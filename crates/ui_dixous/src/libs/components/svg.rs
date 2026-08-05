use crate::libs::hooks::{use_common_css, use_default};
use brick::{Group, Path, Svg};
use dioxus::prelude::*;

#[component]
pub fn svg_(id: Option<String>, brick: Svg, children: Element) -> Element {
    let mut css = vec!["svg"];
    use_common_css(&mut css, &brick);

    let style = brick
        .attrs
        .as_ref()
        .map(|x| x.size_style())
        .unwrap_or("".to_string());
    rsx! {
        svg {
            style: style,
            class: css.join(" "),
            {children}
        }
    }
}

#[component]
pub fn group_(id: Option<String>, brick: Group, children: Element) -> Element {
    let mut css = vec!["group"];
    use_common_css(&mut css, &brick);

    let mut style = String::new();
    if let Some(x) = &brick.attrs
        && let Some(s) = &x.style
    {
        style = s
            .iter()
            .map(|(k, v)| format!("{}: {};", k, v.as_str()))
            .collect::<Vec<String>>()
            .join("\n");
    };
    rsx! {
        g {
            class: css.join(" "),
            style: style,
            {children}
        }
    }
}

#[component]
pub fn path_(id: Option<String>, brick: Path, children: Element) -> Element {
    let mut css = vec!["path"];
    use_common_css(&mut css, &brick);
    if let Some(x) = use_default(&brick)
        && let Some(d) = x.as_str()
    {
        rsx! {
            path {
                class: css.join(" "),
                d: d.to_string(),
                {children}
            }
        }
    } else {
        rsx! {}
    }
}
