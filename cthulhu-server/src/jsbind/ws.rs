use hyper_tungstenite::tungstenite::Message;
use rquickjs::function::Opt;
use rquickjs::{Class, Ctx, Result};

use rquickjs::class::Trace;

use tokio_tungstenite::tungstenite::protocol::frame::{
    coding::{Control, Data, OpCode},
    Frame, FrameHeader,
};

use crate::auto_result;

use super::throw_js_err;

fn to_opcode(opcode: String) -> std::result::Result<OpCode, &'static str> {
    let opcode = {
        let opcode = opcode.to_lowercase();
        let (typ, code) = opcode.split_once(":").unwrap_or(("data", "continue"));
        match typ {
            "data" => {
                let code = match code {
                    "continue" => Data::Continue,
                    "text" => Data::Text,
                    "binary" => Data::Binary,
                    _ => {
                        return Err("<invalid data opcode>");
                    }
                };
                OpCode::Data(code)
            }
            "control" => {
                let code = match code {
                    "close" => Control::Close,
                    "ping" => Control::Ping,
                    "pong" => Control::Pong,
                    _ => {
                        return Err("<invalid control opcode>");
                    }
                };
                OpCode::Control(code)
            }
            _ => {
                return Err("<invalid opcode type>");
            }
        }
    };
    Ok(opcode)
}

fn to_mask(mask: Vec<u8>) -> std::result::Result<[u8; 4], &'static str> {
    let boxed: Box<[u8; 4]> = auto_result!(
        mask.into_boxed_slice().try_into(),
        Err("Expected a array of length 4")
    );
    Ok(*boxed)
}

#[rquickjs::class(rename = "FrameHeader")]
#[derive(Debug, Trace, Clone)]
pub struct JsFrameHeader {
    /// Indicates that the frame is the last one of a possibly fragmented message.
    #[qjs(get, set, enumerable, configurable)]
    pub is_final: bool,
    /// Reserved for protocol extensions.
    #[qjs(get, set, enumerable, configurable)]
    pub rsv1: bool,
    /// Reserved for protocol extensions.
    #[qjs(get, set, enumerable, configurable)]
    pub rsv2: bool,
    /// Reserved for protocol extensions.
    #[qjs(get, set, enumerable, configurable)]
    pub rsv3: bool,

    /// WebSocket protocol opcode.
    #[qjs(skip_trace)]
    pub opcode: OpCode,
    /// A frame mask, if any.
    #[qjs(skip_trace)]
    pub mask: Option<[u8; 4]>,
}
#[rquickjs::methods]
impl JsFrameHeader {
    pub fn new(
        is_final: bool,
        rsv1: bool,
        rsv2: bool,
        rsv3: bool,
        opcode: String,
        mask: Opt<Vec<u8>>,
        ctx: rquickjs::Ctx<'_>,
    ) -> Result<Self> {
        let opcode = to_opcode(opcode).map_err(|e| throw_js_err(e, ctx.clone()))?;
        let mask = match mask.0 {
            Some(v) => {
                let mask = to_mask(v).map_err(|e| throw_js_err(e, ctx.clone()))?;
                Some(mask)
            }
            None => None,
        };
        Ok(Self {
            is_final,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            mask,
        })
    }
    // #[qjs(set, rename = "opcode", enumerable)]
    // pub fn set_opcode(&self, opcode: String, ctx: rquickjs::Ctx<'_>) -> Result<()> {
    //     let opcode = to_opcode(opcode).map_err(|e| throw_js_err(e, ctx.clone()))?;
    //     let ptr = std::ptr::addr_of_mut!(self.opcode);
    //     unsafe {
    //         *ptr = opcode;
    //     }
    //     Ok(())
    // }
    // #[qjs(set, enumerable, configurable, rename = "mask")]
    // pub fn set_mask(&self, mask: Vec<u8>, ctx: rquickjs::Ctx<'_>) -> Result<()> {
    //     let mask = to_mask(mask).map_err(|e| throw_js_err(e, ctx.clone()))?;
    //     let mask = Some(mask);
    //     let ptr = std::ptr::addr_of_mut!(self.mask);
    //     unsafe {
    //         *ptr = mask;
    //     }
    //     Ok(())
    // }
}

