use async_trait::async_trait;
use encoding_rs::Encoding;

use hyper::{
    header::{
        CONTENT_SECURITY_POLICY_REPORT_ONLY, HOST, IF_MODIFIED_SINCE, IF_NONE_MATCH,
        IF_UNMODIFIED_SINCE, LOCATION,
    },
    service::Service,
    Body, Version,
};
use lazy_static::lazy_static;

use crate::{
    handle::model::HostList,
    net_proxy::{
        decode_request, decode_response, encode_body, encode_response, Answer, HttpContext,
        HttpHandler, WebSocketContext, WebSocketHandler,
    },
    reqwest_request_from_hyper, reqwest_response_to_hyper, DOC_URL,
};

use html5ever::{namespace_url, ns};

use hyper::http::{
    header::{
        HeaderValue, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
        ACCESS_CONTROL_ALLOW_ORIGIN, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_ENCODING,
        CONTENT_SECURITY_POLICY, CONTENT_TYPE, ORIGIN, REFERER, REFERRER_POLICY, USER_AGENT,
    },
    Extensions, Method, Request, Response, StatusCode, Uri,
};

use hyper_tungstenite::tungstenite::Message;
use kuchiki::{traits::*, Attribute, ExpandedName, NodeRef};

use markup5ever::{local_name, QualName};

use serde::Serialize;
use serde_json::{json, to_value, Value};
use std::{collections::HashSet, net::SocketAddr, path::Path, str::FromStr};
use tokio::{fs::File, sync::RwLock};

use tracing::{error, instrument};

use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{
    auto_option, auto_result,
    handle::{api::handle_api, socket::handle_socket, web::handle_web},
    jsbind::{http::*, server::Scope, ws::*},
    utils, ASYNC_TASK_MANNAGER, CLIENT_MANAGER, HTTP_CLIENT, PLUGIN_MANAGER,
};

use self::{
    api::config,
    net_agent::{on_message, on_request, on_response, watch_request, watch_response},
};

pub mod api;
pub mod model;
pub mod net_agent;
pub mod plugin_web;
pub mod socket;
pub mod web;

pub fn bad_request() -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .expect("Failed to build response")
}
pub fn response_data<T: Serialize>(data: &T, msg: &str) -> Response<Body> {
    let data = auto_result!(to_value(data),err=>{
        error!("响应值JSON序列化失败：{err}");
        return response_msg(500, &err.to_string());
    });

    let json = json!({
        "data":data,
        "msg":msg,
        "code":200,
    });

    response_json(&json)
}

pub fn response_msg<M: Sized + Into<String>>(code: u16, msg: M) -> Response<Body> {
    let msg: String = msg.into();
    let json = json!({
        "msg":msg,
        "code":code,
    });
    response_json(&json)
}

