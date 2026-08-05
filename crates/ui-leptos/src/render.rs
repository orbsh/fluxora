use brick::Brick;
use ui_leptos_macro::gen_dispatch;

gen_dispatch! {
    file = "../brick/src/lib.rs",
    entry = "Brick",
    object = "brick"
}
