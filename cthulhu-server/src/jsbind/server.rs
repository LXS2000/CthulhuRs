use std::collections::HashMap;

use futures::future::Either;
use futures::SinkExt;

use rquickjs::function::IntoArgs;
use rquickjs::{class::Trace, Class, Ctx};
use rquickjs::{FromJs, IntoJs};

use serde::Serialize;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

use crate::jsbind::throw_js_err;
use crate::{auto_option, utils};

use crate::CLIENT_MANAGER;

use crate::UA_PARSER;

use super::{http::*, to_js_err};

use super::ws::*;

#[rquickjs::class(rename = "UAParser")]
#[derive(Debug, Trace, Clone)]
pub struct UAParser {
    #[qjs(get, enumerable, configurable)]
    ua: String,
    #[qjs(get, enumerable, configurable)]
    product: HashMap<String, String>,
    #[qjs(get, enumerable, configurable)]
    os: HashMap<String, String>,
    #[qjs(get, enumerable, configurable)]
    device: HashMap<String, String>,
    #[qjs(get, enumerable, configurable)]
    cpu: HashMap<String, String>,
    #[qjs(get, enumerable, configurable)]
    engine: HashMap<String, String>,
}

#[rquickjs::methods]
impl UAParser {
    #[qjs(constructor)]
    pub fn new(ua: String) -> Self {
        let cpu = UA_PARSER.parse_cpu(&ua);
        let device = UA_PARSER.parse_device(&ua);
        let engine = UA_PARSER.parse_engine(&ua);
        let os = UA_PARSER.parse_os(&ua);
        let product = UA_PARSER.parse_product(&ua);

        let mut cpu_map = HashMap::new();
        cpu_map.insert(
            "arch".to_string(),
            cpu.architecture.map(|v| v.to_string()).unwrap_or_default(),
        );

        let mut device_map = HashMap::new();
        device_map.insert(
            "brand".to_string(),
            device.brand.map(|v| v.to_string()).unwrap_or_default(),
        );
        device_map.insert(
            "model".to_string(),
            device.model.map(|v| v.to_string()).unwrap_or_default(),
        );
        device_map.insert(
            "name".to_string(),
            device.name.map(|v| v.to_string()).unwrap_or_default(),
        );

        let mut engine_map = HashMap::new();
        engine_map.insert(
            "major".to_string(),
            engine.major.map(|v| v.to_string()).unwrap_or_default(),
        );
        engine_map.insert(
            "minor".to_string(),
            engine.minor.map(|v| v.to_string()).unwrap_or_default(),
        );
        engine_map.insert(
            "name".to_string(),
            engine.name.map(|v| v.to_string()).unwrap_or_default(),
        );
        engine_map.insert(
            "patch".to_string(),
            engine.patch.map(|v| v.to_string()).unwrap_or_default(),
        );

        let mut os_map = HashMap::new();
        os_map.insert(
            "major".to_string(),
            os.major.map(|v| v.to_string()).unwrap_or_default(),
        );
        os_map.insert(
            "minor".to_string(),
            os.minor.map(|v| v.to_string()).unwrap_or_default(),
        );
        os_map.insert(
            "name".to_string(),
            os.name.map(|v| v.to_string()).unwrap_or_default(),
        );
        os_map.insert(
            "patch".to_string(),
            os.patch.map(|v| v.to_string()).unwrap_or_default(),
        );
        os_map.insert(
            "patchMinor".to_string(),
            os.patch_minor.map(|v| v.to_string()).unwrap_or_default(),
        );

        let mut product_map = HashMap::new();
        product_map.insert(
            "major".to_string(),
            product.major.map(|v| v.to_string()).unwrap_or_default(),
        );
        product_map.insert(
            "minor".to_string(),
            product.minor.map(|v| v.to_string()).unwrap_or_default(),
        );
        product_map.insert(
            "name".to_string(),
            product.name.map(|v| v.to_string()).unwrap_or_default(),
        );
        product_map.insert(
            "patch".to_string(),
            product.patch.map(|v| v.to_string()).unwrap_or_default(),
        );
        Self {
            ua,
            device: device_map,
            os: os_map,
            engine: engine_map,
            cpu: cpu_map,
            product: product_map,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Trace, Serialize)]
#[rquickjs::class(rename = "Scope")]
pub struct Scope {
    #[qjs(get, enumerable, configurable)]
    pub id: String,
    #[qjs(get, enumerable, configurable)]
    pub ip: String,
    #[qjs(get, enumerable, configurable)]
    pub scheme: String,
    #[qjs(get, enumerable, configurable)]
    pub host: String, //https?://xxx.com
    #[qjs(get, enumerable, configurable)]
    pub ua: String,
    #[qjs(get, enumerable, configurable)]
    pub email: String,
    #[qjs(get, enumerable, configurable)]
    pub custom: String,
    #[qjs(get, enumerable, configurable)]
    pub window: i32,
    #[qjs(get, enumerable, configurable)]
    pub tab: i32,
    #[qjs(get, enumerable, configurable)]
    pub frame: i32,
}
#[rquickjs::methods]
impl Scope {
    #[qjs(skip)]
    pub fn new(
        ip: String,
        scheme: String,
        host: String,
        ua: String,
        email: String,
        custom: String,
        window: i32,
        tab: i32,
        frame: i32,
    ) -> Self {
        let mut scope = Self {
            ip,
            scheme,
            host,
            ua,
            email,
            custom,
            window,
            tab,
            frame,
            id: String::new(),
        };
        scope.id = scope.hash();
        scope
    }
    fn hash(&self) -> String {
        let s = format!(
            "{}.{}.{}.{}.{}.{}.{}.{}.{}",
            &self.scheme,
            &self.host,
            &self.ua,
            &self.ip,
            &self.email,
            &self.custom,
            &self.window,
            &self.tab,
            &self.frame
        );
        utils::hash(s.as_bytes(), 16, 12)
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("{:?}", &self)
    }
}
#[rquickjs::class(rename = "Server")]
#[derive(Debug, Trace, Clone)]
pub struct Server {
    id: String,
}
#[rquickjs::methods]
impl Server {
    //监听权限
    #[qjs(rename = "watchRequest")]
    pub async fn watch_request(&self, _req: JsRequest, _scope: Scope) -> rquickjs::Result<()> {
        Ok(())
    }

