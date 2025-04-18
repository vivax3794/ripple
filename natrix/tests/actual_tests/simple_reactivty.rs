#![allow(dead_code)]

use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: &str = "__BUTTON";

#[derive(Component)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::button()
            .id(BUTTON_ID)
            .text("value: ")
            .child(|ctx: R<Self>| *ctx.value)
            .on::<events::Click>(|ctx: E<Self>, _| *ctx.value += 1)
    }
}

#[wasm_bindgen_test]
fn renders_initial() {
    crate::mount_test(Counter { value: 0 });

    let button = crate::get(BUTTON_ID);
    assert_eq!(button.text_content(), Some("value: 0".to_owned()));
}

#[wasm_bindgen_test]
fn uses_initial_data() {
    crate::mount_test(Counter { value: 123 });

    let button = crate::get(BUTTON_ID);
    assert_eq!(button.text_content(), Some("value: 123".to_owned()));
}

#[wasm_bindgen_test]
fn updates_text() {
    crate::mount_test(Counter { value: 0 });

    let button = crate::get(BUTTON_ID);

    button.click();
    assert_eq!(button.text_content(), Some("value: 1".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("value: 2".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("value: 3".to_owned()));
}
