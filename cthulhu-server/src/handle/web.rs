use std::path::Path;

use crate::net_proxy::HttpContext;
use hyper::http::{Request, Response};
use hyper::Body;

use crate::auto_option;

use super::{api::config, response_file, response_msg};

pub async fn handle_web(_ctx: &HttpContext, req: Request<Body>) -> Response<Body> {
    let (parts, _) = req.into_parts();
    let path = parts.uri.path();
    let workspace = {
        let workspace = auto_option!(config::get_config("workspace").await, {
            return response_msg(500, "请设置cthulhu server工作目录");
        });
        let workspace = workspace.as_str().unwrap_or("").to_string();

        if workspace.is_empty() {
            return response_msg(500, "请设置cthulhu server工作目录");
        }
        workspace
    };
    let workspace = Path::new(&workspace);
    if !workspace.is_dir() {
        return response_msg(500, "invalid workspace");
    }
    let file_path = relative_path::RelativePath::new(path).to_path(workspace);

    let mut file_path = file_path.to_str().unwrap();

    let std_path = std::path::Path::new(file_path);
    if std_path.is_file() {
        return response_file(&file_path).await;
    }
    //伪静态
    let sites = vec!["/iframe", "/inject"];
    for site in sites {
        if path.starts_with(site) {
            let p = format!("{site}/index.html");
            let path = relative_path::RelativePath::new(&p).to_path(&workspace);
            file_path = path.to_str().unwrap();
            return response_file(&file_path).await;
        }
    }
    return response_msg(404, "file not found");
}
