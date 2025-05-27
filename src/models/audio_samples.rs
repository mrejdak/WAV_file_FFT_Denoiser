use std::fmt::Display;
use crate::models::errors::WavError;

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
    pub fn from_le_bytes(
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
            // Unsupported sample size
            _ => return Err(WavError::InvalidWAudioFormat),
        };
        Ok(data_field)
    }

    pub fn to_le_bytes_vector(&self) -> Vec<u8> {
        match self {
            // 8 bit per sample
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

    pub fn to_f64_mono(&self) -> Result<Vec<f64>, WavError> {
        let data = match self {
            AudioSamples::MonoI8(v) => v.iter().map(|&b| b as f64).collect(),
            AudioSamples::MonoI16(v) => v.iter().map(|&b| b as f64).collect(),
            AudioSamples::MonoI32(v) => v.iter().map(|&b| b as f64).collect(),
            _ => return Err(WavError::InvalidWAudioFormat),
        };
        Ok(data)
    }

    pub fn to_f64_stereo(&self) -> Result<(Vec<f64>, Vec<f64>), WavError> {
        let data: (Vec<f64>, Vec<f64>) = match self {
            AudioSamples::StereoI8(v) => (
                v.iter().map(|pair| pair[0] as f64).collect(),
                v.iter().map(|pair| pair[1] as f64).collect(),
            ),
            AudioSamples::StereoI16(v) => (
                v.iter().map(|pair| pair[0] as f64).collect(),
                v.iter().map(|pair| pair[1] as f64).collect(),
            ),
            AudioSamples::StereoI32(v) => (
                v.iter().map(|pair| pair[0] as f64).collect(),
                v.iter().map(|pair| pair[1] as f64).collect(),
            ),
            _ => return Err(WavError::InvalidWAudioFormat),
        };
        Ok(data)
    }

    pub fn from_f64_mono(channel: &[f64], bits_per_sample: u16) -> Result<AudioSamples, WavError> {
        let data = match bits_per_sample {
            8 => {
                let samples = channel.iter().map(|&b| b.round() as i8).collect();
                AudioSamples::MonoI8(samples)
            }
            16 => {
                let samples = channel.iter().map(|&b| b.round() as i16).collect();
                AudioSamples::MonoI16(samples)
            }
            32 => {
                let samples = channel.iter().map(|&b| b.round() as i32).collect();
                AudioSamples::MonoI32(samples)
            }
            _ => return Err(WavError::InvalidWAudioFormat),
        };
        Ok(data)
    }

    pub fn from_f64_stereo(
        left_channel: &[f64],
        right_channel: &[f64],
        bits_per_sample: u16,
    ) -> Result<AudioSamples, WavError> {
        let n = left_channel.len();
        let data = match bits_per_sample {
            8 => {
                let mut samples = vec![[0_i8; 2]; n];
                for i in 0..n {
                    samples[i][0] = left_channel[i].round() as i8;
                    samples[i][1] = right_channel[i].round() as i8;
                }
                AudioSamples::StereoI8(samples)
            }
            16 => {
                let mut samples = vec![[0_i16; 2]; n];
                for i in 0..n {
                    samples[i][0] = left_channel[i].round() as i16;
                    samples[i][1] = right_channel[i].round() as i16;
                }
                AudioSamples::StereoI16(samples)
            }
            32 => {
                let mut samples = vec![[0_i32; 2]; n];
                for i in 0..n {
                    samples[i][0] = left_channel[i].round() as i32;
                    samples[i][1] = right_channel[i].round() as i32;
                }
                AudioSamples::StereoI32(samples)
            }
            _ => return Err(WavError::InvalidWAudioFormat),
        };
        Ok(data)
    }
}

impl Display for AudioSamples {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioSamples::MonoI8(v) => write!(f, "MonoI8(len: {:?})", v),
            AudioSamples::StereoI8(v) => write!(f, "StereoI8(len: {:?})", v),
            AudioSamples::MonoI16(v) => write!(f, "MonoI16(len: {:?})", v),
            AudioSamples::StereoI16(v) => write!(f, "StereoI16(len: {:?})", v),
            AudioSamples::MonoI32(v) => write!(f, "MonoI32(len: {:?})", v),
            AudioSamples::StereoI32(v) => write!(f, "StereoI32(len: {:?})", v),
        }
    }
}
