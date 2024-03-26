use std::io;

use chrono::Local;
use rquickjs::class::Trace;
use rquickjs::function::Rest;
use rquickjs::{Class, Ctx};

use crate::jsbind::js_to_string;

use crate::{auto_result, utils as rs_utils};

#[rquickjs::class(rename = "Console")]
#[derive(Clone, Trace)]
pub struct JsConsole {
    pub path: String,
}

#[rquickjs::methods]
impl JsConsole {
    #[qjs(skip)]
    pub fn write_to_log(&self, level: &str, mut msg: String) -> io::Result<()> {
        let mut time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        time.push_str(" ");
        msg.insert_str(0, time.as_str());
        let path = format!("{}/LOGS/{}.log", self.path, level);
        msg.push('\n');
    
        rs_utils::write_bytes(path.as_str(), msg.as_bytes(), Some(true))
    }
    #[qjs(skip)]
    fn write<'js>(
        &self,
        level: &str,
        msgs: Rest<rquickjs::Value<'js>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<()> {
        let msgs = msgs.into_inner();
        let mut spans = vec![];
        for msg in msgs {
            let msg_str = js_to_string(msg)?;
            spans.push(msg_str);
        }
        let msg_str = spans.join(", ");
        auto_result!(self.write_to_log(level,msg_str),err=>{
         return Err(ctx.throw(
             rquickjs::String::from_str(ctx.clone(), err.to_string().as_str())
                 .unwrap()
                 .into(),
         ));
        });
        Ok(())
    }
    #[qjs()]
    pub fn log<'js>(&self, msgs: Rest<rquickjs::Value<'js>>, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        self.write("info", msgs, ctx)
    }
    #[qjs()]
    pub fn error<'js>(&self, msgs: Rest<rquickjs::Value<'js>>, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        self.write("error", msgs, ctx)
    }
    #[qjs()]
    pub fn warn<'js>(&self, msgs: Rest<rquickjs::Value<'js>>, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        self.write("warn", msgs, ctx)
    }
    #[qjs()]
    pub fn debug<'js>(&self, msgs: Rest<rquickjs::Value<'js>>, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        self.write("debug", msgs, ctx)
    }
}
pub fn init_def(path: String, ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let console = JsConsole { path };
    let cls = Class::instance(ctx.clone(), console)?;
    ctx.globals().set("console", cls)?;
    Ok(())
}
