use std::{collections::HashMap, str::FromStr};

use async_recursion::async_recursion;
use hyper::http::{Request, Response};

use crate::{handle::model::HostList, net_proxy::HttpContext, proxy, IS_SYS_PROXY};
use hyper::body::Body;

use serde_json::{to_value, Number, Value};
use sqlx::query_as;
use tracing::{error, instrument};

use crate::{
    auto_option, auto_result,
    handle::{
        api::{config, detect},
        model::Config,
        response_data, response_msg,
    },
    wrap, DBPOOL,
};

async fn get_child(id: i64) -> Vec<Config> {
    let pool = &DBPOOL.clone();

    let fetch_all = query_as::<_, Config>("select * from `config` where `parent_id` =?")
        .bind(id)
        .fetch_all(pool)
        .await;
    let child = auto_result!(fetch_all,err=>{
        error!("异常:{:?}", err);
        return vec![];
    });
    child
}
pub async fn get_config_by_key(key: &str) -> Option<Config> {
    let pool = &DBPOOL.clone();
    let fetch_optional = query_as::<_, Config>("select * from `config` where `key` =?")
        .bind(&key)
        .fetch_optional(pool)
        .await;
    auto_result!(fetch_optional,err=>{
        error!(key,"异常:{:?}", err);
        None
    })
}
pub async fn get_config_by_key_and_parent_id(key: &str, parent_id: i64) -> Option<Config> {
    let pool = &DBPOOL.clone();
    let fetch_optional =
        query_as::<_, Config>("select * from `config` where `key` =? and `parent_id`=?")
            .bind(&key)
            .bind(&parent_id)
            .fetch_optional(pool)
            .await;
    auto_result!(fetch_optional,err=>{
        error!(key,"异常:{:?}", err);
        return None;
    })
}

pub async fn get_config(key: &str) -> Option<serde_json::Value> {
    let config = auto_option!(get_config_by_key(key).await, None);
    Some(parse_config_from_sql(config).await)
}
pub async fn get_configs() -> Result<Vec<Value>, sqlx::error::Error> {
    let pool = &DBPOOL.clone();
    let configs_temp = sqlx::query_as::<_, Config>("select * from `config`")
        .fetch_all(pool)
        .await?;

    let mut configs = vec![];
    let mut id_map_childs: HashMap<i64, Vec<Config>> = HashMap::new();
    for config in configs_temp {
        if config.parent_id > 0 {
            let childs = match id_map_childs.get_mut(&config.parent_id) {
                Some(v) => v,
                None => {
                    id_map_childs.insert(config.parent_id, vec![]);
                    id_map_childs.get_mut(&config.parent_id).unwrap()
                }
            };
            childs.push(config);
            continue;
        }
        configs.push(config);
    }

    let mut data = vec![];

    for config in configs {
        let config = config_to_json_value(config, &mut id_map_childs);
        data.push(config);
    }
    Ok(data)
}

pub async fn set_system_proxy(enable: bool) -> Result<(), String> {
    let proxy_addr = {
        let port = config::get_config("port")
            .await
            .map(|v| v.as_i64().unwrap_or(3000))
            .unwrap_or(3000);
        format!("127.0.0.1:{port}")
    };
    let no_proxys = {
        let black_list = config::get_config("blackList").await.expect("获取配置异常");

        let black_list: HostList = serde_json::from_value(black_list).unwrap();
        let enabled = black_list.enabled;
        let mut no_proxys = String::new();

        if enabled {
            for v in &black_list.list {
                no_proxys.push_str(v);
                no_proxys.push(';');
            }
            no_proxys.pop();
        }
        no_proxys
    };
    proxy::set_system_proxy(enable, &proxy_addr, &no_proxys).map_err(|e| e.to_string())?;
    let mut guard = IS_SYS_PROXY.write().map_err(|e| e.to_string())?;
    *guard = enable;
    Ok(())
}

