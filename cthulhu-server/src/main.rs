use core::{PluginManager, ProxyCfg};
use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use async_once_cell::OnceCell;

use clap::{arg, ArgAction};
use futures::stream::SplitSink;
use handle::{api::config::set_system_proxy, Handler};

use hyper::{upgrade::Upgraded, Body, Uri};

use hyper_tungstenite::tungstenite::Message;
use lazy_static::lazy_static;
use local_ip_address::local_ip;
use markup5ever::tendril::fmt::Slice;

use moka::future::Cache;
use net_proxy::{certificate_authority::RcgenAuthority, CustomProxy};
use rand::{rngs::StdRng, Rng};
use reqwest::redirect;
use time::macros::format_description;

use tokio_tungstenite::WebSocketStream;
use tracing::Level;
use tracing_subscriber::{fmt::time::LocalTime, FmtSubscriber};

use rustls_pemfile as pemfile;
use user_agent_parser::UserAgentParser;

use crate::{
    core::{AsyncTaskMannager, ClientManager, PluginCtx},
    handle::api::{config, plugin},
    net_proxy::AddrListenerServer,
};

mod core;
mod handle;

mod ja3;
mod jsbind;
mod net_proxy;
mod proxy;
mod rcgen;
mod utils;

mod macros;

type NetClient = reqwest::Client;
type AppProxy<'ca, P> = CustomProxy<RcgenAuthority, Handler, Handler, P>;

type Sink = SplitSink<WebSocketStream<Upgraded>, Message>;
// const TIME_FMT: &str = "%Y-%m-%d %H:%M:%S";
lazy_static! {

     ///sqlite数据库
    ///数据库连接池
    pub static ref DBPOOL: sqlx::SqlitePool  ={
        let database_url="./cthulhu.db";
        let pool =
            sqlx::SqlitePool::connect_lazy(database_url).expect("实例化连接池失败");
        pool
    } ;


    ///客户端上下文管理器
    pub static ref CLIENT_MANAGER:ClientManager=ClientManager::default();
    //插件hash 映射 对应的js上下文
    pub static ref PLUGIN_MANAGER: PluginManager =PluginManager::default();
    //异步任务管理器
    pub static ref ASYNC_TASK_MANNAGER: AsyncTaskMannager =AsyncTaskMannager::new();

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
    pub static ref UA_PARSER:UserAgentParser= UserAgentParser::from_str(include_str!("../regexes.yaml")).expect("Parser creation failed");

    pub static ref IS_SYS_PROXY:std::sync::RwLock<bool>=std::sync::RwLock::new(false);

    //CA证书
    pub static ref AUTH:OnceCell< Arc <RcgenAuthority>>=OnceCell::new();
    pub static ref PORT:OnceCell<u16>=OnceCell::new();

    pub static ref DOC_URL:&'static str="https://server.cthulhu.fun";

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

    let mut tasks = vec![];
    {
        let mut tasks_guard = ASYNC_TASK_MANNAGER.tasks.write().await;
        while tasks_guard.len() > 0 {
            let join = tasks_guard.remove(0);
            tasks.push(join);
        }
    }
    futures::future::join_all(tasks).await;
    println!("exit...");
}

fn get_cmd() -> clap::Command {
    clap::Command::new("cthulhu")
        .about("a high performance packet capture proxy server")
        .author("li xiu shun 3451743380@qq.com")
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            clap::Command::new("run").about("run the server").arg(
                arg!(sys: -s "set server to be system proxy")
                    .required(false)
                    .action(ArgAction::SetTrue),
            ),
        )
        .subcommand(
            clap::Command::new("install")
                .about("install plugin from current directory")
                .arg(arg!(<DIR> "target dir for install").required(false)),
        )
        .subcommand(
            clap::Command::new("cagen")
                .about("generate self signed cert with random privkey")
                .arg(
                    arg!(<DIR> "cert file output dir")
                        .required(false)
                        .default_missing_value("./ca/")
                        .default_value("./ca/"),
                ),
        )
        .subcommand(
            clap::Command::new("config")
                .about("operate configuration")
                .subcommand(clap::Command::new("list").about("list the all configuration"))
                .subcommand(
                    clap::Command::new("set")
                        .about("set configuration's value by key")
                        .arg(arg!(<KEY> "set configuration's value by key"))
                        .arg(arg!(<VALUE> ... "set configuration's value by key"))
                        .arg_required_else_help(true),
                )
                .arg_required_else_help(true),
        )
}

