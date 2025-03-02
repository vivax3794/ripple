# Getting Started

## Dependencies
While not strictly needed the project template does use the following tools, which are also hightly recommended in general.
* [justfile](TODO) - Task runner
* [Trunk](TODO) - Wasm builder
* [wasm-opt](TODO) - Wasm optimizer
* [wasm-strip](TODO) - strips uneeded code from wasm files

## Install natrix cli
(TODO: make cli)
> This is only strictly needed for project generation and css tree-shacking

## Generate a project
(TODO: make project template)
For more details on the structure and non-default settings in the template see [Project Tempalte](./template.md). It goes into a bit of detail on standard rust-wasm files such as `Trunk.toml` and `index.html`, so its recommended to read if you are new to the ecosystem as the rest of the book will only cover Natrix specific features.

## Run the dev server
you can start a dev server that automatically reload the page on changes with:
```bash
just dev
```

## Build the project
Running
```bash
just build
```
Will compile the project and run the wasm optimizer and stripper. The result is put in `./dist`

## Test the project
You can run your unit tests with 
```bash
just test
```
See [Testing](TODO) for more information
