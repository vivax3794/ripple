use ripple::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod common;

const HELLO_ID: &str = "__HELLO";

#[derive(Component)]
struct HelloWorld {}

impl Component for HelloWorld {
    fn render() -> impl Element<Self::Data> {
        e::h1().id(HELLO_ID).text("Hello World!")
    }
}

#[wasm_bindgen_test]
fn renders_fine() {
    common::setup();
    mount_component(HelloWorld {}, common::MOUNT_POINT);

    let element = common::get(HELLO_ID);
    assert_eq!(element.text_content(), Some("Hello World!".to_owned()));
}
