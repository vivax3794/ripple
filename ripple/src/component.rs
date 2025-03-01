use std::cell::RefCell;
use std::rc::Rc;

use crate::element::Element;
use crate::get_document;
use crate::signal::RenderingState;
use crate::state::{ComponentData, State};

#[diagnostic::on_unimplemented(
    message = "Type `{Self}` is not a component.",
    note = "add `#[derive(Component)]` to the struct"
)]
pub trait ComponentBase: Sized {
    type Data: ComponentData;
    fn into_data(self) -> Self::Data;

    fn into_state(self) -> Rc<RefCell<State<Self::Data>>> {
        State::new(self.into_data())
    }
}

pub trait Component: ComponentBase {
    fn render() -> impl Element<Self::Data>;
}

/// Mounts the component at the target id
/// Replacing the element with the component
/// This should be the entry point to your application
///
/// **WARNING:** This method implicitly leaks the memory of the root component
pub fn mount_component<C: Component>(component: C, target_id: &'static str) {
    let data = component.into_state();
    let element = C::render();

    let mut borrow_data = data.borrow_mut();
    let mut keep_alive = Vec::new();
    let mut state = RenderingState {
        keep_alive: &mut keep_alive,
    };
    let node = element.render(&mut borrow_data, &mut state);

    let document = get_document();
    let target = document
        .get_element_by_id(target_id)
        .expect("Failed to get mount point");
    target
        .replace_with_with_node_1(&node)
        .expect("Failed to replace mount point");

    drop(borrow_data);

    // This is the entry point, this component should be alive FOREVER
    std::mem::forget(data);
    std::mem::forget(keep_alive);
}
