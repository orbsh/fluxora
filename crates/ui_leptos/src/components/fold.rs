use crate::Ctx;
use crate::ctx::{render_brick, render_children};
use crate::hooks::{use_common_css, use_default};
use brick::{Fold, FoldAttr};
use leptos::ev::click;
use leptos::prelude::*;
use leptos::html::*;

/// 折叠容器：`item[0]` 作头部，`show` 信号控制展开/收起。
pub fn fold_(brick: Fold, ctx: &Ctx, id: String) -> AnyView {
    let ctx = ctx.clone();
    let mut css = vec!["g"];
    css.push(id.as_str());
    use_common_css(&mut css, &brick);
    let css = css.join(" ");

    let (replace_header, _float_body) = brick
        .attrs
        .as_ref()
        .map(
            |FoldAttr {
                 replace_header,
                 float_body,
                 ..
             }| (replace_header.unwrap_or(false), float_body.unwrap_or(false)),
        )
        .unwrap_or((false, false));

    let item = brick.item.as_ref().and_then(|i| i.first()).cloned();
    let show = RwSignal::new(
        use_default(&brick)
            .and_then(|x| x.as_bool())
            .unwrap_or_default(),
    );

    move || -> AnyView {
        let s = show.get();
        let onclick = move |_: web_sys::MouseEvent| {
            show.set(!show.get());
        };

        let h: AnyView = if replace_header && s {
            div().into_any()
        } else if let Some(item) = &item {
            render_brick(&ctx, item)
        } else {
            div().into_any()
        };

        let b: AnyView = if s {
            let children = brick
                .sub
                .as_deref()
                .map(|x| render_children(&ctx, x))
                .unwrap_or_default();
            div().child(children).into_any()
        } else {
            div().into_any()
        };

        let icon_class = if s { "icon open" } else { "icon close " };
        div()
            .id(id.as_str())
            .class(css.as_str())
            .style("grid-template-columns: auto 1fr;".to_string())
            .on(click, onclick)
            .child(div().class(icon_class).style("height: 100%; aspect-ratio: 1 / 1;".to_string()))
            .child(h)
            .child(div())
            .child(b)
            .into_any()
    }
    .into_any()
}
