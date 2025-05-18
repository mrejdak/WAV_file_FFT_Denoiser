use std::fs;
use std::path::Path;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum WavError {
    #[error("Invalid WAV header - expected 'RIFF' but found {0:?}")]
    InvalidRiffHeader(Vec<u8>),
    #[error("Invalid WAV format - expected 'WAVE' but found {0:?}")]
    InvalidWaveFormat(Vec<u8>),
    #[error("Invalid audio format - Pcm is the only one handled")]
    InvalidWAudioFormat,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Unexpected length of file")]
    UnexpectedLength,
}

#[derive(Debug, Clone)]
pub struct WavFormat {
  pub audio_format: AudioFormat,
  pub num_channels: u16,
  pub sample_rate: u32,
  pub byte_rate: u32,
  pub block_align: u16,
  pub bits_per_sample: u16,
}
#[derive(Debug, Clone)]
pub enum AudioFormat {
  Pcm,
  Other(u16),
}

#[derive(Debug)]
pub struct WavFile {
  pub format: WavFormat,
  pub data: Vec<u8>,
}