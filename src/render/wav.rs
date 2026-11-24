use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum WavError {
    Io(io::Error),
    SizeOverflow,
    UnsupportedChannelCount(u16),
}

impl fmt::Display for WavError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "could not write WAV: {error}"),
            Self::SizeOverflow => formatter.write_str("WAV size exceeds RIFF limits"),
            Self::UnsupportedChannelCount(channels) => {
                write!(formatter, "unsupported WAV channel count {channels}")
            }
        }
    }
}

impl Error for WavError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::SizeOverflow | Self::UnsupportedChannelCount(_) => None,
        }
    }
}

impl From<io::Error> for WavError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn write_wav_header<W: io::Write>(
    writer: &mut W,
    sample_rate: u32,
    samples: u64,
) -> Result<(), WavError> {
    write_wav_header_with_channels(writer, sample_rate, samples, 1)
}

pub fn write_wav_header_with_channels<W: io::Write>(
    writer: &mut W,
    sample_rate: u32,
    frames: u64,
    channels: u16,
) -> Result<(), WavError> {
    if !matches!(channels, 1 | 2) {
        return Err(WavError::UnsupportedChannelCount(channels));
    }

    let bytes_per_frame = u64::from(channels) * 2;
    let data_size = frames
        .checked_mul(bytes_per_frame)
        .and_then(|size| u32::try_from(size).ok())
        .ok_or(WavError::SizeOverflow)?;
    let riff_size = 36_u32
        .checked_add(data_size)
        .ok_or(WavError::SizeOverflow)?;
    let byte_rate = sample_rate
        .checked_mul(u32::from(channels))
        .and_then(|rate| rate.checked_mul(2))
        .ok_or(WavError::SizeOverflow)?;
    let block_align = channels * 2;
    let mut header = [0_u8; 44];

    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&riff_size.to_le_bytes());
    header[8..12].copy_from_slice(b"WAVE");
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&16_u32.to_le_bytes());
    header[20..22].copy_from_slice(&1_u16.to_le_bytes());
    header[22..24].copy_from_slice(&channels.to_le_bytes());
    header[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    header[32..34].copy_from_slice(&block_align.to_le_bytes());
    header[34..36].copy_from_slice(&16_u16.to_le_bytes());
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&data_size.to_le_bytes());

    writer.write_all(&header)?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonFiniteSample;

impl fmt::Display for NonFiniteSample {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PCM16 input must be finite")
    }
}

impl Error for NonFiniteSample {}

pub fn quantize_pcm16(value: f32) -> Result<i16, NonFiniteSample> {
    if !value.is_finite() {
        return Err(NonFiniteSample);
    }

    let value = value.clamp(-1.0, 1.0);
    Ok((value * 32_767.0).round() as i16)
}
