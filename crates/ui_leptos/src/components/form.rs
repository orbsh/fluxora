use crate::Ctx;
use crate::ctx::render_children;
use crate::hooks::{push_form, pop_form, use_common_css, FormState};
use brick::{Bind, BindVariant, Brick, BrickOps, Form, JsType};
use leptos::prelude::*;
use leptos::html::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    pub data: Value,
    pub payload: Option<Value>,
}

fn default_for(kind: &Option<JsType>, default: &Option<Value>) -> Value {
    match kind {
        Some(JsType::number) => {
            to_value(default.as_ref().and_then(|x| x.as_f64()).unwrap_or(0.0)).unwrap()
        }
        Some(JsType::bool) => {
            to_value(default.as_ref().and_then(|x| x.as_bool()).unwrap_or(false)).unwrap()
        }
        _ => to_value(default.as_ref().and_then(|x| x.as_str()).unwrap_or("")).unwrap(),
    }
}

fn collect_fields(
    b: &Brick,
    fields: &mut HashMap<String, RwSignal<Value>>,
    payloads: &mut HashMap<String, Option<Value>>,
) {
    if let Some(Bind {
        default,
        r#type: kind,
        variant: BindVariant::Field { field, payload, .. },
    }) = b.get_bind().and_then(|x| x.get("value"))
    {
        let v = default_for(kind, default);
        fields
            .entry(field.clone())
            .or_insert_with(|| RwSignal::new(v));
        payloads.insert(field.clone(), payload.clone());
    }
    if let Some(subs) = b.borrow_sub() {
        for c in subs {
            collect_fields(c, fields, payloads);
        }
    }
}

/// 表单：收集 `Field` 字段信号，渲染子组件，`confirm` 为真时发送事件。
pub fn form_(brick: Form, ctx: &Ctx) -> AnyView {
    let ctx = ctx.clone();
    let mut css = vec!["case", "f"];
    if let Some(id) = &brick.id {
        css.push(id.as_str());
    }
    use_common_css(&mut css, &brick);
    let css = css.join(" ");

    let confirm = RwSignal::new(Value::Bool(false));
    let mut fields: HashMap<String, RwSignal<Value>> = HashMap::new();
    let mut payloads: HashMap<String, Option<Value>> = HashMap::new();
    if let Some(subs) = brick.sub.as_deref() {
        for c in subs {
            collect_fields(c, &mut fields, &mut payloads);
        }
    }

    let event = brick
        .get_bind()
        .and_then(|x| x.get("value"))
        .and_then(|b| match &b.variant {
            BindVariant::Event { event } => Some(event.clone()),
            _ => None,
        });

    // 渲染子组件（input_/button_ 会从栈顶取信号）
    push_form(FormState {
        fields: fields.clone(),
        confirm,
    });
    let children = brick
        .sub
        .as_deref()
        .map(|s| render_children(&ctx, s))
        .unwrap_or_default();
    pop_form();

    // confirm 为真时发送
    if let Some(event) = event {
        let ctx = ctx.clone();
        Effect::new(move |_| {
            if confirm.get().as_bool() == Some(true) {
                let ctx = ctx.clone();
                let content: HashMap<String, Message> = fields
                    .iter()
                    .map(|(k, sig)| {
                        let d = Message {
                            data: sig.get(),
                            payload: payloads.get(k).cloned().flatten(),
                        };
                        (k.clone(), d)
                    })
                    .collect();
                let val = to_value(content).unwrap_or(Value::Null);
                let ev = event.clone();
                leptos::task::spawn_local(async move {
                    ctx.send(ev, None, val).await;
                });
                confirm.set(Value::Bool(false));
            }
        });
    }

    div().class(css.as_str()).child(children).into_any()
}
