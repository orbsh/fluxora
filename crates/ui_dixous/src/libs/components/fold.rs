use super::Frame;
use crate::libs::hooks::{use_common_css, use_default};
use brick::{Fold, FoldAttr};
use dioxus::prelude::*;

#[component]
pub fn fold_(id: Option<String>, brick: Fold, children: Element) -> Element {
    let mut css = vec!["g"];
    if let Some(id) = &id {
        css.push(id);
    }
    use_common_css(&mut css, &brick);

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
        .unwrap();

    let item = brick.item.as_ref().context("item")?[0].clone();
    let show = use_signal(|| {
        use_default(&brick)
            .and_then(|x| x.as_bool())
            .unwrap_or_default()
    });

    let onclick = move |_event| {
        let mut s = show;
        s.set(!show());
    };

    let h = if replace_header && show() {
        rsx! { div {} }
    } else {
        rsx! {
            div {
                Frame { brick: item }
            }
        }
    };
    let b = if show() {
        rsx! {
            div {
                {children}
            }
        }
    } else {
        rsx! { div {} }
    };

    let icon_style = r#"
        height: 100%;
        aspect-ratio: 1 / 1;
    "#;
    let grid_style = r#"
        grid-template-columns: auto 1fr;
        "#;
    let icon_class = if show() { "icon open" } else { "icon close " };
    rsx! {
        div {
            class: css.join(" "),
            style: "{grid_style}",
            onclick: onclick,
            div {
                class: icon_class,
                style: "{icon_style}",
            },
            {h},
            div {},
            {b}
        }
    }
}
