use async_trait::async_trait;

use hyper::Body;
use hyper::{
    http::{
        header::{HeaderName, HeaderValue},
        HeaderMap, Method, StatusCode, Version,
    },
    service::Service,
};
use std::collections::HashMap;
use std::fs::File;

use std::sync::Arc;
use tokio_util::codec::{BytesCodec, FramedRead};

use std::{
    fmt::{self, Debug},
    mem,
    str::FromStr,
    sync::{Mutex, RwLock},
};

use rquickjs::{
    class::Trace,
    function::{Async, Func},
    Class, Ctx,
};

use crate::{
    auto_option, auto_result, core::ProxyCfg, create_client, reqwest_request_from_hyper,
    reqwest_response_to_hyper,
};
use rquickjs::Result;

use super::file::JsFile;

use super::{throw_js_err, to_js_err};

fn version_from_str(v: &str) -> Option<hyper::Version> {
    let up = v.to_uppercase();
    let v = match up.as_str() {
        "HTTP/0.9" => hyper::Version::HTTP_09,
        "HTTP/1.0" => hyper::Version::HTTP_10,
        "HTTP/1.1" => hyper::Version::HTTP_11,
        "HTTP/2.0" => hyper::Version::HTTP_2,
        "HTTP/3.0" => hyper::Version::HTTP_3,
        _ => return None,
    };
    Some(v)
}
#[derive(Clone, Debug)]
pub enum HttpAction {
    Reject,
    Delay(JsRequest, u64), //延迟多少毫秒
    Proxy(JsRequest, ProxyCfg),
    Respond(JsResponse),
    Release(JsRequest),
}

#[rquickjs::class(rename = "Uri")]
#[derive(Trace, Clone, Default)]
pub struct JsUri {
    #[qjs(get, set, enumerable, configurable)]
    pub scheme: String,
    #[qjs(get, set, enumerable, configurable)]
    pub authority: String,
    #[qjs(get, set, enumerable, configurable)]
    pub host: String,
    #[qjs(get, set, enumerable, configurable)]
    pub port: u16,
    #[qjs(get, set, enumerable, configurable)]
    pub path: String,
    #[qjs(get, set, enumerable, configurable)]
    pub params: std::collections::HashMap<String, String>,
}

#[rquickjs::methods]
impl JsUri {
    #[qjs(constructor)]
    pub fn new(url: String, ctx: Ctx<'_>) -> rquickjs::Result<Self> {
        let uri = hyper::Uri::from_str(&url).map_err(|e| to_js_err(e, ctx))?;
        Ok(Self::from(uri))
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self, ctx: Ctx<'_>) -> rquickjs::Result<String> {
        self.assemble().map_err(|e| throw_js_err(e, ctx))
    }
    #[qjs(skip)]
    pub fn assemble(&self) -> std::result::Result<String, &'static str> {
        let mut uri = String::new();
        if self.scheme.is_empty() {
            return Err("Invalid scheme");
        }
        uri.push_str(&self.scheme);
        uri.push_str("://");

        if self.authority.is_empty() {
            if self.host.is_empty() {
                return Err("Invalid authority and host");
            }
            uri.push_str(&self.host);
            if self.port != 0 {
                uri.push(':');
                uri.push_str(&self.port.to_string());
            }
        } else {
            uri.push_str(&self.authority);
        }

        if !self.path.is_empty() {
            if !self.path.starts_with("/") {
                uri.push('/');
            }
            let path = &self.path;
            uri.push_str(path);
            while uri.ends_with("/") {
                uri.pop();
            }
        }

        if !self.params.is_empty() {
            let query = self
                .params
                .iter()
                .map(|(k, v)| {
                    let k = urlencoding::encode(k);
                    let v = urlencoding::encode(v);
                    format!("{k}={v}")
                })
                .collect::<Vec<String>>()
                .join("&");
            uri.push('?');
            uri.push_str(&query);
        }
        Ok(uri)
    }
}
impl From<hyper::Uri> for JsUri {
    fn from(uri: hyper::Uri) -> Self {
        Self::from(&uri)
    }
}

