use crate::Ctx;
use crate::ctx::render_children;
use crate::hooks::{use_common_css, use_default};
use brick::{Group, Path, Svg};
use leptos::svg;
use leptos::prelude::*;
use leptos::html::*;

/// SVG 容器：`SizeAttr::size_style()` 尺寸 + 公共 CSS。
pub fn svg_(brick: Svg, ctx: &Ctx) -> AnyView {
    let mut css = vec!["svg"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");
    let style = brick
        .attrs
        .as_ref()
        .map(|x| x.size_style())
        .unwrap_or_default();
    let children = brick
        .sub
        .as_deref()
        .map(|s| render_children(ctx, s))
        .unwrap_or_default();
    svg::svg()
        .class(css.as_str())
        .style(style)
        .child(children)
        .into_any()
}

/// SVG 分组：`StyleAttr.style` 内联样式。
pub fn group_(brick: Group, ctx: &Ctx) -> AnyView {
    let mut css = vec!["group"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");
    let mut style = String::new();
    if let Some(x) = &brick.attrs
        && let Some(s) = &x.style
    {
        style = s
            .iter()
            .map(|(k, v)| format!("{}: {};", k, v))
            .collect::<Vec<String>>()
            .join("\n");
    }
    let children = brick
        .sub
        .as_deref()
        .map(|s| render_children(ctx, s))
        .unwrap_or_default();
    svg::g().class(css.as_str()).style(style).child(children).into_any()
}

/// SVG 路径：`d` 取 `bind["value"].default`。
pub fn path_(brick: Path, _ctx: &Ctx) -> AnyView {
    let mut css = vec!["path"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");
    if let Some(x) = use_default(&brick)
        && let Some(d) = x.as_str()
    {
        let d = d.to_string();
        svg::path()
            .class(css.as_str())
            .attr("d", d)
            .into_any()
    } else {
        div().into_any()
    }
}
