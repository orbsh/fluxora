use crate::Ctx;
use crate::ctx::render_children;
use crate::hooks::use_common_css;
use brick::Float;
use leptos::prelude::*;
use leptos::html::*;

/// 浮动容器：`PositionAttr::into_style()` 定位样式 + 公共 CSS。
pub fn float_(brick: Float, ctx: &Ctx) -> AnyView {
    let mut css = vec!["float", "f"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");
    let style = brick
        .attrs
        .as_ref()
        .map(|x| x.into_style())
        .unwrap_or_default();
    let children = brick
        .sub
        .as_deref()
        .map(|s| render_children(ctx, s))
        .unwrap_or_default();
    div().class(css.as_str()).style(style).child(children).into_any()
}
