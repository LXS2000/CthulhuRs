use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead},
    path::Path,
    str::FromStr,
    vec,
};

use crate::net_proxy::HttpContext;
use hyper::http::{Request, Response};

use hyper::Body;
use tracing::{error, instrument};

use crate::{
    auto_option, auto_result,
    core::PluginCtx,
    handle::{model::Plugin, response_data, response_msg},
    utils::{self, mini_match},
    wrap, DBPOOL, PLUGIN_MANAGER,
};

use super::detect;
#[instrument]
pub async fn get_enabled_plugins() -> Vec<Plugin> {
    let pool = &DBPOOL.clone();
    let sql = "select * from `plugin` where `enable` = 1";
    let vec = sqlx::query_as::<_, Plugin>(sql).fetch_all(pool).await;
    let vec = auto_result!(vec,err=>{
        error!("异常:{:?}", err);
        panic!("{}",err)
    });
    vec
}
#[instrument]
pub async fn get_plugins_by_host(host: &str) -> Vec<Plugin> {
    let vec = get_enabled_plugins().await;

    vec.into_iter()
        .filter(|p| p.matches.is_empty() || p.matches.split(";").any(|m| mini_match(m, host)))
        .collect()
}
#[instrument]
pub async fn get_net_modify_plugin(ids: &HashSet<String>) -> Option<Plugin> {
    if ids.is_empty() {
        return None;
    }
    let pool = &DBPOOL.clone();
    let ids = ids
        .iter()
        .map(|s| format!(r"'{}'", s))
        .collect::<Vec<String>>()
        .join(",");
    let sql = format!(
        "select * from `plugin where `id` in ({ids})order by `net_modify`,`install_time` limit 1"
    );
    let vec = sqlx::query_as::<_, Plugin>(sql.as_str())
        .fetch_optional(pool)
        .await;
    auto_result!(vec,err=>{
        error!("异常:{:?}", err);
        panic!("{}",err)
    })
}