    #[qjs(rename = "watchResponse")]
    pub async fn watch_response(&self, _res: JsResponse, _scope: Scope) -> rquickjs::Result<()> {
        Ok(())
    }
    #[qjs(rename = "watchMessage")]
    pub async fn watch_message(&self, _msg: JsMessage, _scope: Scope) -> rquickjs::Result<()> {
        Ok(())
    }
    //修改权限
    #[qjs(rename = "onRequest")]
    pub async fn on_request(
        &self,
        req: JsRequest,
        _scope: Scope,
    ) -> rquickjs::Result<JsHttpAction> {
        Ok(JsHttpAction::release(req))
    }

    #[qjs(rename = "onResponse")]
    pub async fn on_response(
        &self,
        res: JsResponse,
        _scope: Scope,
    ) -> rquickjs::Result<JsResponse> {
        Ok(res)
    }
    #[qjs(rename = "onMessage")]
    pub async fn on_message(
        &self,
        msg: JsMessage,
        _scope: Scope,
    ) -> rquickjs::Result<JsWsAction> {
        Ok(JsWsAction::release(msg))
    }
    //用户事件通知
    #[qjs(rename = "onClientOpen")]
    pub async fn on_client_open(
        &self,
        _session_type: String,
        _session_id: String,
        _scope: Scope,
    ) -> rquickjs::Result<()> {
        Ok(())
    }
    #[qjs(rename = "onClientClose")]
    pub async fn on_client_close(
        &self,
        _session_type: String,
        _session_id: String,
        _scope: Scope,
    ) -> rquickjs::Result<()> {
        Ok(())
    }
    #[qjs(rename = "onAsk")]
    pub async fn on_ask<'js>(
        &self,
        _key: String,
        _val: rquickjs::Value<'js>,
        _scope: Scope,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<rquickjs::Value<'js>> {
        Ok(rquickjs::Value::new_undefined(ctx))
    }
    #[qjs(rename = "dynamicScript")]
    pub async fn dynamic_script<'js>(
        &self,
        _link: String,
        _scope: Scope,
        _ctx: Ctx<'js>,
    ) -> rquickjs::Result<String> {
        Ok("//dynamic_script".to_string())
    }
    #[qjs(rename = "sendEvent")]
    pub async fn send_event<'js>(
        &self,
        session_id: String,
        event_type: String,
        event_body: std::collections::HashMap<String, rquickjs::Value<'js>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<()> {
        let event_body = event_body
            .into_iter()
            .map(|(k, v)| {
                let ctx = v.ctx();
                let json = ctx.json_stringify(&v).unwrap().unwrap();
                let json = json.to_string().unwrap();
                let body: serde_json::Value = serde_json::from_str(json.as_str()).unwrap();
                (k, body)
            })
            .collect::<serde_json::Map<String, serde_json::Value>>();

        let obj = json!({
            "type":"event",
            "eventType":event_type,
            "eventBody":event_body,
        });
        let json = obj.to_string();
        let msg = Message::Text(json);

        let sinks = CLIENT_MANAGER.sinks.read().await;
        let sink = auto_option!(
            sinks.get(&session_id),
            Err(throw_js_err("invalid sessionId", ctx))
        );
        let mut sink = sink.lock().await;
        let _ = sink
            .send(msg.clone())
            .await
            .map_err(|e| to_js_err(e, ctx))?;
        Ok(())
    }
    #[qjs(rename = "sendScript")]
    pub async fn send_script<'js>(
        &self,
        session_id: String,
        script: String,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<()> {
        let obj = json!({
            "type":"script",
            "script":script,
        });
        let json = obj.to_string();
        let msg = Message::Text(json);

        let sinks = CLIENT_MANAGER.sinks.read().await;
        let sink = auto_option!(
            sinks.get(&session_id),
            Err(throw_js_err("invalid sessionId", ctx))
        );
        let mut sink = sink.lock().await;
        let _ = sink
            .send(msg.clone())
            .await
            .map_err(|e| to_js_err(e, ctx))?;
        Ok(())
    }
}