fn create_client(key: ProxyCfg) -> NetClient {
    let client_config = ja3::random_ja3(key.ja3 as usize);

    let mut builder = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(60 * 60)) //一小时
        .connect_timeout(Duration::from_secs(60)) //一分钟
        .http2_keep_alive_timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(3 * 60 * 1000)
        .redirect(redirect::Policy::limited(100))
        .use_preconfigured_tls(client_config)
        .danger_accept_invalid_certs(true);
    let port = PORT.get().unwrap();

    if proxy::already_sys_proxy(*port, local_ip().ok()) {
        //避免环回代理
        builder = builder.no_proxy(); //禁用自动添加系统代理
    }
    if key.h2 != 0 {
        let mut random: StdRng = rand::SeedableRng::seed_from_u64(key.h2.abs_diff(0));
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
async fn get_client(addr: SocketAddr, uri: Uri) -> NetClient {
    // let key = {
    //     let mut random = rand::thread_rng();
    //     ProxyData {
    //         proxy: None,
    //         ja3: random.gen_range(1..500),
    //         h2: random.gen_range(1..500),
    //     }
    // };
    // let key = &key;
    //=======
    let guard = CLIENT_MANAGER.ctx_map_scope_keys.read().await;

    let scope_key = auto_option!(guard.get(&(addr, uri)), HTTP_CLIENT.clone());

    let keys = CLIENT_MANAGER.proxy_datas.read().await;

    let key = match keys.get(scope_key) {
        Some(v) => v,
        None => {
            return HTTP_CLIENT.clone();
        }
    };
    let clients = &CLITENT_POOL;

    let client = if let Some(client) = clients.get(key).await {
        client.clone()
    } else {
        let client = create_client(key.clone()).into();
        clients.insert(key.clone(), client).await;
        let client = clients.get(key).await.unwrap();
        client.clone()
    };
    client
}

async fn run_server() {
    let port = PORT
        .get_or_init(async {
            let port = config::get_config("port")
                .await
                .map(|v| v.as_i64().unwrap_or(3000) as u16)
                .unwrap_or(3000);
            port
        })
        .await;
    //初始化ca证书
    let _ = AUTH
        .get_or_init(async {
            let certificate = config::get_config("certificate")
                .await
                .expect("读取ca证书路径配置失败!");
            let certificate = certificate.as_object().expect("ca证书路径配置异常!");

            let cert = certificate
                .get("cert")
                .expect("请设置ca证书cert路径!")
                .as_str()
                .unwrap();
            let key = certificate
                .get("key")
                .expect("请设置ca证书密钥key路径!")
                .as_str()
                .unwrap();
            println!("CA_KEY: {key}");
            println!("CA_CERT: {cert}");
            let key = utils::read_bytes(key).expect("读取密钥文件失败!");
            let cert = utils::read_bytes(cert).expect("读取证书文件失败!");

            let mut private_key_bytes: &[u8] = key.as_bytes();
            let mut ca_cert_bytes: &[u8] = cert.as_bytes();

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

            Arc::new(
                RcgenAuthority::new(private_key.into(), ca_cert, 1_000)
                    .expect("Failed to create Certificate Authority"),
            )
        })
        .await;
    {
        //加载插件
        let plugins = plugin::get_enabled_plugins().await;
        for plugin in plugins {
            let ctx = PluginCtx::new(plugin).await.unwrap();
            PLUGIN_MANAGER.set_ctx(ctx).await;
        }
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], *port));

    let rustls = tokio_tungstenite::Connector::Rustls(Arc::new(ja3::random_ja3(0)));
    let proxy = AppProxy::new(
        AUTH.get().unwrap().clone(),
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

async fn list_configs() {
    let configs = auto_result!(config::get_configs().await,err=>{
         println!("系统异常:{err}");
         return ;
    });
    println!("CONFIGS:");
    for config in configs {
        let key = config.get("key").unwrap().as_str().unwrap_or("");
        let label = config.get("label").unwrap().as_str().unwrap_or("");
        let ty = config.get("type").unwrap().as_str().unwrap_or("");
        match ty {
            "str" | "num" | "bool" | "list" => {
                let value = config.get("value").unwrap().to_string();
                println!("\t{key}: {value}\t---{label}")
            }

            "obj" => {
                let temp = vec![];
                let child = config.get("value").unwrap().as_array().unwrap_or(&temp);
                for config in child {
                    let sub_key = config.get("key").unwrap().as_str().unwrap_or("");
                    let sub_label = config.get("label").unwrap().as_str().unwrap_or("");
                    let value = config.get("value").unwrap().to_string();
                    println!("\t{key}.{sub_key}: {value}\t---{label}.{sub_label}")
                }
            }
            _ => {}
        }
    }
}
async fn set_config(full_key: &str, mut values: Vec<String>) {
    let (key, sub_key) = full_key.split_once(".").unwrap_or((full_key, ""));
    let config = auto_option!(config::get_config_by_key(key).await, {
        println!("invalid key");
        return;
    });
    let mut id = config.id;
    let mut ty = config.r#type;
    if !sub_key.is_empty() {
        let sub_config = auto_option!(
            config::get_config_by_key_and_parent_id(sub_key, id).await,
            {
                println!("invalid key");
                return;
            }
        );
        ty = sub_config.r#type;
        id = sub_config.id;
    }
    let value = if ty == "list" {
        values.join("&&")
    } else {
        values.remove(0)
    };
    auto_result!( config::update_config_by_id(id, &value).await,err=>{
        println!("update configration faild:{err}");
        return;
    });
    println!("set config by key '{full_key}' success")
}
#[tokio::main]
async fn main() {
    //初始化环境变量
    // dotenv::dotenv().ok();

    //初始化命令行工具
    let cmd = get_cmd();

    let matches = cmd.get_matches();
    let subcmd = matches.subcommand();
    match subcmd {
        Some(("run", subcmd)) => {
            //注册日志
            let timer = LocalTime::new(format_description!(
                "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second]"
            ));
            let subscriber = FmtSubscriber::builder()
                .with_max_level(Level::ERROR) // 设置最大日志级别为INFO
                .with_timer(timer)
                .pretty()
                .with_ansi(false)
                .finish();

            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");

            //注册系统代理
            if subcmd.get_flag("sys") {
                set_system_proxy(true).await.unwrap();
                println!("设置系统代理成功");
            }
            run_server().await
        }
        Some(("cagen", subcmd)) => {
            let dir = subcmd.get_one::<String>("DIR").unwrap();
            rcgen::ca_gen(dir);
        }
        Some(("install", subcmd)) => {
            let dir = {
                let current_dir = std::env::current_dir().unwrap();
                let dir = subcmd
                    .get_one::<String>("DIR")
                    // .map(|v| std::path::Path::new(v))
                    .map(|dir| {
                        if dir.starts_with(".") {
                            return relative_path::RelativePath::new(dir)
                                .to_logical_path(&current_dir);
                        }
                        relative_path::RelativePath::new(dir).to_path("")
                    })
                    .unwrap_or(current_dir);
                dir
            };
            plugin::install(dir.to_str().unwrap()).await;
        }
        Some(("config", subcmd)) => match subcmd.subcommand() {
            Some(("list", _)) => list_configs().await,
            Some(("set", subcmd)) => {
                let key = auto_option!(subcmd.get_one::<String>("KEY"),v=>v.is_empty(),{
                    println!("'key' is necessary");
                    return;
                });
                let value = auto_option!(subcmd.get_many::<String>("VALUE"), {
                    println!("'value' is necessary");
                    return;
                });
                let values = value.map(|v| v.clone()).collect::<Vec<String>>();
                if values.is_empty() {
                    println!("'value' is necessary");
                    return;
                }
                set_config(key, values).await
            }
            Some((&_, _)) => println!("unknown option"),
            None => {
                println!("unknown option")
            }
        },
        Some((c, _)) => println!("unknown option {c}"),
        None => {
            println!("unknown option")
        }
    }
}
