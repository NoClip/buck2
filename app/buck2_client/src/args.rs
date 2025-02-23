/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::process::Command;
use std::str;

use anyhow::Context as _;
use buck2_client_ctx::immediate_config::ImmediateConfigContext;
use buck2_core::fs::fs_util;
use buck2_core::fs::paths::abs_norm_path::AbsNormPathBuf;
use buck2_core::is_open_source;
use buck2_util::process::background_command;
use termwiz::istty::IsTty;
use thiserror::Error;

#[derive(Error, Debug)]
enum ArgExpansionError {
    #[error("Missing flag file path after --flagfile argument")]
    MissingFlagFilePath,
    #[error("Unable to read flag file at `{path}`")]
    MissingFlagFileOnDisk { source: anyhow::Error, path: String },
    #[error("Unable to read line in flag file `{path}`")]
    FlagFileReadError { source: anyhow::Error, path: String },
    #[error("Python mode file `{path}` output is not UTF-8")]
    PythonOutputNotUtf8 { path: String },
    #[error("No flag file path after @ symbol in argfile argument")]
    MissingFlagFilePathInArgfile,
    #[error("Python argfile at `{path}` exited with non-zero status, stderr: {err:?}")]
    PythonExecutableFailed { path: String, err: String },
    #[error("Python argfile command ({cmd:?}) execution failed")]
    PythonExecutionFailed { source: io::Error, cmd: Command },
    #[error("Unable to read line from stdin")]
    StdinReadError { source: anyhow::Error },
}

/// Log that a relative flag file was not found in CWD, but was found, and used, from the cell root
///
/// This prints directly to stderr (sometimes in color). This should be safe, because flagfile
/// expansion runs *very* early in the CLI process lifetime.
pub fn log_relative_path_from_cell_root(requested_path: &str) -> anyhow::Result<()> {
    let (prefix, reset) = if io::stderr().is_tty() {
        ("\x1b[33m", "\x1b[0m")
    } else {
        ("WARNING: ", "")
    };
    buck2_client_ctx::eprintln!(
        "{}`@{}` was specified, but not found. Using file at `//{}`.",
        prefix,
        requested_path,
        requested_path
    )?;
    buck2_client_ctx::eprintln!(
        "This behavior is being deprecated. Please use `@//{}` instead{}",
        requested_path,
        reset
    )?;
    Ok(())
}

#[derive(Clone, Debug)]
enum ArgFile {
    PythonExecutable(AbsNormPathBuf, Option<String>),
    Path(AbsNormPathBuf),
    Stdin,
}

// Expands any argfiles passed as command line parameters. There are
// two ways to do: `@argfile` or `--flagfile PATH`.
//
// Caveats:
//  - `--` and `--flagfile` cannot be values of other options
//  - `--flagfile=X` is _not_ supported, you need to pass
//    `--flagfile X` instead.
//  - `--flagfil` is _not_ supported.
//
// TODO: This function should also return tracking information, so
//       that we know where args come from. This would be useful
//       in cases where the argfiles contain `--config` flags.
pub fn expand_argfiles_with_context(
    args: Vec<String>,
    context: &mut ImmediateConfigContext,
) -> anyhow::Result<Vec<String>> {
    let mut expanded_args = Vec::new();
    let mut arg_iterator = args.into_iter();

    while let Some(next_arg) = arg_iterator.next() {
        match next_arg.as_str() {
            "--" => {
                expanded_args.push(next_arg);
                expanded_args.extend(arg_iterator);
                break;
            }
            "--flagfile" => {
                let flagfile = match arg_iterator.next() {
                    Some(val) => val,
                    None => return Err(anyhow::anyhow!(ArgExpansionError::MissingFlagFilePath)),
                };
                // TODO: We want to detect cyclic inclusion
                let expanded_flagfile_args = resolve_and_expand_argfile(&flagfile, context)?;
                expanded_args.extend(expanded_flagfile_args);
            }
            next_arg if next_arg.starts_with('@') => {
                let flagfile = next_arg.strip_prefix('@').unwrap();
                if flagfile.is_empty() {
                    return Err(anyhow::anyhow!(
                        ArgExpansionError::MissingFlagFilePathInArgfile
                    ));
                }
                // TODO: We want to detect cyclic inclusion
                let expanded_flagfile_args = resolve_and_expand_argfile(flagfile, context)?;
                expanded_args.extend(expanded_flagfile_args);
            }
            _ => expanded_args.push(next_arg),
        }
    }

    Ok(expanded_args)
}

// Resolves a path argument to an absolute path, reads the flag file and expands
// it into a list of arguments.
fn resolve_and_expand_argfile(
    path: &str,
    context: &mut ImmediateConfigContext,
) -> anyhow::Result<Vec<String>> {
    let flagfile = resolve_flagfile(path, context)
        .with_context(|| format!("Error resolving flagfile `{}`", path))?;
    let flagfile_lines = expand_argfile_contents(&flagfile)?;
    expand_argfiles_with_context(flagfile_lines, context)
}