#[rquickjs::class(rename = "Message")]
#[derive(Debug, Trace, Clone)]
pub struct JsMessage {
    #[qjs(skip_trace)]
    pub msg: Message,
}
#[rquickjs::methods]
impl JsMessage {
    #[qjs(constructor)]
    pub fn new(ctx: rquickjs::Ctx<'_>) -> Result<Self> {
        Err(throw_js_err("Illegal constructor",ctx))
    }
    #[qjs(static)]
    pub fn text(string: String) -> JsMessage {
        JsMessage {
            msg: Message::Text(string),
        }
    }
    #[qjs(static)]
    pub fn binary(bin: Vec<u8>) -> JsMessage {
        JsMessage {
            msg: Message::Binary(bin),
        }
    }
    #[qjs(static)]
    pub fn frame(header: JsFrameHeader, payload: Vec<u8>) -> Result<JsMessage> {
        let JsFrameHeader {
            is_final,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            mask,
        } = header;
        let header = FrameHeader {
            is_final,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            mask,
        };
        let frame = Frame::from_payload(header, payload);

        Ok(JsMessage {
            msg: Message::Frame(frame),
        })
    }
    #[qjs(rename = "type", get)]
    pub fn r#type(&self) -> String {
        match self.msg {
            Message::Text(_) => "text",
            Message::Binary(_) => "binary",
            Message::Ping(_) => "ping",
            Message::Pong(_) => "pong",
            Message::Close(_) => "close",
            Message::Frame(_) => "frame",
        }
        .into()
    }
    #[qjs(rename = "toText")]
    pub fn to_text<'js>(&self, ctx: Ctx<'js>) -> Result<String> {
        match self.msg.to_text() {
            Ok(v) => Ok(v.into()),
            Err(e) => {
                let s = e.to_string();
                let throw = ctx.throw(rquickjs::String::from_str(ctx.clone(), &s).unwrap().into());
                Err(throw)
            }
        }
    }
    #[qjs(rename = "toBytes")]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.msg.clone().into_data()
    }
    #[qjs(rename = "len")]
    pub fn len(&self) -> u64 {
        self.msg.len() as u64
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("{}", &self)
    }
}
impl std::fmt::Display for JsMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}
#[derive(Clone, Debug)]
pub enum WsAction {
    Ignore,
    Delay(JsMessage, u64), //延迟多少毫秒
    Respond(JsMessage),
    Release(JsMessage),
}
#[rquickjs::class(rename = "WsAction")]
#[derive(Debug, Clone, Trace)]
pub struct JsWsAction {
    #[qjs(skip_trace)]
    pub action: WsAction,
}
#[rquickjs::methods]
impl JsWsAction {
    #[qjs(constructor)]
    pub fn new(ctx: rquickjs::Ctx<'_>) -> Result<Self> {
        Err(throw_js_err("Illegal constructor",ctx))
    }
    #[qjs(static)]
    pub fn ignore() -> Self {
        Self {
            action: WsAction::Ignore,
        }
    }
    #[qjs(static)]
    pub fn respond(msg: JsMessage) -> Self {
        Self {
            action: WsAction::Respond(msg),
        }
    }
    #[qjs(static)]
    pub fn release(msg: JsMessage) -> Self {
        Self {
            action: WsAction::Release(msg),
        }
    }
    #[qjs(get)]
    pub fn name(&self) -> String {
        match &self.action {
            WsAction::Delay(_, _) => "delay",
            WsAction::Ignore => "ignore",
            WsAction::Respond(_) => "respond",
            WsAction::Release(_) => "release",
        }
        .into()
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("{}", &self)
    }
}
impl std::fmt::Display for JsWsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WsAction::")?;
        match &self.action {
            WsAction::Ignore => f.write_str("Ignore()"),
            WsAction::Delay(msg, ms) => {
                f.write_str("Delay(msg:\"")?;
                f.write_str(&msg.to_string())?;
                f.write_str("\", ms:")?;
                f.write_str(&ms.to_string())?;
                f.write_str(")")
            }
            WsAction::Respond(msg) => {
                f.write_str("Respond(msg:\"")?;
                f.write_str(&msg.to_string())?;
                f.write_str("\")")
            }
            WsAction::Release(msg) => {
                f.write_str("Release(msg:\"")?;
                f.write_str(&msg.to_string())?;
                f.write_str("\")")
            }
        }
    }
}

pub fn init_def<'js>(_id: &str, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    Class::<'js, JsMessage>::define(&globals)?;
    Class::<'js, JsWsAction>::define(&globals)?;
    Ok(())
}
