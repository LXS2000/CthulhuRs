import{_ as x,i as I,u as z,c as _,r as a,o as p,d,e as t,f as o,g as s,t as h,l as k,w as m,m as N,n as S}from"../assets/index.js";import{I as B}from"./IconBtn.js";import{L as T}from"./local.js";import"./SvgIcon.js";const V={class:"col",style:{height:"100%"}},$={class:"head row",style:{height:"20px"}},F={class:"row",style:{"align-items":"center"}},H={style:{color:"#c7c7c7","font-size":"1rem","font-weight":"bold"}},L=["src"],P={__name:"Func",setup(R){const e=new T({flush:{zh:"刷新插件页面",en:"flush plugin web page"},open:{zh:"在新页面打开",en:"open in new tab"}},!0);e.tlp.bind(e),e.tlf.bind(e);const c=e.tl.bind(e),n=I(!0),r=z();let f=_(()=>r.state.windowHeight-80);const l=r.state.curPlugin,g=self.CTHULHU_SCOPE_ID||"",i=_(()=>`https://${l.id}.plugin.cthulhu.server/${l.webRoot}?scopeId=${g}`);function w(){window.open(i.value)}function v(){n.value=!1,setTimeout(()=>n.value=!0,200)}return(D,u)=>{const b=a("CaretRight"),y=a("el-icon"),C=a("el-scrollbar");return p(),d("div",V,[t("div",$,[t("div",F,[o(B,{size:1.2,name:"flush",color:"#62da59",onClick:v,prompt:s(c)("flush")},null,8,["prompt"])]),t("div",H,h(s(l).name),1),t("div",{class:"row",onClick:w,style:{color:"#5cf1f6","font-size":"0.7rem","align-items":"center"}},[k(h(s(c)("open"))+" ",1),o(y,null,{default:m(()=>[o(b)]),_:1})])]),o(C,{height:"100%",style:S(`height: ${s(f)}px`)},{default:m(()=>[n.value?(p(),d("iframe",{key:0,border:"0",allow:"clipboard-write",frameborder:"0",class:"page",src:i.value},null,8,L)):N("",!0),u[0]||(u[0]=t("div",{style:{height:"3rem"}},null,-1))]),_:1},8,["style"])])}}},q=x(P,[["__scopeId","data-v-f9a14e3a"]]);export{q as default};