fn expand_argfile_contents(flagfile: &ArgFile) -> anyhow::Result<Vec<String>> {
    match flagfile {
        ArgFile::Path(path) => {
            let mut lines = Vec::new();
            let file =
                File::open(path).map_err(|source| ArgExpansionError::MissingFlagFileOnDisk {
                    source: source.into(),
                    path: path.to_string_lossy().into_owned(),
                })?;
            let reader = io::BufReader::new(file);
            for line_result in reader.lines() {
                let line = line_result.map_err(|source| ArgExpansionError::FlagFileReadError {
                    source: source.into(),
                    path: path.to_string_lossy().into_owned(),
                })?;
                if line.is_empty() {
                    continue;
                }
                lines.push(line);
            }
            Ok(lines)
        }
        ArgFile::PythonExecutable(path, flag) => {
            let mut cmd = background_command(if is_open_source() {
                "python3"
            } else {
                "fbpython"
            });
            cmd.env("BUCK2_ARG_FILE", "1");
            cmd.arg(path.as_os_str());
            if let Some(flag) = flag.as_deref() {
                cmd.args(["--flavors", flag]);
            }
            let cmd_out = cmd
                .output()
                .map_err(|source| ArgExpansionError::PythonExecutionFailed { cmd, source })?;
            if cmd_out.status.success() {
                Ok(str::from_utf8(&cmd_out.stdout)
                    .map_err(|_| ArgExpansionError::PythonOutputNotUtf8 {
                        path: path.to_string_lossy().into_owned(),
                    })?
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>())
            } else {
                Err(anyhow::anyhow!(ArgExpansionError::PythonExecutableFailed {
                    path: path.to_string_lossy().into_owned(),
                    err: String::from_utf8_lossy(&cmd_out.stderr).to_string(),
                }))
            }
        }
        ArgFile::Stdin => io::stdin()
            .lock()
            .lines()
            .filter_map(|line| match line {
                Ok(x) if x.is_empty() => None,
                Ok(x) => Some(Ok(x)),
                Err(err) => Some(Err(ArgExpansionError::StdinReadError {
                    source: err.into(),
                }
                .into())),
            })
            .collect(),
    }
}

// Resolves a path argument to an absolute path, so that it can be read.
fn resolve_flagfile(path: &str, context: &mut ImmediateConfigContext) -> anyhow::Result<ArgFile> {
    if path == "-" {
        return Ok(ArgFile::Stdin);
    }

    let (path_part, flag) = match path.split_once('#') {
        Some((pypath, pyflag)) => (pypath, Some(pyflag)),
        None => (path, None),
    };

    let resolved_path = if let Some(cell_resolved_path) = context.resolve_cell_path_arg(path_part) {
        cell_resolved_path.context("Error resolving cell path")?
    } else {
        let p = Path::new(path_part);
        if !p.is_absolute() {
            match context.canonicalize(p) {
                Ok(abs_path) => Ok(abs_path),
                Err(original_error) => {
                    let cell_relative_path = context.resolve_cell_path("", path_part)?;
                    // If the relative path does not exist relative to the cwd,
                    // attempt to make it relative to the cell root. If *that*
                    // doesn't exist, just report the original error back, and
                    // don't tip users off that they can use relative-to-cell paths.
                    // We want to deprecate that.
                    match fs_util::try_exists(&cell_relative_path) {
                        Ok(true) => {
                            log_relative_path_from_cell_root(path_part)?;
                            Ok(cell_relative_path)
                        }
                        _ => Err(ArgExpansionError::MissingFlagFileOnDisk {
                            source: original_error,
                            path: p.to_string_lossy().into_owned(),
                        }),
                    }
                }
            }?
        } else {
            AbsNormPathBuf::try_from(p.to_owned())?
        }
    };

    context.push_trace(&resolved_path);
    if path_part.ends_with(".py") {
        Ok(ArgFile::PythonExecutable(
            resolved_path,
            flag.map(ToOwned::to_owned),
        ))
    } else {
        Ok(ArgFile::Path(resolved_path))
    }
}

#[cfg(test)]
mod tests {
    use buck2_client_ctx::immediate_config::ImmediateConfigContext;
    use buck2_core::fs::fs_util;
    use buck2_core::fs::paths::abs_norm_path::AbsNormPathBuf;
    use buck2_core::fs::paths::abs_path::AbsPath;
    use buck2_core::fs::working_dir::WorkingDir;

    use super::*;

    #[test]
    fn test_expand_argfile_content() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = AbsPath::new(tempdir.path()).unwrap();
        let mode_file = root.join("mode-file");
        // Test skips empty lines.
        fs_util::write(&mode_file, "a\n\nb\n").unwrap();
        let lines = expand_argfile_contents(&ArgFile::Path(
            AbsNormPathBuf::from(mode_file.to_string_lossy().into_owned()).unwrap(),
        ))
        .unwrap();
        assert_eq!(vec!["a".to_owned(), "b".to_owned()], lines);
    }

    #[test]
    fn test_relative_inclusion() {
        // Currently all @-files both on the command line and in files are relative to the current directory.
        // This matches gcc/clang, so write a test we don't inadvertantly change it.
        let tempdir = tempfile::tempdir().unwrap();
        let root = AbsPath::new(tempdir.path()).unwrap();
        fs_util::create_dir(root.join("foo")).unwrap();
        fs_util::create_dir(root.join("foo/bar")).unwrap();
        fs_util::write(root.join("foo/bar/arg1.txt"), "@bar/arg2.txt").unwrap();
        fs_util::write(root.join("foo/bar/arg2.txt"), "--magic").unwrap();
        fs_util::write(root.join(".buckconfig"), "[repositories]\nroot = .").unwrap();
        let cwd = WorkingDir::unchecked_new(
            AbsNormPathBuf::new(root.canonicalize().unwrap().join("foo")).unwrap(),
        );
        let mut context = ImmediateConfigContext::new(&cwd);
        let res =
            expand_argfiles_with_context(vec!["@bar/arg1.txt".to_owned()], &mut context).unwrap();
        assert_eq!(res, vec!["--magic".to_owned()]);
    }
}
