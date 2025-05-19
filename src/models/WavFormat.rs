use std::fs;
use std::path::Path;
use thiserror::Error;
use std::fmt::Display;

// The Scriptures:
// http://soundfile.sapp.org/doc/WaveFormat/

#[derive(Debug, Clone)]
pub enum AudioSamples {
  MonoI8(Vec<i8>),
  StereoI8(Vec<[i8;2]>),
  MonoI16(Vec<i16>),
  StereoI16(Vec<[i16;2]>),
  MonoI32(Vec<i32>),
  StereoI32(Vec<[i32;2]>),
}

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

// Display implementations done using chat

#[derive(Debug, Clone)]
pub struct WavHead {
  pub chunk_id: [u8; 4],
  pub chunk_size: u32,
  pub format: [u8; 4],
}

impl Display for WavHead {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "WavHead {{ chunk_id: {:?}, chunk_size: {}, format: {:?} }}",
      std::str::from_utf8(&self.chunk_id).unwrap_or("????"),
      self.chunk_size,
      std::str::from_utf8(&self.format).unwrap_or("????")
    )
  }
}

#[derive(Debug, Clone)]
pub struct WavFmt {
  pub subchunk_id: [u8; 4],
  pub subchunk_size: u32,
  pub audio_format: AudioFormat,
  pub num_channels: u16,
  pub sample_rate: u32,
  pub byte_rate: u32,
  pub block_align: u16,
  pub bits_per_sample: u16,
}

impl Display for WavFmt {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "WavFmt {{ subchunk_id: {:?}, subchunk_size: {}, audio_format: {:?}, num_channels: {}, sample_rate: {}, byte_rate: {}, block_align: {}, bits_per_sample: {} }}",
      std::str::from_utf8(&self.subchunk_id).unwrap_or("????"),
      self.subchunk_size,
      self.audio_format,
      self.num_channels,
      self.sample_rate,
      self.byte_rate,
      self.block_align,
      self.bits_per_sample
    )
  }
}

#[derive(Debug, Clone)]
pub struct WavData {
  pub subchunk_id: [u8; 4],
  pub subchunk_size: u32,
  pub data: AudioSamples,
}

impl Display for WavData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "WavData {{ subchunk_id: {:?}, subchunk_size: {}, data: ... }}",
      std::str::from_utf8(&self.subchunk_id).unwrap_or("????"),
      self.subchunk_size
    )
  }
}

#[derive(Debug, Clone)]
enum AudioFormat {
  Pcm,
  Other(u16),
}

#[derive(Debug)]
pub struct WavFile {
  pub head: WavHead,
  pub fmt: WavFmt,
  pub data: WavData,
}

impl WavFile {

  // DETACHED METHODS

  pub fn new_head(chunk_size: u32) -> WavHead {
    WavHead {
      chunk_id: *b"RIFF",
      chunk_size,
      format: *b"WAVE",
    }
  }

  pub fn new_fmt(num_channels: u16, sample_rate: u32, bits_per_sample: u16) -> WavFmt {
    let audio_format = AudioFormat::Pcm;
    let subchunk_id = *b"fmt ";
    let subchunk_size = 16; // PCM
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    WavFmt {
      subchunk_id,
      subchunk_size,
      audio_format,
      num_channels,
      sample_rate,
      byte_rate,
      block_align,
      bits_per_sample,
    }
  }

  pub fn new_data(subchunk_size: u32, data: AudioSamples) -> WavData {
    WavData {
      subchunk_id: *b"data",
      subchunk_size,
      data,
    }
  }

