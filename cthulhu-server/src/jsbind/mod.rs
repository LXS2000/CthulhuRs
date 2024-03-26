use std::error::Error;

use futures::future::Either;
use relative_path::RelativePath;
use rquickjs::{
    async_with, function::This, loader::{BuiltinLoader, BuiltinResolver, ModuleLoader, NativeLoader, Resolver, ScriptLoader}, AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Function, Symbol
};
use serde_json::json;

use sled::Db;
use tracing::instrument;

use crate::{auto_result, core::PluginCtx};
use crate::handle::model::Plugin;

use self::server::Scope;

//全局对象
pub mod console;
pub mod server;
pub mod store;

//模块
pub mod file;
pub mod http;
pub mod timer;
pub mod utils;
pub mod ws;

struct PluginResolver {
    pub base_path: String,
}
impl Resolver for PluginResolver {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        _base: &str,
        name: &str,
    ) -> rquickjs::Result<String> {
        let name = if name.ends_with(".js") {
            name.to_string()
        } else {
            format!("{name}.js")
        };
        let path = RelativePath::new(name.as_str()).to_logical_path(&self.base_path);
        if path.is_file() {
            Ok(path.to_str().unwrap().to_string())
        } else {
            Err(rquickjs::Error::new_resolving(&self.base_path, name))
        }
    }
}

pub async fn content(plugin: &Plugin) -> Result<(AsyncContext, AsyncRuntime, Db), String> {
    let path = plugin.path.clone();
    let rt = {
        let rt = AsyncRuntime::new().unwrap();
        let rs = BuiltinResolver::default();
        let ld = ModuleLoader::default();

        // rs.add_module("http")
        //     .add_module("ws")
        //     .add_module("utils")
        //     .add_module("timer")
        //     .add_module("file");
        // ld.add_module("http", js_http)
        //     .add_module("ws", js_ws)
        //     .add_module("utils", js_utils)
        //     .add_module("timer", js_timer)
        //     .add_module("file", js_file);
        let rs = (
            rs,
            PluginResolver {
                base_path: path.clone(),
            },
        );
        let ld = (
            BuiltinLoader::default(),
            NativeLoader::default(),
            ScriptLoader::default(),
            ld,
        );
        rt.set_loader(rs, ld).await;
        rt
    };
    let redb = {
        let db_path = format!("{}/STORE", &plugin.path);
        sled::open(db_path).map_err(|v| v.to_string())?
    };
    let full = AsyncContext::full(&rt).await.map_err(|v| v.to_string())?;

    let id = &plugin.id;

    let db = redb.clone();

    let res = async_with!(full=>|ctx|{
        let init=|ctx|{
            console::init_def(path.clone(), &ctx)?;
            store::init_def(id, &ctx,db)?;
            server::init_def(id, &ctx)?;
            http::init_def(id, &ctx)?;
            ws::init_def(id, &ctx)?;
            utils::init_def(id, &ctx)?;
            timer::init_def(id, &ctx)?;
            file::init_def(id, &ctx)?;
            let globals=ctx.globals();
            globals.set::<_,_>("server_dir", path)?;
            globals.set::<_,_>("global", globals.clone())?;
            if let Ok(code) = plugin.read_server() {
                let _=ctx.clone().compile("server", code)?;
            }
            Ok(())
        };
        auto_result!(init(ctx.clone()).catch(&ctx),err=>{
            return Err(handle_js_error(err, &ctx))
        });
        Ok(())
    })
    .await;
    res.unwrap();
    Ok((full, rt, redb))
}

pub fn js_to_json(js: rquickjs::Value<'_>) -> rquickjs::Result<serde_json::Value> {
    match js.type_of() {
        rquickjs::Type::Uninitialized => Ok(serde_json::Value::Null),
        rquickjs::Type::Undefined => Ok(serde_json::Value::Null),
        rquickjs::Type::Null => Ok(serde_json::Value::Null),
        rquickjs::Type::Bool => {
            let val = js.as_bool().unwrap_or(false);
            Ok(serde_json::Value::from(val))
        }
        rquickjs::Type::Int => {
            let val = js.as_int().unwrap_or(0);
            Ok(serde_json::Value::from(val))
        }
        rquickjs::Type::Float => {
            let val = js.as_float().unwrap_or(0.0);
            Ok(serde_json::Value::from(val))
        }
        rquickjs::Type::String | rquickjs::Type::Symbol | rquickjs::Type::BigInt => {
            let val = match js.as_string() {
                Some(v) => v.to_string()?,
                None => String::new(),
            };
            Ok(serde_json::Value::from(val))
        }
        rquickjs::Type::Array => {
            let array = js.into_array().unwrap();
            let mut vec = vec![];
            for val in array {
                let val = val?;
                let json = js_to_json(val)?;
                vec.push(json);
            }
            Ok(serde_json::Value::from(vec))
        }
        rquickjs::Type::Function | rquickjs::Type::Constructor => Err(rquickjs::Error::FromJs {
            from: "Function",
            to: "Json",
            message: None,
        }),
        rquickjs::Type::Object => {
            let obj = js.into_object().unwrap();
            let mut map = serde_json::Map::new();
            for val in obj {
                let (k, v) = val?;
                map.insert(k.to_string()?, js_to_json(v)?);
            }
            Ok(serde_json::Value::from(map))
        }
        rquickjs::Type::Module => Err(rquickjs::Error::FromJs {
            from: "Module",
            to: "Json",
            message: None,
        }),
        rquickjs::Type::Unknown => Ok(serde_json::Value::Null),
        rquickjs::Type::Exception => {
            let exception = js.try_into_exception().unwrap();
            let message = exception.message();
            let line = exception.line();
            let stack = exception.stack();
            Ok(json!({
             "message":message,
             "line":line,
             "stack":stack
            }))
        }
    }
}

