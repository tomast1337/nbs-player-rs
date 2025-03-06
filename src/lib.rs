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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_nbs_file() {
        let file_data = include_bytes!("../test-assets/nyan_cat.nbs");
        let song_name = play_file(file_data.to_vec());
        assert_eq!(song_name, "Nyan Cat");
    }
}