  // Lifetime parameter
  // Telling rust copmiler that "data" and returned slice will live at least as long as 'a
  fn find_chunk<'a>(data: &'a [u8], chunk_id: &'a [u8; 4]) -> Option<&'a [u8]> {
      let mut offset = 12;

      // Get the next chunk's id and size
      // The first 4 bytes - chunk's id
      // The bytes from 5 to 8 - chunk's size
      // The bytes are also encoded in little-endian, so the from_le_bytes is needed
      while offset + 8 < data.len() {
          let id = &data[offset..offset + 4];
          let chunk_size =
              u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()) as usize;

          if id == chunk_id {
              let end = offset + 8 + chunk_size;
              if end <= data.len() {
                  return Some(&data[offset..end]);
              }
              return None;
          }
          offset += 8 + chunk_size;
      }
      None
  }

  pub fn get_head(data: &Vec<u8>) -> Result<WavHead, WavError> {
    let riff = &data[..4];
    if riff != b"RIFF" {
        return Err(WavError::InvalidRiffHeader(riff.to_vec()));
    }

    let wave = &data[8..12];
    if wave != b"WAVE" {
        return Err(WavError::InvalidWaveFormat(wave.to_vec()));
    }

    let wav_head = WavFile::new_head(data.len() as u32 - 8);

    Ok(wav_head)
  }

  pub fn get_fmt(data: &Vec<u8>) -> Result<WavFmt, WavError> {
    let fmt_subchunk = WavFile::find_chunk(data, b"fmt ").ok_or(WavError::UnexpectedLength)?;

    if fmt_subchunk.len() < 24 {
      return Err(WavError::UnexpectedLength);
    }

    let wav_fmt = WavFile::new_fmt(
      u16::from_le_bytes([fmt_subchunk[10], fmt_subchunk[11]]),
      u32::from_le_bytes([fmt_subchunk[12], fmt_subchunk[13], fmt_subchunk[14], fmt_subchunk[15]]),
      u16::from_le_bytes([fmt_subchunk[22], fmt_subchunk[23]])
    );

    Ok(wav_fmt)
  }

  pub fn get_data(data: &Vec<u8>, fmt: &WavFmt) -> Result<WavData, WavError> {
    let data_subchunk = WavFile::find_chunk(data, b"data").ok_or(WavError::UnexpectedLength)?;
    let subchunk_size = data_subchunk.len() as u32 - 8;
    let audio_data = &data_subchunk[8..];
    let data: AudioSamples = match (fmt.num_channels, fmt.bits_per_sample) {
      (1, 8) => {
        let samples = audio_data.iter().map(|&b| b as i8).collect();
        AudioSamples::MonoI8(samples)
      }
      (2, 8) => {
        let samples = audio_data
          .chunks_exact(2)
          .map(|c| [
            i8::from_le_bytes([c[0]]) , 
            i8::from_le_bytes([c[1]])
          ])
          .collect();
        AudioSamples::StereoI8(samples)
      }
      (1, 16) => {
        let samples = audio_data
          .chunks_exact(2)
          .map(|c| i16::from_le_bytes([c[0], c[1]]))
          .collect();
        AudioSamples::MonoI16(samples)
      }
      (2, 16) => {
        let samples = audio_data
          .chunks_exact(4)
          .map(|c| [
            i16::from_le_bytes([c[0], c[1]]),
            i16::from_le_bytes([c[2], c[3]]),
          ])
          .collect();
        AudioSamples::StereoI16(samples)
      }
      (1, 32) => {
        let samples = audio_data
          .chunks_exact(4)
          .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
          .collect();
        AudioSamples::MonoI32(samples)
      }
      (2, 32) => {
        let samples = audio_data
          .chunks_exact(8)
          .map(|c| [
            i32::from_le_bytes([c[0], c[1], c[2], c[3]]),
            i32::from_le_bytes([c[4], c[5], c[6], c[7]]),
          ])
          .collect();
        AudioSamples::StereoI32(samples)
      }
      _ => return Err(WavError::InvalidWAudioFormat),
    };

    let wav_data = WavData {
      subchunk_id: *b"data",
      subchunk_size,
      data,
    };

    Ok(wav_data)
  }

  // WAVFILE CREATION

  pub fn from_wav_file(file_path: &str) -> Result<WavFile, WavError> {
    let path = Path::new(file_path);
    let data: Vec<u8> = fs::read(path).map_err(WavError::IoError)?;

    let header_chunk = WavFile::get_head(&data)?;
    let fmt_subchunk = WavFile::get_fmt(&data)?;
    let data_subchunk = WavFile::get_data(&data, &fmt_subchunk)?;

    Ok(WavFile { head: header_chunk, fmt: fmt_subchunk, data: data_subchunk })
  }

  // INSTANCE METHODS


}