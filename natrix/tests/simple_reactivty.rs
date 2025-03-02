use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

mod common;

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: &str = "__BUTTON";

#[derive(Component)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(BUTTON_ID)
            .child(|ctx: &S<Self>| *ctx.value)
            .on("click", |ctx: &mut S<Self>| *ctx.value += 1)
    }
}

#[wasm_bindgen_test]
fn renders_inital() {
    common::setup();
    mount_component(Counter { value: 0 }, common::MOUNT_POINT);

    let button = common::get(BUTTON_ID);
    assert_eq!(button.text_content(), Some("0".to_owned()));
}

#[wasm_bindgen_test]
fn uses_inital_data() {
    common::setup();
    mount_component(Counter { value: 123 }, common::MOUNT_POINT);

    let button = common::get(BUTTON_ID);
    assert_eq!(button.text_content(), Some("123".to_owned()));
}

#[wasm_bindgen_test]
fn updates_text() {
    common::setup();
    mount_component(Counter { value: 0 }, common::MOUNT_POINT);

    let button = common::get(BUTTON_ID);

    button.click();
    assert_eq!(button.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("2".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("3".to_owned()));
}
