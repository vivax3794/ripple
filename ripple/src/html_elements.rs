use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Weak;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, intern};

use crate::component::{ComponentData, State};
use crate::element::Element;

pub trait Event<C: ComponentData> {
    fn func(self) -> Box<dyn Fn(&mut State<C>)>;
}
impl<C: ComponentData, F: Fn(&mut State<C>) + 'static> Event<C> for F {
    #[inline(always)]
    fn func(self) -> Box<dyn Fn(&mut State<C>)> {
        Box::new(self)
    }
}

pub struct WebElement<C: ComponentData> {
    name: &'static str,
    events: HashMap<&'static str, Box<dyn Fn(&mut State<C>)>>,
    children: Vec<Box<dyn Element<C>>>,
    styles: HashMap<&'static str, Cow<'static, str>>,
    attributes: HashMap<&'static str, Cow<'static, str>>,
}

impl<C: ComponentData> WebElement<C> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            events: HashMap::new(),
            children: Vec::new(),
            styles: HashMap::new(),
            attributes: HashMap::new(),
        }
    }

    #[inline(always)]
    pub fn on(mut self, event: &'static str, function: impl Event<C>) -> Self {
        self.events
            .insert(event, function.func());
        self
    }

    #[inline(always)]
    pub fn child<E: Element<C> + 'static>(mut self, child: E) -> Self {
        self.children
            .push(Box::new(child));
        self
    }

    #[inline(always)]
    pub fn text<E: Element<C> + 'static>(self, text: E) -> Self {
        self.child(text)
    }

    #[inline(always)]
    pub fn style(mut self, key: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        self.styles
            .insert(key, value.into());
        self
    }

    #[inline(always)]
    pub fn attr(mut self, key: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        self.attributes
            .insert(key, value.into());
        self
    }

    #[inline(always)]
    pub fn id(self, id: &'static str) -> Self {
        self.attr("id", id)
    }
}

impl<C: ComponentData + 'static> Element<C> for WebElement<C> {
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        let Self {
            name,
            events,
            children,
            styles,
            attributes,
        } = *self;

        let document = gloo::utils::document();
        let element = document
            .create_element(intern(name))
            .expect("Failed to get document");

        for child in children {
            let child = child.render_box(ctx);
            element
                .append_child(&child)
                .expect("Failed to append child");
        }

        let ctx_weak = ctx.weak();
        for (event, function) in events {
            let new_ctx = Weak::clone(&ctx_weak);
            let callback: Box<dyn Fn() + 'static> = Box::new(move || {
                let data = new_ctx
                    .upgrade()
                    .expect("Component dropped in event callback");
                let mut data = data.borrow_mut();

                data.clear_state();
                function(&mut data);
                data.update();
            });

            let closure = Closure::<dyn Fn()>::wrap(callback);
            let function = closure
                .as_ref()
                .unchecked_ref();
            element
                .add_event_listener_with_callback(intern(event), function)
                .expect("Failed to add listener");

            ctx.register_event(Box::new(closure));
        }

        let style = styles
            .into_iter()
            .map(|(key, value)| format!("{key}:{value};"))
            .collect::<Vec<_>>()
            .join("");
        element
            .set_attribute(intern("style"), &style)
            .expect("Failed to set style");

        for (key, value) in attributes {
            element
                .set_attribute(intern(key), &value)
                .expect("Failed to set attribute");
        }

        element.into()
    }
}

macro_rules! elements {
    ($($name:ident),*) => {
        $(
            #[inline(always)]
            pub fn $name<C: ComponentData>() -> WebElement<C> {
                WebElement::new(stringify!($name))
            }
        )*
    };
}

elements! {div, p, h1, h2, h3, h4, h5, button}