pub fn response_content(status: u16, content: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str("text/plain;charset=utf8").unwrap(),
        )
        .body(Body::from(content.to_string()))
        .unwrap()
}
pub fn response_headers(mut res: Response<Body>) -> Response<Body> {
    let headers = res.headers_mut();
    headers.insert(
        ACCESS_CONTROL_ALLOW_ORIGIN,
        // HeaderValue::from_str("http://localhost:889").unwrap(),
        HeaderValue::from_str("*").unwrap(),
        // HeaderValue::from_str("https://web.cthulhu.server").unwrap(),
    );
    headers.insert(
        ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_str("*").unwrap(),
    );
    headers.insert(
        ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_str(
            // "Origin, X-Requested-With, Content-Type, Accept, Host, Cthulhu-Extra-Scope,Auth-Token",
            "Origin, X-Requested-With, Content-Type, Accept, Host, Cthulhu-Extra-Scope",
        )
        .unwrap(),
    );
    // headers.insert(
    //     ACCESS_CONTROL_EXPOSE_HEADERS,
    //     HeaderValue::from_str("Auth-Token").unwrap(),
    // );
    // headers.insert(
    //     ACCESS_CONTROL_ALLOW_CREDENTIALS,
    //     HeaderValue::from_str("true").unwrap(),
    // );
    res
}
pub fn response_json<T: ?Sized + Serialize>(json: &T) -> Response<Body> {
    let res = Response::builder()
        .status(200)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str("application/json;charset=utf8").unwrap(),
        )
        .body(Body::from(serde_json::to_string(json).unwrap()))
        .unwrap();
    response_headers(res)
}
pub async fn response_download_file<B>(bytes: B, filename: &str) -> Response<Body>
where
    B: Into<Body>,
{
    let body = bytes.into();
    let name = format!(r#"attachment; filename="{}""#, filename);
    Response::builder()
        .status(200)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str("application/octet-stream").unwrap(),
        )
        .header(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(name.as_str()).unwrap(),
        )
        .body(body)
        .unwrap()
}
#[instrument]
pub async fn response_file(file_path: &str) -> Response<Body> {
    let path = file_path.split(&['/', '\\']).filter(|v| !v.is_empty());
    let file_path = path.collect::<Vec<&str>>().join("/");
    let path = Path::new(&file_path);
    let ext = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    let file = auto_result!(File::open(path).await,err=>{
        error!("读取资源失败：{} {file_path}",&err);
           let err =format!("{file_path}: {err}");
           return response_msg(500, &err);
    });
    let mut temp = String::new();
    let content_type = match ext {
        "html" | "htm" | "xml" => "text/html",
        "svg" => "image/svg+xml",
        "txt" => "text/plain",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff" | "heif" => {
            let len = temp.len();
            let ext = "image/".to_owned() + ext;
            temp = temp + &ext;
            &temp[len..len + &ext.len()]
        }
        _ => "",
    };

    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);
    let any = HeaderValue::from_str("*").unwrap();
    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap())
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, any.clone())
        .header(ACCESS_CONTROL_ALLOW_METHODS, any.clone())
        .header(ACCESS_CONTROL_ALLOW_HEADERS, any.clone())
        .header(
            CACHE_CONTROL,
            HeaderValue::from_str("public,max-age=3600").unwrap(),
        )
        .body(body)
        .unwrap()
}
//代理该host，将其数据发送到url处
pub async fn proxy_host(mut req: Request<Body>, url: &str) -> Response<Body> {
    fn switch_host(origin_uri: &Uri, url: &str) -> Uri {
        let uri = Uri::from_str(url).unwrap();
        let path_and_query = format!(
            "{}{}",
            uri.path(),
            origin_uri.path_and_query().unwrap().clone()
        )
        // .replace("\\\\", "\\")
        .replace("//", "/");
        Uri::builder()
            .scheme(uri.scheme().unwrap().clone())
            .authority(uri.authority().unwrap().clone())
            .path_and_query(path_and_query)
            .build()
            .unwrap()
    }
    let origin_uri = req.uri().clone();
    let uri = switch_host(&origin_uri, url);
    *req.uri_mut() = uri;
    *req.version_mut() = Version::HTTP_2;
    let req = reqwest_request_from_hyper(req).await;
    let response = auto_result!(HTTP_CLIENT.clone().call(req).await,err=>{
       return response_content(500, &err.to_string());
    });
    let mut response = reqwest_response_to_hyper(response).await.unwrap();
    if let Some(localtion) = response.headers_mut().get_mut(LOCATION) {
        let val = localtion.to_str().unwrap();
        let url = format!(
            "{}://{}",
            origin_uri.scheme().unwrap(),
            origin_uri.host().unwrap()
        );
        let uri = switch_host(&Uri::from_str(val).unwrap(), &url).to_string();
        *localtion = HeaderValue::from_str(&uri).unwrap();
    }
    response
}

async fn app_filter(host: &str) -> bool {
    fn host_filter(host: &str, hosts: HashSet<&str>, is_white: bool) -> bool {
        for pattern in hosts {
            let matches_pattern = utils::mini_match(pattern, host);
            if is_white && matches_pattern {
                return true;
            }
            if !is_white && matches_pattern {
                return false;
            }
        }
        true
    }
    let filter = |list: Value, is_white: bool| -> Option<bool> {
        let list: HostList = serde_json::from_value(list).unwrap();
        let enabled = list.enabled;
        if !enabled {
            return None;
        }
        let list = list.list;
        let hosts = list.iter().map(|v| v.as_str()).collect();
        Some(host_filter(host, hosts, is_white))
    };
    if let Some(list) = config::get_config("blackList").await {
        if let Some(v) = filter(list, false) {
            return v;
        }
    }
    if let Some(list) = config::get_config("whiteList").await {
        if let Some(v) = filter(list, true) {
            return v;
        }
    };
    true
}

