# Introduction

Natrix is a ***Rust-first*** frontend framework. Where other frameworks aim to bring React-style development to rust, Natrix embraces Rust’s strengths—leveraging smart pointers, derive macros, the builder pattern, and other idiomatic Rust features to create a truly native experience.

# A Simple Example
A simple counter in Natrix looks like this: 
```rust
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
```
> See the [book](TODO) for more information

## Standout features
* ✅ **No macro DSL** – Macro-based DSLs break formatting & Rust Analyzer support. Natrix avoids them completely for a smoother dev experience.
* ✅ **Derive macros for reactive state** – No need for `useSignal` everywhere, state is directly tied to Rust’s type system.
* ✅ **Callbacks use references to state** – Instead of closures capturing state setters, Natrix callbacks take a reference to the state, which better aligns with Rust’s ownership model.
* ✅ **Fine-grained reactivity** – Natrix only updates what's necessary, minimizing re-renders and maximizing performance.
* ✅ **Smart feature selection** - Natrix will automatically use nightly-only optimizations if possible without needing a explicit `nightly` flag.

# Design Goals
* **Developer experience first** – Natrix is designed to feel natural for Rust developers.
* **Idiomatic Rust** – We use Rust-native features & patterns, not what worked for js.
* **Stop porting JS to Rust** – Rust is an amazing language, let’s build a frontend framework that actually feels like Rust.
