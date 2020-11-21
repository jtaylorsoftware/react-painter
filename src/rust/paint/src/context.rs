use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ContextOptions {
    pub alpha: bool,
    pub antialias: bool,
    pub depth: bool,
    pub powerPreference: &'static str,
    pub preserveDrawingBuffer: bool,
    pub premultipliedAlpha: bool,
}

pub fn get_context(
    canvas: &HtmlCanvasElement,
    options: &ContextOptions,
) -> Result<WebGl2RenderingContext, JsValue> {
    let gl = canvas
        .get_context_with_context_options("webgl2", &JsValue::from_serde(options).unwrap())
        .map_err(|_| JsValue::from_str("WebGl2 not supported"))?
        .unwrap()
        .unchecked_into::<WebGl2RenderingContext>();

    Ok(gl)
}