pub fn json_to_js<'js>(
    json: serde_json::Value,
    ctx: &Ctx<'js>,
) -> rquickjs::Result<rquickjs::Value<'js>> {
    match json {
        serde_json::Value::Null => Ok(rquickjs::Value::new_null(ctx.clone())),
        serde_json::Value::Bool(v) => Ok(rquickjs::Value::new_bool(ctx.clone(), v)),
        serde_json::Value::Number(v) => Ok(rquickjs::Value::new_number(
            ctx.clone(),
            v.as_f64().unwrap_or(0.0),
        )),
        serde_json::Value::String(v) => {
            Ok(rquickjs::String::from_str(ctx.clone(), v.as_str())?.into_value())
        }
        serde_json::Value::Array(v) => {
            let array = rquickjs::Array::new(ctx.clone())?;
            for (i, val) in v.into_iter().enumerate() {
                let js = json_to_js(val, ctx)?;
                array.set(i, js)?;
            }
            Ok(array.into_value())
        }
        serde_json::Value::Object(v) => {
            let obj = rquickjs::Object::new(ctx.clone())?;
            for (k, v) in v {
                obj.set::<String, _>(k, json_to_js(v, ctx)?)?;
            }
            Ok(obj.into_value())
        }
    }
}
fn js_to_string(msg: rquickjs::Value) -> rquickjs::Result<String> {
    if msg.is_exception() {
        let exception = msg.into_exception().unwrap();
        return Ok(format!("{exception}"));
    }
    let msg = match msg.type_of() {
        rquickjs::Type::Uninitialized => "uninitialized".into(),
        rquickjs::Type::Undefined => "undefined".into(),
        rquickjs::Type::Null => "null".into(),
        rquickjs::Type::Array => {
            let array = msg.into_array().unwrap();
            let mut vec = vec![];
            for ele in array {
                let ele = ele?;
                vec.push(js_to_string(ele)?);
            }
            format!("[{}]", vec.join(", "))
        }
        rquickjs::Type::Function => {
            let name = msg.into_object().unwrap().get::<_, String>("name")?;
            format!("<Function {name}>")
        }
        rquickjs::Type::Constructor => {
            let name = msg.into_object().unwrap().get::<_, String>("name")?;
            format!("<Constructor {name}>")
        }
        rquickjs::Type::Object => {
            let obj = msg.into_object().unwrap();
            let constructor = obj.get::<_, rquickjs::Value<'_>>("constructor")?;
            let a_name = "?Anonymous".to_string();
            let name = if constructor.is_constructor() {
                let constructor = constructor.as_constructor().unwrap();
                let name = constructor.get::<_, String>("name")?;
                if name.is_empty() {
                    a_name
                } else {
                    name
                }
            } else {
                a_name
            };
            if name == "Object" {
                let ctx = obj.ctx().clone();
                let json_stringify =ctx.json_stringify(obj)?;
                let s=match json_stringify {
                    Some(v) => {v.to_string()?},
                    None => {
                        String::from("{}")
                    },
                };
                return Ok(s);
            }
            let func = obj.get::<_,Function<'_>>("toString")?;
            let call = func.call::<_,String>((This(obj),))?;
            call
        }
        rquickjs::Type::Module => {
            let name = msg.as_object().unwrap().get::<_, String>("name")?;
            format!("[Module {name}]")
        }
        rquickjs::Type::Unknown => "[Unknown]".into(),
        rquickjs::Type::String => {
            format!(
                "'{}'",
                msg.as_string().map(|v| v.to_string().unwrap()).unwrap()
            )
        }
        rquickjs::Type::Symbol => {
            let sym = Symbol::from_value(msg)?;
            let inner = js_to_string(sym.into_inner())?;
            format!("Symbol({inner})")
        }
        rquickjs::Type::Exception => {
            let exception = msg.into_exception().unwrap();
            let mut name = exception.get::<_, String>("name")?;
            if name.is_empty() {
                name = "Error".to_string();
            }
            let msg = exception.message().unwrap_or_default();
            let stack = exception.stack().unwrap_or_default();
            format!("{name} {msg} \n\t{stack}")
        }

        _ => {
            let json_stringify = msg.ctx().clone().json_stringify(msg)?.unwrap();
            json_stringify.to_string()?
        }
    };
    Ok(msg)
}
#[instrument(skip_all)]
pub fn handle_js_error<'js>(err: CaughtError<'js>, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Error {
    tracing::error!("js执行异常：{err}");
    let globals = ctx.globals();
    let console = globals
        .get::<_, console::JsConsole>("console")
        .catch(ctx)
        .expect("get object 'console' from 'globals' failed");
    {
        console
            .write_to_log("error", format!("{err}"))
            .expect("写入error日志失败");
    }
    err.throw(ctx)
}

