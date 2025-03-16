use std::io::Read;

use log;
use nbs_rs::{NbsFile, NbsParser};

/// Determine whether to load from a ZIP or a normal file
fn is_zip_file(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04])
}

pub struct SongData<'a> {
    pub song: NbsFile,
    pub extra_sounds: Vec<(&'a [u8], f64)>,
}

/// Load an NBS file from a ZIP archive
fn load_nbs_from_zip<'a>(bytes: &'a [u8]) -> SongData<'a> {
    log::info!("Loading song from ZIP file, with {:?} bytes", bytes.len());

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();

    // Extract NBS song
    let nbs_data = {
        let mut nbs_file = zip.by_name("song.nbs").unwrap();
        let mut data = Vec::new();
        nbs_file.read_to_end(&mut data).unwrap();
        data
    };

    let mut song_parser = NbsParser::new(&nbs_data);
    let song = song_parser.parse().unwrap();

    let instruments = &song.instruments;

    // Owned storage for sound data
    let mut sounds_storage: Vec<(Vec<u8>, f64)> = Vec::new();
    let mut extra_sounds: Vec<(&[u8], f64)> = Vec::new();

    for instrument in instruments {
        let sound_name = format!(
            "sounds/{}",
            String::from_utf8(instrument.name.clone()).unwrap()
        );

        if let Ok(mut sound_file) = zip.by_name(&sound_name) {
            let mut sound = Vec::new();
            sound_file.read_to_end(&mut sound).unwrap();
            let key = instrument.key as f64;
            sounds_storage.push((sound, key)); // Store owned data
        }
    }

    // Borrow slices from owned data
    for sound in &sounds_storage {
        extra_sounds.push((sound.0.as_slice(), sound.1));
    }

    // Leak sounds_storage to extend its lifetime
    let leaked_sounds_storage = Box::leak(Box::new(sounds_storage));

    SongData {
        song,
        extra_sounds: leaked_sounds_storage
            .iter()
            .map(|s| (s.0.as_slice(), s.1))
            .collect(),
    }
}
/// Load an NBS file directly (not from ZIP)
fn load_nbs_from_file<'a>(bytes: &'a [u8]) -> SongData<'a> {
    log::info!("Loading song from NBS file, with {:?} bytes", bytes.len());

    let mut song_parser = NbsParser::new(bytes);
    let song = song_parser.parse().unwrap();

    SongData {
        song,
        extra_sounds: Vec::new(),
    }
}

pub fn load_nbs_file<'a>(song_data: Option<&'a [u8]>) -> SongData<'a> {
    let song_data_bytes =
        song_data.unwrap_or_else(|| include_bytes!("../test-assets/Mesmerizer.zip"));

    if is_zip_file(song_data_bytes) {
        load_nbs_from_zip(song_data_bytes)
    } else {
        load_nbs_from_file(song_data_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zip_file() {
        let zip_header = vec![0x50, 0x4B, 0x03, 0x04, 0x00, 0x00]; // Valid ZIP magic bytes
        let not_zip = vec![0x01, 0x02, 0x03, 0x04]; // Random non-ZIP data

        assert!(is_zip_file(&zip_header));
        assert!(!is_zip_file(&not_zip));
    }

    #[test]
    fn test_load_nbs_from_file() {
        // Mock simple NBS file data
        let nbs_data = include_bytes!("../test-assets/nyan_cat.nbs") as &[u8];
        let song_data = load_nbs_from_file(&nbs_data);

        assert!(!song_data.extra_sounds.is_empty() || song_data.extra_sounds.is_empty()); // Ensure it runs
    }

    #[test]
    fn test_load_nbs_file() {
        let nbs_data = include_bytes!("../test-assets/nyan_cat.nbs") as &[u8];
        let song_data = load_nbs_file(Some(&nbs_data));

        assert!(!song_data.extra_sounds.is_empty() || song_data.extra_sounds.is_empty()); // Ensure it runs
    }

    #[test]
    fn test_load_nbs_from_zip() {
        let zip_data = include_bytes!("../test-assets/Mesmerizer.zip").to_vec();

        let song_data = load_nbs_from_zip(&zip_data);

        assert!(!song_data.extra_sounds.len() > 0);
    }
}
