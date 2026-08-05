pub mod ctx;
pub mod render;
pub mod hooks;
pub mod dom;
pub mod ws;

pub mod components;
pub mod widgets;

pub use ctx::Ctx;
use leptos::prelude::Get;

use message::codec::{ActiveCodec, CodecType};
use wasm_bindgen::JsCast;

/// 入口：解析 `#main` 的 host/token/codec，建立 WS，挂载响应式根。
pub fn mount() {
    let doc = web_sys::window().unwrap().document().unwrap();
    let loc = doc.location().unwrap();
    let mut host = "".to_owned();
    let mut token = None;
    let mut codec_type = CodecType::Cbor;

    if let Ok(Some(ele)) = doc.query_selector("#main") {
        if let Some(h) = ele.get_attribute("data-host") {
            host = h;
        } else {
            host = loc.host().unwrap();
        }

        if let Ok(href) = loc.href()
            && let Ok(href) = web_sys::Url::new(&href)
        {
            if let Some(t) = href.search_params().get("token") {
                token = Some(t);
            }
            if let Some(codec_str) = href.search_params().get("codec") {
                if let Ok(t) = codec_str.parse::<CodecType>() {
                    codec_type = t;
                }
            }
        } else if let Some(t) = ele.get_attribute("data-token") {
            token = Some(t);
        }
    }

    let codec_str = match codec_type {
        CodecType::Json => "json",
        CodecType::Cbor => "cbor",
    };
    let query = if let Some(token) = token {
        format!("?token={}&codec={}", token, codec_str)
    } else {
        format!("?codec={}", codec_str)
    };
    let url = format!("ws://{}/channel{}", host, query);

    let parent = doc
        .query_selector("#main")
        .ok()
        .flatten()
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        .expect("no #main element to mount into");

    let _ = leptos::mount::mount_to(parent, move || {
        let ctx = Ctx::new(&url, ActiveCodec::new(codec_type));
        let ctx = ctx.clone();
        // 响应式根：仅当 layout 变化时重建
        move || render::dispatch(&ctx.layout.get(), &ctx)
    });
}