impl From<&hyper::Uri> for JsUri {
    fn from(uri: &hyper::Uri) -> Self {
        let authority = uri.authority().map(|v| v.to_string()).unwrap_or_default();
        let host = uri.host().map(|v| v.to_string()).unwrap_or_default();
        let mut params = HashMap::new();
        let path = if let Some(pq) = uri.path_and_query() {
            if let Some(query) = pq.query() {
                for item in query.split("&") {
                    let (key, value) = match item.split_once("=") {
                        Some(v) => v,
                        None => continue,
                    };
                    let k=urlencoding::decode(key).expect(key);
                    let v=urlencoding::decode(value).expect(value);
                    params.insert(k.to_string(), v.to_string());
                }
            }
            let path = pq.path();
            Some(path.to_string())
        } else {
            None
        }
        .unwrap_or_default();
        let scheme = uri.scheme_str().map(|v| v.to_string()).unwrap_or_default();
        let port = uri.port_u16().unwrap_or(0);
        Self {
            scheme,
            authority,
            host,
            port,
            path,
            params,
        }
    }
}

#[rquickjs::class(rename = "Headers")]
#[derive(Trace, Clone)]
pub struct JsHeaders {
    pub is_mut: bool,
    #[qjs(skip_trace)]
    pub inner: Arc<RwLock<HeaderMap>>,
}

#[rquickjs::methods]
impl JsHeaders {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {
            is_mut: true,
            inner: Arc::new(std::sync::RwLock::new(HeaderMap::new())),
        }
    }
    #[qjs(rename = "get")]
    pub fn get(&self, key: String) -> Option<String> {
        if key.is_empty() {
            return None;
        }
        let headers = self.inner.read().unwrap();
        let k = HeaderName::from_bytes(key.as_bytes()).unwrap();
        headers.get(k).map(|v| v.to_str().unwrap_or("").to_string())
    }
    #[qjs(rename = "keys")]
    pub fn keys(&self) -> Vec<String> {
        let headers = self.inner.read().unwrap();
        let mut keys = vec![];
        for key in headers.keys() {
            let key = key.to_string();
            keys.push(key);
        }
        keys
    }

    #[qjs(rename = "remove")]
    pub fn remove(&self, key: String) -> Option<String> {
        if !self.is_mut || key.is_empty() {
            return None;
        }
        let key = HeaderName::from_bytes(key.as_bytes()).unwrap();
        let mut headers_mut = self.inner.write().unwrap();
        headers_mut
            .remove(key)
            .map(|v| v.to_str().unwrap_or("").to_string())
    }
    #[qjs(rename = "clear")]
    pub fn clear(&self) {
        if !self.is_mut {
            return;
        }
        let mut headers_mut = self.inner.write().unwrap();
        headers_mut.clear();
    }
    #[qjs(rename = "append")]
    pub fn append(&self, key: String, value: String) {
        if !self.is_mut || key.is_empty() || value.is_empty() {
            return;
        }
        let mut headers_mut = self.inner.write().unwrap();
        let k = HeaderName::from_bytes(key.as_bytes()).unwrap();
        let val = auto_result!(HeaderValue::from_str(value.as_str()), ());
        headers_mut.append(k, val);
    }
    #[qjs(rename = "insert")]
    pub fn insert(&self, key: String, value: String) {
        if !self.is_mut || key.is_empty() || value.is_empty() {
            return;
        }
        let mut headers_mut = self.inner.write().unwrap();
        let k = HeaderName::from_bytes(key.as_bytes()).unwrap();
        let val = auto_result!(HeaderValue::from_str(value.as_str()), ());
        headers_mut.insert(k, val);
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        let headers = self.inner.read().unwrap();
        format!("{:?}", &*headers)
    }
    #[qjs(skip)]
    pub fn copy_self(&self) -> Self {
        let headers = self.inner.read().unwrap().clone();
        Self {
            is_mut: self.is_mut,
            inner: Arc::new(RwLock::new(headers)),
        }
    }
}