async fn handle_server(ctx: &HttpContext, mut req: Request<Body>) -> Response<Body> {
    //如果是OPTIONS，返回跨域配置
    if req.method() == Method::OPTIONS {
        let res = Response::builder().status(200).body(Body::empty()).unwrap();
        return response_headers(res).into();
    }

    let host = ctx.uri.host().unwrap_or("");
    // debug!("Request:{}", uri);
    let tag = &host[0..host.len() - BRAND.len()];
    let res = match &tag as &str {
        "api." => {
            let req = decode_request(req).unwrap();
            handle_api(ctx, req).await
        }
        "web." => {
            let req = decode_request(req).unwrap();
            handle_web(ctx, req).await
        }

        "socket." => handle_socket(ctx, req).await,
        "doc." => {
            req.headers_mut().remove(HOST); //避免被icp备案识别拦截
            let res = proxy_host(req, &DOC_URL).await;
            return res.into();
        }
        _ => {
            if tag.ends_with(".plugin.") {
                let req = decode_request(req).unwrap();
                return plugin_web::handle_plugin(ctx, req).await;
            }
            Response::builder().status(404).body(Body::empty()).unwrap()
        }
    };
    res
}
async fn handle_worker(
    ctx: &HttpContext,
    mut req: Request<Body>,
    mut scope_key: Scope,
    dest: &str,
) -> Response<Body> {
    if dest == "serviceworker" {
        let mut uri_parser = JsUri::from(req.uri());
        let scope_id = auto_option!(
            uri_parser.params.remove("SCOPE_ID"),
            response_msg(500, "Invalid scope id")
        );
        let guard = CLIENT_MANAGER.scope_keys.read().await;
        scope_key =
            auto_option!(guard.get(&scope_id), response_msg(500, "Invalid scope id")).clone();
        *req.uri_mut() = hyper::Uri::from_str(&uri_parser.assemble().unwrap()).unwrap();
    }
    let path = ctx.uri.path();
    let host = ctx.uri.host().unwrap_or_default();
    let (parts, body) = {
        let response = if path == "/cthulhu.js" {
            Response::builder()
                .status(200)
                .header(
                    CONTENT_TYPE,
                    HeaderValue::from_str("application/javascript; charset=utf-8").unwrap(),
                )
                .body(hyper::Body::empty())
                .unwrap()
        } else {
            //去除缓存控制
            req.headers_mut().remove(IF_MODIFIED_SINCE);
            req.headers_mut().remove(IF_NONE_MATCH);
            req.headers_mut().remove(IF_UNMODIFIED_SINCE);
            let req = reqwest_request_from_hyper(req).await;
            let response = HTTP_CLIENT.clone().call(req).await;

            let response = auto_result!(response,err=>response_msg(500, err.to_string()).into());
            let response = reqwest_response_to_hyper(response).await.unwrap();
            auto_result!( decode_response(response),err=>response_msg(500, err.to_string()).into())
        };
        response.into_parts()
    };
    let bytes = auto_result!(hyper::body::to_bytes(body).await,err=>response_msg(500, err.to_string()).into());
    let worker_init_js = {
        let workspace = auto_option!(config::get_config("workspace").await, {
            return response_msg(500, "请设置CthulhuRs server工作目录").into();
        });
        let workspace = workspace.as_str().unwrap_or("").to_string();
        if workspace.is_empty() {
            return response_msg(500, "请设置CthulhuRs server工作目录").into();
        }
        let worker_js = format!("{workspace}/worker.js");
        let bytes = auto_result!(utils::read_bytes(worker_js),err=>response_msg(500, err.to_string()).into());
        String::from_utf8(bytes).unwrap()
    };
    let scope_id_inject_js = {
        let hash = scope_key.id.clone();
        format!("self['CTHULHU_SCOPE_ID']='{}';\n", hash)
    };
    let (all, _monitors, _modify) = PLUGIN_MANAGER.ctxs_by_host(host).await;
    let (mut sender, body) = Body::channel();

    tokio::spawn(async move {
        auto_result!(sender.send_data(scope_id_inject_js.into()).await, ());
        auto_result!(sender.send_data(worker_init_js.into()).await, ());

        for ctx in all {
            //注入插件动态脚本
            let scripts = ctx.dynamic_scripts(scope_key.clone()).await;
            for mut script in scripts {
                script.push_str(";\n");
                auto_result!(sender.send_data(script.into()).await, ());
            }
            //注入插件worker脚本
            if let Ok(mut worker) = ctx.plugin.read_worker() {
                worker.push_str(";\n");
                auto_result!(sender.send_data(worker.into()).await, ());
            }
        }
        auto_result!(sender.send_data(bytes.into()).await, ());
    });

    Response::from_parts(parts, body)
}

