use base64::Engine;
use rquickjs::{class::Trace, Class, Ctx};

use crate::utils as rutils;

use super::{throw_js_err, to_js_err};

#[rquickjs::class(rename_all = "camelCase")]
#[derive(Debug, Trace, Clone)]
pub struct StrUtils {}
#[rquickjs::methods]
impl StrUtils {
    #[qjs(constructor)]
    pub fn new(ctx: rquickjs::Ctx<'_>) -> rquickjs::Result<Self> {
        Err(throw_js_err("Illegal constructor", ctx))
    }
    #[qjs(static)]
    pub fn hashstr(s: String, size: u8) -> String {
        rutils::hash(s.as_bytes(), size, 123)
    }
    #[qjs(static)]
    pub fn str_to_bytes(s: String) -> Vec<u8> {
        s.into_bytes()
    }
    #[qjs(static)]
    pub fn bytes_to_utf8(bytes: Vec<u8>) -> String {
        String::from_utf8(bytes).unwrap()
    }
    #[qjs(static)]
    pub fn mini_match(pattern: String, target: String) -> bool {
        rutils::mini_match(pattern.as_str(), target.as_str())
    }
    #[qjs(static)]
    pub fn url_encode(s: String) -> String {
        urlencoding::encode(&s).to_string()
    }
    #[qjs(static)]
    pub fn url_decode(s: String, ctx: Ctx<'_>) -> rquickjs::Result<String> {
        let s = urlencoding::decode(&s).map_err(|e| to_js_err(e, ctx))?;
        Ok(s.to_string())
    }
    #[qjs(static)]
    pub fn base64_encode(bytes: Vec<u8>, _ctx: Ctx<'_>) -> rquickjs::Result<String> {
        let engine = base64::engine::general_purpose::STANDARD;
        let mut buf = String::new();
        engine.encode_string(&bytes, &mut buf);
        Ok(buf)
    }
    #[qjs(static)]
    pub fn base64_decode(s: String, ctx: Ctx<'_>) -> rquickjs::Result<Vec<u8>> {
        let engine = base64::engine::general_purpose::STANDARD;
        let mut buf = vec![];
        engine
            .decode_vec(&s, &mut buf)
            .map_err(|e| to_js_err(e, ctx))?;
        Ok(buf)
    }
}

pub fn init_def(_id: &str, ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    Class::<'_, StrUtils>::define(&globals)?;
    Ok(())
}
