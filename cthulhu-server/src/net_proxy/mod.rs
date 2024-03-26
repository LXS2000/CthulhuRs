#![cfg_attr(docsrs, feature(doc_cfg))]

//! Hudsucker is a MITM HTTP/S proxy that allows you to:
//!
//! - Modify HTTP/S requests
//! - Modify HTTP/S responses
//! - Modify WebSocket messages
//!
//! ## Features
//!
//! - `decoder`: Enables [`decode_request`] and [`decode_response`] helpers (enabled by default).
//! - `full`: Enables all features.
//! - `http2`: Enables HTTP/2 support.
//! - `native-tls-client`: Enables [`ProxyBuilder::with_native_tls_client`].
//! - `openssl-ca`: Enables [`certificate_authority::OpensslAuthority`].
//! - `rcgen-ca`: Enables [`certificate_authority::RcgenAuthority`] (enabled by default).
//! - `rustls-client`: Enables [`ProxyBuilder::with_rustls_client`] (enabled by default).

mod decoder;
mod error;
mod proxy;
mod rewind;

pub mod certificate_authority;

use futures::{Sink, SinkExt, Stream, StreamExt};
use hyper::Body;
use hyper::{Request, Response, StatusCode, Uri};

use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::{self, Message};
use tracing::error;
use error::Error;
pub use decoder::{decode_request, decode_response, encode_body, encode_response};

pub use proxy::*;

#[derive(Debug)]
pub enum Answer<Re, Rs> {
    #[allow(unused)]
    Reject,
    Release(Re),
    Respond(Rs),
}
impl From<Request<Body>> for Answer<Request<Body>, Response<Body>> {
    fn from(req: Request<Body>) -> Self {
        Self::Release(req)
    }
}

impl From<Response<Body>> for Answer<Request<Body>, Response<Body>> {
    fn from(res: Response<Body>) -> Self {
        Self::Respond(res)
    }
}

/// Context for HTTP requests and responses.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct HttpContext {
    /// Address of the client that is sending the request.
    pub client_addr: SocketAddr,
    pub uri: Uri,
}

/// Context for websocket messages.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum WebSocketContext {
    #[non_exhaustive]
    ClientToServer {
        /// Address of the client.
        src: SocketAddr,
        /// URI of the server.
        dst: Uri,
    },
    #[non_exhaustive]
    ServerToClient {
        /// URI of the server.
        src: Uri,
        /// Address of the client.
        dst: SocketAddr,
    },
}
impl WebSocketContext {
    pub fn addr(&self) -> SocketAddr {
        match self {
            WebSocketContext::ClientToServer { src, .. } => src,
            WebSocketContext::ServerToClient { dst, .. } => dst,
        }
        .clone()
    }
    pub fn uri(&self) -> Uri {
        match self {
            WebSocketContext::ClientToServer { dst, .. } => dst,
            WebSocketContext::ServerToClient { src, .. } => src,
        }
        .clone()
    }
    pub fn client_to_server(&self) -> bool {
        match self {
            WebSocketContext::ClientToServer { .. } => true,
            WebSocketContext::ServerToClient { .. } => false,
        }
    }
}

/// Handler for HTTP requests and responses.
///
/// Each request/response pair is passed to the same instance of the handler.
#[async_trait::async_trait]
pub trait HttpHandler: Clone + Send + Sync + 'static {
    /// This handler will be called for each HTTP request. It can either return a modified request,
    /// or a response. If a request is returned, it will be sent to the upstream server. If a
    /// response is returned, it will be sent to the client.
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> Answer<Request<Body>, Response<Body>> {
        req.into()
    }

    /// This handler will be called for each HTTP response. It can modify a response before it is
    /// forwarded to the client.
    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        res
    }

    /// This handler will be called if a proxy request fails. Default response is a 502 Bad Gateway.
    async fn handle_error(&mut self, _ctx: &HttpContext, err:String) -> Response<Body> {
        error!("Failed to forward request: {}", err);
        Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(Body::empty())
            .expect("Failed to build response")
    }

    /// Whether a CONNECT request should be intercepted. Defaults to `true` for all requests.
    async fn should_intercept(&mut self, _ctx: &HttpContext, _req: &Request<Body>) -> bool {
        true
    }
}

/// Handler for WebSocket messages.
///
/// Messages sent over the same WebSocket Stream are passed to the same instance of the handler.
#[async_trait::async_trait]
pub trait WebSocketHandler: Clone + Send + Sync + 'static {
    /// This handler is responsible for forwarding WebSocket messages from a Stream to a Sink and
    /// recovering from any potential errors.
    async fn handle_websocket(
        mut self,
        ctx: WebSocketContext,
        mut stream: impl Stream<Item = Result<Message, tungstenite::Error>> + Unpin + Send + 'static,
        src_sink: Arc<
            Mutex<impl Sink<Message, Error = tungstenite::Error> + Unpin + Send + 'static>,
        >,
        dst_sink: Arc<
            Mutex<impl Sink<Message, Error = tungstenite::Error> + Unpin + Send + 'static>,
        >,
    ) {
        while let Some(message) = stream.next().await {
            match message {
                Ok(message) => match self.handle_message(&ctx, message).await {
                    Answer::Reject => continue,
                    Answer::Release(msg) => {
                        let mut dst_sink = dst_sink.lock().await;
                        match dst_sink.send(msg).await {
                            Err(tungstenite::Error::ConnectionClosed) => break,
                            Err(e) => error!("WebSocket send error: {}", e),
                            _ => (),
                        }
                    }
                    Answer::Respond(msg) => {
                        let mut src_sink = src_sink.lock().await;
                        match src_sink.send(msg).await {
                            Err(tungstenite::Error::ConnectionClosed) => break,
                            Err(e) => error!("WebSocket send error: {}", e),
                            _ => (),
                        }
                    }
                },
                Err(tungstenite::Error::Protocol(e)) => {
                    error!("WebSocket message error: {}", e);
                }
                Err(e) => {
                    error!("WebSocket message error: {}", e);
                    let mut src_sink = src_sink.lock().await;
                    match src_sink.send(Message::Close(None)).await {
                        Err(tungstenite::Error::ConnectionClosed) => break,
                        Err(e) => error!("WebSocket send error: {}", e),
                        _ => (),
                    };
                    let mut dst_sink = dst_sink.lock().await;
                    match dst_sink.send(Message::Close(None)).await {
                        Err(tungstenite::Error::ConnectionClosed) => break,
                        Err(e) => error!("WebSocket send error: {}", e),
                        _ => (),
                    };
                    break;
                }
            }
        }
    }

    /// This handler will be called for each WebSocket message. It can return an optional modified
    /// message. If None is returned the message will not be forwarded.
    async fn handle_message(
        &mut self,
        _ctx: &WebSocketContext,
        message: Message,
    ) -> Answer<Message, Message> {
        Answer::Release(message)
    }
}
