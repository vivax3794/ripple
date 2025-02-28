# Features

## `nightly`
Enables nightly only public facing features, which currently are:
* ... none xD

## `element_unit`
Allows `()` to be used as an element (Same handling as `None`).
This is off by default as there are very few legitemate usecases for this, and it can hide errors such as in the following code:
```rust
fn render() -> impl Element<Self::Data> {
    e::div();
}
```
> Although you will get a "unused `#[must_use]`" in this case

## `web_utils` (Default)
Enables convnient wrappers around certain web apis such as `console.log`

## `nightly_optimization` (Default)
Internal only nightly optimizations, this feature flag is a noop on stable (i.e its safe to enable even if you target stable).

## `intern` (Default)
enabled [`wasm_bindgen` interning] of strings, its generally recommended to keep this on as it massively speeds up dom creation.

## `debug_log`
This enables a bunch of internal logs (to `console.log`), has no affect in release builds.