#[instrument]
pub async fn get_net_monitor_plugins(ids: &HashSet<String>) -> Vec<Plugin> {
    if ids.is_empty() {
        return vec![];
    }
    let pool = &DBPOOL.clone();
    let ids = ids
        .iter()
        .map(|s| format!(r"'{}'", s))
        .collect::<Vec<String>>()
        .join(",");
    let sql = format!("select * from `plugin where `id` in ({ids}) and `net_monitor`!=0");
    let vec = sqlx::query_as::<_, Plugin>(sql.as_str())
        .fetch_all(pool)
        .await;
    auto_result!(vec,err=>{
        error!("异常:{:?}", err);
        panic!("{}",err)
    })
}
pub async fn get_plugin_by_id(id: &str) -> Option<Plugin> {
    let pool = &DBPOOL.clone();
    let res = sqlx::query_as::<_, Plugin>("select * from `plugin` where `id`=?")
        .bind(id)
        .fetch_optional(pool)
        .await;
    let res = auto_result!(res,err=>{
        error!("{err}");
        panic!("{}",err)
    });
    res
}
#[instrument]
pub async fn install(dir: &str) {
    let dir = std::path::Path::new(dir);
    let plugin_info_path = relative_path::RelativePath::new("plugin.json").to_path(&dir);
    let json = {
        let path = plugin_info_path.to_str().unwrap();
        let bytes = utils::read_bytes(path).expect(path);
        let value = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap();
        value
    };
    fn check_path(dir: &Path, path: &str, is_file: bool) -> Option<String> {
        if path.is_empty() {
            return Some(path.to_string());
        }
        if path.starts_with("../") {
            println!(
                "{path} 指向的文件地址不能超过当前目录：{}",
                dir.to_str().unwrap()
            );
            return None;
        }
        let rlt_path = relative_path::RelativePath::new(&path);
        let path = rlt_path.to_logical_path(dir);

        if !path.exists() {
            println!("{} 指向的文件地址不存在", path.to_str().unwrap());
            return None;
        }
        if is_file && !path.is_file() {
            println!("{} 指向的地址不是文件类型", path.to_str().unwrap());
            return None;
        }
        let path = rlt_path.to_logical_path("");
        Some(path.to_str().unwrap().to_string())
    }

    let name = auto_option!(
        json.get("name").map(|v| v.as_str().unwrap_or("").trim()),
        println!("插件名称不能为空")
    );
    if name.is_empty() {
        println!("插件名称不能为空");
        return;
    }
    let version = json
        .get("version")
        .map(|v| v.as_str().unwrap_or("0.0.1"))
        .unwrap_or("0.0.1")
        .trim();
    let intro = json
        .get("intro")
        .map(|v| v.as_str().unwrap_or(""))
        .unwrap_or_default()
        .trim();
    let logo = {
        let logo = json
            .get("logo")
            .map(|v| v.as_str().unwrap_or(""))
            .unwrap_or_default()
            .trim();
        if logo.starts_with("http://") || logo.starts_with("https://") {
            auto_result!(  hyper::Uri::from_str(logo),err=>{
                eprintln!("{err}");
                return;
            });
            logo.to_owned()
        } else {
            let logo = auto_option!(check_path(&dir, &logo, true), {
                return;
            });
            logo
        }
    };
    let empty = vec![];
    let empty_obj = serde_json::json!({});
    let (web_root, web_index) = {
        let web = json.get("web").unwrap_or(&empty_obj);
        let root = web
            .get("root")
            .map(|v| v.as_str().unwrap_or("").trim())
            .unwrap_or_default();
        let root = auto_option!(check_path(&dir, &root, false), {
            return;
        });
        let index = if root.is_empty() {
            ""
        } else {
            let index = web
                .get("index")
                .map(|v| v.as_str().unwrap_or("").trim())
                .unwrap_or_default();
            let _index = auto_option!(check_path(&dir, &format!("{root}/{index}"), true), {
                return;
            });
            index
        };

        (root, index)
    };

    let (net_monitor, net_modify) = {
        let permissions = json.get("permissions").unwrap_or(&empty_obj);
        let net_monitor = permissions
            .get("netMonitor")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);
        let mut net_modify = permissions
            .get("netModify")
            .map(|v| v.as_i64().unwrap_or(0))
            .unwrap_or(0);
        if net_modify < 0 {
            net_modify = 0;
        }
        (net_monitor, net_modify)
    };

    let (server, worker, contents, dynamic_links) = {
        let script = json.get("script").unwrap_or(&empty_obj);
        let server = {
            let server = script
                .get("server")
                .map(|v| v.as_str().unwrap_or("").trim())
                .unwrap_or_default();
            let server = auto_option!(check_path(&dir, &server, true), {
                return;
            });
            server
        };

        let worker = {
            let worker = script
                .get("worker")
                .map(|v| v.as_str().unwrap_or("").trim())
                .unwrap_or_default();
            let worker = auto_option!(check_path(&dir, &worker, true), {
                return;
            });
            worker
        };
        let contents = {
            let contents = script
                .get("contents")
                .map(|v| v.as_array().unwrap())
                .unwrap_or(&empty);
            let mut paths = vec![];
            for content in contents {
                let content = content.as_str().unwrap_or("").trim();
                if content.is_empty() {
                    continue;
                }
                let content = auto_option!(check_path(&dir, &content, true), {
                    return;
                });

                paths.push(content);
            }
            paths
        };
        let dynamic_links = {
            let dynamic_links = script
                .get("dynamic_links")
                .map(|v| v.as_array().unwrap())
                .unwrap_or(&empty);
            let mut links = vec![];
            for link in dynamic_links {
                let link = link.as_str().unwrap_or("").trim();
                if link.is_empty() {
                    continue;
                }
                links.push(link);
            }
            links
        };
        (server, worker, contents, dynamic_links)
    };
    let matches = {
        let matches = json
            .get("matches")
            .map(|v| v.as_array().unwrap())
            .unwrap_or(&empty);
        matches
            .into_iter()
            .map(|v| v.as_str().unwrap_or("").trim())
            .filter(|v| !v.is_empty())
            .collect::<Vec<&str>>()
    };

    let id = {
        let id = uuid::Uuid::new_v4().simple().to_string();
        utils::hash(id.as_bytes(), 10,12)
    };
    let pool = &DBPOOL.clone();
    {
        let sql = r"select * from `plugin` where `path`=?";
        let res = sqlx::query_as::<_, Plugin>(sql)
            .bind(dir.to_str().unwrap())
            .fetch_optional(pool)
            .await;
        let op = auto_result!(res,err=>{
            println!("安装插件失败:{err}");
            return;
        });
        if let Some(plugin) = op {
            let sql = r"update `plugin` set `name`=?,`intro`=?,`version`=?,
        `logo_path`=?,`web_root`=?,`web_index`=?,`server_path`=?,`content_paths`=?,`worker_path`=?,`dynamic_links`=?,`matches`=?,
        `net_monitor`=?,`net_modify`=? where `id`=?";
            let res = sqlx::query(sql)
                .bind(name)
                .bind(intro)
                .bind(version)
                .bind(logo)
                .bind(web_root)
                .bind(web_index)
                .bind(server)
                .bind(contents.join(","))
                .bind(worker)
                .bind(dynamic_links.join(","))
                .bind(matches.join(","))
                .bind(net_monitor)
                .bind(net_modify)
                .bind(&plugin.id)
                .execute(pool)
                .await;
            auto_result!(res,err=>{
                println!("更新插件失败:{err}");
                return;
            });
            println!("更新插件成功");
            return;
        }
    }

    let sql = r"insert into `plugin`(`id`,`name`,`version`,`intro`,`path`,`logo_path`,`web_root`,
        `web_index`,`server_path`,`content_paths`,`worker_path`,`dynamic_links`,`matches`,`net_monitor`,`net_modify`)
        values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

    let res = sqlx::query(sql)
        .bind(id)
        .bind(name)
        .bind(version)
        .bind(intro)
        .bind(dir.to_str().unwrap())
        .bind(logo)
        .bind(web_root)
        .bind(web_index)
        .bind(server)
        .bind(contents.join(","))
        .bind(worker)
        .bind(dynamic_links.join(","))
        .bind(matches.join(","))
        .bind(net_monitor)
        .bind(net_modify)
        .execute(pool)
        .await;
    auto_result!(res,err=>{
        println!("安装插件失败:{err}");
        return;
    });
    println!("安装插件成功");
}
#[instrument(skip_all)]
pub async fn list(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _data) = auto_result!(detect(req).await);
    let key = params.remove("key").unwrap_or_default();
    let enable = params.remove("isEnable").unwrap_or_default();
    let enable = enable.parse().unwrap_or(-1);
    let pool = &DBPOOL.clone();
    let mut sql = "select * from `plugin`".to_string();

    if key != "" || (enable > 0 && enable <= 2) {
        sql += " where ";
        if key != "" && enable > 0 {
            sql += &format!("`name` like '%{key}%' and `enable` = {enable}");
        } else if key != "" {
            sql += &format!("`name` like '%{key}%'");
        } else if enable > 0 {
            sql += &format!("`enable` = {enable}");
        }
    }
    let vec = sqlx::query_as::<_, Plugin>(sql.as_str())
        .fetch_all(pool)
        .await;
    let vec = auto_result!(vec,err=>{
        error!("异常:{:?}", err);
        return response_msg(500, "系统异常");
    });

    return response_data(&vec, "");
}