pub fn scope_key_from_request(addr: &SocketAddr, req: &mut Request<Body>) -> Result<Scope, String> {
    let (email, custom, window, tab, frame) = {
        let headers = req.headers_mut();
        if let Some(extra) = headers.remove("cthulhu-extra-scope") {
            let extra = extra.to_str().unwrap_or_default();
            let mut email = "";
            let mut window = 0;
            let mut tab = 0;
            let mut frame = 0;
            let mut custom = "";
            for item in extra.split(";") {
                let (key, value) = item.split_once("=").unwrap_or_default();
                let key = key.trim();
                let value = value.trim();
                match key {
                    "email" => email = value,
                    "custom" => custom = value,
                    "window" => window = value.parse::<i32>().unwrap_or(0),
                    "tab" => tab = value.parse::<i32>().unwrap_or(0),
                    "frame" => frame = value.parse::<i32>().unwrap_or(0),
                    _ => {
                        return Err("value of header 'cthulhu-extra-scope' invalid".to_string());
                    }
                }
            }
            (email.to_string(), custom.to_string(), window, tab, frame)
        } else {
            Default::default()
        }
    };
    let headers = req.headers();
    let ua = headers
        .get(USER_AGENT)
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    let origin = {
        if let Some(origin) = headers.get(ORIGIN) {
            origin.to_str().unwrap()
        } else {
            if let Some(referrer) = headers.get(REFERER) {
                referrer.to_str().unwrap()
            } else {
                //什么都没有 猜测是文档请求 直接使用请求地址本身构建scopekey
                let uri = req.uri();
                let ip = addr.ip().to_string();
                let scheme = uri.scheme_str().unwrap_or("").to_string();
                let host = uri.host().unwrap_or("").to_string();
                let ua = ua.to_string();

                let scope_key = Scope::new(ip, scheme, host, ua, email, custom, window, tab, frame);
                return Ok(scope_key);
            }
        }
    };
    let origin = auto_result!(
        Uri::from_str(origin),
        Err("Origin 或 Referrer uri地址解析错误，代理服务器无法辨识来源".into())
    );
    let ip = addr.ip().to_string();
    let scheme = origin.scheme_str().unwrap_or("").to_string();
    let host = origin.host().unwrap_or("").to_string();
    let ua = ua.to_string();

    let scope_key = Scope::new(ip, scheme, host, ua, email, custom, window, tab, frame);
    Ok(scope_key)
}

pub const BRAND: &str = "cthulhu.server";
lazy_static! {
    static ref LAST_CHECK: RwLock<(u64, bool)> = RwLock::new((0, false));
}

#[derive(Clone, Debug)]
pub struct Handler;

#[async_trait]
impl HttpHandler for Handler {
    // #[instrument(skip(err),parent=None)]
    async fn handle_error(&mut self, ctx: &HttpContext, err: String) -> Response<Body> {
        error!("{err}, {:?}", ctx);
        response_msg(500, err.as_str())
    }

