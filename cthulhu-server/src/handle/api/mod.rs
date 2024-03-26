use std::{collections::HashMap, pin::Pin, str::FromStr};

use crate::net_proxy::HttpContext;

use futures::Future;
use hyper::body::Body;

use hyper::http::{Request, Response};

use serde_json::Value;
use tokio_tungstenite::tungstenite::http::uri::PathAndQuery;
use tracing::{error, instrument};

use crate::{auto_option, auto_result};

use super::{response_headers, response_msg};

pub mod config;
pub mod plugin;
pub mod server;

pub async fn detect(
    req: Request<Body>,
) -> Result<(HashMap<String, String>, Value), Response<Body>> {
    let (parts, mut body) = req.into_parts();
    let empty = PathAndQuery::from_static("/");
    let path_and_quary = parts.uri.path_and_query().unwrap_or(&empty);

    let quary = path_and_quary.query().unwrap_or("");

    let mut params: HashMap<String, String> = HashMap::new();
    quary.split("&").into_iter().for_each(|pair| {
        let mut split = pair.split("=");
        let key = split.next().unwrap_or_default();
        if key.is_empty() {
            return;
        }
        let value = split.next().unwrap_or_default();
        params.insert(key.to_string(), value.to_string());
    });

    let bytes = auto_result!(hyper::body::to_bytes(&mut body).await,err=>{
        error!("异常：{}",err);
        return Err(response_msg(500,"读取请求体异常"));
    });
    if bytes.is_empty() {
        return Ok((params, serde_json::Value::Null));
    }
    let bytes = bytes.to_vec();
    let body = auto_result!(String::from_utf8(bytes),err=>{
        error!("异常：{}",err);
        return Err(response_msg(500,"读取请求体异常"));
    });
    let body = auto_result!( serde_json::Value::from_str(body.as_str()),err=>{
        error!("异常：{}",err);
        return Err(response_msg(500,"JSON序列化请求体异常"));
    });

    Ok((params, body))
}

pub type AsyncFn =
    fn(HttpContext, Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>>;
#[macro_export]
macro_rules! wrap {
    ($expr:expr) => {
        Box::new(|ctx, req| Box::pin(async move { $expr(ctx, req).await }))
    };
}
lazy_static::lazy_static! {
    pub static ref ROUTER: HashMap<&'static str, Box<AsyncFn>> = {
        let mut router: HashMap<&'static str, Box<AsyncFn>> = HashMap::new();
        server::route(&mut router);
        plugin::route(&mut router);
        config::route(&mut router);
        router
    };
}

#[instrument(skip(req))]
pub async fn handle_api(ctx: &HttpContext, req: Request<Body>) -> Response<Body> {
    let ctx = ctx.clone();

    let path = req.uri().path().to_string();

    let handler = auto_option!(
        ROUTER.get(&path.as_str()),
        Response::builder().status(404).body(Body::empty()).unwrap()
    );
    let res = handler(ctx, req).await;

    response_headers(res)
}