#[instrument(skip_all)]
pub async fn enable(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _body) = auto_result!(detect(req).await);
    let id = auto_option!(params.remove("id"), response_msg(500, "缺少插件id"));
    let plugin = auto_option!(get_plugin_by_id(&id).await, response_msg(500, "插件不存在"));
    let _ = PLUGIN_MANAGER.del_ctx(&id).await;
    let enable = if plugin.enable == 0 {
        let ctx = auto_result!(PluginCtx::new(plugin).await,err=>{
          return  response_msg(500, err.as_str());
        });
        let _ = PLUGIN_MANAGER.set_ctx(ctx).await;
        1
    } else {
        // let _ = PLUGIN_MANAGER.set_ctx(ctx).await;
        0
    };
    let sql = "update `plugin` set `enable`=? where id=?";
    // let sql = "update `plugin` set `enable`=(~(`enable` &1))&(`enable` |1) where id=?";
    let pool = &DBPOOL.clone();
    let res = sqlx::query(sql).bind(enable).bind(id).execute(pool).await;
    auto_result!(res,err=>{
        error!("{err}");
        return response_msg(500, "修改插件异常");
    });

    response_msg(200, "")
}

#[instrument(skip_all)]
pub async fn del(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _body) = auto_result!(detect(req).await);
    let id = auto_option!(params.remove("id"), response_msg(500, "缺少插件id"));

    let _plugin = auto_option!(get_plugin_by_id(&id).await, response_msg(500, "插件不存在"));
    let pool = &DBPOOL.clone();

    let sql = "delete from `plugin` where id=?";
    auto_result!(sqlx::query(sql).bind(&id).execute(pool).await,err=>{
        error!("{err}");
        return response_msg(500, "删除插件异常");
    });

    // if utils::exists_path(plugin.path.as_str()) {
    //     auto_result!( utils::remove_path(plugin.path.as_str()),err=>{
    //         error!("{err}");
    //         begin.rollback().await.expect("事务提交失败");
    //         return response_msg(500, "删除插件文件异常");
    //     });
    // }
    PLUGIN_MANAGER.del_ctx(&id).await;
    response_msg(200, "删除插件成功")
}

