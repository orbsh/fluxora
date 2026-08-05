use crate::Ctx;
use crate::ctx::render_brick;
use crate::hooks::use_common_css;
use brick::Popup;
use leptos::prelude::*;
use leptos::html::*;

/// 弹窗：sub[0] 为触发占位，sub[1] 为模态内容。
pub fn popup_(brick: Popup, ctx: &Ctx) -> AnyView {
    let mut css = vec!["popup", "f"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");
    let style = brick.attrs.as_ref().map(|x| x.into_style()).unwrap_or_default();

    if let Some(subs) = brick.sub.as_deref()
        && let Some(placeholder) = subs.first()
        && let Some(modal) = subs.get(1)
    {
        div()
            .class(css.as_str())
            .style(style)
            .child(div().class("f").child(render_brick(ctx, placeholder)))
            .child(div().class("f body").child(render_brick(ctx, modal)))
            .into_any()
    } else {
        div().into_any()
    }
}
