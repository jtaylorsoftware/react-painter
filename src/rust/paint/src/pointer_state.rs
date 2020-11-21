pub struct PointerState {
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