#[instrument(skip_all)]
pub async fn log(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _body) = auto_result!(detect(req).await);

    let id = auto_option!(params.remove("id"), response_msg(500, "缺少插件id"));
    let level = params.remove("level").unwrap_or_default();
    let index = params.remove("index").unwrap_or_default();
    let index = auto_result!(index.parse::<u64>(),err=>{
        error!("异常：{}",err);
        return response_msg(500, "index传参异常");
    });
    let levels = vec!["info", "debug", "error", "warn"];
    if !levels.contains(&level.as_str()) {
        return response_msg(500, "无效的日志等级");
    }
    let plugin = auto_option!(get_plugin_by_id(&id).await, response_msg(500, "插件不存在"));

    let path = format!("{}\\LOGS\\{level}.log", &plugin.path);

    let path = Path::new(&path);
    let file = auto_result!(File::open(&path), response_data::<Vec<String>>(&vec![], ""));
    let size = 150;
    let index = index as usize;
    let reader = io::BufReader::new(file);
    let lines = reader
        .lines()
        .skip(index)
        .take(size)
        .map(|s| s.unwrap_or_default())
        .collect::<Vec<String>>();
    let log = lines;

    response_data(&log, "")
}
#[instrument(skip_all)]
pub async fn clear_log(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _body) = auto_result!(detect(req).await);

    let id = auto_option!(params.remove("id"), response_msg(500, "缺少插件id"));
    let level = params.remove("level").unwrap_or_default();

    let levels = vec!["info", "debug", "error", "warn"];
    if !levels.contains(&level.as_str()) {
        return response_msg(500, "无效的日志等级");
    }
    let plugin = auto_option!(get_plugin_by_id(&id).await, response_msg(500, "插件不存在"));

    let path = format!("{}\\LOGS\\{level}.log", &plugin.path);
    auto_result!(  utils::remove_path(&path),err=>{
        return response_msg(500, err.to_string());
    });

    response_msg(200, format!("{level} 日志已清空"))
}

// 重新加载插件
#[instrument(skip_all)]
pub async fn reload(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _body) = auto_result!(detect(req).await);
    let id = auto_option!(params.remove("id"), response_msg(500, "缺少插件id"));
    let plugin = auto_option!(get_plugin_by_id(&id).await, response_msg(500, "插件不存在"));
    let _ = PLUGIN_MANAGER.del_ctx(&id).await;
    //重新加载配置
    install(&plugin.path).await;
    let plugin = auto_option!(get_plugin_by_id(&id).await, response_msg(500, "插件不存在"));
    let ctx = auto_result!(PluginCtx::new(plugin).await,err=>{
      return  response_msg(500, err);
    });
    let _ = PLUGIN_MANAGER.set_ctx(ctx).await;
    response_msg(200, "插件重新加载成功")
}

#[instrument(skip_all)]
pub async fn tree_names(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _body) = auto_result!(detect(req).await);
    let id = auto_option!(params.remove("id"), response_msg(500, "缺少插件id"));
    let ctx = auto_option!(
        PLUGIN_MANAGER.get_ctx(&id).await,
        response_msg(500, "插件未被启用加载")
    );
    let collect = ctx
        .db
        .tree_names()
        .into_iter()
        .map(|v| String::from_utf8(v.to_vec()).unwrap())
        .collect::<Vec<String>>();
    response_data(&collect, "")
}
#[instrument(skip_all)]
pub async fn tree_list(_ctx: HttpContext, req: Request<Body>) -> Response<Body> {
    let (mut params, _body) = auto_result!(detect(req).await);
    let id = auto_option!(params.remove("id"), response_msg(500, "缺少插件id"));
    let tree = auto_option!(params.remove("tree"), response_msg(500, "缺少tree名称"));
    let ctx = auto_option!(
        PLUGIN_MANAGER.get_ctx(&id).await,
        response_msg(500, "插件未被启用加载")
    );
    let tree = auto_result!(ctx.db.open_tree(tree),err=>response_msg(500, err.to_string()));
    let mut kvs = vec![];
    for ele in tree.iter() {
        let (k, v) = auto_result!(ele,err=>response_msg(500, err.to_string()));
        kvs.push((
            String::from_utf8(k.to_vec()).unwrap(),
            String::from_utf8(v.to_vec()).unwrap(),
        ));
    }
    response_data(&kvs, "")
}
pub fn route(router: &mut HashMap<&'static str, Box<super::AsyncFn>>) {
    router.insert("/plugin/list", wrap!(list));
    router.insert("/plugin/enable", wrap!(enable));
    router.insert("/plugin/del", wrap!(del));
    router.insert("/plugin/log", wrap!(log));
    router.insert("/plugin/clearLog", wrap!(clear_log));

    router.insert("/plugin/reload", wrap!(reload));
    router.insert("/plugin/treeNames", wrap!(tree_names));
    router.insert("/plugin/treeList", wrap!(tree_list));
}