pub async fn update_config_by_id(id: i64, val: &str) -> Result<(), String> {
    let pool = &DBPOOL.clone();
    let fetch_optional = query_as::<_, Config>("select * from `config` where `id` =?")
        .bind(id)
        .fetch_optional(pool)
        .await;
    let op: Option<Config> = auto_result!(fetch_optional,err=>{
        error!("异常:{:?}", err);
        return Err("系统异常".into());
    });
    let config = auto_option!(op, Err("配置不存在".into()));
    let key = config.key.as_str();
    let tag = config.r#type;
    match tag.as_str() {
        "str" | "list" => {}
        "num" => {
            if Number::from_str(val).is_err() {
                return Err(format!(
                    "值:{}无法转换位配置id:{}的指定类型:{}",
                    val, id, tag
                ));
            }
        }
        "bool" => {
            if bool::from_str(val).is_err() {
                return Err(format!(
                    "值:{}无法转换位配置id:{}的指定类型:{}",
                    val, id, tag
                ));
            }
        }
        "obj" => return Err("不能直接修改obj类型的配置的值，请修改它的属性值".into()),
        _ => return Err("类型异常".into()),
    }
    match key {
        "port" => {
            let port = u32::from_str(val).unwrap_or(0);
            if !(port > 100 && port <= 65535) {
                return Err("端口范围不支持".into());
            }
        }
        "enabled" => {
            let pool = &DBPOOL.clone();
            auto_result!(sqlx::query("update `config` set `value`='false' where `type`='bool' and `key` =? ")
            .bind("enabled")
            .execute(pool)
            .await,err=>{
                error!("异常:{:?}", err);
                return Err("修改失败".into())

            });
            auto_result!(sqlx::query("update `config` set `value`='true' where `id` =?")
            .bind(config.id)
            .execute(pool)
            .await,err=>{
                error!("异常:{:?}", err);
                return Err("修改失败".into())
            });
        }
        "key" | "cert" => {
            let path = std::path::Path::new(val);
            if !path.exists() {
                return Err("文件不存在".into());
            }
        }
        "systemProxy" => set_system_proxy(bool::from_str(val).unwrap_or(false)).await?,
        _ => {}
    }
    let execute = sqlx::query("update `config` set `value`=? where `id`=?")
        .bind(val)
        .bind(id)
        .execute(pool)
        .await;

    auto_result!(execute,e=>{
        error!("异常:{:?}", e);
        return Err("修改配置异常".into())

    });
    Ok(())
}
#[async_recursion]
async fn parse_config_from_sql(config: Config) -> serde_json::Value {
    let id = config.id;
    let ty = config.r#type.clone();
    let ty = ty.as_str();
    match ty {
        "obj" => {
            let child = get_child(id).await;
            let mut map = serde_json::Map::new();
            for ele in child {
                let key = ele.key.clone();
                let val = parse_config_from_sql(ele).await;
                map.insert(key, val);
            }
            return serde_json::Value::Object(map);
        }
        "str" => serde_json::from_str(&config.value).expect("字符串格式异常"),
        "list" => config.value.split("&&").filter(|v| !v.is_empty()).collect::<Vec<&str>>().into(),
        "num" => serde_json::Value::Number(Number::from_str(&config.value).unwrap()),
        "bool" => serde_json::Value::Bool(bool::from_str(&config.value).unwrap_or(false)),
        _ => {
            panic!("invalid type: {ty}")
        }
    }
}

fn config_to_json_value(
    config: Config,
    id_map_childs: &mut HashMap<i64, Vec<Config>>,
) -> serde_json::Value {
    let id = config.id;
    let ty = config.r#type.clone();
    let ty = ty.as_str();
    let value = config.value.clone();
    let mut val = auto_result!(to_value(config),e=>{
        error!("序列化失败:{e}");
        panic!("{}",e);
    });
    let value = match ty {
        "obj" => {
            let child = id_map_childs.remove(&id).unwrap_or(vec![]);
            let mut list = vec![];
            for ele in child {
                let val = config_to_json_value(ele, id_map_childs);
                list.push(val);
            }
            serde_json::Value::Array(list)
        }
        "str" => serde_json::from_str(&value).expect("字符串格式异常"),
        "list" => value.split("&&").filter(|v| !v.is_empty()).collect::<Vec<&str>>().into(),
        "num" => serde_json::Value::Number(Number::from_str(&value).unwrap()),
        "bool" => serde_json::Value::Bool(bool::from_str(&value).unwrap_or(false)),
        _ => {
            panic!("invalid type: {ty}")
        }
    };
    val.as_object_mut()
        .unwrap()
        .insert("value".to_string(), value);
    return val;
}

#[instrument]
pub async fn get(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _body) = auto_result!(detect(req).await);
    let key = params.remove("key").unwrap_or_default();
    let value = get_config(key.as_str()).await;
    response_data(&value, "")
}

#[instrument]
pub async fn list(_ctx: HttpContext, _req: Request<Body>) -> Response<Body> {
    let data = auto_result!( get_configs().await,err=>{
        error!("异常:{:?}", err);
        return response_msg(500, "系统异常");
    });
    response_data(&data, "")
}

#[instrument()]
pub async fn update(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (_params, mut body) = auto_result!(detect(req).await);
    let body = auto_option!(
        body.as_object_mut(),
        response_msg(500, "JSON对象序列化请求体异常")
    );
    let id = auto_option!(
        body.remove("id").map(|s| s.as_i64()),
        response_msg(500, "id传参异常")
    );
    let id = auto_option!(id, response_msg(500, "id传参异常"));
    let value = auto_option!(body.remove("value"), response_msg(500, "value传参异常"));
    let value = if value.is_string() {
        serde_json::value::from_value::<String>(value).unwrap()
    } else {
        value.to_string()
    };
    auto_result!(update_config_by_id(id,&value).await, err=>{
        return response_msg(500, err.as_str());
    });
    return response_msg(200, "");
}

pub fn route(router: &mut HashMap<&'static str, Box<super::AsyncFn>>) {
    router.insert("/config/get", wrap!(get));
    router.insert("/config/list", wrap!(list));
    router.insert("/config/update", wrap!(update));
}
