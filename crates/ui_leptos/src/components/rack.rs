use crate::Ctx;
use crate::ctx::render_brick;
use crate::hooks::{use_common_css, use_source_id};
use brick::classify::Classify;
use brick::{Brick, BrickOps, Rack, RackAttr};
use leptos::prelude::*;
use leptos::html::*;
use std::collections::HashMap;

#[derive(Debug)]
struct ItemContainer {
    default: Option<Brick>,
    index: HashMap<String, Brick>,
}

impl From<Vec<Brick>> for ItemContainer {
    fn from(data: Vec<Brick>) -> Self {
        let mut default = None;
        let mut index = HashMap::new();
        for l in &data {
            if let Some(x) = l.get_selector() {
                index.insert(x.to_owned(), l.clone());
            } else {
                default = Some(l.clone());
            }
        }
        ItemContainer { index, default }
    }
}

impl ItemContainer {
    fn select(&self, child: &Brick) -> Option<Brick> {
        if let Some(s) = child.get_selector()
            && let Some(i) = self.index.get(s)
        {
            return Some(i.clone());
        }
        self.default.clone()
    }
}

/// 列表容器：按 selector 索引 `item` 模板，遍历 `ctx.list[source]` 渲染。
pub fn rack_(brick: Rack, ctx: &Ctx, id: String) -> AnyView {
    let ctx = ctx.clone();
    let mut css = vec!["rack", "f"];
    use_common_css(&mut css, &brick);
    let css = css.join(" ");

    let item: ItemContainer = brick.item.clone().unwrap_or_default().into();
    let Some(source) = use_source_id(&brick).cloned() else {
        return div().into_any();
    };
    let scroll = brick
        .attrs
        .as_ref()
        .map(|RackAttr { scroll, .. }| *scroll)
        .unwrap_or(false);

    move || -> AnyView {
        let c = ctx.list.get().get(&source).cloned().unwrap_or_default();
        let children = c.iter().enumerate().map(|(idx, child)| {
            let key = child.get_id().clone().unwrap_or(idx.to_string());
            let _ = key;
            match item.select(child) {
                Some(mut template) => {
                    // 模板外壳 + child 作为其 children
                    template.set_sub(vec![child.clone()]);
                    let ctx = ctx.clone();
                    render_brick(&ctx, &template)
                }
                None => {
                    let ctx = ctx.clone();
                    render_brick(&ctx, child)
                }
            }
        });

        if scroll {
            let id_ = id.clone();
            leptos::task::spawn_local(async move {
                crate::dom::eval(&format!(
                    r#"
                    var e = document.getElementById("{id_extra}");
                    if (e && Math.abs(e.scrollHeight - e.offsetHeight - e.scrollTop) < e.offsetHeight) {{
                        e.scrollTop = e.scrollHeight;
                    }}
                    "#,
                    id_extra = id_
                ));
            });
        }

        div()
            .id(id.as_str())
            .class(css.as_str())
            .child(Vec::from_iter(children))
            .into_any()
    }
    .into_any()
}
