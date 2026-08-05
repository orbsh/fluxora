use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::WebSocketError;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub use gloo_net::websocket::Message;

/// 持有 WS 写半部分。`Rc<RefCell<SplitSink>>` 非 Send/Sync，
/// 用 `SendWrapper` 包一层（wasm 单线程，安全）。
#[derive(Clone)]
pub struct WebSocketHandle {
    ws_write: SendWrapper<Rc<RefCell<SplitSink<WebSocket, Message>>>>,
    state: RwSignal<websocket::State>,
    message_bytes: RwSignal<Vec<u8>>,
}

impl WebSocketHandle {
    pub fn new(url: &str) -> Self {
        let state = RwSignal::new(websocket::State::Closed);
        let message_bytes = RwSignal::new(Vec::new());

        let ws = WebSocket::open(url).expect("ws open failed");
        let (write, mut read) = ws.split();

        let mb = message_bytes;
        leptos::task::spawn_local(async move {
            while let Some(Ok(m)) = read.next().await {
                match m {
                    Message::Text(t) => mb.set(t.into_bytes()),
                    Message::Bytes(b) => mb.set(b),
                }
            }
        });

        Self {
            ws_write: SendWrapper::new(Rc::new(RefCell::new(write))),
            state,
            message_bytes,
        }
    }

    #[allow(clippy::await_holding_refcell_ref)]
    pub async fn send(&self, message: Message) -> Result<(), WebSocketError> {
        let sink = self.ws_write.deref().clone();
        sink.borrow_mut().send(message).await
    }

    #[allow(unused)]
    pub fn status(&self) -> RwSignal<websocket::State> {
        self.state
    }

    pub fn message_bytes(&self) -> RwSignal<Vec<u8>> {
        self.message_bytes
    }
}