#[rquickjs::class(rename = "Body")]
#[derive(Trace, Clone, Default)]
pub struct JsBody {
    #[qjs(skip_trace)]
    pub inner: Arc<Mutex<Body>>,
}
#[rquickjs::methods]
impl JsBody {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(throw_js_err("Illegal constructor", ctx))
    }
    #[qjs(static, rename = "empty")]
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Body::empty())),
        }
    }
    #[qjs(static, rename = "str")]
    pub fn str(s: String) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Body::from(s))),
        }
    }
    #[qjs(static, rename = "bytes")]
    pub fn bytes(bytes: Vec<u8>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Body::from(bytes))),
        }
    }
    #[qjs(static, rename = "file")]
    pub fn file(file: JsFile, ctx: Ctx<'_>) -> Result<Self> {
        let path = { file.path };
        let file = File::open(path.inner).map_err(|e| to_js_err(e, ctx))?;
        let file = tokio::fs::File::from_std(file);
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        Ok(Self {
            inner: Arc::new(Mutex::new(body)),
        })
    }
    #[qjs(rename = "toBytes")]
    async fn to_bytes_js(&self, ctx: Ctx<'_>) -> Result<Vec<u8>> {
        let bytes = self.to_bytes().await.map_err(|e| to_js_err(e, ctx))?;
        Ok(bytes.to_vec())
    }
    #[qjs(skip)]
    pub async fn to_bytes(&self) -> hyper::Result<Vec<u8>> {
        let mut empty = Body::empty();
        {
            let mut body = self.inner.lock().unwrap();
            std::mem::swap(&mut *body, &mut empty);
            //交换
        };
        let bytes = hyper::body::to_bytes(&mut empty).await?;
        {
            let mut body = self.inner.lock().unwrap();
            std::mem::swap(&mut *body, &mut empty);
            //恢复
        };
        Ok(bytes.to_vec())
    }
    #[qjs(skip)]
    pub fn replace(&self, body: Body) {
        let mut guard = self.inner.lock().unwrap();
        *guard = body;
    }
}

#[rquickjs::class(rename = "Request")]
#[derive(Debug, Trace, Clone)]
pub struct JsRequest {
    pub is_mut: bool,
    #[qjs(skip_trace)]
    pub parts: Arc<RwLock<(Method, JsUri, Version)>>,
    #[qjs(skip_trace)]
    pub inner: Arc<RwLock<(JsHeaders, JsBody)>>,
}

#[rquickjs::methods]
impl JsRequest {
    #[qjs(constructor)]
    fn new(
        method: rquickjs::function::Opt<String>,
        uri: rquickjs::function::Opt<JsUri>,
        headers: rquickjs::function::Opt<JsHeaders>,
        body: rquickjs::function::Opt<JsBody>,
        ctx: Ctx<'_>,
    ) -> Result<Self> {
        let method = hyper::Method::from_bytes(method.0.unwrap_or("get".to_owned()).as_bytes())
            .map_err(|err| to_js_err(err, ctx.clone()))?;
        let uri = uri.0.unwrap_or_default();

        let headers = headers.0.unwrap_or_default();
        let body = body.0.unwrap_or_default();
        Ok(Self {
            is_mut: true,
            parts: Arc::new(RwLock::new((method, uri, Version::HTTP_11))),
            inner: Arc::new(RwLock::new((headers, body))),
        })
    }

    #[qjs(set, rename = "method", enumerable)]
    pub fn set_method(&self, method: String, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        if !self.is_mut {
            return Ok(());
        }

        let method = hyper::Method::from_bytes(method.as_bytes()).map_err(|e| to_js_err(e, ctx))?;
        let mut parts = self.parts.write().unwrap();
        parts.0 = method;
        Ok(())
    }
    #[qjs(get, rename = "method", enumerable, configurable)]
    pub fn get_method(&self) -> String {
        self.parts.read().unwrap().0.to_string()
    }

    #[qjs(set, rename = "version", enumerable)]
    pub fn set_version(&self, version: String, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        if !self.is_mut {
            return Ok(());
        }

        let version = auto_option!(
            version_from_str(&version),
            Err(throw_js_err("invalid http version", ctx))
        );
        let mut parts = self.parts.write().unwrap();
        parts.2 = version;
        Ok(())
    }
    #[qjs(get, rename = "version", enumerable, configurable)]
    pub fn get_version(&self) -> String {
        format!("{:?}", &self.parts.read().unwrap().2)
    }

