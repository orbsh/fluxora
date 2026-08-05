use crate::Ctx;
use crate::hooks::{use_common_css, use_source, use_source_value, use_target_value};
use brick::TextArea;
use leptos::ev;
use leptos::prelude::*;
use leptos::html::*;
use serde_json::{Value, to_value};
use wasm_bindgen::JsCast;

/// 多行输入：初值取 `bind["value"].default`，Enter 发送 `bind["value"]` 事件。
pub fn textarea_(brick: TextArea, ctx: &Ctx) -> AnyView {
    let ctx = ctx.clone();
    let mut css = vec!["textarea", "shadow"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");

    let slot = RwSignal::new(
        use_source_value(&ctx, &brick).unwrap_or_else(|| to_value("").unwrap()),
    );
    let placeholder = use_source(&ctx, &brick, "placeholder")
        .and_then(|d| d.as_str().map(String::from));

    move || -> AnyView {
        let ctx = ctx.clone();
        let brick = brick.clone();
        let p = placeholder.clone();
        let oninput = move |event: web_sys::Event| {
            let v = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                .map(|e| e.value())
                .unwrap_or_default();
            slot.set(to_value(v).unwrap());
        };
        let onkeydown = move |ev: web_sys::KeyboardEvent| {
            if ev.key() == "Enter" {
                let ctx = ctx.clone();
                let b = brick.clone();
                let val = slot.get();
                if let Some(emitter) = use_target_value(ctx.clone(), &b) {
                    emitter(val.clone());
                }
                slot.set(Value::Null);
            }
        };

        let val = slot.get().as_str().unwrap_or("").to_string();
        let base = input()
            .class(css.as_str())
            .value(val.as_str())
            .on(ev::input, oninput)
            .on(ev::keydown, onkeydown);
        match &p {
            Some(ph) => base.placeholder(ph.as_str()).into_any(),
            None => base.into_any(),
        }
    }
    .into_any()
}
