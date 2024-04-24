mod net;

use crate::net_proxy::{
    certificate_authority::CertificateAuthority, Error, HttpHandler, WebSocketHandler,
};

use hyper::Uri;
use hyper::{
    server::conn::{AddrIncoming, AddrStream},
    service::{make_service_fn, service_fn},
    Server,
};
use reqwest::Client;
// use internal::InternalProxy;

use std::{
    convert::Infallible,
    future::Future,
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
use tokio_tungstenite::Connector;

use self::net::NetProxy;

/// A proxy server. This must be constructed with a [`ProxyBuilder`].
///
/// # Examples
///
/// ```rust
/// use hudsucker::Proxy;
/// # use rustls_pemfile as pemfile;
/// # use tokio_rustls::rustls;
/// #
/// # #[cfg(all(feature = "rcgen-ca", feature = "rustls-client"))]
/// # #[tokio::main]
/// # async fn main() {
/// # use hudsucker::certificate_authority::RcgenAuthority;
/// #
/// # let mut private_key_bytes: &[u8] = include_bytes!("../../examples/ca/hudsucker.key");
/// # let mut ca_cert_bytes: &[u8] = include_bytes!("../../examples/ca/hudsucker.cer");
/// # let private_key = rustls::PrivateKey(
/// #     pemfile::pkcs8_private_keys(&mut private_key_bytes)
/// #         .expect("Failed to parse private key")
/// #         .remove(0),
/// # );
/// # let ca_cert = rustls::Certificate(
/// #     pemfile::certs(&mut ca_cert_bytes)
/// #         .expect("Failed to parse CA certificate")
/// #         .remove(0),
/// # );
/// #
/// # let ca = RcgenAuthority::new(private_key, ca_cert, 1_000)
/// #     .expect("Failed to create Certificate Authority");
///
/// // let ca = ...;
///
/// let proxy = Proxy::builder()
///     .with_addr(std::net::SocketAddr::from(([127, 0, 0, 1], 0)))
///     .with_rustls_client()
///     .with_ca(ca)
///     .build();
///
/// let (stop, done) = tokio::sync::oneshot::channel();
///
/// tokio::spawn(proxy.start(async {
///     done.await.unwrap_or_default();
/// }));
///
/// // Do something else...
///
/// stop.send(()).unwrap();
/// # }
/// #
/// # #[cfg(not(all(feature = "rcgen-ca", feature = "rustls-client")))]
/// # fn main() {}
/// ```
// pub struct Proxy<C, CA, H, W> {
//     als: AddrListenerServer,
//     ca: Arc<CA>,
//     client: Client<C>,
//     http_handler: H,
//     websocket_handler: W,
//     websocket_connector: Option<Connector>,
// }

// impl Proxy<(), (), (), ()> {
//     /// Create a new [`ProxyBuilder`].
//     pub fn builder() -> ProxyBuilder<WantsAddr> {
//         ProxyBuilder::new()
//     }
// }

// impl<C, CA, H, W> Proxy<C, CA, H, W>
// where
//     C: Connect + Clone + Send + Sync + 'static,
//     CA: CertificateAuthority,
//     H: HttpHandler,
//     W: WebSocketHandler,
// {
//     /// Attempts to start the proxy server.
//     ///
//     /// # Errors
//     ///
//     /// This will return an error if the proxy server is unable to be started.
//     pub async fn start<F: Future<Output = ()>>(self, shutdown_signal: F) -> Result<(), Error> {
//         let make_service = make_service_fn(move |conn: &AddrStream| {
//             let client = self.client.clone();
//             let ca = Arc::clone(&self.ca);
//             let http_handler = self.http_handler.clone();
//             let websocket_handler = self.websocket_handler.clone();
//             let websocket_connector = self.websocket_connector.clone();

//             let client_addr = conn.remote_addr();
//             async move {
//                 Ok::<_, Infallible>(service_fn(move |req| {
//                     InternalProxy {
//                         ca: Arc::clone(&ca),
//                         client: client.clone(),
//                         http_handler: http_handler.clone(),
//                         websocket_handler: websocket_handler.clone(),
//                         websocket_connector: websocket_connector.clone(),
//                         client_addr,
//                     }
//                     .proxy(req)
//                 }))
//             }
//         });

//         let server_builder = match self.als {
//             AddrListenerServer::Addr(addr) => Server::try_bind(&addr)?
//                 .http1_preserve_header_case(true)
//                 .http1_title_case_headers(true),
//             AddrListenerServer::Listener(listener) => Server::from_tcp(listener)?
//                 .http1_preserve_header_case(true)
//                 .http1_title_case_headers(true),
//             AddrListenerServer::Server(server) => *server,
//         };

//         server_builder
//             .serve(make_service)
//             .with_graceful_shutdown(shutdown_signal)
//             .await
//             .map_err(Into::into)
//     }
// }
#[derive(Debug)]

pub enum AddrListenerServer {
    Addr(SocketAddr),
    #[allow(unused)]
    Listener(TcpListener),
    #[allow(unused)]
    Server(Box<hyper::server::Builder<AddrIncoming>>),
}

pub struct CustomProxy<CA, H, W> {
    ca: Arc<CA>,
    als: AddrListenerServer,
    http_handler: H,
    websocket_handler: W,
    websocket_connector: Option<Connector>,
}
impl<CA, H, W> CustomProxy<CA, H, W>
where
    CA: CertificateAuthority,
    H: HttpHandler,
    W: WebSocketHandler,

{
    pub fn new(
        ca: Arc<CA>,
        als: AddrListenerServer,
        http_handler: H,
        websocket_handler: W,
        websocket_connector: Option<Connector>,
    ) -> Self {
        Self {
            ca,
            als,
            http_handler,
            websocket_handler,
            websocket_connector,
        }
    }

    pub async fn start<F: Future<Output = ()>>(self, shutdown_signal: F) -> Result<(), Error> {
        let make_service = make_service_fn(move |conn: &AddrStream| {
            let ca = Arc::clone(&self.ca);
            let http_handler = (&self.http_handler).clone();
            let websocket_handler = self.websocket_handler.clone();
            let websocket_connector = self.websocket_connector.clone();

            let client_addr = conn.remote_addr();

            async move {
                let service = service_fn(move |req| {
                    let net_proxy = NetProxy {
                        ca: Arc::clone(&ca),
                        http_handler: http_handler.clone(),
                        websocket_handler: websocket_handler.clone(),
                        websocket_connector: websocket_connector.clone(),
                        client_addr: client_addr.clone(),
                    };
                    async { net_proxy.proxy(req).await }
                });
                Ok::<_, Infallible>(service)
            }
        });

        let server_builder = match self.als {
            AddrListenerServer::Addr(addr) => Server::try_bind(&addr)?
                .http1_preserve_header_case(true)
                .http1_title_case_headers(true),
            AddrListenerServer::Listener(listener) => Server::from_tcp(listener)?
                .http1_preserve_header_case(true)
                .http1_title_case_headers(true),
            AddrListenerServer::Server(server) => *server,
        };

        server_builder
            .serve(make_service)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(Into::into)
    }
}