    #[qjs(set, rename = "uri", enumerable)]
    pub fn set_uri(&self, uri: JsUri, _ctx: Ctx<'_>) -> rquickjs::Result<()> {
        if !self.is_mut {
            return Ok(());
        }
        let mut parts = self.parts.write().unwrap();
        parts.1 = uri;
        Ok(())
    }
    #[qjs(get, rename = "uri", enumerable, configurable)]
    pub fn get_uri(&self) -> JsUri {
        self.parts.read().unwrap().1.clone()
    }

    #[qjs(get, rename = "headers", enumerable, configurable)]
    pub fn get_headers(&self) -> JsHeaders {
        self.inner.read().unwrap().0.clone()
    }
    #[qjs(set, rename = "headers", enumerable, configurable)]
    pub fn set_headers(&self, headers: JsHeaders) {
        let mut guard = self.inner.write().unwrap();
        *(&mut guard.0) = headers;
    }
    #[qjs(get, rename = "body", enumerable, configurable)]
    pub fn get_body<'js>(&self) -> JsBody {
        self.inner.read().unwrap().1.clone()
    }
    #[qjs(set, rename = "body", enumerable, configurable)]
    pub fn set_body<'js>(&self, body: JsBody) {
        let mut guard = self.inner.write().unwrap();
        *(&mut guard.1) = body;
    }

    #[qjs(skip)]
    pub async fn copy_self(&self) -> std::result::Result<JsRequest, hyper::Error> {
        let parts = self.parts.read().unwrap().clone();

        let (headers, bytes) = {
            let mut empty = Body::empty();
            {
                let guard = self.inner.read().unwrap();
                let mut body = guard.1.inner.lock().unwrap();
                std::mem::swap(&mut *body, &mut empty);
            }
            let bytes = hyper::body::to_bytes(&mut empty).await?.to_vec();
            let headers = {
                let guard = self.inner.read().unwrap();
                guard.1.replace(Body::from(bytes.clone())); //恢复body的可读

                let headers = guard.0.copy_self();
                headers
            };

            (headers, bytes)
        };

        Ok(JsRequest {
            is_mut: self.is_mut,
            parts: Arc::new(RwLock::new(parts)),
            inner: Arc::new(RwLock::new((headers, JsBody::bytes(bytes)))),
        })
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("{}", &self)
    }
}

#[rquickjs::class(rename = "Response")]
#[derive(Debug, Trace, Clone)]
pub struct JsResponse {
    pub is_mut: bool,
    #[qjs(skip_trace)]
    pub parts: Arc<RwLock<(StatusCode, Version)>>,
    #[qjs(skip_trace)]
    pub inner: Arc<RwLock<(JsHeaders, JsBody)>>,
}

