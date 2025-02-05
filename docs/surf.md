(TODO: this is a draft document, none of the features mentioned are implmented :P)

# Running

## Cli
* `surf fmt` (`f`/`format`): format provided folders and files, or everything in current directory.
* `surf lint`: Runs a single diagnostics pass on the provided files and folders
* `surf`: starts the lsp server, this should almost never be ran manually and instead be spawned by your ide

## LSP
(TODO: make vscode,nvim,... plugins)

the surf lsp is spawned using `surf`, the instructions are different for each editor, just search up "running custom lsp for X", for example in neovim its:
```lua
require('lspconfig.configs').surf = {
    default_config = {
        cmd = { "use/bin/surf" }, -- Replace with path to surf binary
        filetypes = { "css", "html" },
        settings = {},
        root_dir = function(fname)
            return lspconfig.util.find_git_ancestor(fname)
        end;
    },
}

-- Enable your custom server
lspconfig.surf.setup({
    on_attach = function(client, bufnr)
    end,
})
```

# Config file
each lint can be disabled using a `surf.toml` file in the root of the project, surf will also read from `ripple.toml` if present. surf will read values either from `surf` header or a `lsp` header.
These settings can also be set in your IDE.

* `disable_lints` - a list of lint codes that will be ignored.
* ... and more (listed in their respective feature sections.)

# Terms
Heres some terms for the following docs.

## Parent Context
Surf will in some cases be able to tell when a selector is a more specific version of another (`:hover`, `:clikced`, etc) and provided enchanced diagnostics and completions.
When a section talks about a "selector context" below it can be impled it includes information from these other selectors.

## Inlay hints
inlay hints will be shown using comments in order for the syntax highligthing in markdown to make them gray, etc. but in editor they will show up as just the comment content.
i.e when the docs show `/* some hint */ abc` what will actually be in editor is just `some hint abc` where `some hint` is grayed out.

# Features

Surf aims to have features parity with VsCode, so only additional features are listed here.


## Sane defaults
Okay these arent "new features", we just have them on by default unlike vscode, so you might not even realized vscode had these :P

* Duplicate properties.
* Putting units on `0`

## Argument inlay hints
Surf will insert inlay hints for multi argument properties.
```css
button {
    margin /* vertical: */ 0 /* horizontal: */ 0;
}
```
can be disabled with `arugment_hints = false`

## Argument inlay hints, properties style (Default: OFF)
A alternative way to display the argument info is:
```css
button {
    margin 0 10px;
    /*
    margin-top: 0;
    margin-bottom: 0;
    margin-left: 10px;
    margin-right: 10px;
    */
}
```
Which takes up much more space, but might be preffered by some people.
can be enabled with `argument_hints_property = true`

## Bad Contrast
Surf will compare the contrast between specific attributes in a block, such as below: 
```css
button {
    color: #FFF; /* WARNING: Bad contrast between `color` and `background-color` */
    background-color: #FFE; /* WARNING: Bad contrast between `color` and `background-color` */
}
```

This also works between multiple bodies in the same selector context {
```css
button {
    color: #FFF;
    background-color: #000;
}

button:hover {
    background-color: #FFE; /* WARNING: Bad contrast between `color` and `background-color` */
}
```
The contrast threshold can be configured with `contrast_threshold = ...` (TODO: figure out this value range.)

## Parent inlay hints
When a value is overwriten in a "child" in a selector context you will get a inlay hint with the previous value:
```css
button {
    font-size: 12px;
}
button:hover {
    font-size: 14px; /* button: ~12 px~ */
}
```

can be disabled with `overwrite_hints = false`

(TODO: SO SO MUCH MORE)