    #[instrument(skip_all,fields(ctx),parent=None)]
    async fn handle_request(
        &mut self,
        ctx: &HttpContext,
        mut req: Request<Body>,
    ) -> Answer<Request<Body>, Response<Body>> {
        let uri = &ctx.uri;
        let host = uri.host().unwrap_or("");

        let _client_addr = ctx.client_addr;
        // println!("{:#?}\n",&req);
        if req.method() == Method::CONNECT {
            let _ = req.headers_mut().remove("cthulhu-extra-scope");
            return req.into();
        }

        if host.ends_with(BRAND) {
            return handle_server(ctx, req).await.into();
        }

        // //通过域名名单过滤
        let allow = app_filter(host).await;
        if !allow {
            let _ = req.headers_mut().remove("cthulhu-extra-scope");
            return req.into();
        }
        // if uri.path() == "/creepjs/creep.js" {
        //     println!("{:#?}", &req)
        // }
        let scope_key = {
            //处理scope
            let scope_key = auto_result!(scope_key_from_request(&ctx.client_addr,&mut req),err=>{
                return response_msg(500, err).into();
            });
            //将scopekey与客户端地址和接口关联起来，方便response找到自己的scopekey
            let mut guard = CLIENT_MANAGER.ctx_map_scope_keys.write().await;
            guard.insert((ctx.client_addr, ctx.uri.clone()), scope_key.clone());
            //方便插件端通过scopeid得到scopekey
            CLIENT_MANAGER.set_scope_key(scope_key.clone()).await;
            scope_key
        };
        let headers = req.headers();
        //dest在谷歌系列浏览器上生效
        let dest = headers
            .get("sec-fetch-dest")
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_string();
        if dest.ends_with("worker") {
            return handle_worker(ctx, req, scope_key, &dest).await.into();
        }
        let extensions = {
            //将extensions保留起来，因为http Request转为JsRequst 这些数据会丢失影响网络连接 比如ws升级
            let mut extensions = Extensions::new();
            std::mem::swap(req.extensions_mut(), &mut extensions);
            extensions
        };

        let js_req = JsRequest::from_hyper(req);
        let action = on_request(&scope_key, js_req).await;

        match action.action {
            HttpAction::Reject => return response_content(500, "<server rejected>").into(),
            HttpAction::Proxy(req, proxy_data) => {
                let mut keys = CLIENT_MANAGER.proxy_datas.write().await;
                keys.insert(scope_key.clone(), proxy_data);
                let new_one = req.clone();
                let join = tokio::task::spawn(async move {
                    watch_request(&scope_key, new_one).await;
                });
                let mut tasks = ASYNC_TASK_MANNAGER.tasks.write().await;
                tasks.push(join);

                let mut req: Request<Body> = req.into_hyper().await;
                *req.extensions_mut() = extensions;
                req.into()
            }
            HttpAction::Delay(req, ms) => {
                let new_one = req.clone();
                let join = tokio::task::spawn(async move {
                    watch_request(&scope_key, new_one).await;
                });
                let mut tasks = ASYNC_TASK_MANNAGER.tasks.write().await;
                tasks.push(join);

                tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
                let mut req: Request<Body> = req.into_hyper().await;
                *req.extensions_mut() = extensions;
                req.into()
            }
            HttpAction::Respond(res) => {
                let new_one = res.clone();
                let join = tokio::task::spawn(async move {
                    watch_response(&scope_key, new_one).await;
                });
                let mut tasks = ASYNC_TASK_MANNAGER.tasks.write().await;
                tasks.push(join);

                let res: Response<Body> = res.into_hyper().await;
                res.into()
            }
            HttpAction::Release(req) => {
                let new_one = req.clone();
                let join = tokio::task::spawn(async move {
                    watch_request(&scope_key, new_one).await;
                });
                let mut tasks = ASYNC_TASK_MANNAGER.tasks.write().await;
                tasks.push(join);
                let mut req: Request<Body> = req.into_hyper().await;
                *req.extensions_mut() = extensions;
                req.into()
            }
        }
    }

