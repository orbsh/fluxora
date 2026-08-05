use crate::Ctx;
use crate::hooks::{use_common_css, use_default};
use brick::Diagram;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::html::*;

/// Mermaid 图表：数据取 `bind["value"].default`，挂载后 `mermaid.init`。
pub fn diagram_(brick: Diagram, _ctx: &Ctx, id: String) -> AnyView {
    let mut css = vec!["diagram"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");

    if let Some(x) = use_default(&brick)
        && let Some(y) = x.as_str()
    {
        let val = y.to_string();
        let id_ = id.clone();
        let nr = NodeRef::<Div>::new();
        nr.on_load(move |_el| {
            crate::dom::eval(&format!("mermaid.init({{}}, '#{id_extra}')", id_extra = id_));
        });
        div()
            .id(id.as_str())
            .class(css.as_str())
            .child(val)
            .node_ref(nr)
            .into_any()
    } else {
        div().into_any()
    }
}
