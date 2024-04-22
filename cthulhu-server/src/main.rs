use std::{collections::HashMap, fs, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use async_trait::async_trait;

use hyper::{Body, Uri};

use lazy_static::lazy_static;
use local_ip_address::local_ip;

use moka::future::Cache;
use net_proxy::{
    certificate_authority::RcgenAuthority, CustomProxy, HttpHandler, WebSocketHandler,
};
use rand::{rngs::StdRng, Rng};
use reqwest::redirect;
use serde::Deserialize;

use rustls_pemfile as pemfile;

use crate::net_proxy::AddrListenerServer;

mod core;

mod ja3;
mod net_proxy;
mod proxy;
mod rcgen;
mod utils;

mod macros;

type NetClient = reqwest::Client;
type AppProxy<'ca, P> = CustomProxy<RcgenAuthority, Handler, Handler, P>;

// const TIME_FMT: &str = "%Y-%m-%d %H:%M:%S";
lazy_static! {


    ///客户端池
    pub static ref CLITENT_POOL: Cache<ProxyCfg,NetClient> =Cache::builder()
    .max_capacity(10000)
    .time_to_live(std::time::Duration::from_secs(60*10))
    .build();

    ///sever http客户端
    pub static ref HTTP_CLIENT: NetClient = {
       let client= create_client(ProxyCfg::default());
        client
    };

    pub static ref IS_SYS_PROXY:std::sync::RwLock<bool>=std::sync::RwLock::new(false);

    //CA证书
    pub static ref AUTH:Arc <RcgenAuthority>=Arc::new(read_ca());
    pub static ref CONFIG:Config={
        let bytes = fs::read("./config.json").expect("配置文件读取失败");
        let cfg = serde_json::from_slice::<Config>(&bytes).expect("配置文件格式不正确");
        cfg
    };


    pub static ref HOST_CFG:HashMap<String,ProxyCfg>={
        let mut map=HashMap::new();
        for Match { host, ja3, h2, proxy } in &CONFIG.matches {
           let proxy= match proxy {
                Some(value) => {
                    if value.is_empty() {
                        None
                    }else{
                        Some(Uri::from_str(value).expect("代理地址格式错误"))
                    }

                },
                None => {None},
            };

            map.insert(host.to_string(), ProxyCfg{ja3:*ja3,h2:*h2,proxy});
        }
        map
    };



}

pub async fn reqwest_response_to_hyper(
    res: reqwest::Response,
) -> Result<hyper::Response<Body>, Box<dyn std::error::Error>> {
    let status = res.status();
    let version = res.version();
    let headers = res.headers().clone();

    let bytes = res.bytes().await?;
    let mut response = hyper::Response::builder()
        .version(version)
        .status(status)
        .body(Body::from(bytes))?;
    *response.headers_mut() = headers;
    Ok(response)
}

pub async fn reqwest_request_from_hyper(req: hyper::Request<Body>) -> reqwest::Request {
    let (parts, body) = req.into_parts();
    let mut request = reqwest::Request::new(
        parts.method,
        reqwest::Url::from_str(&parts.uri.to_string()).unwrap(),
    );

    *request.headers_mut() = parts.headers;
    *request.version_mut() = parts.version;
    let bytes = hyper::body::to_bytes(body).await.unwrap();
    *request.body_mut() = Some(reqwest::Body::from(bytes));
    request
}
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");

    println!("exit...");
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct ProxyCfg {
    pub ja3: i32,
    pub h2: i32,
    pub proxy: Option<Uri>,
}

#[derive(Debug, Deserialize)]
pub struct Match {
    pub host: String,
    pub ja3: i32,
    pub h2: i32,
    pub proxy: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct Config {
    pub port: u16,
    pub matches: Vec<Match>,
}
#[derive(Debug, Clone)]
struct Handler;
#[async_trait]
impl HttpHandler for Handler {}

#[async_trait]
impl WebSocketHandler for Handler {}

fn create_client(key: ProxyCfg) -> NetClient {
    let client_config = ja3::random_ja3(key.ja3 as usize);

    let mut builder = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(60 * 60)) //一小时
        .connect_timeout(Duration::from_secs(60)) //一分钟
        .http2_keep_alive_timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(3 * 60 * 1000)
        .redirect(redirect::Policy::limited(100))
        .use_preconfigured_tls(client_config);
    // .danger_accept_invalid_certs(true);
    let port = CONFIG.port;

    if proxy::already_sys_proxy(port, local_ip().ok()) {
        //避免环回代理
        builder = builder.no_proxy(); //禁用自动添加系统代理
    }
    if key.h2 != 0 {
        let mut random: StdRng = rand::SeedableRng::seed_from_u64(key.h2 as u64);
        let base = 1024 * 1024; //1mb
        builder = builder
            .http2_max_frame_size(random.gen_range(16_384..((1 << 24) - 1)) / 1024 * 1024)
            // .http2_max_send_buf_size(random.gen_range(2..20) * base)
            .http2_initial_connection_window_size(random.gen_range(2..20) * base as u32)
            .http2_initial_stream_window_size(random.gen_range(2..20) * base as u32);
    }
    if let Some(uri) = key.proxy {
        let proxy = reqwest::Proxy::all(&uri.to_string()).unwrap();
        builder = builder.proxy(proxy);
    }
    builder.build().unwrap()
}
async fn get_client(_addr: SocketAddr, uri: Uri) -> NetClient {
    let host = uri.host().unwrap();
    for (host_p, cfg) in HOST_CFG.iter() {
        if utils::mini_match(host_p, host) {
            return create_client(cfg.clone());
        }
    }
    HTTP_CLIENT.clone()
}

fn read_ca() -> RcgenAuthority {
    let key = utils::read_bytes("./ca/ca.key").expect("读取密钥文件失败!");
    let cert = utils::read_bytes("./ca/ca.cer").expect("读取证书文件失败!");

    let mut private_key_bytes: &[u8] = &key;
    let mut ca_cert_bytes: &[u8] = &cert;

    let private_key = rustls::PrivateKey(
        pemfile::pkcs8_private_keys(&mut private_key_bytes)
            .expect("Failed to parse private key")
            .remove(0),
    );

    let ca_cert = rustls::Certificate(
        pemfile::certs(&mut ca_cert_bytes)
            .expect("Failed to parse CA certificate")
            .remove(0),
    );

    let auth = RcgenAuthority::new(private_key.into(), ca_cert, 1_000)
        .expect("Failed to create Certificate Authority");
    auth
}
async fn run_server() {
    //初始化ca证书
    let port = CONFIG.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let rustls = tokio_tungstenite::Connector::Rustls(Arc::new(ja3::random_ja3(0)));
    let proxy = AppProxy::new(
        AUTH.clone(),
        AddrListenerServer::Addr(addr),
        Handler,
        Handler,
        Some(rustls),
        get_client,
    );
    let local_ip = auto_result!(local_ip(),err=>{
        panic!("获取本机内网地址失败：{}",err);
    });
    println!("server at port {local_ip}:{port}");
    if let Err(e) = proxy.start(shutdown_signal()).await {
        panic!("{}", e);
    }
}

#[tokio::main]
async fn main() {
    run_server().await
}
