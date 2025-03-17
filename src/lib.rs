use macroquad::{
    self,
    miniquad::{
        conf::{Platform, WebGLVersion},
        start,
    },
};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn run_demo(demo_data: Vec<u8>, canvas_id: String, window_width: f32, window_height: f32) {
    // execute macroquad on the canvas
    let conf = macroquad::window::Conf {
        window_title: "BasicShapes".to_string(),
        window_width: window_width as i32,
        window_height: window_height as i32,
        high_dpi: false,
        fullscreen: false,
        sample_count: 1,
        window_resizable: false,
        icon: Default::default(),
        platform: Platform {
            webgl_version: WebGLVersion::WebGL2,
            ..Default::default()
        },
    };

    //native::wasm::run(&conf, || {});
}
