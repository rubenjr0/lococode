use wasm_bindgen::prelude::*;

use crate::Coordinates;
use crate::Decoded;

#[wasm_bindgen]
pub struct Coords(Coordinates<f64>);

#[wasm_bindgen]
impl Coords {
    #[wasm_bindgen(constructor)]
    pub fn new(latitude: f64, longitude: f64) -> Coords {
        Self(Coordinates::new(latitude, longitude))
    }

    pub fn encode(&self, code_length: u8) -> String {
        self.0.encode(code_length)
    }
}

#[wasm_bindgen]
pub fn decode(code: &str) -> Decoded {
    Coordinates::<f64>::decode(code)
}
