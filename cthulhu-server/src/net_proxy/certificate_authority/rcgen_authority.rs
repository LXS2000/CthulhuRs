use crate::net_proxy::{
    certificate_authority::{CertificateAuthority, CACHE_TTL, NOT_BEFORE_OFFSET, TTL_SECS},
    error::Error,
};
use async_trait::async_trait;

use hyper::http::uri::Authority;
use moka::future::Cache;
use rand::{thread_rng, Rng};
use rcgen::{DistinguishedName, DnType, KeyPair, RcgenError, SanType};
use time::OffsetDateTime;

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
    sync::Arc, time::Duration,
};

use tokio_rustls::rustls::{self, ServerConfig};
use tracing::debug;

/// Issues certificates for use when communicating with clients.
///
/// Issues certificates for communicating with clients over TLS. Certificates are cached in memory
/// up to a max size that is provided when creating the authority. Certificates are generated using
/// the `rcgen` crate.
///
/// # Examples
///
/// ```rust
/// use hudsucker::{certificate_authority::RcgenAuthority, rustls};
/// use rustls_pemfile as pemfile;
///
/// let mut private_key_bytes: &[u8] = include_bytes!("../../examples/ca/hudsucker.key");
/// let mut ca_cert_bytes: &[u8] = include_bytes!("../../examples/ca/hudsucker.cer");
/// let private_key = rustls::PrivateKey(
///     pemfile::pkcs8_private_keys(&mut private_key_bytes)
///         .unwrap()
///         .remove(0),
/// );
/// let ca_cert = rustls::Certificate(
///     pemfile::certs(&mut ca_cert_bytes)
///         .unwrap()
///         .remove(0),
/// );
///
/// let ca = RcgenAuthority::new(private_key, ca_cert, 1_000).unwrap();
/// ```

pub struct RcgenAuthority {
    pub private_key: rustls::PrivateKey,
    pub ca_cert: rustls::Certificate,
    cache: Cache<Authority, Arc<ServerConfig>>,
}

impl RcgenAuthority {
    /// Attempts to create a new rcgen authority.
    ///
    /// # Errors
    ///
    /// This will return an error if the provided key or certificate is invalid, or if the key does
    /// not match the certificate.
    pub fn new(
        private_key: rustls::PrivateKey,
        ca_cert: rustls::Certificate,
        cache_size: u64,
    ) -> Result<RcgenAuthority, Error> {
        let ca = Self {
            private_key,
            ca_cert,
            cache: Cache::builder()
                .max_capacity(cache_size)
                .time_to_live(std::time::Duration::from_secs(CACHE_TTL))
                .build(),
        };

        ca.validate()?;
        Ok(ca)
    }

    fn gen_cert(&self, authority: &Authority) -> rustls::Certificate {
        let mut params = rcgen::CertificateParams::default();
        params.serial_number = Some(thread_rng().gen::<u64>().into());
        
        let not_before = OffsetDateTime::now_utc() - Duration::from_secs(NOT_BEFORE_OFFSET as u64);
        params.not_before = not_before;
        params.not_after = not_before + Duration::from_secs(TTL_SECS  as u64);

        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, authority.host());
        params.distinguished_name = distinguished_name;
        let host = authority.host().to_owned();
        // println!("Generating server config:{:?}", &host);
        let san = match Ipv4Addr::from_str(&host) {
            Ok(ipv4) => SanType::IpAddress(std::net::IpAddr::V4(ipv4)),
            Err(_) => match Ipv6Addr::from_str(&host) {
                Ok(ipv6) => SanType::IpAddress(std::net::IpAddr::V6(ipv6)),
                Err(_) => SanType::DnsName(host),
            },
        };

        params.subject_alt_names.push(san);

        let key_pair = KeyPair::from_der(&self.private_key.0).expect("Failed to parse private key");
        params.alg = key_pair
            .compatible_algs()
            .next()
            .expect("Failed to find compatible algorithm");
        params.key_pair = Some(key_pair);

        let key_pair = KeyPair::from_der(&self.private_key.0).expect("Failed to parse private key");

        let ca_cert_params =
            rcgen::CertificateParams::from_ca_cert_der(self.ca_cert.as_ref(), key_pair)
                .expect("Failed to parse CA certificate");
        let ca_cert = rcgen::Certificate::from_params(ca_cert_params)
            .expect("Failed to generate CA certificate");

        let cert = rcgen::Certificate::from_params(params).expect("Failed to generate certificate");

        rustls::Certificate(
            cert.serialize_der_with_signer(&ca_cert)
                .expect("Failed to serialize certificate"),
        )
    }

    fn validate(&self) -> Result<(), RcgenError> {
        let key_pair = rcgen::KeyPair::from_der(&self.private_key.0)?;
        rcgen::CertificateParams::from_ca_cert_der(self.ca_cert.as_ref(), key_pair)?;
        Ok(())
    }
}

#[async_trait]
impl CertificateAuthority for RcgenAuthority {
    async fn gen_server_config(&self, authority: &Authority, alpn: Vec<Vec<u8>>) -> Arc<ServerConfig> {
        if let Some(server_cfg) = self.cache.get(authority).await {
            debug!("Using cached server config");
            return server_cfg;
        }
        debug!("Generating server config:{:?}", &authority);

        let certs = vec![self.gen_cert(authority)];

        let mut server_cfg = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, self.private_key.clone())
            .expect("Failed to build ServerConfig");

        // server_cfg.alpn_protocols = vec![
        //     b"h2".to_vec(),
        //     b"http/1.1".to_vec(),
        // ];
        server_cfg.alpn_protocols = alpn;

        let server_cfg = Arc::new(server_cfg);

        self.cache
            .insert(authority.clone(), Arc::clone(&server_cfg))
            .await;

        server_cfg
    }
}
