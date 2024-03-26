use std::{error::Error, net::IpAddr, str::FromStr};

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn set_system_proxy(_enable: bool, _proxy_addr: &str, _no_proxys: &str) -> Result<(), String> {
    Err("程序所在的系统暂时不支持设置系统代理".into())
}
#[cfg(target_os = "linux")]
pub fn set_system_proxy(
    enable: bool,
    proxy_addr: &str,
    no_proxys: &str,
) -> Result<(), Box<dyn Error>> {
    use std::process::Command;
    if enable {
        let _out = Command::new(format!("export http_proxy={proxy_addr}")).output()?;
        let _out = Command::new(format!("export https_proxy={proxy_addr}")).output()?;
        if !no_proxys.is_empty() {
            let _out = Command::new(format!("export no_proxy={no_proxys}")).output()?;
        }
    } else {
        let _out = Command::new(format!("unset http_proxy")).output()?;
        let _out = Command::new(format!("unset https_proxy")).output()?;
        let _out = Command::new(format!("unset no_proxy")).output()?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn set_system_proxy(
    enable: bool,
    proxy_addr: &str,
    no_proxys: &str,
) -> Result<(), Box<dyn Error>> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (settings, _) =
        hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")?;
    let e: u32 = if enable { 1 } else { 0 };
    settings.set_value("ProxyEnable", &e)?;
    if enable {
        // 开启代理
        if !no_proxys.is_empty() {
            settings.set_value("ProxyOverride", &no_proxys)?;
        }
        settings.set_value("ProxyServer", &proxy_addr)?;
    }
    Ok(())
}
use hyper::Uri;
use reqwest::Proxy;
use std::collections::HashMap;
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_system_proxies_map() -> HashMap<&'static str, Uri> {
    HashMap::new()
}

#[cfg(target_os = "windows")]
pub fn get_system_proxies_map() -> HashMap<&'static str, Uri> {
    use winreg::enums::*;
    use winreg::RegKey;

    use crate::auto_result;
    let mut map = HashMap::new();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (settings, _) = auto_result!(
        hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings"),
        map
    );
    let enable = auto_result!(settings.get_value::<u32, _>("ProxyEnable"), map);
    if enable == 0 {
        return map;
    }

    let addr = auto_result!(settings.get_value::<String, _>("ProxyServer"), map);

    map.insert("all", Uri::from_str(&addr).unwrap());
    map
}

#[cfg(target_os = "linux")]
pub fn get_system_proxies_map() -> HashMap<&'static str, Uri> {
    let http_proxy = std::env::var("http_proxy").ok();
    let https_proxy = std::env::var("https_proxy").ok();
    let mut map = HashMap::new();

    if let Some(addr) = http_proxy {
        map.insert("http", Uri::from_str(&addr).unwrap());
    }
    if let Some(addr) = https_proxy {
        map.insert("https", Uri::from_str(&addr).unwrap());
    }
    map
}
#[allow(dead_code)]
pub fn get_system_proxies() -> Vec<Proxy> {
    let system_proxies_map = get_system_proxies_map();
    let mut proxies = vec![];
    for (key, val) in system_proxies_map {
        let uri = to_proxy_uri(val);
        let p = match key {
            "all" => Proxy::all(uri.to_string()).unwrap(),
            "http" => Proxy::http(uri.to_string()).unwrap(),
            "https" => Proxy::https(uri.to_string()).unwrap(),
            _ => panic!("unknown proxy type"),
        };
        proxies.push(p)
    }
    proxies
}
fn to_proxy_uri(addr: Uri) -> Uri {
    Uri::builder()
        .scheme(addr.scheme_str().unwrap_or("http"))
        .authority(addr.authority().unwrap().clone())
        .path_and_query(addr.path_and_query().map(|v| v.as_str()).unwrap_or(""))
        .build()
        .unwrap()
}

pub fn already_sys_proxy(port: u16, ip: Option<IpAddr>) -> bool {
    let system_proxies_map = get_system_proxies_map();
    for (_key, val) in system_proxies_map {
        let host = val.host().unwrap_or_default();
        let p_port = val.port_u16().unwrap_or_default();
        if let Ok(p_ip) = IpAddr::from_str(host) {
            if p_ip.is_loopback() || p_ip.is_unspecified() {
                return p_port == port;
            }
            if let Some(ip) = ip {
                if ip == p_ip && (ip.is_loopback() || p_ip.is_unspecified()) {
                    return p_port == port;
                }
            }
        }
        if host == "localhost" {
            return p_port == port;
        }
    }
    false
}
// pub fn system_proxy_connector(
//     conn: HttpsConnector<HttpConnector>,
// ) -> ProxyConnector<HttpsConnector<HttpConnector>> {
//     let mut conn = ProxyConnector::new(conn).expect("构建系统代理客户端失败");
//     if !*IS_SYS_PROXY.read().unwrap() {
//         //避免循环代理
//         let proxies = get_system_proxies();
//         conn.extend_proxies(proxies);
//     }
//     conn
// }
