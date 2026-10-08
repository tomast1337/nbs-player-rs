use std::io::Read;

use nbs_rs::{NbsFile, NbsParser};

/// A custom instrument sound shipped inside a song archive.
#[derive(Debug, Clone)]
pub struct ExtraSound {
    /// Encoded `.ogg` bytes.
    pub ogg: Vec<u8>,
    /// Base key (NBS `instrument.key`) the sample is tuned to.
    pub key: f64,
}

pub struct SongData {
    pub song: NbsFile,
    pub extra_sounds: Vec<ExtraSound>,
}

#[derive(Debug)]
pub enum SongError {
    Zip(String),
    MissingSong,
    Parse(String),
}

impl std::fmt::Display for SongError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SongError::Zip(e) => write!(f, "invalid zip archive: {e}"),
            SongError::MissingSong => write!(f, "song.nbs not found in zip archive"),
            SongError::Parse(e) => write!(f, "failed to parse NBS file: {e}"),
        }
    }
}

impl std::error::Error for SongError {}

/// Determine whether to load from a ZIP or a normal file
#[inline]
fn is_zip_file(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04])
}

fn parse_nbs(bytes: &[u8]) -> Result<NbsFile, SongError> {
    // nbs-rs unwraps on truncated input; turn that into an error where unwinding is available.
    std::panic::catch_unwind(|| NbsParser::new(bytes).parse())
        .map_err(|_| SongError::Parse("parser panicked on malformed input".into()))?
        .map_err(|e| SongError::Parse(e.to_string()))
}

/// Load an NBS file from a ZIP archive
fn load_nbs_from_zip(bytes: &[u8]) -> Result<SongData, SongError> {
    log::info!("Loading song from ZIP file, with {:?} bytes", bytes.len());

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| SongError::Zip(e.to_string()))?;

    let nbs_data = {
        let mut nbs_file = zip.by_name("song.nbs").map_err(|_| SongError::MissingSong)?;
        let mut data = Vec::new();
        nbs_file
            .read_to_end(&mut data)
            .map_err(|e| SongError::Zip(e.to_string()))?;
        data
    };

    let song = parse_nbs(&nbs_data)?;
    log::info!("Successfully parsed NBS file");

    let mut extra_sounds = Vec::new();
    for instrument in &song.instruments {
        let file_name = String::from_utf8_lossy(&instrument.file).into_owned();
        if file_name.is_empty() {
            log::warn!("Empty file name for instrument {:?}!", instrument);
            continue;
        }
        let file_path = format!("sounds/{}", file_name);
        if let Ok(mut sound_file) = zip.by_name(&file_path) {
            let mut ogg = Vec::new();
            if let Err(e) = sound_file.read_to_end(&mut ogg) {
                log::warn!("Failed to read {}: {}", file_path, e);
                continue;
            }
            extra_sounds.push(ExtraSound {
                ogg,
                key: instrument.key as f64,
            });
        }
    }

    Ok(SongData { song, extra_sounds })
}

/// Load an NBS file directly (not from ZIP)
fn load_nbs_from_file(bytes: &[u8]) -> Result<SongData, SongError> {
    log::info!("Loading song from NBS file, with {:?} bytes", bytes.len());
    Ok(SongData {
        song: parse_nbs(bytes)?,
        extra_sounds: Vec::new(),
    })
}

/// Load a song from raw bytes (plain `.nbs`, or a zip with `song.nbs` + `sounds/`).
pub fn load_song_bytes(bytes: &[u8]) -> Result<SongData, SongError> {
    if is_zip_file(bytes) {
        load_nbs_from_zip(bytes)
    } else {
        load_nbs_from_file(bytes)
    }
}

/// Load `song_data`, or the bundled demo song when `None`.
pub fn load_nbs_file(song_data: Option<&[u8]>) -> Result<SongData, SongError> {
    let bytes = song_data
        .unwrap_or_else(|| include_bytes!("../../../test-assets/Bad Piggies Theme.nbs"));
    load_song_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zip_file() {
        assert!(is_zip_file(&[0x50, 0x4B, 0x03, 0x04, 0x00, 0x00]));
        assert!(!is_zip_file(&[0x01, 0x02, 0x03, 0x04]));
    }

    #[test]
    fn load_plain_nbs() {
        let data = load_nbs_file(None).unwrap();
        assert!(data.extra_sounds.is_empty());
        assert!(data.song.header.song_length > 0);
    }

    #[test]
    fn load_zip_has_custom_sounds() {
        let zip = include_bytes!("../../../test-assets/Mesmerizer.zip");
        let data = load_song_bytes(zip).unwrap();
        assert!(!data.extra_sounds.is_empty());
    }

    #[test]
    fn garbage_is_an_error_not_a_panic() {
        assert!(load_song_bytes(&[1, 2, 3, 4, 5]).is_err());
    }
}
