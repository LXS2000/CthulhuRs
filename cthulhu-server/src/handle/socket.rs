use crate::net_proxy::HttpContext;
use futures::{SinkExt, StreamExt};

use hyper::http::{uri::Scheme, Request, Response, Uri};
use hyper::Body;
use hyper_tungstenite::tungstenite::{self, protocol::frame::Frame, Message};
use rquickjs::{async_with, CatchResultExt};

use tracing::{error, instrument};

use crate::{
    auto_option, auto_result,
    jsbind::{
        self,
        server::{self, Scope},
    },
    CLIENT_MANAGER, PLUGIN_MANAGER,
};

use super::{bad_request, scope_key_from_request};

async fn linked(
    session_type: &String,
    session_id: &String,
    scope_key: &Scope,
) -> Result<(), String> {
    let (all, _monitors, _modify) = PLUGIN_MANAGER.ctxs_by_host(&scope_key.host).await;
    if all.is_empty() {
        return Ok(());
    }
    for plugin in all {
        let ctx = &plugin.ctx;
        let ctx = ctx.lock().await;
        let _: rquickjs::Result<()> = async_with!(ctx=>|ctx|{
            let res=server::call_function::<(),_>(&ctx, "onClientOpen", (session_type.clone(),session_id.clone(),scope_key.clone())).await
                    .catch(&ctx);
                     auto_result!(res,err=>{
                        return Err(jsbind::handle_js_error(err,&ctx));
                    });
                    Ok(())
        })
        .await;
    }
    Ok(())
}

#[instrument]
pub async fn answer_text(_tabid: &String, text: String) -> Message {
    // let tabid = client_key.tabid.to_string();
    // let obj: HashMap<&str, serde_json::Value> = auto_result!(serde_json::from_str(&text),_err=>{
    //     return response_msg(500,&"json格式错误","err",tabid,"".to_string(),0);
    // });

    // let empty = serde_json::Value::from_str(r#""""#).unwrap();
    // let ty = obj
    //     .get("type")
    //     .unwrap_or(&empty)
    //     .as_str()
    //     .unwrap_or_default();

    // if ty == "link" {
    //     return link(client_key, obj).await;
    // }
    Message::from("<nothing todo>")
}

pub fn answer_binary(_tabid: &String, _bin: Vec<u8>) -> Message {
    Message::from("<nothing todo>")
}
pub fn answer_frame(_tabid: &String, frame: Frame) -> Message {
    let _payload = frame.payload();
    Message::from("<nothing todo>")
}

pub async fn after_close(session_type: &String, session_id: &String, scope_key: &Scope) {
    let (all, _monitors, _modify) = PLUGIN_MANAGER.ctxs_by_host(&scope_key.host).await;
    if all.is_empty() {
        return;
    }
    let mut future_vec = vec![];
    for plugin in all {
        let f = async move {
            let ctx = &plugin.ctx;
            let ctx = ctx.lock().await;
            let session_type = session_type.clone();
            let session_id = session_id.clone();
            let scope_key = scope_key.clone();
            let _: rquickjs::Result<()> = async_with!(ctx=>|ctx|{

                let res=server::call_function::<(),_>(&ctx, "onClientClose",(session_type,session_id,scope_key)).await
                .catch(&ctx);
                 auto_result!(res,err=>{
                    return Err(jsbind::handle_js_error(err,&ctx));
                });
                Ok(())
            })
            .await;
        };
        future_vec.push(f);
    }
    futures::future::join_all(future_vec).await;
    let mut guard = CLIENT_MANAGER.sinks.write().await;
    guard.remove(session_id);
    let mut guard = CLIENT_MANAGER.sessions.write().await;
    guard.get_mut(scope_key).unwrap().remove(session_id);
}
#[instrument(skip(req))]
pub async fn handle_socket(ctx: &HttpContext, mut req: Request<Body>) -> Response<Body> {
    let (session_type, session_id) = {
        let mut items = req
            .uri()
            .path()
            .split(&['/', '\\'])
            .filter(|v| !v.is_empty());
        let session_type = items.next().unwrap_or("").to_string();
        let types = &["content", "worker", "serviceworker", "sharedworker"];
        if !types.contains(&session_type.as_str()) {
            return super::response_msg(500, "invalid session type");
        }
        let session_id = {
            auto_option!(
                items.next(),
                super::response_msg(500, "session id required")
            )
            .to_string()
        };
        (session_type, session_id)
    };

    let scope_key = {
        auto_result!(scope_key_from_request(&ctx.client_addr,&mut req),err=>{
            return super::response_msg(500, err);
        })
    };

    let mut req = {
        let (mut parts, _) = req.into_parts();
        parts.uri = {
            let mut parts = parts.uri.into_parts();
            parts.scheme = if parts.scheme.unwrap_or(Scheme::HTTP) == Scheme::HTTP {
                Some("ws".try_into().expect("Failed to convert scheme"))
            } else {
                Some("wss".try_into().expect("Failed to convert scheme"))
            };
            auto_result!( Uri::from_parts(parts),err=>{
                error!("解析uri失败{err}");
                return bad_request();
            })
        };
        Request::from_parts(parts, ())
    };
    let (res, websocket) = auto_result!(hyper_tungstenite::upgrade(&mut req, None) ,err=>{
        error!("WebSocket upgrade error: {}", err);
        return bad_request();
    });
    tokio::spawn(async move {
        let ws = auto_result!(websocket.await,err=>{
            error!("WebSocket send error: {}", err);
            return ;
        });
        let (sink, mut stream) = ws.split();
        CLIENT_MANAGER
            .add_session_sink(scope_key.clone(), session_id.clone(), sink)
            .await;
        if let Err(e) = linked(&session_type, &session_id, &scope_key).await {
            tracing::error!("{e}");
        }
        while let Some(msg) = stream.next().await {
            let message = auto_result!(msg,err=>{
                error!("WebSocket send error: {}", err);
                return ;
            });
            // debug!("收到消息,{:?}", &message);
            let response = match message {
                Message::Text(text) => answer_text(&session_id, text).await,
                Message::Binary(bin) => answer_binary(&session_id, bin),
                Message::Frame(frame) => answer_frame(&session_id, frame),
                Message::Ping(_) => Message::Pong(vec![]),
                Message::Pong(_) => break,
                Message::Close(_) => {
                    after_close(&session_type, &session_id, &scope_key).await;
                    break;
                }
            };
            let sinks = CLIENT_MANAGER.sinks.read().await;

            let sink = auto_option!(sinks.get(&session_id), {
                after_close(&session_type, &session_id, &scope_key).await;
                break;
            });

            match sink.lock().await.send(response).await {
                Err(tungstenite::Error::ConnectionClosed) => (),
                Err(e) => error!("WebSocket send error: {}", e),
                _ => (),
            };
        }
    });

    return res;
}
