use std::collections::HashMap;
use std::rc::{Rc, Weak};

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use crate::component::ComponentData;
use crate::element::Element;

pub struct WebElement<C: ComponentData> {
    name: &'static str,
    events: HashMap<&'static str, Box<dyn Fn(&mut C)>>,
}

impl<C: ComponentData> WebElement<C> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            events: HashMap::new(),
        }
    }

    pub fn on(mut self, event: &'static str, function: impl Fn(&mut C) + 'static) -> Self {
        self.events.insert(event, Box::new(function));
        self
    }
}

impl<C: ComponentData + 'static> Element<C> for WebElement<C> {
    fn render(mut self, ctx: &mut C) -> web_sys::Node {
        let document = gloo::utils::document();
        let element = document.create_element(self.name).unwrap();

        let ctx = ctx.weak();

        for (event, function) in self.events.drain() {
            let new_ctx = Weak::clone(&ctx);
            let callback: Box<dyn Fn() + 'static> = Box::new(move || {
                let mut data = new_ctx.upgrade().unwrap();
                // We know that no other people are holding a mut reference since single threaded
                // yada yada
                let data = unsafe { Rc::get_mut_unchecked(&mut data) };

                data.clear_state();
                function(data);
                data.update();
            });

            let closure = Closure::<dyn Fn()>::wrap(callback);
            let function = closure.as_ref().unchecked_ref();
            element
                .add_event_listener_with_callback(event, function)
                .unwrap();

            // MASSIVE FUCKING TODO: dont do this, actually cleanup memory somehow
            // We would need to know when this element leaves the dom... which because soft immediate mode is hard...
            // I guess mutation observers are the "correct" way to go here.
            // But they are annoying to setup... **TODO**
            closure.forget();
        }

        element.into()
    }
}

macro_rules! elements {
    ($($name:ident),*) => {
        $(
            pub fn $name<C: ComponentData>() -> WebElement<C> {
                WebElement::new(stringify!($name))
            }
        )*
    };
}

elements! {div, p, h1, h2, h3, h4, h5}
