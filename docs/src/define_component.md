# Defining a Component

A component is created using both the `#[derive(Component)]` macro ***AND*** doing `impl Component` for the type. This is because the derive macro actually implements `ComponentBase` (which `Component` requires). A minimal component looks like the following.

```rust
# use natrix::prelude::*;
#
#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {
    fn render() -> impl Element<Self::Data> {
        e::h1().text("Hello World!")
    }
}
```

the most important part of a component is the `render` function, which can return anything that implements [`Element`](TODO), most commonly that will be [html nodes](TODO) such as the `e::h1()` above. You will notice that the `Element` trait is generic, and specifically it should be `Element<Self::Data>`, the reason for this is providing strong compiletime checking for reactive elements and event handlers, as covered in [Reactivity](TODO). 

This is a very simple component which simply generates 
```html
<h1>Hello World!</h1>
```

## Mounting a component
Now that you have your component you can mount it to your app using [`mount_component`](TODO)

```rust
# use natrix::prelude::*;
#
# #[derive(Component)]
# struct HelloWorld;
#
# impl Component for HelloWorld {
#    fn render() -> impl Element<Self::Data> {
#        e::h1().text("Hello World!")
#    }
# }
#
fn main() {
    mount_component(HelloWorld, "mount");
}
```
The second argument to `mount_component` is the id of the element to replace. For example if your `index.html` looks like
```html
<body>
    <div id="mount" />
</body>
```
After natrix loads it would be transformed into
```html
<body>
    <h1>Hello World!</h1>
</body>
```

<div class="warning">

Unlike frameworks like React that mount inside an existing element, Natrix replaces the target element completely. This means any content inside `<div id="mount">` will be removed when the component is mounted.

</div>

This can be used to create a "javascript required message"
```html
<body>
    <div id="mount">
        JavaScript is required for this site to function
    </div>
</body>
```

## Next steps
Next you should give the [`Element`](TODO) and [Reactivity](TODO) chapthers a read, and then you are ready to start writing simple apps in Natrix. But its highly recommended to at least skim the whole book to get familiar with most features available to you.
