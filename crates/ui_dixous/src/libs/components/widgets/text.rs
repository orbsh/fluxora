use crate::libs::hooks::use_common_css;
use crate::libs::hooks::use_source_value;
use brick::{Text, TextAttr};
use dioxus::prelude::*;
use markdown::{Options, to_html_with_options};
use std::sync::LazyLock;

#[component]
pub fn text_(id: Option<String>, brick: Text) -> Element {
    let mut css = vec!["text"];
    if let Some(id) = &id {
        css.push(id);
    }
    use_common_css(&mut css, &brick);

    let text_content = if let Some(json_data) = use_source_value(&brick) {
        if json_data.is_string() {
            json_data.as_str().unwrap().to_owned()
        } else {
            json_data.to_string()
        }
    } else {
        "".to_string()
    };

    static MDFMT: LazyLock<Vec<String>> = LazyLock::new(|| {
        ["markdown", "md"]
            .iter()
            .map(|fmt| fmt.to_string())
            .collect()
    });

    if let Some(attrs) = &brick.attrs
        && let TextAttr {
            format: Some(text_format),
            ..
        } = attrs
        && (*MDFMT).contains(text_format)
    {
        let v = &text_content;
        if let Ok(md_html) = to_html_with_options(v, &Options::gfm()) {
            css.push("markdown");
            return rsx! {
                div {
                    class: css.join(" "),
                    dangerous_inner_html: md_html
                }
            };
        }
    };

    rsx! {
        div {
            class: css.join(" "),
            {text_content}
        }
    }
}
