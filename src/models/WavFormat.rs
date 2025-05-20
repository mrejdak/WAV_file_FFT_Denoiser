use std::fmt::{Display};
use std::path::Path;
use std::{fs};
use thiserror::Error;

// The Scriptures:
// http://soundfile.sapp.org/doc/WaveFormat/

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
pub enum AudioSamples {
    MonoI8(Vec<i8>),
    StereoI8(Vec<[i8; 2]>),
    MonoI16(Vec<i16>),
    StereoI16(Vec<[i16; 2]>),
    MonoI32(Vec<i32>),
    StereoI32(Vec<[i32; 2]>),
}

impl AudioSamples {
    fn from_le_bytes(
        audio_data: &[u8],
        num_channels: u16,
        bits_per_sample: u16,
    ) -> Result<AudioSamples, WavError> {
        let data_field: AudioSamples = match (num_channels, bits_per_sample) {
            // 8 bits per sample
            (1, 8) => {
                let samples = audio_data.iter().map(|&b| b as i8).collect();
                AudioSamples::MonoI8(samples)
            }

            (2, 8) => {
                let samples = audio_data
                    .chunks_exact(2)
                    .map(|c| [i8::from_le_bytes([c[0]]), i8::from_le_bytes([c[1]])])
                    .collect();
                AudioSamples::StereoI8(samples)
            }

            // 16 bits per sample
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
                    .map(|c| {
                        [
                            i16::from_le_bytes([c[0], c[1]]),
                            i16::from_le_bytes([c[2], c[3]]),
                        ]
                    })
                    .collect();
                AudioSamples::StereoI16(samples)
            }

            // 32 bits per sample
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
                    .map(|c| {
                        [
                            i32::from_le_bytes([c[0], c[1], c[2], c[3]]),
                            i32::from_le_bytes([c[4], c[5], c[6], c[7]]),
                        ]
                    })
                    .collect();
                AudioSamples::StereoI32(samples)
            }

            // Unsupportex sample size
            _ => return Err(WavError::InvalidWAudioFormat),
        };
        Ok(data_field)
    }

    fn to_le_bytes_vector(&self) -> Vec<u8> {
        match self {
            // 8 bit per sample
            // Casting an i8 to a u8 causes the underlying binary representation (8 bits) to remain unchanged,
            // but the type system now treats it as unsigned

            // Iterator hell below, enjoy the functional programming paradigm
            AudioSamples::MonoI8(v) => v.iter().map(|&b| b as u8).collect(),

            AudioSamples::StereoI8(v) => {
                v.iter().flat_map(|c| c.iter().map(|&b| b as u8)).collect()
            }

            // 16 bit per sample
            AudioSamples::MonoI16(v) => v.iter().flat_map(|&b| b.to_le_bytes()).collect(),

            AudioSamples::StereoI16(v) => v
                .iter()
                .flat_map(|c| c.iter().flat_map(|&b| b.to_le_bytes()))
                .collect(),

            // 32 bit per sample
            AudioSamples::MonoI32(v) => v.iter().flat_map(|&b| b.to_le_bytes()).collect(),

            AudioSamples::StereoI32(v) => v
                .iter()
                .flat_map(|c| c.iter().flat_map(|&b| b.to_le_bytes()))
                .collect(),
        }
    }
}

// Display implementations done using chat

#[derive(Debug, Clone)]
pub(crate) struct WavHead {
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
pub(crate) struct WavFmt {
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
pub(crate) struct WavData {
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
pub(crate) enum AudioFormat {
    Pcm,
    Other(u16),
}

impl AudioFormat {
    fn value(&self) -> u16 {
        match self {
            AudioFormat::Pcm => 1 as u16,
            AudioFormat::Other(x) => *x,
        }
    }
}

// Offset  Size  Name             Description
// 0         4   ChunkID          Contains the letters "RIFF" in ASCII form
//                                (0x52494646 big-endian form).
// 4         4   ChunkSize        36 + SubChunk2Size, or more precisely:
//                                4 + (8 + SubChunk1Size) + (8 + SubChunk2Size)
//                                This is the size of the rest of the chunk
//                                following this number.  This is the size of the
//                                entire file in bytes minus 8 bytes for the
//                                two fields not included in this count:
//                                ChunkID and ChunkSize.
// 8         4   Format           Contains the letters "WAVE"
//                                (0x57415645 big-endian form).

pub fn new_head(chunk_size: u32) -> WavHead {
    WavHead {
        chunk_id: *b"RIFF",
        chunk_size,
        format: *b"WAVE",
    }
}

// Offset  Size  Name             Description
// 12        4   Subchunk1ID      Contains the letters "fmt "
//                                (0x666d7420 big-endian form).
// 16        4   Subchunk1Size    16 for PCM.  This is the size of the
//                                rest of the Subchunk which follows this number.
// 20        2   AudioFormat      PCM = 1 (i.e. Linear quantization)
//                                Values other than 1 indicate some
//                                form of compression.
// 22        2   NumChannels      Mono = 1, Stereo = 2, etc.
// 24        4   SampleRate       8000, 44100, etc.
// 28        4   ByteRate         == SampleRate * NumChannels * BitsPerSample/8
// 32        2   BlockAlign       == NumChannels * BitsPerSample/8
//                                The number of bytes for one sample including
//                                all channels. I wonder what happens when
//                                this number isn't an integer?
// 34        2   BitsPerSample    8 bits = 8, 16 bits = 16, etc.

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

// Offset  Size  Name             Description
// 36        4   Subchunk2ID      Contains the letters "data"
//                                (0x64617461 big-endian form).
// 40        4   Subchunk2Size    == NumSamples * NumChannels * BitsPerSample/8
//                                This is the number of bytes in the data.
//                                You can also think of this as the size
//                                of the read of the subchunk following this
//                                number.
// 44        *   Data             The actual sound data.

pub fn new_data(subchunk_size: u32, data: AudioSamples) -> WavData {
    WavData {
        subchunk_id: *b"data",
        subchunk_size,
        data,
    }
}

#[derive(Debug)]
pub struct WavFile {
    pub head: WavHead,
    pub fmt: WavFmt,
    pub data: WavData,
}

impl WavFile {
    // STRUCT READING FROM FILE

