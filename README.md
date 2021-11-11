# 🖼 wasm-ui

Heyo, welcome to my little project. This is my attempt to build a web
application with Rust. Instead of using HTML+CSS like a normal person, I want to
build everything using WebGL. 

It's painful. It's not super efficient. But it's a fun and interesting
challenge.

Basically nothing works yet.

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