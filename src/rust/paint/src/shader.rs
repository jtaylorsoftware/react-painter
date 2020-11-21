pub const BRUSH_VERTEX_SHADER_SRC: &str = r#"#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

in vec4 position;
void main() {
    gl_Position = position;
}
"#;

pub const BRUSH_FRAGMENT_SHADER_SRC: &str = r#"#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform vec4 color;
out vec4 out_color;
void main() {
    out_color = color;
}
"#;
