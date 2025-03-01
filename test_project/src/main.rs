use ripple::prelude::*;

#[derive(Component, Default)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    fn render() -> impl Element<Self::Data> {
        e::button()
            .style("font-size", "4rem")
            .text(|ctx: &S<Self>| *ctx.value)
            .on("click", |ctx: &mut S<Self>| {
                *ctx.value += 1;
            })
    }
}

fn main() {
    mount_component(Counter::default(), "mount");
}
