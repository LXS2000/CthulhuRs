

use crate::net_proxy::HttpContext;
use crate::CLIENT_MANAGER;


use futures::future::Either;
use hyper::http::{header::CONTENT_TYPE, Request, Response};
use hyper::Body;
use rquickjs::{async_with, CatchResultExt};
use serde_json::json;

use crate::{auto_option, auto_result, handle::scope_key_from_request, jsbind::{self, server}, PLUGIN_MANAGER};

use super::api::{self, detect};
use super::response_json;
use super::{response_file, response_msg};
fn split_path(path:&str)->(&str,&str){
    let mut iter = path.splitn(3, "/").skip(1);
  (  iter.next().unwrap_or_default(),iter.next().unwrap_or_default())
}
pub async fn handle_plugin(ctx: &HttpContext, req: Request<Body>) -> Response<Body> {
    let host = req.uri().host().unwrap();
    let (id,_) = host.split_once(".").unwrap_or_default();
    if id.is_empty(){
        return  response_msg(404, "Invalid plugin id");
    }
    let id=id.to_owned();
    let path = req.uri().path();
    
    let plugin = auto_option!(
        api::plugin::get_plugin_by_id(&id).await,
        response_msg(404, "Invalid plugin id")
    );
    if path.starts_with("/ask"){
        return ask(ctx, req, &id).await;
    }if path.starts_with("/dynamic/"){
        return dynamic_script(ctx, req, &id).await;
    }
   
    let base = &plugin.path.replace("\\", "/");
    let root = &plugin.web_root;
  
    let file_path = relative_path::RelativePath::new(&path).to_logical_path(base);
    if file_path.is_file() {
        return response_file(file_path.to_str().unwrap()).await;
    }
    if let Some((_,end))= path.split_once(".") {
        if !end.is_empty() {
            return response_msg(404,"File not found");
        }
    }
    let root_path = relative_path::RelativePath::new(root).to_logical_path(base);
    let index_path = relative_path::RelativePath::new(&plugin.web_index).to_logical_path(root_path);
    response_file(index_path.to_str().unwrap()).await
}


pub async fn dynamic_script(ctx: &HttpContext, mut req: Request<Body>,id:&str) -> Response<Body> {
    
    let path = req.uri().path().to_owned();
    let (_,link) = split_path(&path);
  
    let scope_key = {
        auto_result!(scope_key_from_request(&ctx.client_addr,&mut req),err=>{
            return super::response_msg(500, err);
        })
    };
    let ctx = auto_option!(
        PLUGIN_MANAGER.get_ctx(id).await,
        response_msg(500, "Invalid plugin id")
    );
    let ctx = ctx.ctx.as_ref().unwrap();
    let ctx = ctx.lock().await;
    let response = async_with!(ctx=>|ctx|{
        let res=server::call_function::<Option<String>,_>(&ctx, "dynamicScript", (link, scope_key)).await.catch(&ctx);
        let res=auto_result!(res,err=>{
            let e=err.to_string();
            jsbind::handle_js_error(err,&ctx);
            return response_msg(500, e);
        });
        match res {
           Either::Left(value) =>{
                hyper::Response::builder()
                .status(200)
                .header(CONTENT_TYPE, "application/javascript")
                .body(value.unwrap_or_default().into()).unwrap() 
            },
           Either::Right(_args) => {
                    return response_msg(500, "dynamicScript not a function")
                },
            }
       
    })
    .await;
    response
}

async fn ask(_ctx: &HttpContext, req: Request<Body>,id:&str) -> Response<Body> {
    let (mut params, data) = auto_result!(detect(req).await);
    //插件请求
   
    let key = params.remove("key").unwrap_or_default();
    let scope_id = params.remove("scopeId").unwrap_or_default();

    let guard = CLIENT_MANAGER.scope_keys.read().await;
    let scope_key = auto_option!(
        guard.get(&scope_id),
        response_msg(500, "invalid scope id")
    ).clone();

    let ctx = auto_option!(
        PLUGIN_MANAGER.get_ctx(id).await,
        response_msg(500, "invalid plugin id")
    );
    let ctx =ctx.ctx.as_ref().unwrap();
    let ctx = ctx.lock().await;
    let json = async_with!(ctx=>|ctx|{
        let body=jsbind::json_to_js(data, &ctx).unwrap();
        let res=server::call_function::<rquickjs::Value<'_>,_>(&ctx, "onAsk", (key, body, scope_key)).await.catch(&ctx);
        let res=auto_result!(res,err=>{
            let e=err.to_string();
            jsbind::handle_js_error(err,&ctx);
            let json=json!({"code":500,"msg":e});
            return json;
        });
        match res {
   Either::Left(value) =>{
        let json=auto_result!( jsbind::js_to_json(value),err=>{
            return json!({"code":500,"msg":err.to_string()});
         });
         json
    },
       Either::Right(_args) => {
        return json!({"code":500,"msg":"onAsk not a function"});
        },
    }
       
    })
    .await;
    response_json( &json)
}
