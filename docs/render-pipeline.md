# Render Pipeline

The render pipeline is roughly based on [this tech talk](https://www.youtube.com/watch?v=UUfXWzp0-DU)
about Flutter's render pipeline. However, it is *much* simpler and probably less
efficient.

There are two stages:

1. Generate widget tree
2. Layout
3. Paint

A more detailed time looks like this:

1. The platform-specific code calls the `tick` method on the `AppDriver` instance.
2. This returns the widget tree.
3. This widget tree is used to generate a `LayoutTree`, which contains a hierarchy of `LayoutBox` elements.
4. These boxes are passed to the platform driver, which then paints them to the screen.

`tick` is called up to 60 times a second, which means steps 1-4 must be
completed in under 16ms.