# User facing crates

Ripple has the following crates that users interact with:

* `ripple_cli` - This contains the project generator, formatter, and lsp for ripple and is the tool you will install to use rippe.
    * `ripple lsp` will spawn the lsp server, for editors without ripple plugins this is how you would setup your editor to use ripple.
    * `ripple new` will generate a new ripple project with a hello world component, and the needed `trunk` config and `build.rs` files.
    * `ripple fmt` will run the ripple formatter on `.ripple` files (it will also run `rustfmt` for your connvinence)
* `ripple_transpiler` - This crate you will see in your build depdendncies and is used in `build.rs` to transpile `.ripple` files in the source directory
* `ripple_runtime` - contains a lot of runtime code that the transpiled files will use, also contains some public facing apis for spawning components as well as utility wrappers around js functions like `interval`

# Surf
Surf is a html and css lsp written in rust with many modern features, its part of the ripple project, but is targeted towards everyone excetp ripple devs. Surf is already included in the ripple lsp, and the surf standalone binary and formatter are for generic `.html` and `.css` files.

# Private crates
These crates are part of the ripple projects as dependencies of other crates and you shouldnt really need to worry about them.

* `ripple_parser` - This includes a parser for html, css and ripple files and is depdended on by all other crates (including `surf`).
* `ripple_generator` - this converts the syntax tree from ripple_parser into a well formatted css and html output, used both by the transpiler to reemit modified css and html, and alsos by the lsps and formatters. 
* `ripple_intel` - includes information and lsp-like insihgt for css, html. primarly used by `surf`, but also used by the transpiler for a few decisions (primerly for knowing wether a html tag is real or should be treated as a custom component).
* `ripple_lsp` - the actual ripple lsp, can be installed as a standalone binary (which most editor extensions do)

# Full crate graph
* `ripple_cli` - `ripple_lsp`, `ripple_generator`, `ripple_parser`
* `ripple_transpiler` - `ripple_parser`, `ripple_generator`, `ripple_intel`
* `ripple_runtime` - `web-sys`
* `ripple_lsp` - `surf`, `ripple_transpiler`, `ripple_parser`, `ripple_generator`
* `surf` - `ripple_intel`, `ripple_parser`, `ripple_generator`,
* `ripple_intel` - 
* `ripple_generator` - 
* `ripple_parser` - TreeSitter

