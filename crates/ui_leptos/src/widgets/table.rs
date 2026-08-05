use crate::Ctx;
use crate::ctx::render_children;
use brick::{Table, Tbody, Td, Th, Thead, Tr};
use leptos::prelude::*;
use leptos::html::*;

pub fn table_(brick: Table, ctx: &Ctx) -> AnyView {
    let children = brick.sub.as_deref().map(|s| render_children(ctx, s)).unwrap_or_default();
    table().child(children).into_any()
}

pub fn thead_(brick: Thead, ctx: &Ctx) -> AnyView {
    let children = brick.sub.as_deref().map(|s| render_children(ctx, s)).unwrap_or_default();
    thead().child(children).into_any()
}

pub fn tbody_(brick: Tbody, ctx: &Ctx) -> AnyView {
    let children = brick.sub.as_deref().map(|s| render_children(ctx, s)).unwrap_or_default();
    tbody().child(children).into_any()
}

pub fn tr_(brick: Tr, ctx: &Ctx) -> AnyView {
    let children = brick.sub.as_deref().map(|s| render_children(ctx, s)).unwrap_or_default();
    tr().child(children).into_any()
}

pub fn th_(brick: Th, ctx: &Ctx) -> AnyView {
    let children = brick.sub.as_deref().map(|s| render_children(ctx, s)).unwrap_or_default();
    th().child(children).into_any()
}

pub fn td_(brick: Td, ctx: &Ctx) -> AnyView {
    let children = brick.sub.as_deref().map(|s| render_children(ctx, s)).unwrap_or_default();
    td().child(children).into_any()
}
