use crate::Ctx;
use brick::Render;
use leptos::prelude::*;
use leptos::html::*;

/// 占位渲染：无内容。
pub fn render_(_brick: Render, _ctx: &Ctx) -> AnyView {
    div().into_any()
}
