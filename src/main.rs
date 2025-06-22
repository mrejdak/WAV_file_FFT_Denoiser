mod models;

use std::{io, thread};
use std::sync::mpsc;
use std::time::Duration;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, Borders, Gauge, Widget};
use crate::models::tui_app::{Event, App, handle_input_events};

fn main() -> io::Result<()> {
    // let file_path = "noise_example.wav";
    // let mut wav = WavFile::from_wav_file(file_path).unwrap();
    // wav.denoise_data_fft(0.001).expect("Błont");
    //
    // wav.save_to_file("new_file.wav");

    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let app_tx = event_tx.clone();

    let mut app = App::new(app_tx);

    let input_tx = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(input_tx);
    });

    // let background_tx = event_tx.clone();
    // thread::spawn(move || {
    //     run_background_thread(background_tx);
    // });

    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result
  }



