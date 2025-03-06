use nbs_rs::NbsParser;
use std::str;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn play_file(nbs_file_data: Vec<u8>) -> String {
    let mut parser = NbsParser::new(nbs_file_data.as_slice());
    let nbs_file = parser.parse().unwrap();
    let song_name_bytes = nbs_file.header.song_name;
    str::from_utf8(&song_name_bytes).unwrap().to_string()
}
