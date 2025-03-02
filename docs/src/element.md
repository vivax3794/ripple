# Element

In the context of this framework a "Element" is anything that implements the `Element` trait, which is not limited to HTML elements and includes everything from `String` to `Result`.

> For HTML elements see [Html Element](TODO)

## Text Rendering
Both `String` and `&'static str` get rendered as simple text. Integers (`u8`, `u16`, `i8`, ...) are also rendered as text via the [`itoa`](TODO) crate.
Text is inserted as `Text` nodes and is ***not*** vulnerable to xss.

## Conditional Rendering
Both `Option<T>` and `Result<T, E>` can be used as elements (assuming their contents are also elements). In `Option`s case `None` is rendered as a HTML comment (to facilitate easy swapping for a real element)

# Examples
| Rust | Html |
| --- | --- |
| `10` | `10` |
| `"Hello World"` | `Hello World` |
| `format!("ABC {}", 123)` | `ABC 123` |
| `"<h1>Hello</h1>"` | `&lt;h1&gt;Hello&lt;/h1&gt;` |
| `Some("Hello")` | `Hello` |
| `None` | `<!---->` |
| `Ok("Hello")` | `Hello` |
| `Err("Oh No!")` | `Oh No!` | 
