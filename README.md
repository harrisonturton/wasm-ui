# 🖼 wasm-ui

Heyo, welcome to my little project. This is my attempt to build a web
application with Rust. Instead of using HTML+CSS like a normal person, I want to
build everything using WebGL. 

It's painful. It's not super efficient. But it's a fun and interesting
challenge.

Basically nothing works yet.

## Usage

I personally find Flutter really nice to use, so I've tried to design this
interface to be fairly similar. It's a bit more verbose due to Rust's
explicitness, but still readable.

I'm hoping to eventually write a procedural macro to make writing the widget
tree a bit nicer.

### App Boilerplate

The library only requires that your application implements the `AppDriver`
trait. This allows your app to be "driven" by a variety of different platforms.

```rust
pub trait AppDriver {
    fn tick(&mut self, time: f32) -> Box<dyn Layout>;
}
```

This is called every frame. The `Layout` trait is implemented by widgets that
can be rendered by the `wasm-ui` library. For example, a simple app might look
like:

```rust
pub struct App {
    position: (f32, f32),
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        Box::new(Container {
            size: (100.0, 100.0).into(),
            color: Color::blue(),
            ..Default::default()
        })
    }
}
```

This will render a blue square that is 100 pixels wide and 100 pixels tall.

Note the usage of `Default::default()`. This allows us to only define the fields
we need, rather than being forced to specify every single in a widget. In this case, `Container` is defined like this:

```rust
pub struct Container {
    pub size: Vector2,
    pub color: Color,
    pub child: Option<Box<dyn Layout>>,
}
```

By using `..Default::default()`, it automatically sets `Container.child` to
`None`. In this example it doesn't help us too much, but it's more useful with
widgets that are highly configurable.

### Flex Containers

`wasm-ui` has two main flex containers, `Row` and `Column`.

```rust
Box::new(Row {
    cross_axis_alignment: CrossAxisAlignment::Center,
    main_axis_alignment: MainAxisAlignment::SpaceEvenly,
    children: vec![
        Flex::Fixed {
            child: Box::new(Container{
                size: (100.0, 200.0).into(),
                color: Color::red(),
                ..Default::default()
            }),
        },
        Flex::Fixed {
            child: Box::new(Container{
                size: (100.0, 100.0).into(),
                color: Color::green(),
                ..Default::default()
            }),
        },
        Flex::Fixed {
            child: Box::new(Container{
                size: (100.0, 100.0).into(),
                color: Color::blue(),
                ..Default::default()
            }),
        },
    ]
})
```

This will position three rectangles – red, green and blue – horizontally in the
center of the screen. The red rectangle will be twice as tall as the green and blue squares.

<img width="1074" alt="Screen Shot 2021-11-14 at 2 11 00 pm" src="https://user-images.githubusercontent.com/20736299/141665966-1013fc2b-0f72-490a-a5e8-101eb2fd97a9.png">

If we change the green square to this:

```rust
Flex::Flexible {
    flex: 1.0,
    child: Box::new(Container{
        size: (100.0, 100.0).into(),
        color: Color::green(),
        ..Default::default()
    }),
},
```

Then it will expand to fill the screen in the horizontal direction, pushing the
red and blue squares to the edges of the screen.

<img width="1074" alt="Screen Shot 2021-11-14 at 2 11 15 pm" src="https://user-images.githubusercontent.com/20736299/141665968-8d6606e5-bb8b-4a8d-9df4-7e80c0a1a240.png">

## Roadmap

### Figuring out what's possible

These are messy one-off experiments to make sure I can actually build
everything.

- [x] Build + deploy Rust on the web
- [x] Control a WebGL canvas with Rust
- [x] Render glyphs from a font
- [x] Implement a basic box layout algorithm (based on Flutter + CSS box model)
- [x] Pass browser events to the Rust library (cross the JS-WASM boundary)

### Actual Roadmap

Now that I've finished with the little experiments, I'm confident that building
the UI library is *possible*. Not easy, but possible. The current challenge is
to combine all the individual spike projects, ideally with some half-decent
architecture.

- [ ] Build WebGL render driver
- [ ] Build data structure to represent the UI
- [ ] Implement box layout algorithm

Maybe I should write an article for each roadmap item 🤔

## The codebase

`core/` is the Rust implementation. This gets compiled to WASM and async loaded
in the browser.

`core/crates/render` implements the rendering logic and app drivers. The drivers
sit between the application logic and the deploy target. For instance, the
browser driver translates browser events into something the application can
understand. It also takes the render primitive output of the application and
draws it on the WebGL canvas.

Currently I've just implemented this for the browser, but I'm trying to keep it
generic. There should be pretty easy to port to any OpenGL target, so I imagine
it wouldn't be too much extra work to support native platforms.

`core/crates/math` a generic linear algebra math library. Implements things like
vectors and matrices.

`core/src` is the demo application to demonstrate how the UI libraries could be
used.

`bin/` holds some helper scripts to build the project and run the dev server.

`web/` contains the scaffolding for the web target. This is a small React
project that does nothing except async load the WASM libraries and initialises
some event listeners to pass browser events to the browser driver.

## Building

`./bin/build` will build everything and shove it in the `/build` folder. You can
run `./bin/build <core|web>` to only build the core or web parts.

`./bin/start` will start the dev server on `:8080`. This will automatically
detect changes in the web directory, but you'll have to manually rebuild any
Rust changes with `./bin/build core`.
