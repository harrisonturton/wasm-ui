# Accessibility

Accessibility is required for production release. I expect this to be one of the
most platform-specific parts of the implementation. I've mostly focused on
backend work, so I haven't dived deep into accessibility before. Time to learn
something new!

## Web

On the web accessibility is done through a set of attributes in the DOM, [called ARIA.](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA) These attributes introduce more metadata so that an [accessibility tree](https://developer.mozilla.org/en-US/docs/Glossary/Accessibility_tree) can be generated from the DOM structure, which is then consumed by screenreaders.

So on the web, I think the only option is to maintain a parallel DOM structure.
This won't need to be rendered, so I assume this will be pretty efficient. It
could be set as `display: none` so it doesn't undergo layout or painting.

This would have to be generated from the component tree *before* layout, because
once we have a `LayoutTree`, we've lost all the semantic information about the
components. So this would have to be an additional part of the render pipeline
between creating the widget tree and layout.

## Mac OSX + iOS

[Apple seems to have great support.](https://developer.apple.com/accessibility/macos/) There is a lot of inspiration to be taken from their work.

I wonder if we can do something similar to accessibility on the web, but instead
of maintaining a parallel DOM structure, we do it with native OSX components?

## Windows

*Todo*

## Linux

From the tiny bit of searching I've done, it seems like accessibility on Linux
is not in the greatest state. It'd be easy to include some things, like
increasing constrast for visual impairment, but interop with screenreaders might
be difficult. Maybe it'd be best to provide two builds, one that renders
directly to OpenGL without accessibility built in, and one that uses a webview
like Electron that can use the same accessibility tooling that would be built
for the web?