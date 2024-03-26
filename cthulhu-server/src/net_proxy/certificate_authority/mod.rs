
mod rcgen_authority;

use async_trait::async_trait;
use hyper::http::uri::Authority;
use std::sync::Arc;
use tokio_rustls::rustls::ServerConfig;


pub use rcgen_authority::*;

const TTL_SECS: i64 = 365 * 24 * 60 * 60;
const CACHE_TTL: u64 = TTL_SECS as u64 / 2;
const NOT_BEFORE_OFFSET: i64 = 60;

/// Issues certificates for use when communicating with clients.
///
/// Clients should be configured to either trust the provided root certificate, or to ignore
/// certificate errors.
#[async_trait]
pub trait CertificateAuthority: Send + Sync + 'static {
    /// Generate ServerConfig for use with rustls.
    async fn gen_server_config(&self, authority: &Authority,alpn:Vec<Vec<u8>>) -> Arc<ServerConfig>;
}
