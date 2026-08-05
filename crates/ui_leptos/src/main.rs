use tracing_wasm::WASMLayerConfigBuilder;

fn main() {
    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );
    ui_leptos::mount();
}
