use crate::Ctx;
use crate::hooks::use_default;
use brick::{Image, ImageAttr};
use leptos::prelude::*;
use leptos::html::*;

/// 图片：`src` 取 `bind["value"].default`，尺寸/描述取 `ImageAttr`。
pub fn image_(brick: Image, _ctx: &Ctx) -> AnyView {
    if let Some(src) = use_default(&brick)
        && let Some(src) = src.as_str()
        && let Some(x) = brick.attrs
    {
        let ImageAttr { desc, .. } = &x;
        let style = x.size_style();
        let desc = desc.clone().unwrap_or_default();
        img()
            .src(src)
            .alt(desc.as_str())
            .style(style)
            .into_any()
    } else {
        div().into_any()
    }
}
