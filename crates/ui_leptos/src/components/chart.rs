use crate::Ctx;
use crate::hooks::use_default;
use brick::Chart;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::html::*;

/// ApexCharts 图表：数据取 `bind["value"].default`，挂载后执行 JS 渲染。
pub fn chart_(brick: Chart, _ctx: &Ctx, id: String) -> AnyView {
    if let Some(val) = use_default(&brick) {
        let val = val.to_string();
        let id_ = id.clone();
        let nr = NodeRef::<Div>::new();
        nr.on_load(move |_el| {
            crate::dom::eval(&format!(
                r#"
                var chart = new ApexCharts(document.getElementById("{id_extra}"), {val_body});
                chart.render();
                "#,
                id_extra = id_,
                val_body = val
            ));
        });
        div().id(id.as_str()).node_ref(nr).into_any()
    } else {
        div().into_any()
    }
}
