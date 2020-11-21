#![allow(dead_code)]

mod brush;
mod context;
mod engine;
mod pointer_state;
use context::{get_context, ContextOptions};
use engine::Engine;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlDivElement};

#[wasm_bindgen]
pub struct Painter {
    engine: Rc<RefCell<Engine>>,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Painter {
    #[wasm_bindgen(constructor)]
    pub fn new(canvasTarget: &HtmlDivElement) -> Result<Painter, JsValue> {
        // create canvas
        let canvas: Option<HtmlCanvasElement>;
        {
            let window = web_sys::window().ok_or(JsValue::from_str("Could not get window"))?;
            let document = window
                .document()
                .ok_or(JsValue::from_str("Could not get document"))?;
            canvas = Some(match document.create_element("canvas") {
                Ok(element) => Ok(element.unchecked_into::<HtmlCanvasElement>()),
                Err(_) => Err(JsValue::from_str("Could not create canvas element")),
            }?);
        }
        // append to canvasTarget
        canvasTarget
            .append_child(canvas.as_ref().unwrap())
            .map_err(|_| JsValue::from_str("Could not append canvas"))?;
        // get webgl2 context
        let gl = Some(get_context(
            canvas.as_ref().unwrap(),
            &ContextOptions {
                alpha: true,
                antialias: true,
                depth: true,
                powerPreference: "high-performance",
                preserveDrawingBuffer: false,
                premultipliedAlpha: true,
            },
        )?);
        // initialize private impl
        let engine = Engine::new(gl, canvas)?;

        Ok(Self { engine })
    }

    pub fn changeColor(&mut self, color: &[f32]) -> Result<(), JsValue> {
        self.engine.borrow_mut().change_color(color)
    }
}
