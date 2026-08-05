use crate::Ctx;
use crate::ctx::render_brick;
use crate::ctx::render_children;
use crate::hooks::use_common_css;
use brick::{BindVariant, BrickOps, Case, CaseAttr, Placeholder};
use leptos::prelude::*;
use leptos::html::*;

/// 容器：`case` class + grid 样式 + 公共 CSS。
pub fn case_(brick: Case, ctx: &Ctx) -> AnyView {
    let mut css = vec!["case", "f"];
    if let Some(id) = &brick.id {
        css.push(id.as_str());
    }
    let mut f = true;
    let mut style = String::new();
    if let Some(CaseAttr { grid, .. }) = &brick.attrs {
        if let Some(g) = grid {
            f = false;
            css.push("g");
            style = g
                .iter()
                .map(|(k, v)| format!("{}: {};", k, v.as_str().unwrap_or("")))
                .collect::<Vec<String>>()
                .join("\n");
        }
        if f {
            css.push("f");
        }
    }
    use_common_css(&mut css, &brick);
    let css = css.join(" ");

    let children = brick
        .sub
        .as_deref()
        .map(|s| render_children(ctx, s))
        .unwrap_or_default();
    div()
        .class(css.as_str())
        .style(style)
        .child(children)
        .into_any()
}

/// 占位符：绑定 `Source` 时从 `ctx.data` 取源渲染并做淡入淡出，否则渲染 children。
pub fn placeholder_(brick: Placeholder, ctx: &Ctx, id: String) -> AnyView {
    let ctx = ctx.clone();
    let mut css = vec!["placeholder", "f"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");
    let id_ = id.clone();

    let source = brick
        .get_bind()
        .and_then(|x| x.get("value"))
        .and_then(|b| match &b.variant {
            BindVariant::Source { source } => Some(source.clone()),
            _ => None,
        });

    move || -> AnyView {
        if let Some(source) = &source
            && let Some(data) = ctx.data.get().get(source).cloned()
        {
            crate::dom::eval(&format!(
                r#"
                let x = document.getElementById('{id_extra}');
                x.classList.add('fade-in-and-out');
                setTimeout(() => x.classList.remove('fade-in-and-out'), 1000);
                "#,
                id_extra = id_
            ));
            let ctx = ctx.clone();
            div()
                .id(id_.as_str())
                .class(css.as_str())
                .child(render_brick(&ctx, &data))
                .into_any()
        } else {
            let children = brick
                .sub
                .as_deref()
                .map(|s| render_children(&ctx, s))
                .unwrap_or_default();
            div()
                .id(id_.as_str())
                .class(css.as_str())
                .child(children)
                .into_any()
        }
    }
    .into_any()
}
