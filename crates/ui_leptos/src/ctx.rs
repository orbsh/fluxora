use crate::render::dispatch;
use crate::ws::WebSocketHandle;
use brick::{
    Brick, BrickOps,
    merge::{BrickOp, Concat, Delete, Replace},
};
use content::{Content, Message, Method, Outflow};
use leptos::prelude::*;
use message::codec::ActiveCodec;
use minijinja::Environment;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

static TMPL: LazyLock<RwLock<Environment>> = LazyLock::new(|| {
    let env = Environment::new();
    RwLock::new(env)
});

/// 全局共享状态容器：掌管布局、数据、列表与 WS 发送。
#[derive(Clone)]
pub struct Ctx {
    pub ws: WebSocketHandle,
    pub codec: ActiveCodec,
    pub layout: RwSignal<Brick>,
    pub data: RwSignal<HashMap<String, Brick>>,
    pub list: RwSignal<HashMap<String, Vec<Brick>>>,
}

impl Ctx {
    pub fn new(url: &str, codec: ActiveCodec) -> Self {
        let ws = WebSocketHandle::new(url);
        let layout = RwSignal::new(Brick::text(Default::default()));
        let data = RwSignal::new(HashMap::new());
        let list = RwSignal::new(HashMap::new());

        let ctx = Ctx {
            ws,
            codec,
            layout,
            data,
            list,
        };

        // 订阅 WS 消息并分发
        let bytes = ctx.ws.message_bytes();
        let ctx_clone = ctx.clone();
        Effect::new(move |_| {
            let b = bytes.get();
            if !b.is_empty() {
                if let Ok(act) = ctx_clone.codec.decode::<Message<Brick>>(&b) {
                    dispatch_msg(&act, &ctx_clone);
                }
            }
        });

        ctx
    }

    pub async fn send(&self, event: impl AsRef<str>, id: Option<String>, content: Value) {
        let x = Outflow {
            event: event.as_ref().to_string(),
            id,
            data: content,
        };
        if let Ok(buf) = self.codec.encode(&x) {
            let msg = match &self.codec {
                ActiveCodec::Json => gloo_net::websocket::Message::Text(
                    String::from_utf8(buf).unwrap_or_default(),
                ),
                ActiveCodec::Cbor => gloo_net::websocket::Message::Bytes(buf),
            };
            let _ = self.ws.send(msg).await;
        }
    }

    pub fn set(&self, name: impl AsRef<str>, brick: Brick) {
        self.data.update(|d| {
            d.insert(name.as_ref().to_string(), brick);
        });
    }
}

fn dispatch_msg(act: &Message<Brick>, ctx: &Ctx) {
    for c in &act.content {
        match c {
            Content::Tmpl(x) => {
                let n = x.name.clone();
                let d = x.data.clone();
                let _ = TMPL
                    .write()
                    .expect("write TMPL failed")
                    .add_template_owned(n, d);
            }
            Content::Create(x) => {
                let mut d = x.data.clone();
                let env = TMPL.read().expect("read TMPL failed");
                d.render(&env);
                ctx.layout.set(d);
            }
            Content::Set(x) => {
                let mut d = x.data.clone();
                let env = TMPL.read().expect("read TMPL failed");
                d.render(&env);
                ctx.data
                    .update(|m| {
                        m.insert(x.event.clone(), d);
                    });
            }
            Content::Join(x) => {
                let mut d = x.data.clone();
                let env = TMPL.read().expect("read TMPL failed");
                d.render(&env);
                let vs: &dyn BrickOp = match x.method {
                    Method::Replace => &Replace,
                    Method::Concat => &Concat,
                    Method::Delete => &Delete,
                };
                if d.get_id().is_some() {
                    let mut l = ctx.list.get();
                    let list = l.entry(x.event.clone()).or_default();
                    let mut is_merge = false;
                    for i in list.iter_mut() {
                        if i.cmp_id(&d) {
                            is_merge = true;
                            let mut rhs = d.clone();
                            i.merge(vs, &mut rhs);
                        }
                    }
                    if !is_merge {
                        list.push(d.clone());
                    }
                    ctx.list.set(l);
                } else {
                    ctx.list.update(|m| {
                        m.entry(x.event.clone()).or_default().push(d.clone());
                    });
                }
            }
            Content::Empty => {}
        }
    }
}

/// 渲染一个 brick 为视图（供 external 触发）。
pub fn render_brick(ctx: &Ctx, brick: &Brick) -> AnyView {
    dispatch(brick, ctx)
}

/// 渲染一组子 brick。
pub fn render_children(ctx: &Ctx, subs: &[Brick]) -> Vec<AnyView> {
    subs.iter().map(|b| dispatch(b, ctx)).collect()
}
