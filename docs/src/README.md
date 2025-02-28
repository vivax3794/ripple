# Introduction

Ripple is a ***Rust-first*** frontend framework. Where other frameworks aim to bring React-style development to rust, Ripple embraces Rust’s strengths—leveraging smart pointers, derive macros, the builder pattern, and other idiomatic Rust features to create a truly native experience.

# A Simple Example
A simple counter in Ripple looks like this:
```rust
# use ripple::prelude::*;
#
#[derive(Component)]
struct Counter(usize);

impl Component for Counter {
    fn render() -> impl Element<Self::Data> {
        e::div().child(
            e::button()
                .child(|ctx: &S<Self>| *ctx.0)
                .style("font-size", "4rem")
                .on("click", |ctx: &mut S<Self>| {
                    *ctx.0 += 1;
                }),
        )
    }
}
#
# fn main() {
#   mount_component(Counter(0), "mount");
# }
```
> The rest of this book dives deeper into each of these features in detail.

## Standout features
* ✅ **No macro DSL** – Macro-based DSLs break formatting & Rust Analyzer support. Ripple avoids them completely for a smoother dev experience.
* ✅ **Derive macros for reactive state** – No need for `useSignal` everywhere, state is directly tied to Rust’s type system.
* ✅ **Callbacks use references to state** – Instead of closures capturing state setters, Ripple callbacks take a reference to the state, which better aligns with Rust’s ownership model.
* ✅ **Fine-grained reactivity** – Ripple only updates what's necessary, minimizing re-renders and maximizing performance.
* ✅ **Smart feature selection** - Ripple will automatically use nightly-only optimizations if possible without needing a explicit `nightly` flag.
* ✅ **Opt-In unsafe** - Ripple contains a set of off-by-default unsafe optimizations

# Design Goals
* **Developer experience first** – Ripple is designed to feel natural for Rust developers.
* **Idiomatic Rust** – We use Rust-native features & patterns, not what worked for js.
* **Stop porting JS to Rust** – Rust is an amazing language, let’s build a frontend framework that actually feels like Rust.