    #[instrument(skip_all,fields(ctx),parent=None)]
    async fn handle_response(
        &mut self,
        ctx: &HttpContext,
        mut res: Response<Body>,
    ) -> Response<Body> {
        let host = ctx.uri.host().unwrap_or_default();
        let allow = app_filter(host).await;

        if !allow {
            return res;
        }

        let headers = res.headers();
        let empty = HeaderValue::from_str("").unwrap();
        let encodings = headers
            .get(CONTENT_ENCODING)
            .unwrap_or(&empty)
            .to_str()
            .unwrap()
            .to_string();
        let mut split = headers
            .get(CONTENT_TYPE)
            .unwrap_or(&empty)
            .to_str()
            .unwrap()
            .split(";");
        let ctype = split.next().unwrap_or("");

        let charset = {
            let charset = split.next().unwrap_or("").trim();
            if charset.len() > 8 {
                let charset = &charset[8..].trim();
                if charset.is_empty() {
                    "utf-8"
                } else {
                    charset
                }
            } else {
                "utf-8"
            }
        }
        .to_owned();
        let mut guard = CLIENT_MANAGER.ctx_map_scope_keys.write().await;
        let scope_key = auto_option!(guard.remove(&(ctx.client_addr, ctx.uri.clone())), res);

        // println!("URL:{:?},{}", _ctx.uri, ctype);
        if ctype.starts_with("text/html") {
            let res = decode_response(res).unwrap();
            let (mut parts, mut body) = res.into_parts();
            parts.headers.remove(CONTENT_SECURITY_POLICY_REPORT_ONLY); //刪除 禁止报告
            let (all, _monitors, _modify) = PLUGIN_MANAGER.ctxs_by_host(host).await;

            {
                //修改内容安全策略
                if let Some(val) = parts.headers.get_mut(CONTENT_SECURITY_POLICY) {
                    let csp = val.to_str().unwrap_or("");
                    if csp != "" {
                        let csp = net_agent::content_security_policy(&all, csp).await;
                        *val = HeaderValue::from_str(&csp).unwrap();
                    }
                }
            }
            {
                //修改referrer policy
                let policy = parts
                    .headers
                    .get(REFERRER_POLICY)
                    .map(|v| v.to_str().unwrap_or(""))
                    .unwrap_or("")
                    .trim();
                if policy.is_empty() || policy.starts_with("no-referrer") {
                    parts
                        .headers
                        .insert(REFERRER_POLICY, HeaderValue::from_str("origin").unwrap());
                }
            }

            let encoder = Encoding::for_label(charset.as_bytes()).expect("charset not support");

            let bytes = auto_result!(hyper::body::to_bytes(&mut body).await,err=>{
                error!("{err}");
                return response_content(500, "<proxy server error>");
            });
            let bytes = bytes.to_vec();
            let (decoding, _, err) = encoder.decode(&bytes);
            if err {
                return Response::from_parts(parts, Body::from(bytes));
            }

            let html = {
                let body_str = decoding.into_owned();

                let html_inject = |html: String| -> String {
                    //有的接口响应头返回的是text/html 实际却是其他格式 所以通过<!DOCTYPE html>区分
                  
                    let tag = &html.trim_start()[0..15];
                    if !tag.eq_ignore_ascii_case("<!doctype html>") {
                        return html;
                    }
                    let document = kuchiki::parse_html().one(html);

                    let head = document.select_first("head").unwrap();
                    {
                        //注入插件js 3
                        for ctx in all {
                            let id = &ctx.plugin.id;
                            //注入静态js
                            ctx.plugin
                                .content_paths
                                .split(",")
                                .map(|v| format!("https://{id}.plugin.cthulhu.server/{v}"))
                                .for_each(|url| {
                                    let script = new_script(url.as_str(), &charset);
                                    head.as_node().prepend(script);
                                });
                            //注入动态js
                            ctx.plugin
                                .dynamic_links
                                .split(",")
                                .map(|v| v.trim())
                                .filter(|v| !v.is_empty())
                                .map(|link| {
                                    format!("https://{id}.plugin.cthulhu.server/dynamic/{link}")
                                })
                                .for_each(|url| {
                                    let script = new_script(url.as_str(), &charset);
                                    head.as_node().prepend(script);
                                });
                        }
                    }
                    {
                        //注入全局初始化js 2
                        let script = new_script("https://web.cthulhu.server/content.js", &charset);
                        head.as_node().prepend(script);
                    }

                    {
                        //将分配的scope_key直接注入到html中 1
                        let scope_id = scope_key.id;
                        let set_scope_key_js = format!("self['CTHULHU_SCOPE_ID']='{}'", scope_id);
                        let script = new_text_script(set_scope_key_js.as_str(), &charset);
                        head.as_node().prepend(script);
                    }
                    let html = document.to_string();
                    println!("injected html '{host}'");
                    // println!("head:\n{}",head.as_node().to_string());
                    html
                };
                html_inject(body_str)
            };

            let (decoding, _e, _err) = encoder.encode(&html);

            // if err {
            //     eprintln!("编码失败：{charset}");
            //     return Response::from_parts(parts, Body::from(html));
            // }

            let html = decoding.into_owned();
            if encodings == "" {
                let res = Response::from_parts(parts, Body::from(html));
                return encode_response("gzip", res).expect("compress failed");
            }
            for code in encodings.split(",") {
                let body = auto_result!(encode_body(code, Body::from(html.clone())),err=>{
                    error!("compress respond failed {err}");
                    continue
                });
                parts
                    .headers
                    .append(CONTENT_ENCODING, HeaderValue::from_str(code).unwrap());

                return Response::from_parts(parts, body);
            }
            return response_msg(500, "compress respond failed");
        }
        if ctype.ends_with("application/javascript") {}
        let res = {
            let extensions = {
                //将extensions保留起来，因为http Response转为JsResponse 这些数据会丢失影响网络连接 比如ws升级
                let mut extensions = Extensions::new();
                std::mem::swap(res.extensions_mut(), &mut extensions);
                extensions
            };

            let res = JsResponse::from_hyper(res);
            let js_res = on_response(&scope_key, res.into()).await;
            let new_one = js_res.clone();
            let scope_key = scope_key.clone();

            let join = tokio::task::spawn(async move {
                watch_response(&scope_key, new_one).await;
            });
            let mut tasks = ASYNC_TASK_MANNAGER.tasks.write().await;
            tasks.push(join);

            let mut res: Response<Body> = js_res.into_hyper().await;
            *res.extensions_mut() = extensions;
            res
        };

        res
    }
}
fn new_script(src: &str, charset: &str) -> NodeRef {
    let tag = QualName::new(None, ns!(html), local_name!("script"));
    let script = NodeRef::new_element(
        tag,
        vec![
            (
                ExpandedName::new("", "src"),
                Attribute {
                    prefix: None,
                    value: src.to_string(),
                },
            ),
            (
                ExpandedName::new("", "charset"),
                Attribute {
                    prefix: None,
                    value: charset.to_owned(),
                },
            ),
            (
                ExpandedName::new("", "type"),
                Attribute {
                    prefix: None,
                    value: "text/javascript".to_owned(),
                },
            ),
        ],
    );
    script
}

