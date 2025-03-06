use nbs_rs::NbsParser;
use raylib::prelude::*;
use std::str;

pub fn play_file(nbs_file_data: Vec<u8>) -> String {
    let mut parser = NbsParser::new(nbs_file_data.as_slice());
    let nbs_file = parser.parse().unwrap();
    let song_name_bytes = nbs_file.header.song_name;
    str::from_utf8(&song_name_bytes).unwrap().to_string()
}
fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    let file_data = include_bytes!("../test-assets/nyan_cat.nbs");
    let mut parser = NbsParser::new(file_data.as_slice());
    let nbs_file = parser.parse().unwrap();
    let song_name_bytes = nbs_file.header.song_name;
    let name = str::from_utf8(&song_name_bytes).unwrap().to_string();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text(name.as_str(), 12, 12, 20, Color::BLACK);
    }
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
