mod models;
use models::WavFormat::*;

fn main() {
    let file_path = "file_example.wav";
    let wav = WavFile::from_wav_file(file_path).unwrap();
    println!("{}", wav.head);
    println!("{}", wav.fmt);
    println!("{}", wav.data);
    }