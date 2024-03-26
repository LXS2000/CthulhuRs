use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use content_security_policy::{Directive, Policy, PolicyDisposition, PolicySource};
use futures::future::Either;
use rquickjs::{async_with, CatchResultExt, Object};

use tracing::instrument;

use crate::{
    auto_option, auto_result,
    core::PluginCtx,
    jsbind::{
        self,
        http::*,
        server::{self, Scope},
        ws::*,
    },
    PLUGIN_MANAGER,
};

#[instrument(skip(jsreq))]
pub async fn on_request(scope_key: &Scope, jsreq: JsRequest) -> JsHttpAction {
    let (_all, _monitors, modify) = PLUGIN_MANAGER.ctxs_by_host(&scope_key.host).await;
    let modify = auto_option!(modify, JsHttpAction::release(jsreq));
    let ctx = &modify.ctx.as_ref().unwrap();
    let ctx = ctx.lock().await;
    let scope_key = scope_key.clone();
    return async_with!(ctx=>|ctx|{

        let res=server::call_function::<JsHttpAction,_>(&ctx, "onRequest", ( jsreq.clone(),scope_key)).await
        .catch(&ctx);

        let res= auto_result!(res,err=>{
                jsbind::handle_js_error(err,&ctx);
                return JsHttpAction::release(jsreq);
        });
        match res {
           Either::Left(v) => v,
           Either::Right((jsreq,_)) => JsHttpAction::release(jsreq),
        }
})    .await;
}

#[instrument(skip(jsres))]
pub async fn on_response(scope_key: &Scope, jsres: JsResponse) -> JsResponse {
    let (_all, _monitors, modify) = PLUGIN_MANAGER.ctxs_by_host(&scope_key.host).await;
    let modify = auto_option!(modify, jsres);

    let ctx = modify.ctx.as_ref().unwrap();
    let ctx = ctx.lock().await;
    let scope_key = scope_key.clone();
    let result: JsResponse = async_with!(ctx=>|ctx|{
        let res=server::call_function::<JsResponse,_>(&ctx, "onResponse", ( jsres.clone(),scope_key)).await
                .catch(&ctx);

        let res= auto_result!(res,err=>{
                jsbind::handle_js_error(err,&ctx);
                return jsres;
        });
        match res {
           Either::Left(v) => v,
           Either::Right((jsres,_)) => jsres,
        }

    })
    .await;
    result
}

#[instrument(skip(msg))]
pub async fn on_message(scope_key: &Scope, msg: JsMessage, client_to_server: bool) -> JsWsAction {
    let (_all, _monitors, modify) = PLUGIN_MANAGER.ctxs_by_host(&scope_key.host).await;
    let modify = auto_option!(modify, JsWsAction::release(msg));

    let ctx = modify.ctx.as_ref().unwrap();
    let ctx = ctx.lock().await;

    let scope_key = scope_key.clone();
    let result = async_with!(ctx=>|ctx|{
        let res=server::call_function::<JsWsAction,_>(&ctx, "onMessage", ( msg.clone(),scope_key)).await
                .catch(&ctx);

        let res= auto_result!(res,err=>{
                jsbind::handle_js_error(err,&ctx);
                return JsWsAction::release(msg);
        });
        match res {
            Either::Left(v) => v,
            Either::Right((msg, _)) => JsWsAction::release(msg),
         }
    })
    .await;
    
    result
}

#[instrument(skip(jsreq))]
pub async fn watch_request(scope_key: &Scope, jsreq: JsRequest) {
    let (_all, monitors, _modify) = PLUGIN_MANAGER.ctxs_by_host(&scope_key.host).await;
    if monitors.is_empty() {
        return;
    }
    let mut vec = vec![];
    let mut jsreq = jsreq.copy_self().await.unwrap();
    jsreq.is_mut = false;
    for plugin in monitors {
        let jsreq = jsreq.clone();
        let fut = async move {
            let ctx = plugin.ctx.as_ref().unwrap();
            let ctx = ctx.lock().await;
            let scope_key = scope_key.clone();
            async_with!(ctx=> |ctx|{
                let res=server::call_function::<(),_>(&ctx, "watchRequest", (jsreq,scope_key)).await
                .catch(&ctx);
                 auto_result!(res,err=>{
                    jsbind::handle_js_error(err,&ctx);
                    return;
                });
            })
            .await;
        };
        vec.push(fut);
    }
    futures::future::join_all(vec).await;
}

