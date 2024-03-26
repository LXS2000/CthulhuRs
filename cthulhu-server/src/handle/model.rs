use std::io;

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub intro: String,
    pub logo_path: String,
    pub web_root: String,
    pub web_index: String,
    #[serde(skip)]
    pub path: String,
    #[serde(skip)]
    pub server_path: String,
    #[serde(skip)]
    pub worker_path: String,
    #[serde(skip)]
    pub content_paths: String,
    #[serde(skip)]
    pub dynamic_links: String,
    pub matches: String,
    pub net_monitor: i64, //网络监听权限
    pub net_modify: i64,  //网络修改权限
    pub enable: i64,
    pub install_time: chrono::DateTime<Local>,
}
impl Plugin {
    pub fn read_worker(&self) -> io::Result<String> {
        let relative_path = relative_path::RelativePath::new(&self.worker_path);
        let path = relative_path.to_path(&self.path);
        let vec = utils::read_bytes(path)?;
        Ok(String::from_utf8(vec).unwrap())
    }
    pub fn read_server(&self) -> io::Result<String> {
        let relative_path = relative_path::RelativePath::new(&self.server_path);
        let path = relative_path.to_path(&self.path);
        let vec = utils::read_bytes(path)?;
        Ok(String::from_utf8(vec).unwrap())
    }
    pub fn read_contents(&self) -> io::Result<Vec<String>> {
        let mut js = vec![];
        for ctx_path in self.content_paths.split(",") {
            let relative_path = relative_path::RelativePath::new(ctx_path);
            let path = relative_path.to_path(&self.path);
            let vec = utils::read_bytes(path)?;
            let from_utf8 = String::from_utf8(vec).unwrap();
            js.push(from_utf8)
        }
        Ok(js)
    }
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub id: i64,
    pub key: String,
    pub label: String,
    pub r#type: String,
    pub value: String,
    pub parent_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct HostList {
    pub enabled: bool,
    pub list: Vec<String>,
}
// impl HostList {
//     pub fn list(&self) -> Vec<&str> {
//         self.list.split("&&").collect()
//     }
// }
#[derive(Serialize, Deserialize)]
pub struct CertCfg {
    pub key: String,
    pub cert: String,
}
