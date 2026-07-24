mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, npm-wasm!");
}

fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

#[wasm_bindgen]
pub fn u32_add(a: u32, b: u32) -> u32 {
    add(a, b)
}