#[instrument(skip(jsres))]
pub async fn watch_response(scope_key: &Scope, jsres: JsResponse) {
    let (_all, monitors, _modify) = PLUGIN_MANAGER.ctxs_by_host(&scope_key.host).await;
    if monitors.is_empty() {
        return;
    }
    let mut vec = vec![];
    let mut jsres = jsres.copy_self().await.unwrap();
    jsres.is_mut = false;
    for plugin in monitors {
        let jsres = jsres.clone();
        let fut = async move {
            let ctx = plugin.ctx.as_ref().unwrap();
            let ctx = ctx.lock().await;
            let scope_key = scope_key.clone();
            async_with!(ctx=> |ctx|{
                let res=server::call_function::<(),_>(&ctx, "watchResponse", ( jsres,scope_key)).await
                .catch(&ctx);
                 auto_result!(res,err=>{
                     jsbind::handle_js_error(err,&ctx);
                    return;
                });
            })
            .await;
        };
        vec.push(fut);
    }
    futures::future::join_all(vec).await;
}
#[instrument(skip(jsmsg))]
pub async fn watch_message(scope_key: &Scope, jsmsg: JsMessage, client_to_server: bool) {
    let (_all, monitors, _modify) = PLUGIN_MANAGER.ctxs_by_host(&scope_key.host).await;
    if monitors.is_empty() {
        return;
    }
    let mut vec = vec![];
    for plugin in monitors {
        let jsmsg = jsmsg.clone();
        let fut = async move {
            let ctx = plugin.ctx.as_ref().unwrap();
            let ctx = ctx.lock().await;
            let scope_key = scope_key.clone();
            async_with!(ctx=> |ctx|{
                let res=server::call_function::<(),_>(&ctx, "watchMessage", (jsmsg,scope_key)).await
                .catch(&ctx);

                 auto_result!(res,err=>{
                     jsbind::handle_js_error(err,&ctx);
                    return;
                });

            })
            .await;
        };
        vec.push(fut);
    }
    futures::future::join_all(vec).await;
}

#[instrument]
pub async fn content_security_policy(plugins: &Vec<Arc<PluginCtx>>, csp_str: &str) -> String {
    let base_policy = Policy::parse(csp_str, PolicySource::Header, PolicyDisposition::Enforce);
    let mut policies = vec![];

    let script = Policy::parse(
        "script-src 'self' 'unsafe-eval' 'unsafe-inline'  https://*.cthulhu.server",
        PolicySource::Header,
        PolicyDisposition::Enforce,
    );
    let script_elem = Policy::parse(
        "script-src-elem 'self' 'unsafe-eval' 'unsafe-inline'  https://*.cthulhu.server",
        PolicySource::Header,
        PolicyDisposition::Enforce,
    );
    policies.push(script);
    policies.push(script_elem);

    let style = Policy::parse(
        "style-src 'self' 'unsafe-inline' https://*.cthulhu.server",
        PolicySource::Header,
        PolicyDisposition::Enforce,
    );
    policies.push(style);
    let connect = Policy::parse(
        "connect-src 'self' 'unsafe-inline' https://*.cthulhu.server wss://*.cthulhu.server",
        PolicySource::Header,
        PolicyDisposition::Enforce,
    );
    policies.push(connect);
    let items = vec![
        "worker-src",
        "object-src",
        "frame-src",
        "child-src",
        "img-src",
        // "font-src",
    ];
    for item in items {
        let item = format!("{item} 'self' https://*.cthulhu.server");
        let item = Policy::parse(
            item.as_str(),
            PolicySource::Header,
            PolicyDisposition::Enforce,
        );
        policies.push(item);
    }

    for plugin in plugins {
        let ctx = plugin.ctx.as_ref().unwrap();

        let ctx = ctx.lock().await;

        let csp = async_with!(ctx=> |ctx|{
            let globals=ctx.globals();
            let server=globals.get::<_,Object>("server").unwrap();
            let csp= server.get::<_,String>("csp").unwrap_or_default();
            let policy = Policy::parse(csp.as_str(), PolicySource::Header, PolicyDisposition::Enforce);
            if !policy.is_valid() {
                let _=ctx.throw(rquickjs::Value::from_string(rquickjs::String::from_str(ctx.clone(), "invalid content security policy").unwrap()));
                return None;
            }
            Some(policy)
        })
        .await;
        if let Some(csp) = csp {
            policies.push(csp);
        }
    }

    let mut base_csp: HashMap<String, HashSet<String>> = HashMap::new();
    let mut csp: HashMap<String, HashSet<String>> = HashMap::new();
    fn to_map(policy: Policy, map: &mut HashMap<String, HashSet<String>>) {
        for directive in policy.directive_set {
            let name = directive.name;
            let values = match map.get_mut(&name) {
                Some(v) => v,
                None => {
                    map.insert(name.clone(), HashSet::new());
                    map.get_mut(&name).unwrap()
                }
            };
            directive.value.into_iter().for_each(|v| {
                values.insert(v);
            });
        }
    }
    to_map(base_policy, &mut base_csp);
    for policy in policies {
        to_map(policy, &mut csp);
    }
    //将网页原本csp中没有的指令都删掉，避免加了来自插件的限制后影响到网页的资源加载
    csp.into_iter().for_each(|(name, v)| {
        if let Some(values) = base_csp.get_mut(&name) {
            values.extend(v);
        }
    });

    let directives = base_csp
        .into_iter()
        .map(|(name, mut values)| {
            values.remove("'none'"); //none的意思是不加载任何资源，必须删除
            if name == "script-src" || name == "script-src-elem" {
                //不删除的话不能加载外部脚本
                values.remove("'strict-dynamic'");
                values.retain(|v| !(v.starts_with("'nonce-") || v.starts_with("'sha")));
            }
            Directive {
                name,
                value: Vec::from_iter(values.into_iter()),
            }
        })
        .collect::<Vec<Directive>>();
    let policy = Policy {
        directive_set: directives,
        disposition: PolicyDisposition::Enforce,
        source: PolicySource::Header,
    };
    policy.to_string()
}
