use rodio::Source;
use std::time::Duration;
use crate::models::wav_file::WavFile;
use crate::models::audio_samples::AudioSamples;

pub struct WavSource {
    samples: std::vec::IntoIter<i16>,
    sample_rate: u32,
    channels: u16,
}

impl Iterator for WavSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples.next()
    }
}

impl Source for WavSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl WavSource {
    pub fn from_wav_file(wav: WavFile) -> Self {
        Self {
            samples: Self::from_audio_samples(wav.data.data).into_iter(),
            sample_rate: wav.fmt.sample_rate,
            channels: wav.fmt.num_channels,
        }
    }

    fn from_audio_samples(samples: AudioSamples) -> Vec<i16> {
        fn clamp_i32_to_i16(v: i32) -> i16 {
            v.max(i16::MIN as i32).min(i16::MAX as i32) as i16
        }

        fn convert_i8_to_i16(v: i8) -> i16 {
            (v as i16) << 8
        }

        match samples {
            AudioSamples::MonoI8(v) => {
                let data = v.into_iter().map(convert_i8_to_i16).collect();
                data
            }
            AudioSamples::StereoI8(v) => {
                let data = v
                    .into_iter()
                    .flat_map(|[l, r]| [convert_i8_to_i16(l), convert_i8_to_i16(r)])
                    .collect();
                data
            }
            AudioSamples::MonoI16(v) => v,
            AudioSamples::StereoI16(v) => {
                let data = v.into_iter().flat_map(|[l, r]| [l, r]).collect();
                data
            }
            AudioSamples::MonoI32(v) => {
                let data = v.into_iter().map(clamp_i32_to_i16).collect();
                data
            }
            AudioSamples::StereoI32(v) => {
                let data = v
                    .into_iter()
                    .flat_map(|[l, r]| [clamp_i32_to_i16(l), clamp_i32_to_i16(r)])
                    .collect();
                data
            }
        }
    }
}