#[rquickjs::methods]
impl JsResponse {
    #[qjs(constructor)]
    pub fn new<'js>(
        status: rquickjs::function::Opt<u16>,
        headers: rquickjs::function::Opt<JsHeaders>,
        body: rquickjs::function::Opt<JsBody>,
        ctx: Ctx<'js>,
    ) -> Result<Self> {
        let status =
            hyper::StatusCode::from_u16(status.0.unwrap_or(200)).map_err(|e| to_js_err(e, ctx))?;
        let headers = headers.0.unwrap_or_default();
        let body = body.0.unwrap_or_default();
        Ok(Self {
            is_mut: true,
            parts: Arc::new(RwLock::new((status, Version::HTTP_11))),
            inner: Arc::new(RwLock::new((headers, body))),
        })
    }
    #[qjs(set, rename = "status", enumerable, configurable)]
    pub fn set_status(&self, status: u16, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        if !self.is_mut {
            return Ok(());
        }
        let status = hyper::StatusCode::from_u16(status).map_err(|e| to_js_err(e, ctx))?;
        let mut parts = self.parts.write().unwrap();
        parts.0 = status;
        Ok(())
    }
    #[qjs(get, rename = "status", enumerable, configurable)]
    pub fn get_status(&self) -> u16 {
        self.parts.read().unwrap().0.as_u16()
    }
    #[qjs(set, rename = "version", enumerable)]
    pub fn set_version(&self, version: String, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        if !self.is_mut {
            return Ok(());
        }

        let version = auto_option!(
            version_from_str(&version),
            Err(throw_js_err("invalid http version", ctx))
        );
        let mut part = self.parts.write().unwrap();
        part.1 = version;
        Ok(())
    }
    #[qjs(get, rename = "version", enumerable, configurable)]
    pub fn get_version(&self) -> String {
        format!("{:?}", &self.parts.read().unwrap().1)
    }
    #[qjs(get, rename = "headers", enumerable, configurable)]
    pub fn get_headers(&self) -> JsHeaders {
        self.inner.read().unwrap().0.clone()
    }

    #[qjs(set, rename = "headers", enumerable, configurable)]
    pub fn set_headers(&self, headers: JsHeaders) {
        let mut guard = self.inner.write().unwrap();
        *(&mut guard.0) = headers;
    }
    #[qjs(get, rename = "body", enumerable, configurable)]
    pub fn get_body<'js>(&self) -> JsBody {
        self.inner.read().unwrap().1.clone()
    }
    #[qjs(set, rename = "body", enumerable, configurable)]
    pub fn set_body<'js>(&self, body: JsBody) {
        let mut guard = self.inner.write().unwrap();
        *(&mut guard.1) = body;
    }
    #[qjs(skip)]
    pub async fn copy_self(&self) -> std::result::Result<JsResponse, hyper::Error> {
        let parts = self.parts.read().unwrap().clone();
        let (headers, bytes) = {
            let mut empty = Body::empty();
            {
                let guard = self.inner.read().unwrap();
                let mut body = guard.1.inner.lock().unwrap();
                std::mem::swap(&mut *body, &mut empty);
            }
            let bytes = hyper::body::to_bytes(&mut empty).await?.to_vec();
            let headers = {
                let guard = self.inner.read().unwrap();
                guard.1.replace(Body::from(bytes.clone())); //恢复body的可读

                let headers = guard.0.copy_self();
                headers
            };

            (headers, bytes)
        };

        Ok(JsResponse {
            is_mut: self.is_mut,
            parts: Arc::new(RwLock::new(parts)),
            inner: Arc::new(RwLock::new((headers, JsBody::bytes(bytes)))),
        })
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("{}", &self)
    }
}

fn opt_to_proxy_data(
    cfg: rquickjs::function::Opt<rquickjs::Value<'_>>,
) -> rquickjs::Result<ProxyCfg> {
    let cfg = cfg.0;
    let (proxy, ja3, h2) = if cfg.is_none() {
        (None, 0, 0)
    } else {
        let cfg = cfg.unwrap();
        if !cfg.is_object() {
            (None, 0, 0)
        } else {
            let cfg = cfg.into_object().unwrap();
            let proxy = cfg.get::<_, String>("proxy").unwrap_or_default();
            let ja3 = cfg.get::<_, i64>("ja3").unwrap_or(0);
            let h2 = cfg.get::<_, i64>("h2").unwrap_or(0);
            let proxy = if proxy.is_empty() {
                None
            } else {
                let proxy = auto_result!(hyper::Uri::from_str(proxy.as_str()),err=>Err(to_js_err(err,cfg.ctx().clone())));
                Some(proxy)
            };
            (proxy, ja3, h2)
        }
    };
    let proxy_cfg = ProxyCfg { ja3, h2, proxy };
    Ok(proxy_cfg)
}
#[rquickjs::function]
pub async fn fetch<'js>(
    jsreq: JsRequest,
    cfg: rquickjs::function::Opt<rquickjs::Value<'_>>,
    ctx: Ctx<'js>,
) -> Result<JsResponse> {
    let req: hyper::Request<Body> = jsreq.into_hyper().await;
    if req.uri().host().unwrap_or("") == "api.cthulhu.server" {
        return Err(throw_js_err("the host not allow", ctx));
    }
    let proxy_data = opt_to_proxy_data(cfg)?;
    let res = {
        let mut client = create_client(proxy_data);
        let req = reqwest_request_from_hyper(req).await;
        client.call(req).await
    };
    let res = auto_result!(res,err=>Err(to_js_err(err,ctx)));
    let res = reqwest_response_to_hyper(res).await.unwrap();
    Ok(JsResponse::from_hyper(res))
}

