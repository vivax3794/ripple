# Project template

## index.html
This file is what gets loaded in the browser, with a bit of transformation from trunk. You will see it contains a reference to our crate, as well as a `<div id="mount"/>` which is where our app will be mounted. This is where you would also add stuff to the `<head></head>` section of the page. See [Trunk Docs](TODO) for more information on this file.

<div class="warning">
Avoid including css references in this file as it wont be tree-shaken. instead use the `#[style]` macro as outlined in [Styles](TODO).
</div>

## Trunk.toml
This is the trunk config file which controls the build process, it should contain
```toml
[build]
minify = "on_release"
filehash = false
```
The `minify` settings shrinks your js bundle making it load faster, `filehash=false` makes dist outputs have a consistent file name which is used by the `build` task in `./justfile` to optimize the wasm.

If you selected `tailwind` as a option this file will also include the needed trunk config for tailwind to pick up css classes from natrix components (see [Tailwind](TODO) for specifics on how classes are picked up)

See [Trunk Docs](TODO) for additional options.


## justfile
a [justfile](TODO) defines task you can easially run, most of these are covered in [Getting Started](./getting_started.md), but can also be listed with
```bash
just --list
```

## Cargo.toml features
Depending on your selected options when generating the `natrix` depedency might contain some feature flags, see [Features](./features.md) for explanation on what they do.

## Optimizations
You will find that `Cargo.toml` contains some agresssive `release` profile optimizations. You will also find that in `.cargo/config.toml` the project is set to recompile the stdlib with `bulk-memory` enabled.
