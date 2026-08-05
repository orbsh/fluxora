use crate::Ctx;
use brick::{Bind, BindVariant, Brick, BrickOps, classify::Classify};
use leptos::prelude::*;
use serde_json::Value;

/// 追加公共 CSS：非横向的 Box/Case/Rack/Text/Tab/Select 加 `col`，并扩展 class 列表。
pub fn use_common_css<'a, 'b: 'a, T>(css: &'a mut Vec<&'b str>, brick: &'b T)
where
    T: Classify + BrickOps,
{
    let t = brick.get_type();
    let mut v = ["Box", "Case", "Rack", "Text", "Tab", "Select"].contains(&t);
    if let Some(a) = brick.borrow_attrs() {
        if a.is_horizontal() {
            v = false;
        }
        if let Some(cc) = a.get_class() {
            let c = cc.iter().map(|x| &**x).collect::<Vec<_>>();
            css.extend(c);
        }
    }
    if v {
        css.push("col");
    }
}

/// 取 `bind["value"].default`。
pub fn use_default(brick: &impl BrickOps) -> Option<Value> {
    brick
        .get_bind()
        .and_then(|x| x.get("value"))
        .and_then(|x| x.default.clone())
}

/// 取 `bind["value"]` 的 Source 来源名。
pub fn use_source_id(brick: &impl BrickOps) -> Option<&String> {
    if let Bind {
        variant: BindVariant::Source { source },
        ..
    } = brick.get_bind().and_then(|x| x.get("value"))?
    {
        Some(source)
    } else {
        None
    }
}

/// 从 `ctx.list[source]` 取列表（无则返回空 vec）。
pub fn use_source_list(ctx: &Ctx, brick: &impl BrickOps, key: &str) -> Option<Vec<Brick>> {
    let list = ctx.list.get();
    if let Some(Bind {
        variant: BindVariant::Source { source },
        default: _,
        r#type: _kind,
    }) = brick.get_bind().and_then(|x| x.get(key))
        && let Some(l) = list.get(source)
    {
        Some(l.clone())
    } else {
        Some(Vec::new())
    }
}

/// 取 `bind[key]` 对应的值：Source 则从 `ctx.data[source]` 取，否则取 brick 自身。
pub fn use_source<'a>(ctx: &Ctx, brick: &'a impl BrickOps, key: &'a str) -> Option<Value> {
    let data = ctx.data.get();
    let value = if let Some(Bind {
        variant: BindVariant::Source { source },
        default: _,
        r#type: _kind,
    }) = brick.get_bind().and_then(|x| x.get(key))
        && let Some(d) = data.get(source)
    {
        Some(d as &dyn BrickOps)
    } else {
        Some(brick as &dyn BrickOps)
    };
    if let Some(comp) = value
        && let Some(bind) = &comp.get_bind()
        && let Some(value) = bind.get(key)
    {
        value.default.clone()
    } else {
        None
    }
}

/// `use_source(ctx, brick, "value")`。
pub fn use_source_value(ctx: &Ctx, brick: &impl BrickOps) -> Option<Value> {
    use_source(ctx, brick, "value")
}

/// `bind[key]` 为 Event 时，返回一个发送该事件的闭包。
pub fn use_target<'a>(
    ctx: Ctx,
    brick: &'a impl BrickOps,
    key: &'a str,
) -> Option<impl Fn(Value)> {
    if let Some(Bind {
        variant: BindVariant::Event { event },
        default: _,
        r#type: _,
    }) = brick.get_bind().and_then(|x| x.get(key))
    {
        let ev = event.clone();
        Some(move |val| {
            let ctx = ctx.clone();
            let ev = ev.clone();
            leptos::task::spawn_local(async move {
                ctx.send(ev, None, val).await;
            });
        })
    } else {
        None
    }
}

/// `use_target(ctx, brick, "value")`。
pub fn use_target_value(ctx: Ctx, brick: &impl BrickOps) -> Option<impl Fn(Value)> {
    use_target(ctx, brick, "value")
}
/// 表单信号共享：`form_` 构建 `FormState` 并压栈，`input_`/`button_`
/// 在渲染期间从栈顶取字段/确认信号。因 `BindVariant` 的 `signal` 字段
/// 在非 dioxus 构建下被 cfg 掉，改用线程栈传递信号句柄。
#[derive(Clone)]
pub struct FormState {
    pub fields: std::collections::HashMap<String, RwSignal<Value>>,
    pub confirm: RwSignal<Value>,
}

thread_local! {
    static FORM_STACK: std::cell::RefCell<Vec<std::rc::Rc<FormState>>> =
        std::cell::RefCell::new(Vec::new());
}

pub fn push_form(fs: FormState) {
    FORM_STACK.with(|s| s.borrow_mut().push(std::rc::Rc::new(fs)));
}

pub fn pop_form() {
    FORM_STACK.with(|s| {
        s.borrow_mut().pop();
    });
}

pub fn peek_form() -> Option<std::rc::Rc<FormState>> {
    FORM_STACK.with(|s| s.borrow().last().cloned())
}
