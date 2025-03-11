use nbs_rs;

#[derive(Clone, Debug)]
pub struct NoteBlock {
    pub was_played: bool,
    pub tick: u16,
    pub layer: u16,
    pub instrument: u8,
    pub key: u8,
    pub velocity: u8,
    pub panning: i8,
    pub pitch: i16,
}

pub fn get_note_blocks(song: &nbs_rs::NbsFile) -> Vec<Vec<NoteBlock>> {
    // it groups the notes by tick
    let mut note_blocks: Vec<Vec<NoteBlock>> = vec![];
    for tick in 0..song.header.song_length {
        let mut note_block: Vec<NoteBlock> = vec![];
        let note = song.notes.iter().find(|note| note.tick == tick);
        if let Some(note) = note {
            note_block.push(NoteBlock {
                was_played: false,
                tick: note.tick,
                layer: note.layer,
                instrument: note.instrument,
                key: note.key,
                velocity: note.velocity,
                panning: note.panning,
                pitch: note.pitch,
            });
        }

        note_blocks.push(note_block);
    }
    note_blocks
}