    pub fn from_wav_file(file_path: &str) -> Result<WavFile, WavError> {
        // Helper functions

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

        fn get_head_chunk(data: &Vec<u8>) -> Result<WavHead, WavError> {
            let riff = &data[..4];
            if riff != b"RIFF" {
                return Err(WavError::InvalidRiffHeader(riff.to_vec()));
            }
            let wave = &data[8..12];
            if wave != b"WAVE" {
                return Err(WavError::InvalidWaveFormat(wave.to_vec()));
            }

            let wav_head = new_head(data.len() as u32 - 8);
            Ok(wav_head)
        }

        pub fn get_fmt_subchunk(data: &Vec<u8>) -> Result<WavFmt, WavError> {
            let fmt_subchunk = find_chunk(data, b"fmt ").ok_or(WavError::UnexpectedLength)?;
            if fmt_subchunk.len() < 24 {
                return Err(WavError::UnexpectedLength);
            }

            let wav_fmt = new_fmt(
                u16::from_le_bytes([fmt_subchunk[10], fmt_subchunk[11]]),
                u32::from_le_bytes([
                    fmt_subchunk[12],
                    fmt_subchunk[13],
                    fmt_subchunk[14],
                    fmt_subchunk[15],
                ]),
                u16::from_le_bytes([fmt_subchunk[22], fmt_subchunk[23]]),
            );

            Ok(wav_fmt)
        }

        fn get_data_subchunk(data: &Vec<u8>, fmt: &WavFmt) -> Result<WavData, WavError> {
            let data_subchunk = find_chunk(data, b"data").ok_or(WavError::UnexpectedLength)?;
            let subchunk_size = data_subchunk.len() as u32 - 8;
            let audio_data = &data_subchunk[8..];

            let data_field =
                AudioSamples::from_le_bytes(audio_data, fmt.num_channels, fmt.bits_per_sample)?;

            let wav_data = new_data(subchunk_size, data_field);

            Ok(wav_data)
        }

        let path = Path::new(file_path);
        let data: Vec<u8> = fs::read(path).map_err(WavError::IoError)?;

        let header_chunk = get_head_chunk(&data)?;
        let fmt_subchunk = get_fmt_subchunk(&data)?;
        let data_subchunk = get_data_subchunk(&data, &fmt_subchunk)?;

        Ok(WavFile {
            head: header_chunk,
            fmt: fmt_subchunk,
            data: data_subchunk,
        })
    }

    // STRUCT FROM SUBCHUNKS

    pub fn from_subchunks(head: WavHead, fmt: WavFmt, data: WavData) -> WavFile {
        WavFile { head, fmt, data }
    }

    // STRUCT WRITING TO FILE

    fn create_le_bytes_vector(&self) -> Vec<u8> {
        fn write_head_subchunk_to_vec(head: &WavHead, v: &mut Vec<u8>) {
            v.extend_from_slice(&head.chunk_id);
            v.extend_from_slice(&head.chunk_size.to_le_bytes());
            v.extend_from_slice(&head.format);
        }

        fn write_fmt_subchunk_to_vec(fmt: &WavFmt, v: &mut Vec<u8>) {
            v.extend_from_slice(&fmt.subchunk_id);
            v.extend_from_slice(&fmt.subchunk_size.to_le_bytes());
            v.extend_from_slice(&fmt.audio_format.value().to_le_bytes());
            v.extend_from_slice(&fmt.num_channels.to_le_bytes());
            v.extend_from_slice(&fmt.sample_rate.to_le_bytes());
            v.extend_from_slice(&fmt.byte_rate.to_le_bytes());
            v.extend_from_slice(&fmt.block_align.to_le_bytes());
            v.extend_from_slice(&fmt.bits_per_sample.to_le_bytes());
        }

        fn write_data_subchunk_to_vec(data: &WavData, v: &mut Vec<u8>) {
            v.extend_from_slice(&data.subchunk_id);
            v.extend_from_slice(&data.subchunk_size.to_le_bytes());
            v.extend(data.data.to_le_bytes_vector());
        }

        let mut v: Vec<u8> = Vec::new();

        write_head_subchunk_to_vec(&self.head, &mut v);
        write_fmt_subchunk_to_vec(&self.fmt, &mut v);
        write_data_subchunk_to_vec(&self.data, &mut v);

        v
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), WavError> {
        let v = self.create_le_bytes_vector();
        fs::write(file_path, &v).map_err(WavError::IoError)
    }
}
