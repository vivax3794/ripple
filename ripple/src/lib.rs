#![feature(get_mut_unchecked)]
mod component;
mod element;

pub mod html_elements;

pub mod prelude {
    pub use super::html_elements as e;
}
