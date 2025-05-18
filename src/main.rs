mod models;
use models::WavFormat::*;
use std::fs;
use std::path::Path;

// Tis' the Book of Specification
// https://dylanmeeus.github.io/posts/audio-from-scratch-pt2/

fn read_wav_file(file_path: &str) -> Result<WavFile, WavError> {
    let path = Path::new(file_path);
    let data: Vec<u8> = fs::read(path)?;
    check_header(&data)?;
    let fmt_chunk = find_chunk(&data, b"fmt ").ok_or(WavError::UnexpectedLength)?;
    let wav_format = parse_fmt_chunk(fmt_chunk)?;
    let data_chunk = find_chunk(&data, b"data").ok_or(WavError::UnexpectedLength)?;
    let audio_data = data_chunk[8..].to_vec();

    Ok(WavFile {
      format: wav_format,
      data: audio_data
    })
}

fn check_header(data: &Vec<u8>) -> Result<(), WavError> {
    let riff = &data[..4];
    if riff != b"RIFF" {
        return Err(WavError::InvalidRiffHeader(riff.to_vec()));
    }

    let wave = &data[8..12];
    if wave != b"WAVE" {
        return Err(WavError::InvalidWaveFormat(wave.to_vec()));
    }

    Ok(())
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

fn parse_fmt_chunk(data: &[u8]) -> Result<WavFormat, WavError> {
    // fmt cannot be lesser than 24 bytes
    if data.len() < 24 {
        return Err(WavError::UnexpectedLength);
    }

    let audio_format = match u16::from_le_bytes([data[8], data[9]]) {
        1 => AudioFormat::Pcm,
        _ => return Err(WavError::InvalidWAudioFormat),
    };

    Ok(WavFormat {
        audio_format: audio_format,
        num_channels: u16::from_le_bytes([data[10], data[11]]),
        sample_rate: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        byte_rate: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
        block_align: u16::from_le_bytes([data[20], data[21]]),
        bits_per_sample: u16::from_le_bytes([data[22], data[23]]),
    })
}

fn main() -> Result<(), WavError> {
    let file_path = "file_example.wav";
    let wav = read_wav_file(file_path)?;
    println!("{:?}", wav);
    Ok(())
}
