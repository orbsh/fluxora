//! 容器组件与叶子组件入口。宏 `gen_dispatch!` 分派到 `crate::components::*_`，
//! 故此处 re-export 所有组件函数，使它们在 `crate::components` 命名空间可见。
pub mod case;
pub use case::{case_, placeholder_};
pub mod chart;
pub use chart::chart_;
pub mod diagram;
pub use diagram::diagram_;
pub mod float;
pub use float::float_;
pub mod fold;
pub use fold::fold_;
pub mod form;
pub use form::form_;
pub mod popup;
pub use popup::popup_;
pub mod rack;
pub use rack::rack_;
pub mod render;
pub use render::render_;
pub mod svg;
pub use svg::{path_, group_, svg_};

pub use crate::widgets::*;
