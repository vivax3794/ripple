#![allow(dead_code)]

use natrix::html_elements::ToAttribute;
use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
wasm_bindgen_test_configure!(run_in_browser);

mod common;

const ROOT: &str = "ROOT";

#[derive(Component, Default)]
struct Generic<T>(T);

impl<T: ToAttribute<Self::Data> + Copy> Component for Generic<T> {
    fn render() -> impl Element<Self::Data> {
        e::div().attr("abc", |ctx: &S<Self>| *ctx.0).id(ROOT)
    }
}

#[wasm_bindgen_test]
fn simple_true() {
    common::setup();
    mount_component(Generic(true), common::MOUNT_POINT);

    let button = common::get(ROOT);
    assert_eq!(button.get_attribute("abc"), Some("".to_owned()));
}
#[wasm_bindgen_test]
fn simple_false() {
    common::setup();
    mount_component(Generic(false), common::MOUNT_POINT);

    let button = common::get(ROOT);
    assert_eq!(button.get_attribute("abc"), None);
}
#[wasm_bindgen_test]
fn simple_string() {
    common::setup();
    mount_component(Generic("hello"), common::MOUNT_POINT);

    let button = common::get(ROOT);
    assert_eq!(button.get_attribute("abc"), Some("hello".to_owned()));
}
#[wasm_bindgen_test]
fn simple_some() {
    common::setup();
    mount_component(Generic(Some("hello")), common::MOUNT_POINT);

    let button = common::get(ROOT);
    assert_eq!(button.get_attribute("abc"), Some("hello".to_owned()));
}
#[wasm_bindgen_test]
fn simple_none() {
    common::setup();
    mount_component(Generic(None::<u8>), common::MOUNT_POINT);

    let button = common::get(ROOT);
    assert_eq!(button.get_attribute("abc"), None);
}

#[derive(Component, Default)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(ROOT)
            .attr("abc", |ctx: &S<Self>| format!("{}", *ctx.value))
            .on("click", |ctx: &mut S<Self>| {
                *ctx.value += 1;
            })
    }
}

#[wasm_bindgen_test]
fn reactive_attribute() {
    common::setup();
    mount_component(Counter::default(), common::MOUNT_POINT);

    let button = common::get(ROOT);

    assert_eq!(button.get_attribute("abc"), Some("0".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("1".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("2".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("3".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("4".to_owned()));
}

#[derive(Component, Default)]
struct Toggle {
    value: bool,
}

impl Component for Toggle {
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(ROOT)
            .attr("abc", |ctx: &S<Self>| *ctx.value)
            .on("click", |ctx: &mut S<Self>| {
                *ctx.value = !*ctx.value;
            })
    }
}

#[wasm_bindgen_test]
fn reactive_bool() {
    common::setup();
    mount_component(Toggle::default(), common::MOUNT_POINT);

    let button = common::get(ROOT);

    assert_eq!(button.get_attribute("abc"), None);

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), None);
}
