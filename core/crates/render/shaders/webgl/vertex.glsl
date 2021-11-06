attribute vec3 position;

uniform mat4 m_model_view; // The modelview matrix
//uniform mat4 m_view; // The position of the camera
//uniform mat4 m_projection; // The projection to the camera (perspective or ortho)

uniform vec2 viewport; // The dimensions of the viewport

void main() {
    gl_Position = vec4(position, 1.0) * m_model_view;


    //gl_Position = m_view * position * m_model;
    // Matrix multiplication needs to be backwards
//    v_color = vec4(0.95, 0.47, 0.39, 1.0);
}