use std::{
    collections::HashMap,
    process::{exit, Command},
    time::Duration,
};

use crate::{jsbind::server::Scope, net_proxy::HttpContext};
use hyper::http::{Request, Response};
use hyper::Body;
use local_ip_address::local_ip;

use serde_json::json;
use tracing::error;

use crate::{
    auto_result,
    handle::{response_data, response_download_file, response_msg},
    wrap, AUTH, CLIENT_MANAGER,
};

use super::config;

async fn download_ca(_ctx: HttpContext, _req: Request<Body>) -> Response<Body> {
    let auth = AUTH.get().unwrap().clone();
    let ca_cert = &auth.ca_cert;
    response_download_file(ca_cert.0.clone(), "cthulhu.cer").await
}
async fn restart(_ctx: HttpContext, _req: Request<Body>) -> Response<Body> {
    let path = std::env::current_exe().expect("Failed to get current exe");
    let path = path.to_str().unwrap();
    println!("application: {path}");
    Command::new(format!("{path} run"))
        .spawn()
        .expect("Failed to start new instance of application");

    // Exit the current process.
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;

        exit(0);
    });

    return response_msg(200, "正在重启中...");
}
async fn server_info(_ctx: HttpContext, _req: Request<Body>) -> Response<Body> {
    let local_ip = auto_result!(local_ip(),err=>{
        error!("获取本机内网地址失败：{}",err);
        return response_msg(500, "获取本机内网地址失败");
    });
    let port = config::get_config("port")
        .await
        .map(|v| v.as_i64().unwrap_or(3000))
        .unwrap_or(3000);

    let guard = CLIENT_MANAGER.scope_keys.read().await;
    let scopes = guard.values().collect::<Vec<&Scope>>();
    let json = json!({
        "addr":local_ip,
        "port":port,
        "scopes":scopes,
    });
    response_data(&json, "")
}

pub fn route(router: &mut HashMap<&'static str, Box<super::AsyncFn>>) {
    // router.insert("/server/ask", wrap!(ask));
    router.insert("/server/serverInfo", wrap!(server_info));
    router.insert("/server/restart", wrap!(restart));
    router.insert("/server/downloadCa", wrap!(download_ca));
}
