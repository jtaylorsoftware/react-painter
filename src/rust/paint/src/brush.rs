use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Brush {
    #[wasm_bindgen(skip)]
    pub color: [f32; 4],
}
#[wasm_bindgen]
impl Brush {
    #[wasm_bindgen(constructor)]
    pub fn new(color: &[f32]) -> Result<Brush, JsValue> {
        match color.len() {
            4 => {
                let mut color_arr = [0f32; 4];
                color_arr.copy_from_slice(color);
                Ok(Self { color: color_arr })
            }
            _ => Err("Invalid color length".into()),
        }
    }
}
