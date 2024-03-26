use std::time::Duration;

use rquickjs::function::Func;
use rquickjs::{Class, Ctx, Function};

use rquickjs::class::Trace;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

#[derive(Trace, Clone)]
#[rquickjs::class(rename = "CancellationToken")]
pub struct CancellationTokenWrapper {
    #[qjs(skip_trace)]
    pub token: CancellationToken,
}

#[rquickjs::methods]
impl CancellationTokenWrapper {
    pub fn cancel(&self) {
        self.token.cancel()
    }
}

#[rquickjs::function(rename = "setInterval")]
pub fn set_interval<'js>(
    func: Function<'js>,
    delay: Option<usize>,
    immediately: Option<bool>,
    ctx: Ctx<'js>,
) -> rquickjs::Result<CancellationTokenWrapper> {
    let delay = delay.unwrap_or(0) as u64;
    let duration = Duration::from_millis(delay);
    let mut interval = time::interval(duration);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
    let token = CancellationToken::new();
    ctx.spawn({
        let token = token.clone();
        async move {
            let immediately = immediately.unwrap_or(false);
            if immediately {
                func.call::<_, ()>(()).unwrap();
            }
            // ignore the first tick
            // let _ = interval.tick().await;
            loop {
                select! {
                    _ = token.cancelled() => {
                        // Token被取消
                        // println!("Token被取消");
                        return ;
                    }
                   _ = interval.tick() => {
                        func.call::<_, ()>(()).unwrap();
                    }
                }
            }
        }
    });

    Ok(CancellationTokenWrapper { token })
}

#[rquickjs::function(rename = "clearInterval")]
pub fn clear_interval(token: CancellationTokenWrapper) {
    token.cancel();
}

#[rquickjs::function(rename = "setTimeout")]
pub fn set_timeout<'js>(
    func: Function<'js>,
    delay: Option<usize>,
    ctx: Ctx<'js>,
) -> rquickjs::Result<CancellationTokenWrapper> {
    let delay = delay.unwrap_or(0) as u64;
    let duration = Duration::from_millis(delay);
    let token = CancellationToken::new();

    ctx.spawn({
        let token = token.clone();

        async move {
            // 等待取消或者很长时间
            select! {
                _ = token.cancelled() => {
                    // Token被取消
                    // println!("Token被取消");
                }
                _ = tokio::time::sleep(duration) => {
                    func.call::<_, ()>(()).unwrap();
                }
            }
        }
    });

    Ok(CancellationTokenWrapper { token })
}

#[rquickjs::function(rename = "clearTimeout")]
pub fn clear_timeout(token: CancellationTokenWrapper) {
    token.cancel();
}


pub fn init_def(_id: &str, ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    Class::<'_,CancellationTokenWrapper>::define(&globals)?;

    globals.set("setInterval", Func::new(set_interval))?;
    globals.set("clearInterval", Func::new(clear_interval))?;
    globals.set("setTimeout", Func::new(set_timeout))?;
    globals.set("clearTimeout", Func::new(clear_timeout))?;
    Ok(())
}