#[rquickjs::class(rename = "HttpAction")]
#[derive(Debug, Clone, Trace)]
pub struct JsHttpAction {
    #[qjs(skip_trace)]
    pub action: HttpAction,
}
#[rquickjs::methods]
impl JsHttpAction {
    #[qjs(constructor)]
    pub fn new(ctx: rquickjs::Ctx<'_>) -> rquickjs::Result<Self> {
        Err(throw_js_err("Illegal constructor", ctx))
    }
    #[qjs(static)]
    pub fn reject() -> Self {
        Self {
            action: HttpAction::Reject,
        }
    }
    #[qjs(static)]
    pub fn release(req: JsRequest) -> Self {
        Self {
            action: HttpAction::Release(req),
        }
    }
    #[qjs(static)]
    pub fn respond(res: JsResponse) -> Self {
        Self {
            action: HttpAction::Respond(res),
        }
    }

    #[qjs(static)]
    pub fn proxy(
        req: JsRequest,
        cfg: rquickjs::function::Opt<rquickjs::Value<'_>>,
    ) -> rquickjs::Result<Self> {
        if !cfg.is_none() {
            return Ok(Self::release(req));
        }
        let proxy_data = opt_to_proxy_data(cfg)?;
        if proxy_data.proxy.is_none() && proxy_data.ja3 == 0 && proxy_data.h2 == 0 {
            return Ok(Self::release(req));
        }

        Ok(Self {
            action: HttpAction::Proxy(req, proxy_data),
        })
    }
    #[qjs(get)]
    pub fn name(&self) -> String {
        match &self.action {
            HttpAction::Delay(_, _) => "delay",
            HttpAction::Proxy(_, _) => "proxy",
            HttpAction::Reject => "reject",
            HttpAction::Respond(_) => "respond",
            HttpAction::Release(_) => "release",
        }
        .into()
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("{:?}", &self.action)
    }
}

impl Default for JsHeaders {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for JsRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts = self.parts.read().unwrap();
        let mut d = f.debug_struct("Request");
        d.field("method", &parts.0);
        d.field("url", &parts.1);
        d.field("version", &parts.2);
        let guard = self.inner.read().unwrap();
        d.field("headers", &guard.0);
        d.field("body", &guard.1);
        d.finish()
    }
}
impl fmt::Display for JsResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts = self.parts.read().unwrap();
        let mut d = f.debug_struct("Response");
        d.field("status", &parts.0);
        d.field("version", &parts.1);
        let guard = self.inner.read().unwrap();
        d.field("headers", &guard.0);
        d.field("body", &guard.1);
        d.finish()
    }
}

impl fmt::Display for JsUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Uri");
        d.field("scheme", &self.scheme);
        d.field("authority", &self.authority);
        d.field("host", &self.host);
        d.field("port", &self.port);
        d.field("path", &self.path);
        d.field("params", &self.params);
        d.finish()
    }
}

impl fmt::Display for JsHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let guard = self.inner.read().unwrap();
        (&*guard).fmt(f)
    }
}

impl fmt::Display for JsBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let guard = &self.inner.lock().unwrap();
        (&*guard).fmt(f)
    }
}

