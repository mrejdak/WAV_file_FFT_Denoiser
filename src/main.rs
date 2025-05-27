mod models;
use models::wav_file::*;

fn main() {
    let file_path = "noise_example.wav";
    let mut wav = WavFile::from_wav_file(file_path).unwrap();
    wav.denoise_data_fft(0.001);
    
    wav.save_to_file("new_file.wav");
  }