pub fn to_js_err<E: Error>(e: E, ctx: Ctx<'_>) -> rquickjs::Error {
    let s = e.to_string();
    ctx.throw(rquickjs::String::from_str(ctx.clone(), &s).unwrap().into())
}
pub fn throw_js_err<'js, E: Into<&'js str>>(e: E, ctx: Ctx<'js>) -> rquickjs::Error {
    let e = e.into();
    ctx.throw(rquickjs::String::from_str(ctx.clone(), e).unwrap().into())
}

// pub fn get_constructor_name(obj: &rquickjs::Object<'_>) -> rquickjs::Result<String> {
//     let constructor = obj.get::<_, rquickjs::Object<'_>>("constructor")?;
//     constructor.get::<_, String>("name")
// }

// pub fn to_promise<'a, T: FromJs<'a> + 'a>(
//     ctx: &Ctx<'a>,
//     val: rquickjs::Value<'a>,
// ) -> Maybe<T, rquickjs::promise::Promise<'a, T>> {
//     if val.is_object() {
//         let obj = val.into_object().unwrap();
//         if let Ok(name) = get_constructor_name(&obj) {
//             if name == "Promise" {
//                 let val = obj.into_value();
//                 let promise = rquickjs::promise::Promise::<'a, T>::from_js(ctx, val).catch(ctx);
//                 let promise = auto_result!(promise,err=>{
//                     handle_js_error(err,ctx);
//                     panic!()
//                 });
//                 return Maybe::Right(promise);
//             }
//             return Maybe::Left(T::from_js(ctx, obj.into_value()).unwrap());
//         }
//         return Maybe::Left(T::from_js(ctx, obj.into_value()).unwrap());
//     }
//     Maybe::Left(T::from_js(ctx, val).unwrap())
// }

// mod test {
//     use rquickjs::{FromJs, IntoJs, Module};

//     use crate::handle::api::plugin;

//     use super::*;
//     #[test]
//     pub fn test_to_promise() {
//         let runtime = tokio::runtime::Runtime::new().unwrap();
//         runtime.block_on(async {
//             let rt = AsyncRuntime::new().unwrap();
//             let ctx = AsyncContext::full(&rt).await.unwrap();

//             let js = r"export function test(){return 1}";
//             let a = async_with!(ctx=>|ctx|{
//                    let m= ctx.clone().compile("a", js).unwrap();
//                    let function= m.get::<_,rquickjs::Function<'_>>("test").unwrap();
//                    let pro=rquickjs::promise::Promised::from(async move{
//                     function.call::<_,i32>(()).unwrap()
//                    }).into_js(&ctx).unwrap();
//                    rquickjs::promise::Promise::<'_,i32>::from_js(&ctx,pro).unwrap().await.unwrap()
//             })
//             .await;
//             println!("a={a}");

//             let js = r"export async function test(){return 3}";
//             let b = async_with!(ctx=>|ctx|{
//                 let m= ctx.clone().compile("a", js).unwrap();
//                 let function= m.get::<_,rquickjs::Function<'_>>("test").unwrap();
//                    let pro=rquickjs::promise::Promised::from(async move{
//                     function.call::<_,rquickjs::Value<'_>>(()).unwrap()
//                    }).into_js(&ctx).unwrap();
//                    rquickjs::promise::Promise::<'_,i32>::from_js(&ctx,pro).unwrap().await.unwrap()
//             })
//             .await;
//             println!("b={b}");
//         })
//     }
// }


impl PluginCtx {
    pub async fn dynamic_scripts(&self,scope_key:Scope)->Vec<String>{
        let mut scripts=vec![];
        for link in self
.plugin
.dynamic_links
.split(",")
.map(|v| v.trim())
.filter(|v| !v.is_empty())
{
let ctx = &self.ctx;
let ctx = ctx.lock().await;
let scope_key = scope_key.clone();
let script = async_with!(ctx=>|ctx|{
    let res=server::call_function::<Option<String>,_>(&ctx, "dynamicScript", (link, scope_key)).await.catch(&ctx);
    let res=auto_result!(res,err=>{
        let e= format!(r##"throw new Error("{err}")"##);
        handle_js_error(err,&ctx);
        return Some(e);
    });
    match res {
       Either::Left(value) =>{
            return value;
        },
       Either::Right(_args) => {
                let e= format!(r##"throw new Error("dynamicScript not a function")"##);
                return Some(e);
            },
        }
   
})
.await;
scripts.push(script.unwrap_or_default());


    }
    scripts
}

}
