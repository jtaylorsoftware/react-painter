#![allow(dead_code)]

use js_sys::Float32Array;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    console, HtmlCanvasElement, HtmlDivElement, MouseEvent, UiEvent, WebGl2RenderingContext,
    WebGlProgram, WebGlShader,
};

#[wasm_bindgen]
pub struct Painter {
    engine: Rc<Engine>,
}

struct Engine {
    gl: Option<Rc<WebGl2RenderingContext>>,
    canvas: Option<Rc<HtmlCanvasElement>>,
    vert_shader: Rc<RefCell<Option<WebGlShader>>>,
    frag_shader: Rc<RefCell<Option<WebGlShader>>>,
    program: Rc<RefCell<Option<WebGlProgram>>>,
    pointer_state: Rc<RefCell<PointerState>>,
    brush: Rc<RefCell<Brush>>,
}

struct PointerState {
    pressed: bool,
}

impl PointerState {
    pub fn new() -> Self {
        Self { pressed: false }
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }
}

#[wasm_bindgen]
pub struct Brush {
    color: [f32; 3],
}

impl Brush {
    pub fn new(color: &[f32]) -> Self {
        let mut arr: [f32; 3] = [0.0; 3];
        arr.copy_from_slice(color);
        Self { color: arr }
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct ContextOptions {
    pub alpha: bool,
    pub antialias: bool,
    pub depth: bool,
    pub powerPreference: &'static str,
    pub preserveDrawingBuffer: bool,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Painter {
    #[wasm_bindgen(constructor)]
    pub fn new(canvasTarget: &HtmlDivElement) -> Result<Painter, JsValue> {
        // create canvas
        let canvas: HtmlCanvasElement;
        {
            let window = web_sys::window().ok_or(JsValue::from_str("Could not get window"))?;
            let document = window
                .document()
                .ok_or(JsValue::from_str("Could not get document"))?;
            canvas = match document.create_element("canvas") {
                Ok(element) => Ok(element.unchecked_into::<HtmlCanvasElement>()),
                Err(_) => Err(JsValue::from_str("Could not create canvas element")),
            }?;
        }
        // append to canvasTarget
        canvasTarget
            .append_child(&canvas)
            .map_err(|_| JsValue::from_str("Could not append canvas"))?;
        // get webgl2 context
        let gl = Rc::new(get_context(&canvas)?);
        let canvas = Rc::new(canvas);
        // initialize private impl
        let engine = Rc::new(Engine {
            gl: Some(gl.clone()),
            canvas: Some(canvas.clone()),
            vert_shader: Rc::new(RefCell::new(None)),
            frag_shader: Rc::new(RefCell::new(None)),
            program: Rc::new(RefCell::new(None)),
            pointer_state: Rc::new(RefCell::new(PointerState::new())),
            brush: Rc::new(RefCell::new(Brush::new(&[0.5, 0.5, 0.5]))),
        });

        // set initial viewport size (initial canvas width &  clientWidth will not match)
        engine.resize_canvas();
        engine.clear_canvas();
        // add handlers
        // TODO - use event listeners to drop closures
        {
            // window resize - call gl.viewport
            let engine = engine.clone();
            let resize = Closure::wrap(Box::new(move |event: UiEvent| {
                engine.resize_canvas();
                engine.clear_canvas();
            }) as Box<dyn FnMut(_)>);
            web_sys::window()
                .unwrap()
                .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())
                .map_err(|_| JsValue::from_str("Error adding window onresize listener"))?;
            resize.forget();
        }
        {
            // mousemove - draw if pressed
            let engine = engine.clone();
            let canvas_clone = engine.canvas.as_ref().unwrap().clone();
            let mouse_move = Closure::wrap(Box::new(move |event: MouseEvent| {
                if !engine.pointer_state.borrow().pressed() {
                    return;
                }

                let width = canvas_clone.width() as f32;
                let height = canvas_clone.height() as f32;
                let offset_x = 2.0 * event.offset_x() as f32 / width - 1.0;
                let offset_y = -(2.0 * event.offset_y() as f32 / height - 1.0);
                // console::log_4(
                //     &"Offset: ".into(),
                //     &offset_x.into(),
                //     &", ".into(),
                //     &offset_y.into(),
                // );
                match engine.draw_tri(0.1, offset_x, offset_y) {
                    Ok(_) => {}
                    Err(_) => {
                        console::log_1(&"engine.draw_tri error".into());
                    }
                }
            }) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref())
                .map_err(|_| JsValue::from_str("Error adding mousemove listener"))?;
            mouse_move.forget();
        }
        {
            // mousedown - set pressed
            let engine = engine.clone();
            let canvas_clone = engine.canvas.as_ref().unwrap().clone();
            let mouse_down = Closure::wrap(Box::new(move |event: MouseEvent| {
                engine.pointer_state.borrow_mut().set_pressed(true);

                // draw one triangle at mouse pos
                let width = canvas_clone.width() as f32;
                let height = canvas_clone.height() as f32;
                let offset_x = 2.0 * event.offset_x() as f32 / width - 1.0;
                let offset_y = -(2.0 * event.offset_y() as f32 / height - 1.0);
                // console::log_4(
                //     &"Offset: ".into(),
                //     &offset_x.into(),
                //     &", ".into(),
                //     &offset_y.into(),
                // );
                match engine.draw_tri(0.1, offset_x, offset_y) {
                    Ok(_) => {}
                    Err(_) => {
                        console::log_1(&"engine.draw_tri error".into());
                    }
                }
            }) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref())
                .map_err(|_| JsValue::from_str("Error adding mousedown listener"))?;
            mouse_down.forget();
        }
        {
            // mouseup - unset pressed
            let engine = engine.clone();
            let mouse_up = Closure::wrap(Box::new(move |event: MouseEvent| {
                engine.pointer_state.borrow_mut().set_pressed(false);
            }) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("mouseup", mouse_up.as_ref().unchecked_ref())
                .map_err(|_| JsValue::from_str("Error adding mouseup listener"))?;
            mouse_up.forget();
        }

        engine.compile_shaders()?;

        Ok(Self { engine })
    }

    pub fn changeColor(&mut self, color: &[f32]) {
        *self.engine.brush.borrow_mut() = Brush::new(color);
    }
}

impl Engine {
    fn resize_canvas(&self) {
        let gl = self.gl.as_ref().unwrap();
        let canvas = self.canvas.as_ref().unwrap();
        let rect = canvas.get_bounding_client_rect();
        let client_width = rect.width() as u32;
        let client_height = rect.height() as u32;

        let width = canvas.width();
        let height = canvas.height();

        if width != client_width || height != client_height {
            canvas.set_width(client_width);
            canvas.set_height(client_height);
        }

        // console::log_4(
        //     &"Resize: w:".into(),
        //     &client_width.into(),
        //     &", h:".into(),
        //     &client_height.into(),
        // );

        gl.viewport(0, 0, client_width as i32, client_height as i32);
    }

    fn clear_canvas(&self) {
        let gl = self.gl.as_ref().unwrap();
        // clear to black
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }

    fn draw_tri(&self, len: f32, pos_x: f32, pos_y: f32) -> Result<(), JsValue> {
        let gl = self.gl.as_ref().unwrap();

        let d = len / 3.0f32.sqrt();

        // ccw vertices
        let vertices: [f32; 9] = [
            // left
            -d + pos_x,
            -d + pos_y,
            0.0,
            // right
            d + pos_x,
            -d + pos_y,
            0.0,
            //top
            pos_x,
            d + pos_y,
            0.0,
        ];

        let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let vert_array = Float32Array::view(&vertices);

            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        let program = self.program.borrow();
        gl.use_program(program.as_ref());
        let uniform_loc = gl.get_uniform_location(program.as_ref().unwrap(), "color");
        let color = &self.brush.borrow().color[..];
        gl.uniform3fv_with_f32_array(uniform_loc.as_ref(), color);
        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            (vertices.len() / 3) as i32,
        );
        Ok(())
    }

    fn compile_shaders(&self) -> Result<(), JsValue> {
        let gl = self.gl.as_ref().unwrap();

        // compile shaders
        let compiled = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            r#"#version 300 es

        #ifdef GL_FRAGMENT_PRECISION_HIGH
            precision highp float;
        #else
            precision mediump float;
        #endif
        
        in vec4 position;
        void main() {
            gl_Position = position;
        }
        "#,
        )
        .map_err(|shader_log| {
            JsValue::from_str(format!("Unable to compile vertex shader:\n{}", shader_log).as_str())
        })?;
        *self.vert_shader.borrow_mut() = Some(compiled);

        let compiled = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r#"#version 300 es

        #ifdef GL_FRAGMENT_PRECISION_HIGH
            precision highp float;
        #else
            precision mediump float;
        #endif

        uniform vec3 color;
        out vec4 out_color;
        void main() {
            out_color = vec4(color, 1.0);
        }
        "#,
        )
        .map_err(|shader_log| {
            JsValue::from_str(
                format!("Unable to compile fragment shader:\n{}", shader_log).as_str(),
            )
        })?;
        *self.frag_shader.borrow_mut() = Some(compiled);
        let linked = link_program(
            &gl,
            self.vert_shader.borrow_mut().as_ref().unwrap(),
            self.frag_shader.borrow_mut().as_ref().unwrap(),
        )
        .map_err(|_| JsValue::from_str("Unable to link shader program"))?;
        self.program.replace(Some(linked));

        Ok(())
    }
}

fn get_context(canvas: &HtmlCanvasElement) -> Result<WebGl2RenderingContext, JsValue> {
    let gl = canvas
        .get_context_with_context_options(
            "webgl2",
            &JsValue::from_serde(&ContextOptions {
                alpha: true,
                antialias: true,
                depth: true,
                powerPreference: "high-performance",
                preserveDrawingBuffer: true,
            })
            .unwrap(),
        )
        .map_err(|_| JsValue::from_str("WebGl2 not supported"))?
        .unwrap()
        .unchecked_into::<WebGl2RenderingContext>();

    Ok(gl)
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
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

fn link_program(
    gl: &WebGl2RenderingContext,
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
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
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
