import{_ as E,u as N,h as k,i as f,r as y,o as c,j as V,w as u,e as t,d as m,k as C,F as x,t as l,g as z,f as v,l as K,n as S}from"../assets/index.js";import{t as F,a as L}from"./api.js";import{t as $}from"./local.js";const P={class:"row",style:{"margin-top":"10px"}},T={class:"names col"},j=["onClick"],D={class:"omit",style:{width:"80%"}},I={class:"col",style:{width:"76%"}},M={class:"row",style:{"margin-bottom":"0.3rem"}},R={style:{"font-weight":"bold","font-size":"0.8rem",color:"#b2b2b2"}},Y={class:"row",style:{background:"#262626","font-size":"0.9rem","font-weight":"bold",color:"#c7c7c7"}},q={style:{width:"30%"}},A={style:{width:"70%"}},G={class:"kvs col"},H=["onClick"],J={class:"key omit"},O={class:"value omit"},Q={class:"row",style:{"margin-bottom":"0.3rem","margin-top":"1rem"}},U={style:{"font-weight":"bold","font-size":"0.9rem",color:"#b2b2b2"}},W={class:"valueContent col"},X={__name:"Store",setup(Z){const g=N().state.curPlugin,a=k([]),n=f(""),i=k({}),r=f(""),b=f("");async function w(o){n.value=o;let e=i[o];e||await L(g.id,o).then(d=>{i[o]=d,e=d});let _=e[0]||[];h(_[0],_[1])}function h(o,e){r.value=o,b.value=e}return F(g.id).then(async o=>{a.splice(0,a.length),a.push(...o),n.value=a[0],await w(n.value);let e=i[n.value][0]||[];h(e[0],e[1])}),(o,e)=>{const _=y("Coin"),d=y("el-icon"),p=y("el-scrollbar");return c(),V(p,{height:"100%",style:{height:"100%"}},{default:u(()=>[t("div",P,[t("div",T,[(c(!0),m(x,null,C(a,s=>(c(),m("div",{class:"row name",onClick:B=>w(s),style:S(`color:${s===n.value?"#5cf1f6":"#c7c7c7"}`)},[v(d,{style:{width:"20%",margin:"auto"}},{default:u(()=>[v(_)]),_:1}),t("span",D,l(s),1)],12,j))),256))]),t("div",I,[t("div",M,[e[0]||(e[0]=t("span",{style:{"font-size":"0.8rem",color:"#c5c5c5","font-weight":"bold"}},"TREE：",-1)),t("span",R,l(n.value),1)]),t("div",Y,[t("span",q,l(z($)("k")),1),t("span",A,l(z($)("v")),1)]),t("div",G,[v(p,null,{default:u(()=>[(c(!0),m(x,null,C(i[n.value],s=>(c(),m("div",{class:"row",onClick:B=>h(s[0],s[1]),style:S(`background:${s[0]===r.value?"rgba(121,120,120,0.92)":""};color:${s[0]===r.value?"rgb(245,245,245)":"#b6b6b6"}`)},[t("span",J,l(s[0]),1),t("span",O,l(s[1]),1)],12,H))),256))]),_:1})]),t("div",Q,[e[1]||(e[1]=t("span",{style:{"font-size":"0.9rem",color:"#c5c5c5","font-weight":"bold"}},"KEY：",-1)),t("span",U,l(r.value),1)]),t("div",W,[v(p,{style:{padding:"0.3rem"}},{default:u(()=>[K(l(b.value),1)]),_:1})]),e[2]||(e[2]=t("div",{style:{height:"1rem"}},null,-1))])]),e[3]||(e[3]=t("div",{style:{height:"3rem"}},null,-1))]),_:1})}}},lt=E(X,[["__scopeId","data-v-5580a53a"]]);export{lt as default};