fn new_text_script(text: &str, charset: &str) -> NodeRef {
    let tag = QualName::new(None, ns!(html), local_name!("script"));
    let node = kuchiki::NodeRef::new_text(text);
    let script = NodeRef::new_element(
        tag,
        vec![
            (
                ExpandedName::new("", "charset"),
                Attribute {
                    prefix: None,
                    value: charset.to_owned(),
                },
            ),
            (
                ExpandedName::new("", "type"),
                Attribute {
                    prefix: None,
                    value: "text/javascript".to_owned(),
                },
            ),
        ],
    );
    script.append(node);
    script
}

#[async_trait]
impl WebSocketHandler for Handler {
    #[instrument(skip_all,fields(ctx),parent=None)]
    async fn handle_message(
        &mut self,
        ctx: &WebSocketContext,
        msg: Message,
    ) -> Answer<Message, Message> {
        let uri = ctx.uri();
        let addr = ctx.addr();
        let host = uri.host().unwrap_or_default();
        let allow = app_filter(host).await;

        if !allow {
            return Answer::Release(msg);
        }

        let jsmsg = JsMessage { msg };
        let client_to_server = ctx.client_to_server();

        let guard = CLIENT_MANAGER.ctx_map_scope_keys.read().await;
        //由于ws是长连接所以不能将scopekey直接取出，只能使用它的引用clone，否则下次消息就无法找到它的scopekey
        let scope_key = auto_option!(guard.get(&(addr, uri)), Answer::Release(jsmsg.msg)).clone();

        let action = on_message(&scope_key, jsmsg, client_to_server).await;
        match action.action {
            WsAction::Ignore => panic!(""),
            WsAction::Delay(msg, ms) => {
                let new_one = msg.clone();
                let join = tokio::task::spawn(async move {
                    net_agent::watch_message(&scope_key, new_one, client_to_server).await;
                });
                let mut tasks = ASYNC_TASK_MANNAGER.tasks.write().await;
                tasks.push(join);

                tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
                Answer::Release(msg.msg)
            }
            WsAction::Respond(msg) => {
                let new_one = msg.clone();
                let join = tokio::task::spawn(async move {
                    net_agent::watch_message(&scope_key, new_one, client_to_server).await;
                });
                let mut tasks = ASYNC_TASK_MANNAGER.tasks.write().await;
                tasks.push(join);

                Answer::Respond(msg.msg)
            }
            WsAction::Release(msg) => {
                let new_one = msg.clone();
                let join = tokio::task::spawn(async move {
                    net_agent::watch_message(&scope_key, new_one, client_to_server).await;
                });
                let mut tasks = ASYNC_TASK_MANNAGER.tasks.write().await;
                tasks.push(join);

                Answer::Release(msg.msg)
            }
        }
    }
}
