pub mod ctx;
pub mod render;
pub mod components;
pub mod widgets;
pub mod dom;
pub mod ws;

pub use ctx::Ctx;

/// 由 ui_leptos_macro 生成 `dispatch`，入口。
pub fn mount(root: &str) {
    let ctx = Ctx::new();
    let view = render::dispatch(&ctx.layout.get(), &ctx);
    leptos::mount::mount_to(root, move || view);
}