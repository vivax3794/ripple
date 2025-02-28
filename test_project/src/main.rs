use ripple::prelude::*;

#[derive(Component)]
struct Bench(bool, usize);

impl Component for Bench {
    fn render() -> impl Element<Self::Data> {
        e::div()
            .child(
                e::button()
                    .text("TOGGLE")
                    .on("click", |ctx: &mut S<Self>| {
                        *ctx.0 = !*ctx.0;
                    }),
            )
            .child(|ctx: &S<Self>| {
                if *ctx.0 {
                    let mut result = e::div();
                    for _ in 0..10_000 {
                        result = result.child(
                            e::button()
                                .text(|ctx: &S<Self>| *ctx.1)
                                .on("click", |ctx: &mut S<Self>| {
                                    *ctx.1 += 1;
                                }),
                        );
                    }
                    Some(result)
                } else {
                    None
                }
            })
    }
}

fn main() {
    mount_component(Bench(false, 0), "mount");
}
