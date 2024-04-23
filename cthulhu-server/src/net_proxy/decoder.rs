use crate::net_proxy::Error;
use async_compression::tokio::bufread::{
    BrotliDecoder, BrotliEncoder, GzipDecoder, GzipEncoder, ZlibDecoder, ZlibEncoder, ZstdDecoder,
    ZstdEncoder,
};

use bstr::ByteSlice;
use bytes::Bytes;
use futures::Stream;
use hyper::{
    header::{HeaderMap, HeaderValue, CONTENT_ENCODING, CONTENT_LENGTH},
    Request, Response,
};
use hyper::{Body, Error as HyperError};
use std::{
    io,
    io::Error as IoError,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncBufRead, AsyncRead, BufReader};
use tokio_util::{
    bytes,
    io::{ReaderStream, StreamReader},
};
struct IoStream<T: Stream<Item = Result<Bytes, HyperError>> + Unpin>(T);

impl<T: Stream<Item = Result<Bytes, HyperError>> + Unpin> Stream for IoStream<T> {
    type Item = Result<Bytes, IoError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        
        match futures::ready!(Pin::new(&mut self.0).poll_next(cx)) {
            Some(Ok(chunk)) => Poll::Ready(Some(Ok(chunk))),
            Some(Err(err)) => Poll::Ready(Some(Err(IoError::new(io::ErrorKind::Other, err)))),
            None => Poll::Ready(None),
        }
    }
}

enum Decoder {
    Body(Body),
    Decoder(Box<dyn AsyncRead + Send + Unpin>),
}

impl Decoder {
    pub fn decode(self, encoding: &[u8]) -> Result<Self, Error> {
        if encoding == b"identity" {
            return Ok(self);
        }

        let reader: Box<dyn AsyncBufRead + Send + Unpin> = match self {
            Self::Body(body) => Box::new(StreamReader::new(IoStream(body))),
            Self::Decoder(decoder) => Box::new(BufReader::new(decoder)),
        };

        // decoder::Decoder
        let decoder: Box<dyn AsyncRead + Send + Unpin> = match encoding {
            b"gzip" | b"x-gzip" => Box::new(GzipDecoder::new(reader)),
            b"deflate" => Box::new(ZlibDecoder::new(reader)),
            b"br" => Box::new(BrotliDecoder::new(reader)),
            b"zstd" => Box::new(ZstdDecoder::new(reader)),
            _ => return Err(Error::Decode),
        };

        Ok(Self::Decoder(decoder))
    }
}
enum Encoder {
    Body(Body),
    Encoder(Box<dyn AsyncRead + Send + Unpin>),
}
impl Encoder {
    pub fn encode(self, encoding: &[u8]) -> Result<Self, Error> {
        let reader: Box<dyn AsyncBufRead + Send + Unpin> = match self {
            Self::Body(body) => Box::new(StreamReader::new(IoStream(body))),
            Self::Encoder(encoder) => Box::new(BufReader::new(encoder)),
        };

        let encoder: Box<dyn AsyncRead + Send + Unpin> = match encoding {
            b"gzip" | b"x-gzip" => Box::new(GzipEncoder::new(reader)),
            b"deflate" => Box::new(ZlibEncoder::new(reader)),
            b"br" => Box::new(BrotliEncoder::new(reader)),
            b"zstd" => Box::new(ZstdEncoder::new(reader)),
            _ => return Err(Error::Decode),
        };

        Ok(Self::Encoder(encoder))
    }
}
impl From<Decoder> for Body {
    fn from(decoder: Decoder) -> Body {
        match decoder {
            Decoder::Body(body) => body,
            Decoder::Decoder(decoder) => Body::wrap_stream(ReaderStream::new(decoder)),
        }
    }
}

impl From<Encoder> for Body {
    fn from(encoder: Encoder) -> Body {
        match encoder {
            Encoder::Body(body) => body,
            Encoder::Encoder(encoder) => Body::wrap_stream(ReaderStream::new(encoder)),
        }
    }
}

fn extract_encodings(headers: &HeaderMap<HeaderValue>) -> impl Iterator<Item = &[u8]> {
    headers
        .get_all(CONTENT_ENCODING)
        .iter()
        .rev()
        .flat_map(|val| val.as_bytes().rsplit_str(b",").map(|v| v.trim()))
}

