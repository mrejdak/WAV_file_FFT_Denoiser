import numpy as np
from scipy.io import wavfile
import argparse

def add_noise(input_file, output_file, noise_level=0.05):
  # Read the WAV file
  sample_rate, audio_data = wavfile.read(input_file)
  
  # Convert audio to float between -1 and 1
  audio_data = audio_data.astype(float) / np.iinfo(audio_data.dtype).max
  
  # Generate noise
  noise = np.random.normal(0, noise_level, audio_data.shape)
  
  # Add noise to the audio
  noisy_audio = audio_data + noise
  
  # Clip the signal to prevent distortion
  noisy_audio = np.clip(noisy_audio, -1.0, 1.0)
  
  # Convert back to original data type
  noisy_audio = (noisy_audio * np.iinfo(np.int16).max).astype(np.int16)
  
  # Save the noisy audio
  wavfile.write(output_file, sample_rate, noisy_audio)

if __name__ == "__main__":
  # parser = argparse.ArgumentParser(description='Add noise to a WAV file')
  # parser.add_argument('input_file', help='Input WAV file path')
  # parser.add_argument('output_file', help='Output WAV file path')
  # parser.add_argument('--noise_level', type=float, default=0.05,
  #           help='Noise level (default: 0.05)')
  
  # args = parser.parse_args()
  add_noise('file_example.wav', 'noise_example.wav', 0.001)