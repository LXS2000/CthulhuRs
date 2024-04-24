use std::{fs, net::SocketAddr, sync::Arc};

use async_trait::async_trait;

use hyper::{
    client::HttpConnector, header::HeaderValue, service::Service, Body, Method, Request, Response,
    Uri,
};

use lazy_static::lazy_static;
use local_ip_address::local_ip;

use net_proxy::{
    certificate_authority::RcgenAuthority, Answer, CustomProxy, HttpContext, HttpHandler,
    WebSocketHandler,
};

use serde::Deserialize;

use rustls_pemfile as pemfile;

use crate::net_proxy::AddrListenerServer;

mod ja3;
mod net_proxy;
mod proxy;
mod rcgen;
mod utils;

mod macros;

type NetClient = hyper::Client<HttpConnector, Body>;
type AppProxy<'ca> = CustomProxy<RcgenAuthority, Handler, Handler>;

// const TIME_FMT: &str = "%Y-%m-%d %H:%M:%S";
lazy_static! {



    ///sever http客户端
    pub static ref HTTP_CLIENT: NetClient = Default::default();

    pub static ref IS_SYS_PROXY:std::sync::RwLock<bool>=std::sync::RwLock::new(false);

    //CA证书
    pub static ref AUTH:Arc <RcgenAuthority>=Arc::new(read_ca());
    pub static ref CONFIG:Config={
        let bytes = fs::read("./config.json").expect("配置文件读取失败");
        let cfg = serde_json::from_slice::<Config>(&bytes).expect("配置文件格式不正确");
        cfg
    };

}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");

    println!("exit...");
}

#[derive(Debug, Deserialize)]
pub struct Match {
    pub host: String,
    pub ja3: String,
    pub akamai: String,
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
impl HttpHandler for Handler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        mut req: Request<Body>,
    ) -> Answer<Request<Body>, Response<Body>> {
        let uri = req.uri().clone();
        println!("req:{},{}", req.method(), uri.to_string());
        let headers = req.headers_mut();

        headers.append("mitm-uri", HeaderValue::from_str(&uri.to_string()).unwrap());
        let host_ = uri.host().unwrap();
        for Match {
            host,
            ja3,
            akamai,
            proxy,
        } in &CONFIG.matches
        {
            if utils::mini_match(&host, host_) {
                headers.append(
                    "mitm-proxy",
                    HeaderValue::from_str(&proxy.clone().unwrap_or_default()).unwrap(),
                );
                headers.append("mitm-akamai", HeaderValue::from_str(akamai).unwrap());
                headers.append("mitm-ja3", HeaderValue::from_str(ja3).unwrap());
                break;
            }
        }
        if req.method() == Method::CONNECT {
            *req.uri_mut() = Uri::from_static("127.0.0.1:520");
        } else {
            *req.uri_mut() = Uri::from_static("https://127.0.0.1:520/");
        }

        let call = HTTP_CLIENT.clone().call(req).await;
        match call {
            Ok(res) => Answer::Respond(res),
            Err(e) => {
                eprintln!("{:?}", &e);
                let res = Response::builder()
                    .status(500)
                    .body(Body::from(e.to_string()))
                    .unwrap();
                Answer::Respond(res)
            }
        }
    }
}

#[async_trait]
impl WebSocketHandler for Handler {}

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
