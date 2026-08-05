use crate::Ctx;
use crate::hooks::{use_common_css, use_source_value};
use brick::{Text, TextAttr};
use leptos::prelude::*;
use leptos::html::*;
use markdown::{Options, to_html_with_options};
use std::sync::LazyLock;

static MDFMT: LazyLock<Vec<String>> = LazyLock::new(|| {
    ["markdown", "md"]
        .iter()
        .map(|fmt| fmt.to_string())
        .collect()
});

/// 文本：`bind["value"].default` 取值；`TextAttr.format` 为 markdown/md 时渲染 HTML。
pub fn text_(brick: Text, ctx: &Ctx) -> AnyView {
    let ctx = ctx.clone();
    let mut css = vec!["text"];
    if let Some(id) = &brick.id {
        css.push(id);
    }
    use_common_css(&mut css, &brick);
    let base_css = css.join(" ");

    move || -> AnyView {
        let text_content = match use_source_value(&ctx, &brick) {
            Some(j) if j.is_string() => j.as_str().unwrap().to_owned(),
            Some(j) => j.to_string(),
            None => "".to_string(),
        };

        if let Some(TextAttr { format: Some(fmt), .. }) = &brick.attrs
            && MDFMT.contains(fmt)
            && let Ok(md_html) = to_html_with_options(&text_content, &Options::gfm())
        {
            div()
                .class(base_css.as_str())
                .class("markdown")
                .inner_html(md_html)
                .into_any()
        } else {
            div().class(base_css.as_str()).child(text_content).into_any()
        }
    }
    .into_any()
}