impl fmt::Debug for JsUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
impl fmt::Debug for JsHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
impl fmt::Debug for JsBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
#[async_trait]
pub trait Convert<T> {
    async fn into_hyper(self) -> T;
    fn from_hyper(value: T) -> Self;
}
#[async_trait]
impl Convert<hyper::Request<Body>> for JsRequest {
    async fn into_hyper(self) -> hyper::Request<Body> {
        let parts = self.parts.read().unwrap().clone();

        let (headers, body) = {
            let inner = self.inner.read().unwrap();
            let headers = {
                let mut guard = inner.0.inner.write().unwrap();
                let headers = mem::replace(&mut *guard, HeaderMap::new());
                headers
            };
            let body = {
                let mut guard = inner.1.inner.lock().unwrap();
                let body = mem::replace(&mut *guard, Body::empty());
                body
            };
            (headers, body)
        };
        let mut req = hyper::Request::builder()
            .method(parts.0)
            .uri(parts.1.assemble().unwrap())
            .version(parts.2)
            .body(body)
            .unwrap();
        *req.headers_mut() = headers;
        req
    }
    fn from_hyper(value: hyper::Request<Body>) -> Self {
        let (parts, body) = value.into_parts();
        let method = parts.method;

        let uri = parts.uri;
        let version = parts.version;
        let body = JsBody {
            inner: Arc::new(Mutex::new(body)),
        };
        let headers = JsHeaders {
            is_mut: true,
            inner: Arc::new(RwLock::new(parts.headers)),
        };
        JsRequest {
            is_mut: true,
            parts: Arc::new(RwLock::new((method, JsUri::from(uri), version))),
            inner: Arc::new(RwLock::new((headers, body))),
        }
    }
}
#[async_trait]
impl Convert<hyper::Response<Body>> for JsResponse {
    async fn into_hyper(self) -> hyper::Response<Body> {
        let parts = self.parts.read().unwrap().clone();
        let (headers, body) = {
            let inner = self.inner.read().unwrap();
            let headers = {
                let mut guard = inner.0.inner.write().unwrap();
                let headers = mem::replace(&mut *guard, HeaderMap::new());
                headers
            };
            let body = {
                let mut guard = inner.1.inner.lock().unwrap();
                let body = mem::replace(&mut *guard, Body::empty());
                body
            };
            (headers, body)
        };
        let mut res = hyper::Response::builder()
            .status(parts.0)
            .version(parts.1)
            .body(body)
            .unwrap();
        *res.headers_mut() = headers;
        res
    }
    fn from_hyper(value: hyper::Response<Body>) -> Self {
        let (parts, body) = value.into_parts();
        let body = JsBody {
            inner: Arc::new(Mutex::new(body)),
        };
        let headers = JsHeaders {
            is_mut: true,
            inner: Arc::new(RwLock::new(parts.headers)),
        };
        JsResponse {
            is_mut: true,
            parts: Arc::new(RwLock::new((parts.status, parts.version))),
            inner: Arc::new(RwLock::new((headers, body))),
        }
    }
}

pub fn init_def(_id: &str, ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    Class::<'_, JsUri>::define(&globals)?;
    Class::<'_, JsHeaders>::define(&globals)?;
    Class::<'_, JsBody>::define(&globals)?;
    Class::<'_, JsRequest>::define(&globals)?;
    Class::<'_, JsResponse>::define(&globals)?;
    Class::<'_, JsHttpAction>::define(&globals)?;
    globals.set("fetch", Func::new(Async(fetch)))?;
    Ok(())
}

// #[rquickjs::module]
// pub mod http {
//     use rquickjs::{
//         function::{Async, Func},
//         module::Exports,
//         Class, Ctx,
//     };

//     pub use super::*;
//     #[qjs(declare)]
//     pub fn declare(declare: &mut rquickjs::module::Declarations) -> rquickjs::Result<()> {
//         declare.declare("Request")?;
//         declare.declare("Response")?;
//         declare.declare("HttpAction")?;
//         declare.declare("fetch")?;
//         Ok(())
//     }
//     #[qjs(evaluate)]
//     pub fn evaluate<'js>(ctx: &Ctx<'js>, exports: &mut Exports<'js>) -> rquickjs::Result<()> {
//         exports.export(
//             "Request",
//             Class::<'_, super::JsRequest>::create_constructor(ctx)?,
//         )?;
//         exports.export(
//             "Response",
//             Class::<'_, super::JsResponse>::create_constructor(ctx)?,
//         )?;
//         exports.export(
//             "HttpAction",
//             Class::<'_, super::JsHttpAction>::create_constructor(ctx)?,
//         )?;
//         exports.export("fetch", Func::new(Async(super::fetch)))?;

//         let globals = ctx.globals();
//         Class::<'_, super::JsRequest>::define(&globals)?;
//         Class::<'_, super::JsResponse>::define(&globals)?;
//         Class::<'_, super::JsHttpAction>::define(&globals)?;
//         globals.set("fetch", Func::new(Async(super::fetch)))?;
//         Ok(())
//     }
// }
