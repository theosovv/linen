use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VmConfig {
    pub sample_rate: u32,
    pub block_size: usize,
}

impl VmConfig {
    pub fn validate(self) -> Result<(), VmConfigError> {
        if !(8_000..=384_000).contains(&self.sample_rate) {
            return Err(VmConfigError::InvalidSampleRate(self.sample_rate));
        }
        if self.block_size == 0 {
            return Err(VmConfigError::ZeroBlockSize);
        }
        Ok(())
    }
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            block_size: 128,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VmConfigError {
    InvalidSampleRate(u32),
    ZeroBlockSize,
}

impl fmt::Display for VmConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidSampleRate(rate) => write!(
                formatter,
                "invalid VM sample rate {rate}; expected 8000..=384000"
            ),
            Self::ZeroBlockSize => formatter.write_str("VM block size must be greater than zero"),
        }
    }
}

impl Error for VmConfigError {}
