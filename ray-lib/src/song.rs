use log;
use nbs_rs::{NbsFile, NbsParser};

pub fn load_nbs_file(song_data: Option<Vec<u8>>) -> NbsFile {
    let song_data_bytes = song_data
        .unwrap_or_else(|| include_bytes!("../test-assets/Metropolis of Illusion.nbs").to_vec());
    log::info!("Loading song with {:?} bytes", song_data_bytes.len());
    let mut song_file = NbsParser::new(&song_data_bytes);
    let song = song_file.parse().unwrap();
    log::info!("Loaded song successfully");
    song
}
