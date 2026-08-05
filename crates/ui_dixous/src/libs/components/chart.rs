use crate::libs::hooks::use_default;
use brick::Chart;
use dioxus::prelude::*;

#[component]
pub fn chart_(id: Option<String>, brick: Chart) -> Element {
    if let Some(val) = use_default(&brick)
        && let Some(id) = id
    {
        let id_ = id.clone();
        use_effect(move || {
            let js = format!(
                r#"
                var chart = new ApexCharts(document.getElementById("{id_}"), {val});
                chart.render();
            "#
            );
            document::eval(&js);
        });
        rsx! {
            div {
                id: id,
            }
        }
    } else {
        rsx! {
            div {}
        }
    }
}
