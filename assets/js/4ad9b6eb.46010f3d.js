"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[6],{3905:(e,t,r)=>{r.r(t),r.d(t,{MDXContext:()=>u,MDXProvider:()=>p,mdx:()=>v,useMDXComponents:()=>d,withMDXComponents:()=>l});var n=r(67294);function a(e,t,r){return t in e?Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}):e[t]=r,e}function o(){return o=Object.assign||function(e){for(var t=1;t<arguments.length;t++){var r=arguments[t];for(var n in r)Object.prototype.hasOwnProperty.call(r,n)&&(e[n]=r[n])}return e},o.apply(this,arguments)}function i(e,t){var r=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),r.push.apply(r,n)}return r}function c(e){for(var t=1;t<arguments.length;t++){var r=null!=arguments[t]?arguments[t]:{};t%2?i(Object(r),!0).forEach((function(t){a(e,t,r[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(r)):i(Object(r)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(r,t))}))}return e}function s(e,t){if(null==e)return{};var r,n,a=function(e,t){if(null==e)return{};var r,n,a={},o=Object.keys(e);for(n=0;n<o.length;n++)r=o[n],t.indexOf(r)>=0||(a[r]=e[r]);return a}(e,t);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);for(n=0;n<o.length;n++)r=o[n],t.indexOf(r)>=0||Object.prototype.propertyIsEnumerable.call(e,r)&&(a[r]=e[r])}return a}var u=n.createContext({}),l=function(e){return function(t){var r=d(t.components);return n.createElement(e,o({},t,{components:r}))}},d=function(e){var t=n.useContext(u),r=t;return e&&(r="function"==typeof e?e(t):c(c({},t),e)),r},p=function(e){var t=d(e.components);return n.createElement(u.Provider,{value:t},e.children)},m="mdxType",f={inlineCode:"code",wrapper:function(e){var t=e.children;return n.createElement(n.Fragment,{},t)}},y=n.forwardRef((function(e,t){var r=e.components,a=e.mdxType,o=e.originalType,i=e.parentName,u=s(e,["components","mdxType","originalType","parentName"]),l=d(r),p=a,m=l["".concat(i,".").concat(p)]||l[p]||f[p]||o;return r?n.createElement(m,c(c({ref:t},u),{},{components:r})):n.createElement(m,c({ref:t},u))}));function v(e,t){var r=arguments,a=t&&t.mdxType;if("string"==typeof e||a){var o=r.length,i=new Array(o);i[0]=y;var c={};for(var s in t)hasOwnProperty.call(t,s)&&(c[s]=t[s]);c.originalType=e,c[m]="string"==typeof e?e:a,i[1]=c;for(var u=2;u<o;u++)i[u]=r[u];return n.createElement.apply(null,i)}return n.createElement.apply(null,r)}y.displayName="MDXCreateElement"},1973:(e,t,r)=>{r.r(t),r.d(t,{assets:()=>s,contentTitle:()=>i,default:()=>d,frontMatter:()=>o,metadata:()=>c,toc:()=>u});var n=r(87462),a=(r(67294),r(3905));const o={id:"restarter",title:"Restarter"},i=void 0,c={unversionedId:"users/advanced/restarter",id:"users/advanced/restarter",title:"Restarter",description:"The Restarter can automatically restart Buck2 when Buck2 detects that it hit a",source:"@site/../docs/users/advanced/restarter.md",sourceDirName:"users/advanced",slug:"/users/advanced/restarter",permalink:"/docs/users/advanced/restarter",draft:!1,tags:[],version:"current",frontMatter:{id:"restarter",title:"Restarter"},sidebar:"manualSidebar",previous:{title:"Deferred Materialization",permalink:"/docs/users/advanced/deferred_materialization"},next:{title:"In Memory Cache",permalink:"/docs/users/advanced/in_memory_cache"}},s={},u=[{value:"Enabling the Restarter",id:"enabling-the-restarter",level:2}],l={toc:u};function d(e){let{components:t,...r}=e;return(0,a.mdx)("wrapper",(0,n.Z)({},l,r,{components:t,mdxType:"MDXLayout"}),(0,a.mdx)("p",null,"The Restarter can automatically restart Buck2 when Buck2 detects that it hit a\ncondition that may be recovered by restarting the Buck2 daemon."),(0,a.mdx)("p",null,"This is particularly useful with\n",(0,a.mdx)("a",{parentName:"p",href:"/docs/users/advanced/deferred_materialization"},"Deferred Materialization"),", which may require a\ndaemon restart if your daemon holds references to artifacts that have expired in\nyour Remote Execution backend."),(0,a.mdx)("h2",{id:"enabling-the-restarter"},"Enabling the Restarter"),(0,a.mdx)("p",null,"To enable, add this to your Buckconfig:"),(0,a.mdx)("pre",null,(0,a.mdx)("code",{parentName:"pre"},"[buck2]\nrestarter = true\n")))}d.isMDXComponent=!0}}]);