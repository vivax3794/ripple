#![allow(dead_code)]

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

const MOUNT_PARENT: &str = "__TESTING_PARENT";
pub const MOUNT_POINT: &str = "__TESTING_MOUNT_POINT";

pub fn setup() {
    let document = web_sys::window().unwrap().document().unwrap();

    if let Some(element) = document.get_element_by_id(MOUNT_PARENT) {
        element.remove();
    }

    let parent = document.create_element("div").unwrap();
    parent.set_id(MOUNT_PARENT);

    let mount = document.create_element("div").unwrap();
    mount.set_id(MOUNT_POINT);

    parent.append_child(&mount).unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
}

pub fn get(id: &'static str) -> HtmlElement {
    let document = web_sys::window().unwrap().document().unwrap();

    document
        .get_element_by_id(id)
        .unwrap()
        .dyn_ref::<HtmlElement>()
        .unwrap()
        .clone()
}
