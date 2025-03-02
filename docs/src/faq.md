# FAQ

## Do I need to use the natrix cli?
No! the natrix cli (currently) is only for project generation and [css treeshaking](TODO).

## Do I need to use Trunk?
Nope! All natrix needs is a html element to mount to, it doesnt care how its built.

## Can I use natrix in a bigger application?
Yep! Just expose a rust function using [`wasm_bindgen`](TODO) that calls [`mount_component`](TODO) with the target node

<div class="warning">

`mount_component` assumes its target will never be removed and as such is a memory leak if called multiple times.
Currently theres isnt a mechanism to pass component lifetime mangement to JS. If you are embedding natrix in another rust framework consider [`render_component`](TODO) which will return the component;

</div>

## Is natrix ever gonna have a `html!` macro?
Short answer, no.

Long answer, its this frameworks opinion that DSL macros do more harm than good because of formatting and lsp issues.
