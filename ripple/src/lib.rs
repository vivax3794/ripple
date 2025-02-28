#![deny(clippy::unwrap_used, unsafe_code)]
#![allow(clippy::type_complexity, private_interfaces, incomplete_features)]
#![cfg_attr(feature = "nightly", feature(specialization))]

mod component;
mod element;

pub mod html_elements;
mod utils;

pub mod prelude {
    pub use ripple_macros::Component;

    pub use super::component::{Component, ComponentData, S, State, mount_component};
    pub use super::element::Element;
    pub use super::html_elements as e;
    pub use super::html_elements::Event;

    #[cfg(feature = "web_utils")]
    pub fn log(msg: &str) {
        let msg = wasm_bindgen::JsValue::from_str(msg);
        web_sys::console::log_1(&msg);
    }
}

#[doc(hidden)]
pub mod macro_ref {
    pub use super::component::{ComponentBase, ComponentData, DataTracker, DataTrackerMethods};
}
