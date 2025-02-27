use ripple::prelude::*;

#[derive(Component)]
struct Counter(usize);

impl Component for Counter {
    fn render() -> impl Element<Self::Data> {
        e::div()
            .style("color", "white")
            .child(
                e::h1()
                    .style("font-size", "5rem")
                    .child(|ctx: &S<Self>| format!("{}", *ctx.0)),
            )
            .child(
                e::button()
                    .child("Click me!")
                    .style("font-size", "4rem")
                    .on("click", |ctx: &mut S<Self>| {
                        *ctx.0 += 1;
                    }),
            )
    }
}

fn main() {
    mount_component(Counter(0), "mount");
}
