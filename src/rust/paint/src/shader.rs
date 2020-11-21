use web_sys::{WebGl2RenderingContext as WGL2, WebGlProgram, WebGlShader};

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

pub fn compile_shader(gl: &WGL2, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WGL2::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    gl: &WGL2,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WGL2::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
