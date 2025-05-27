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
