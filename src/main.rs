mod models;

use std::fs::File;
use std::io;
use std::io::BufReader;
use crossterm::event::KeyEvent;
use ratatui::{DefaultTerminal, Frame};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Layout, Constraint};
use ratatui::prelude::Stylize;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Gauge, Widget};
use ratatui::widgets::canvas::Canvas;
use models::wav_file::*;
use crate::models::wav_source::WavSource;

fn main() -> io::Result<()> {
    // let file_path = "noise_example.wav";
    // let mut wav = WavFile::from_wav_file(file_path).unwrap();
    // wav.denoise_data_fft(0.001);
    //
    // wav.save_to_file("new_file.wav");

    let mut terminal = ratatui::init();

    let mut app = App {
        exit: false,
    };

    let app_result = app.run(&mut terminal);

    ratatui::restore();
    app_result
  }

pub struct App {
    exit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.is_press() && key_event.code == crossterm::event::KeyCode::Char('q') {
            self.exit = true;
        } else if key_event.is_press() && key_event.code == crossterm::event::KeyCode::Char('p') {
            self.play_file()?;
        }
        Ok(())
    }

    fn play_file(&mut self) -> io::Result<()> {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        // let file = BufReader::new(File::open("C:\\Users\\Work\\Desktop\\Rust\\rust-project\\src\\noise_example.wav").unwrap());
        // let source = rodio::Decoder::new(file).unwrap();
        let file_path = "C:\\Users\\Work\\Desktop\\Rust\\rust-project\\src\\noise_example.wav";
        let mut wav = WavFile::from_wav_file(file_path).unwrap();
        let source = WavSource::from_wav_file(wav);
        sink.append(source);
        sink.sleep_until_end(); // sink plays in a separate thread, no idea what happens when App.run() is implemented using threads
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_layout = Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(40), Constraint::Percentage(40)]);
        let [file_selection_area, raw_wave_area, denoised_wave_area] = vertical_layout.areas(area);

        Line::from("press q to quit, press p to play").bold().centered().render(file_selection_area, buf);

        let instructions = Line::from(vec![" Quit ".into(),"<Q>".blue().bold(), " Play ".into(), "<P> ".blue().bold()]).centered();

        let block = Block::bordered().title(" Controls ").title_bottom(instructions).borders(Borders::ALL).border_set(border::THICK);

        let sound_wave = Gauge::default().block(block);
        // let sound_wave = Canvas::default().block(block);

        sound_wave.render(raw_wave_area, buf);
    }


}