# Rust WAV Audio FFT Denoiser 🎵
## Computer Science | AGH 2025

## Project Description

This repository contains a Rust-based implementation for reading, processing, and denoising WAV audio files using Fast Fourier Transform (FFT) techniques. The project is designed as part of *Rust Programming* coursework, being a comprehensive introduction to Rust language.

The codebase includes modules for WAV file parsing, FFT-based denoising, and audio data manipulation, providing a foundation for further exploration in audio processing and Rust development.

## Table of Contents

| Module         | Description                                 | Source                                  |
|----------------|---------------------------------------------|-----------------------------------------|
| WAV Parser     | Read and write standard WAV files           | [src/models/wav_file.rs](src/models/wav_file.rs) |
| FFT Utilities  | Perform FFT and IFFT on audio samples       | [src/fft.rs](src/fft.rs)                |
| Denoising      | Remove noise using frequency thresholding   | [src/models/wav_file.rs](src/models/wav_file.rs) |
| AudioSamples   | Audio data abstraction and conversion       | [src/models/audio_samples.rs](src/models/audio_samples.rs) |
| TUI            | Terminal UI of the application              | [src/models/tui_app.rs](src/models/tui_app.rs) |

<br/>
<p align="center">
  <img src="https://github.com/user-attachments/assets/95827f3f-3f20-4e2c-ba86-7b4c9574833b" width="70%"/>
  <p align="center"><i>Terminal UI of the application</i></p>
</p>
<br/>

## Prerequisites

- Rust (edition 2021 or higher)
- Cargo (Rust package manager)

## Setup Instructions

You can either download the latest release binary (built for Windows), or build from source by following the steps below:

### 1. Clone the repository
```bash
git clone https://github.com/mrejdak/WAV_file_FFT_Denoiser
cd WAV_file_FFT_Denoiser
```

### 2. Build the project
```bash
cargo build --release
```
