use super::super::super::store::Store;
use super::super::utils::merge_css_class;
use dioxus::prelude::*;
use layout::{Bind, JsKind, Layout};
use serde_json::{Value, to_value};
use std::ops::Deref;
use std::rc::Rc;

fn default_option_jskind(v: &Option<JsKind>) -> Value {
    v.as_ref()
        .map(|x| x.default_value())
        .unwrap_or_else(|| to_value("").unwrap())
}

#[component]
pub fn Input(layout: Layout) -> Element {
    let store = use_context::<Store>();
    let mut css = vec!["input", "f", "shadow"];
    merge_css_class(&mut css, &layout);

    return match &layout.bind {
        Some(Bind::Field {
            field,
            kind,
            payload,
            signal,
        }) => {
            let mut slot = signal.unwrap_or_else(|| use_signal(|| default_option_jskind(&kind)));
            let oninput = move |event: Event<FormData>| {
                let event_value = event.value();
                let parsed_value = match kind {
                    Some(JsKind::bool) => to_value(event_value == "true"),
                    Some(JsKind::number) => to_value(event_value.parse::<f64>().unwrap()),
                    _ => to_value(event_value),
                }
                .unwrap();
                slot.set(parsed_value);
            };
            rsx!()
        }
        Some(Bind::Event { event, kind }) => {
            let onkeydown = move |ev: Event<KeyboardData>| {
                let mut s = store.clone();
                async move {
                    if ev.data.key() == Key::Enter {
                        todo! {
                            //s.send(event.deref(), None, val).await;
                        }
                    }
                }
            };
            rsx!()
        }
        Some(Bind::Variable { variable, kind }) => {
            rsx!()
        }
        _ => {
            rsx!()
        }
    };
}
