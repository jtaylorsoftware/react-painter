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

pub const QUAD_VERTEX_SHADER_SRC: &str = r#"#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 texcoords;

out vec2 out_texcoords;

void main() {
    out_texcoords = texcoords;
    gl_Position = vec4(position, 0.0, 1.0);
}
"#;

pub const QUAD_FRAGMENT_SHADER_SRC: &str = r#"#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

in vec2 out_texcoords;
out vec4 out_color;
uniform sampler2D tex;

void main() {
    out_color = vec4(texture(tex, out_texcoords).rgb, 1.0);
    // out_color = vec4(0.0, 1.0, 0.0, 1.0);
} 
"#;
