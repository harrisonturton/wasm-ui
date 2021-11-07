// Position of the vertex
attribute vec2 a_position;

// Pixel dimensions of the canvas
uniform vec2 u_viewport;

void main() {
    vec2 zero_to_one = a_position / u_viewport;
    vec2 zero_to_two = zero_to_one * 2.0;
    vec2 clip_space = zero_to_two - 1.0;
    gl_Position = vec4(clip_space, 0.0, 1.0);
}