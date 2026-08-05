use crate::Ctx;
use crate::hooks::{peek_form, use_common_css};
use brick::{Bind, BindVariant, BrickOps, Input, JsType};
use leptos::ev;
use leptos::prelude::*;
use leptos::html::*;
use serde_json::{Value, to_value};
use wasm_bindgen::JsCast;

fn default_option_jskind(v: &Option<JsType>) -> Value {
    v.as_ref()
        .map(|x| x.default_value())
        .unwrap_or_else(|| to_value("").unwrap())
}

/// 输入框：`Field` 绑定写 form 字段信号；`Event` 绑定在 Enter 时发送事件。
pub fn input_(brick: Input, ctx: &Ctx) -> AnyView {
    let ctx = ctx.clone();
    let mut css = vec!["input", "f", "shadow"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");

    let (bind_type, key, kind) = brick
        .get_bind()
        .and_then(|x| x.get("value"))
        .cloned()
        .map(|x| match x {
            Bind {
                variant: BindVariant::Field { field, .. },
                r#type: kind,
                ..
            } => ("field", field, kind),
            Bind {
                variant: BindVariant::Event { event },
                r#type: kind,
                ..
            } => ("event", event, kind),
            _ => ("", "".to_string(), Default::default()),
        })
        .unwrap_or(("", "".to_string(), Default::default()));

    let field_sig = if bind_type == "field" {
        peek_form().and_then(|fs| fs.fields.get(&key).copied())
    } else {
        None
    };

    let slot = field_sig.unwrap_or_else(|| RwSignal::new(default_option_jskind(&kind)));

    move || -> AnyView {
        // 每次闭包重跑时重建事件回调（回调捕获 Copy 信号与 owned ctx）
        let ctx = ctx.clone();
        let k1 = kind.clone();
        let oninput = move |event: web_sys::Event| {
            let event_value: String = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|e| e.value())
                .unwrap_or_default();
            let parsed = match k1.as_ref() {
                Some(JsType::bool) => to_value(event_value == "true"),
                Some(JsType::number) => to_value(event_value.parse::<f64>().unwrap_or(0.0)),
                _ => to_value(event_value),
            }
            .unwrap();
            slot.set(parsed);
        };

        let k2 = kind.clone();
        let k3 = key.clone();
        let onkeydown = move |ev: web_sys::KeyboardEvent| {
            if ev.key() == "Enter" {
                match bind_type {
                    "field" => {
                        if let Some(sig) = field_sig {
                            sig.set(slot.get());
                        }
                    }
                    "event" => {
                        let ctx = ctx.clone();
                        let key = k3.clone();
                        let kk = k2.clone();
                        let val = slot.get();
                        leptos::task::spawn_local(async move {
                            ctx.send(key, None, val).await;
                            slot.set(default_option_jskind(&kk));
                        });
                    }
                    _ => {}
                }
            }
        };

        let v = slot.get();
        let val: String = match &v {
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => v.as_str().unwrap_or("").to_string(),
        };
        let ty = match &kind {
            Some(JsType::number) => "number",
            Some(JsType::bool) => "checkbox",
            Some(x) => x.input_type(),
            None => "text",
        };
        let base = input().class(css.as_str()).r#type(ty);
        match &kind {
            Some(JsType::bool) => base
                .checked(v.as_bool().unwrap_or(false))
                .on(ev::input, oninput)
                .on(ev::keydown, onkeydown)
                .into_any(),
            _ => base
                .value(val.as_str())
                .on(ev::input, oninput)
                .on(ev::keydown, onkeydown)
                .into_any(),
        }
    }
    .into_any()
}
