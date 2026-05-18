//! Audio I/O: WAV export and JACK preview.

use std::path::Path;

/// Export a mono f32 buffer to a WAV file.
pub fn export_wav_mono(
    path: &Path,
    samples: &[f32],
    sample_rate: u32,
) -> Result<(), std::io::Error> {
    // TODO: implement WAV export using symphonia
    let _ = (path, samples, sample_rate);
    Ok(())
}

/// JACK client for REPL preview.
pub struct JackPreview {
    // TODO: implement JACK client
}

impl JackPreview {
    pub fn new(_client_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
    }

    pub fn play_buffer(&mut self, _buffer: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
