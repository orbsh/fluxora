use crate::libs::hooks::{use_common_css, use_default};
use brick::Diagram;
use dioxus::prelude::*;

#[component]
pub fn diagram_(id: Option<String>, brick: Diagram) -> Element {
    let mut css = vec!["diagram"];
    use_common_css(&mut css, &brick);
    if let Some(x) = use_default(&brick)
        && let Some(y) = x.as_str()
        && let Some(eid) = id.clone()
    {
        let val = y.to_string();
        use_effect(move || {
            let js = format!(
                r#"
                mermaid.init({{}}, '#{eid}')
            "#
            );
            document::eval(&js);
        });
        rsx! {
            div {
                id: id,
                class: css.join(" "),
                {val}
            }
        }
    } else {
        rsx! {
            div {}
        }
    }
}