pub async fn call_function<'js, T: FromJs<'js> + 'js, A: IntoArgs<'js>>(
    ctx: &Ctx<'js>,
    name: &str,
    args: A,
) -> rquickjs::Result<Either<T, A>> {
    let globals = ctx.globals();
    let server = globals.get::<_, rquickjs::Object<'js>>("server")?;

    let value = server.get::<_, rquickjs::Value<'js>>(name)?;
    if !value.is_function() {
        return Ok(Either::Right(args));
    }
    let function = value.into_function().unwrap();

    let mut js_args = rquickjs::function::Args::new_unsized(ctx.clone());
    args.into_args(&mut js_args)?;
    js_args.this(server)?;

    let promised = rquickjs::promise::Promised::from(async move {
        function.call_arg::<rquickjs::Value<'js>>(js_args)
    });
    let promised = promised.into_js(ctx)?;
    let promise = rquickjs::promise::Promise::<'js, T>::from_js(ctx, promised)?;
    let promise = promise.await?;
    return Ok(Either::Left(promise));
}

pub fn init_def(id: &str, ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let server = Server { id: id.to_string() };
    let globals = ctx.globals();
    Class::<Scope>::define(&globals)?;
    Class::<UAParser>::define(&globals)?;

    let cls = Class::instance(ctx.clone(), server)?;
    globals.set("server", cls)?;

    Ok(())
}
