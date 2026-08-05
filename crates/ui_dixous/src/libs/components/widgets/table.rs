use brick::{Table, Tbody, Td, Th, Thead, Tr};
use dioxus::prelude::*;

#[component]
pub fn table_(id: Option<String>, brick: Table, children: Element) -> Element {
    rsx! {
        table {
            {children}
        }
    }
}

#[component]
pub fn thead_(id: Option<String>, brick: Thead, children: Element) -> Element {
    rsx! {
        thead {
            {children}
        }
    }
}

#[component]
pub fn tbody_(id: Option<String>, brick: Tbody, children: Element) -> Element {
    rsx! {
        tbody {
            {children}
        }
    }
}

#[component]
pub fn tr_(id: Option<String>, brick: Tr, children: Element) -> Element {
    rsx! {
        tr {
            {children}
        }
    }
}

#[component]
pub fn th_(id: Option<String>, brick: Th, children: Element) -> Element {
    rsx! {
        th {
            {children}
        }
    }
}

#[component]
pub fn td_(id: Option<String>, brick: Td, children: Element) -> Element {
    rsx! {
        td {
            {children}
        }
    }
}
