use super::brush::Brush;
use super::pointer_state::PointerState;
use super::shader;
use js_sys::Float32Array;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    console, HtmlCanvasElement, MouseEvent, UiEvent, WebGl2RenderingContext as WGL2,
    WebGlFramebuffer, WebGlProgram, WebGlTexture,
};

pub struct Engine {
    gl: Option<WGL2>,
    canvas: Option<HtmlCanvasElement>,
    canvas_tex: Option<WebGlTexture>,
    canvas_fb: Option<WebGlFramebuffer>,
    tri_program: Option<WebGlProgram>,
    quad_program: Option<WebGlProgram>,
    pointer_state: PointerState,
    brush: Brush,
}

impl Engine {
    pub fn new(
        gl: Option<WGL2>,
        canvas: Option<HtmlCanvasElement>,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        let this = Rc::new(RefCell::new(Engine {
            gl,
            canvas,
            canvas_tex: None,
            canvas_fb: None,
            tri_program: None,
            quad_program: None,
            pointer_state: PointerState::new(),
            brush: Brush::new(&[0.5, 0.5, 0.5, 1.0])?,
        }));

        // set blend func, call glenable, etc
        this.borrow().set_gl_capabilities();

        // set initial viewport size (initial canvas width &  clientWidth will not match)
        this.borrow().resize_canvas();

        // create the render target for canvas composite
        this.borrow_mut().create_canvas_fb()?;

        // compile all shaders
        this.borrow_mut().compile_shaders()?;

        // clear default framebuffer
        this.borrow().clear(1.0, 1.0, 1.0, 1.0);

        // add all event handlers
        Self::init_handlers(this.clone())?;
        Ok(this)
    }

    pub fn change_color(&mut self, color: &[f32]) -> Result<(), JsValue> {
        self.brush = Brush::new(color)?;
        Ok(())
    }

    fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        let gl = self.gl.as_ref().unwrap();
        gl.clear_color(r, g, b, a);
        gl.clear(WGL2::COLOR_BUFFER_BIT);
    }

    fn set_gl_capabilities(&self) {
        let gl = self.gl.as_ref().unwrap();
        gl.enable(WGL2::BLEND);
        gl.blend_func(WGL2::SRC_ALPHA, WGL2::ONE_MINUS_SRC_ALPHA);
    }

    fn get_canvas_size(&self) -> (f32, f32) {
        let canvas = self.canvas.as_ref().unwrap();
        let width = canvas.width() as f32;
        let height = canvas.height() as f32;
        (width, height)
    }

    fn create_canvas_fb(&mut self) -> Result<(), JsValue> {
        let gl = self.gl.as_ref().unwrap();
        let canvas = self.canvas.as_ref().unwrap();
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        let level = 0;
        let border = 0;
        let texture = gl.create_texture();
        gl.bind_texture(WGL2::TEXTURE_2D, texture.as_ref());
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WGL2::TEXTURE_2D,
            level,
            WGL2::RGBA as i32,
            width,
            height,
            border,
            WGL2::RGBA,
            WGL2::UNSIGNED_BYTE,
            None,
        )?;
        gl.tex_parameteri(
            WGL2::TEXTURE_2D,
            WGL2::TEXTURE_MIN_FILTER,
            WGL2::LINEAR as i32,
        );
        gl.tex_parameteri(
            WGL2::TEXTURE_2D,
            WGL2::TEXTURE_WRAP_S,
            WGL2::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WGL2::TEXTURE_2D,
            WGL2::TEXTURE_WRAP_T,
            WGL2::CLAMP_TO_EDGE as i32,
        );
        let fb = gl.create_framebuffer();
        gl.bind_framebuffer(WGL2::FRAMEBUFFER, fb.as_ref());
        gl.framebuffer_texture_2d(
            WGL2::FRAMEBUFFER,
            WGL2::COLOR_ATTACHMENT0,
            WGL2::TEXTURE_2D,
            texture.as_ref(),
            level,
        );
        gl.viewport(0, 0, width, height);
        self.clear(1.0, 1.0, 1.0, 1.0);
        gl.bind_framebuffer(WGL2::FRAMEBUFFER, None);
        gl.bind_texture(WGL2::TEXTURE_2D, None);
        self.canvas_fb = fb;
        self.canvas_tex = texture;
        Ok(())
    }

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

    fn draw_tri(&self, len: f32, pos_x: f32, pos_y: f32) -> Result<(), JsValue> {
        let gl = self.gl.as_ref().unwrap();

        let d = len / 3.0f32.sqrt();

        // triangle vertices
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
        gl.bind_buffer(WGL2::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let vert_array = Float32Array::view(&vertices);

            gl.buffer_data_with_array_buffer_view(
                WGL2::ARRAY_BUFFER,
                &vert_array,
                WGL2::STATIC_DRAW,
            );
        }

        gl.vertex_attrib_pointer_with_i32(0, 3, WGL2::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        let program = self.tri_program.as_ref();
        gl.use_program(program);

        let uniform_loc = gl.get_uniform_location(program.unwrap(), "color");
        gl.uniform4fv_with_f32_array(uniform_loc.as_ref(), &self.brush.color);
        // draw to canvas framebuffer
        gl.bind_framebuffer(WGL2::FRAMEBUFFER, self.canvas_fb.as_ref());

        let canvas = self.canvas.as_ref().unwrap();
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        gl.viewport(0, 0, width, height);

        gl.draw_arrays(WGL2::TRIANGLES, 0, 3);
        gl.flush();
        Ok(())
    }

    fn draw_canvas(&self) -> Result<(), JsValue> {
        let gl = self.gl.as_ref().unwrap();

        let vertices: [f32; 24] = [
            -1.0, 1.0, 0.0, 1.0, //
            -1.0, -1.0, 0.0, 0.0, //
            1.0, -1.0, 1.0, 0.0, //
            -1.0, 1.0, 0.0, 1.0, //
            1.0, -1.0, 1.0, 0.0, //
            1.0, 1.0, 1.0, 1.0,
        ];

        let vert_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WGL2::ARRAY_BUFFER, Some(&vert_buffer));

        unsafe {
            let vert_array = Float32Array::view(&vertices);

            gl.buffer_data_with_array_buffer_view(
                WGL2::ARRAY_BUFFER,
                &vert_array,
                WGL2::STATIC_DRAW,
            );
        }

        gl.vertex_attrib_pointer_with_i32(
            0,
            2,
            WGL2::FLOAT,
            false,
            4 * 4, /* sizeof float */
            0,
        );
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_with_i32(1, 2, WGL2::FLOAT, false, 4 * 4, 2 * 4);
        gl.enable_vertex_attrib_array(1);

        let program = self.quad_program.as_ref();
        gl.use_program(program);

        // draw to default framebuffer
        gl.bind_framebuffer(WGL2::FRAMEBUFFER, None);
        self.clear(0.0, 0.0, 0.0, 0.0);
        let canvas = self.canvas.as_ref().unwrap();
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        gl.viewport(0, 0, width, height);

        gl.bind_texture(WGL2::TEXTURE_2D, self.canvas_tex.as_ref());
        gl.draw_arrays(WGL2::TRIANGLES, 0, 6);
        gl.flush();
        Ok(())
    }

    fn compile_shaders(&mut self) -> Result<(), JsValue> {
        let gl = self.gl.as_ref().unwrap();

        /*
         Triangle shader
        */
        let vert =
            shader::compile_shader(&gl, WGL2::VERTEX_SHADER, shader::BRUSH_VERTEX_SHADER_SRC)
                .map_err(|shader_log| {
                    JsValue::from_str(
                        format!("Unable to compile vertex shader:\n{}", shader_log).as_str(),
                    )
                })?;

        let frag = shader::compile_shader(
            &gl,
            WGL2::FRAGMENT_SHADER,
            shader::BRUSH_FRAGMENT_SHADER_SRC,
        )
        .map_err(|shader_log| {
            JsValue::from_str(
                format!("Unable to compile fragment shader:\n{}", shader_log).as_str(),
            )
        })?;
        let linked = shader::link_program(&gl, &vert, &frag)
            .map_err(|_| JsValue::from_str("Unable to link shader program"))?;
        self.tri_program = Some(linked);
        gl.delete_shader(Some(&vert));
        gl.delete_shader(Some(&frag));
        /*
         Screen quad shader
        */
        let vert = shader::compile_shader(&gl, WGL2::VERTEX_SHADER, shader::QUAD_VERTEX_SHADER_SRC)
            .map_err(|shader_log| {
                JsValue::from_str(
                    format!("Unable to compile vertex shader:\n{}", shader_log).as_str(),
                )
            })?;

        let frag =
            shader::compile_shader(&gl, WGL2::FRAGMENT_SHADER, shader::QUAD_FRAGMENT_SHADER_SRC)
                .map_err(|shader_log| {
                    JsValue::from_str(
                        format!("Unable to compile fragment shader:\n{}", shader_log).as_str(),
                    )
                })?;
        let linked = shader::link_program(&gl, &vert, &frag)
            .map_err(|_| JsValue::from_str("Unable to link shader program"))?;
        self.quad_program = Some(linked);
        gl.delete_shader(Some(&vert));
        gl.delete_shader(Some(&frag));
        Ok(())
    }

    fn init_handlers(this: Rc<RefCell<Self>>) -> Result<(), JsValue> {
        /*
         Add handlers
        */
        // TODO - use event listeners to drop closures
        {
            // window resize - call gl.viewport
            let this_clone = this.clone();
            let resize = Closure::wrap(Box::new(move |event: UiEvent| {
                // TODO - perspective projection, zoom & pan, etc
                this_clone.borrow().resize_canvas();
            }) as Box<dyn FnMut(_)>);
            web_sys::window()
                .unwrap()
                .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())
                .map_err(|_| JsValue::from_str("Error adding window onresize listener"))?;
            resize.forget();
        }
        {
            // mousemove - draw if pressed
            let this_clone = this.clone();
            let mouse_move = Closure::wrap(Box::new(move |event: MouseEvent| {
                if !this_clone.borrow().pointer_state.pressed() {
                    return;
                }
                let (width, height) = this_clone.borrow().get_canvas_size();
                let offset_x = 2.0 * event.offset_x() as f32 / width - 1.0;
                let offset_y = -(2.0 * event.offset_y() as f32 / height - 1.0);
                // console::log_4(
                //     &"Offset: ".into(),
                //     &offset_x.into(),
                //     &", ".into(),
                //     &offset_y.into(),
                // );
                match this_clone.borrow().draw_tri(0.1, offset_x, offset_y) {
                    Ok(_) => {}
                    Err(_) => {
                        console::log_1(&"engine.draw_tri error".into());
                    }
                }
                match this_clone.borrow().draw_canvas() {
                    Ok(_) => {}
                    Err(_) => {
                        console::log_1(&"engine.draw_canvas error".into());
                    }
                }
            }) as Box<dyn FnMut(_)>);
            this.borrow()
                .canvas
                .as_ref()
                .unwrap()
                .add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref())
                .map_err(|_| JsValue::from_str("Error adding mousemove listener"))?;
            mouse_move.forget();
        }
        {
            // mousedown - set pressed
            let this_clone = this.clone();
            let mouse_down = Closure::wrap(Box::new(move |event: MouseEvent| {
                this_clone.borrow_mut().pointer_state.set_pressed(true);

                // draw one triangle at mouse pos
                let (width, height) = this_clone.borrow().get_canvas_size();
                let offset_x = 2.0 * event.offset_x() as f32 / width - 1.0;
                let offset_y = -(2.0 * event.offset_y() as f32 / height - 1.0);
                // console::log_4(
                //     &"Offset: ".into(),
                //     &offset_x.into(),
                //     &", ".into(),
                //     &offset_y.into(),
                // );
                match this_clone.borrow().draw_tri(0.1, offset_x, offset_y) {
                    Ok(_) => {}
                    Err(_) => {
                        console::log_1(&"engine.draw_tri error".into());
                    }
                }
                match this_clone.borrow().draw_canvas() {
                    Ok(_) => {}
                    Err(_) => {
                        console::log_1(&"engine.draw_canvas error".into());
                    }
                }
            }) as Box<dyn FnMut(_)>);
            this.borrow()
                .canvas
                .as_ref()
                .unwrap()
                .add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref())
                .map_err(|_| JsValue::from_str("Error adding mousedown listener"))?;
            mouse_down.forget();
        }
        {
            // mouseup - unset pressed
            let this_clone = this.clone();
            let mouse_up = Closure::wrap(Box::new(move |event: MouseEvent| {
                this_clone.borrow_mut().pointer_state.set_pressed(false);
            }) as Box<dyn FnMut(_)>);
            this.borrow()
                .canvas
                .as_ref()
                .unwrap()
                .add_event_listener_with_callback("mouseup", mouse_up.as_ref().unchecked_ref())
                .map_err(|_| JsValue::from_str("Error adding mouseup listener"))?;
            mouse_up.forget();
        }

        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let gl = self.gl.as_ref().unwrap();
        // TODO - safely delete everything
        gl.delete_program(self.tri_program.as_ref());
        gl.delete_program(self.quad_program.as_ref());
        gl.delete_framebuffer(self.canvas_fb.as_ref());
        gl.delete_texture(self.canvas_tex.as_ref());
    }
}
