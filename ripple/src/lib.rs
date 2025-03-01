#![deny(clippy::unwrap_used, unsafe_code, clippy::todo)]
#![allow(private_interfaces, private_bounds, clippy::type_complexity)]

use std::cell::OnceCell;

mod callbacks;
mod component;
mod element;
pub mod html_elements;
mod render_callbacks;
mod signal;
mod state;
mod utils;

thread_local! {
    static DOCUMENT: OnceCell<web_sys::Document> = const { OnceCell::new() };
}

pub(crate) fn get_document() -> web_sys::Document {
    DOCUMENT.with(|doc_cell| {
        doc_cell
            .get_or_init(|| {
                web_sys::window()
                    .expect("No window object")
                    .document()
                    .expect("No document")
            })
            .clone()
    })
}

pub mod prelude {
    pub use ripple_macros::Component;

    pub use super::callbacks::Event;
    pub use super::component::{Component, mount_component};
    pub use super::element::Element;
    pub use super::html_elements as e;
    pub use super::state::{ComponentData, S, State};

    #[cfg(feature = "web_utils")]
    pub fn log(msg: &str) {
        let msg = wasm_bindgen::JsValue::from_str(msg);
        web_sys::console::log_1(&msg);
    }

    #[inline(always)]
    #[allow(unused_variables)]
    pub fn debug(msg: &str) {
        #[cfg(debug_log)]
        {
            crate::prelude::log(msg);
        }
    }
}

#[doc(hidden)]
pub mod macro_ref {
    pub use super::component::ComponentBase;
    pub use super::signal::{Signal, SignalMethods};
    pub use super::state::ComponentData;
}