fn decode_body<'a>(
    encodings: impl IntoIterator<Item = &'a [u8]>,
    body: Body,
) -> Result<Body, Error> {
    let mut decoder = Decoder::Body(body);

    for encoding in encodings {
        decoder = decoder.decode(encoding)?;
    }

    Ok(decoder.into())
}

pub fn encode_body<'a>(encoding: &str, body: Body) -> Result<Body, Error> {
    let mut encoder = Encoder::Body(body);
    {
        encoder = encoder.encode(encoding.as_bytes())?;
    }

    Ok(encoder.into())
}
/// Decode the body of a request.
///
/// # Errors
///
/// This will return an error if either of the `content-encoding` or `content-length` headers are
/// unable to be parsed, or if one of the values specified in the `content-encoding` header is not
/// supported.
///
/// # Examples
///
/// ```rust
/// use hudsucker::{
///     async_trait::async_trait,
///     decode_request,
///     hyper::{Body, Request, Response},
///     Error, HttpContext, HttpHandler, RequestOrResponse,
/// };
///
/// #[derive(Clone)]
/// pub struct MyHandler;
///
/// #[async_trait]
/// impl HttpHandler for MyHandler {
///     async fn handle_request(
///         &mut self,
///         _ctx: &HttpContext,
///         req: Request<Body>,
///     ) -> RequestOrResponse {
///         let req = decode_request(req).unwrap();
///
///         // Do something with the request
///
///         RequestOrResponse::Request(req)
///     }
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "decoder")))]
pub fn decode_request(mut req: Request<Body>) -> Result<Request<Body>, Error> {
    if !req.headers().contains_key(CONTENT_ENCODING) {
        return Ok(req);
    }

    if let Some(val) = req.headers_mut().remove(CONTENT_LENGTH) {
        if val == "0" {
            return Ok(req);
        }
    }

    let (mut parts, body) = req.into_parts();

    let body = {
        let encodings = extract_encodings(&parts.headers);
        decode_body(encodings, body)?
    };

    parts.headers.remove(CONTENT_ENCODING);

    Ok(Request::from_parts(parts, body))
}
#[allow(unused)]
pub fn encode_request(encoding: &str, mut req: Request<Body>) -> Result<Request<Body>, Error> {
    if let Some(val) = req.headers_mut().remove(CONTENT_LENGTH) {
        if val == "0" {
            return Ok(req);
        }
    }

    let (mut parts, body) = req.into_parts();

    let body = encode_body(encoding, body)?;
    parts
        .headers
        .append(CONTENT_ENCODING, HeaderValue::from_str(encoding).unwrap());

    Ok(Request::from_parts(parts, body))
}

/// Decode the body of a response.
///
/// # Errors
///
/// This will return an error if either of the `content-encoding` or `content-length` headers are
/// unable to be parsed, or if one of the values specified in the `content-encoding` header is not
/// supported.
///
/// # Examples
///
/// ```rust
/// use hudsucker::{
///     async_trait::async_trait,
///     decode_response,
///     hyper::{Body, Request, Response},
///     Error, HttpContext, HttpHandler, RequestOrResponse,
/// };
///
/// #[derive(Clone)]
/// pub struct MyHandler;
///
/// #[async_trait]
/// impl HttpHandler for MyHandler {
///     async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
///         let res = decode_response(res).unwrap();
///
///         // Do something with the response
///
///         res
///     }
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "decoder")))]
pub fn decode_response(mut res: Response<Body>) -> Result<Response<Body>, Error> {
    if !res.headers().contains_key(CONTENT_ENCODING) {
        return Ok(res);
    }

    if let Some(val) = res.headers_mut().remove(CONTENT_LENGTH) {
        if val == "0" {
            return Ok(res);
        }
    }

    let (mut parts, body) = res.into_parts();

    let body = {
        let encodings = extract_encodings(&parts.headers);
        decode_body(encodings, body)?
    };

    parts.headers.remove(CONTENT_ENCODING);
    parts.headers.remove(CONTENT_LENGTH);

    Ok(Response::from_parts(parts, body))
}
pub fn encode_response(encoding: &str, mut res: Response<Body>) -> Result<Response<Body>, Error> {
    if let Some(val) = res.headers_mut().remove(CONTENT_LENGTH) {
        if val == "0" {
            return Ok(res);
        }
    }

    let (mut parts, body) = res.into_parts();

    let body = encode_body(encoding, body)?;

    parts
        .headers
        .append(CONTENT_ENCODING, HeaderValue::from_str(encoding).unwrap());

    Ok(Response::from_parts(parts, body))
}

