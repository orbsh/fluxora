#![allow(unused_imports)]
use crate::libs::components::Frame;
use crate::libs::hooks::{use_common_css, use_source_value};
use brick::{Bind, Float, JsType};
use dioxus::prelude::*;
use serde_json::{Value, to_value};
use std::ops::Deref;
use std::rc::Rc;

#[component]
pub fn float_(id: Option<String>, brick: Float, children: Element) -> Element {
    let mut css = vec!["float", "f"];
    use_common_css(&mut css, &brick);
    let style = &brick
        .attrs
        .as_ref()
        .map(|x| x.into_style())
        .unwrap_or("".to_string());
    rsx! {
        div {
            style: style.clone(),
            class: css.join(" "),
            {children}
        }
    }
}
