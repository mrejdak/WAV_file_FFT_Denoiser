use std::{io, thread};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use ratatui::{DefaultTerminal, Frame};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Gauge, List};
use rodio::Source;
use crate::models::wav_file::WavFile;
use crate::models::wav_source::WavSource;


pub(crate) enum Event {
    Input(crossterm::event::KeyEvent),
    // FileSelected(WavFile),
    SoundProgress(f64),
    SinksReady(rodio::Sink, rodio::Sink, Instant, Duration),
}

pub struct App {
    exit: bool,
    progress_bar_color: ratatui::style::Color,
    sound_progress: f64,
    tx: Sender<Event>,
    sink_original: Option<rodio::Sink>,
    sink_denoised: Option<rodio::Sink>,
    start_time: Option<Instant>,
    duration: Option<Duration>,
}

// pub(crate) fn run_background_thread(tx: mpsc::Sender<Event>) {
//     let mut progress = 0.0;
//     loop {
//         thread::sleep(Duration::from_secs(1));
//
//         // set progress to current time / total_duration
//         tx.send(Event::SoundProgress(progress)).unwrap();
//     }
// }

fn play_file( playback_tx: Sender<Event>) -> io::Result<()> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink1 = rodio::Sink::try_new(&stream_handle).unwrap();
    let sink2 = rodio::Sink::try_new(&stream_handle).unwrap();

    let file_path = "C:\\Users\\Work\\Desktop\\Rust\\rust-project\\src\\noise_example.wav";
    let mut wav = WavFile::from_wav_file(file_path).unwrap();


    let mut denoised_wav = wav.clone();
    playback_tx.send(Event::SoundProgress(0.0)).unwrap();
    denoised_wav.denoise_data_fft(0.1).expect("denoise panic");
    denoised_wav.save_to_file("C:\\Users\\Work\\Desktop\\Rust\\rust-project\\src\\new_file.wav").expect("save panic");
    let source = WavSource::from_wav_file(&wav);
    let denoised_source = WavSource::from_wav_file(&denoised_wav);

    let total_duration = source.total_duration().unwrap();

    sink1.append(source);
    sink2.append(denoised_source);

    sink1.set_volume(1.0);
    sink2.set_volume(0.0);

    playback_tx.send(Event::SinksReady(sink1, sink2, Instant::now(), total_duration)).unwrap();

    thread::sleep(Duration::from_secs(total_duration.as_secs()));

    Ok(())
}

fn load_progress_bar(progress_tx: Sender<Event>, start_time: Instant, total_duration: Duration) -> io::Result<()> {
    let mut progress = 0.0;
    while progress < 1.0 {
        let elapsed = start_time.elapsed().as_secs_f64();
        progress = (elapsed / total_duration.as_secs_f64()).min(1.0);
        progress_tx.send(Event::SoundProgress(progress)).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

pub(crate) fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

impl App {
    pub fn new(tx: Sender<Event>) -> App {
        Self { exit: false,
            progress_bar_color: ratatui::style::Color::Green,
            sound_progress: 0.0,
            tx,
            sink_original: None,
            sink_denoised: None,
            start_time: None,
            duration: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::SoundProgress(progress) => self.sound_progress = progress,
                Event::SinksReady(sink_orig, sink_denoised, start_time, duration) => {
                    self.sink_original = Some(sink_orig);
                    self.sink_denoised = Some(sink_denoised);
                    self.start_time = Some(start_time);
                    self.duration = Some(duration);
                    self.handle_sound(start_time, duration);
                }
            }
        }
        Ok(())
    }


    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_sound(&mut self, start_time: Instant, duration: Duration) {
        let progress_tx = self.tx.clone();
        thread::spawn(move || {
            load_progress_bar(progress_tx, start_time, duration).expect("tu tez sie nie wywal");
        });
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.is_press() && key_event.code == crossterm::event::KeyCode::Char('q') {
            self.exit = true;
        } else if key_event.is_press() && key_event.code == crossterm::event::KeyCode::Char('p') {
            let playback_tx = self.tx.clone();  // need to play file in a thread
            thread::spawn(move || {
                play_file(playback_tx).expect("pls nie wywal sie");
            });
            // self.play_file(playback_tx)?;
        } else if key_event.is_press() && key_event.code == crossterm::event::KeyCode::Char('c') {
            if let (Some(orig), Some(denoised)) = (&self.sink_original, &self.sink_denoised) {
                if orig.volume() > 0.0 {
                    orig.set_volume(0.0);
                    denoised.set_volume(1.0);
                    self.progress_bar_color = ratatui::style::Color::Red;
                } else {
                    orig.set_volume(1.0);
                    denoised.set_volume(0.0);
                    self.progress_bar_color = ratatui::style::Color::Green;
                }
            }
        }
        Ok(())
    }

}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_layout = Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(40), Constraint::Percentage(40)]);
        let horizontal_layout = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let [top_area, raw_wave_area, denoised_wave_area] = vertical_layout.areas(area);
        let [file_selection_area, progress_bar_area] = horizontal_layout.areas(top_area);

        let controls = Line::from(vec![" Change File ".into(), "<Up/Down>".blue().bold(),
            " Select ".into(), "<Enter>".blue().bold(), " Quit ".into(), "<Q>".blue().bold()]).bold().centered();
        let controls_bloc = Block::bordered().title(" Select File ").title_bottom(controls).borders(Borders::ALL).border_set(border::THICK);
        // let file_selector = List::new(items).block(controls_bloc).

        let instructions = Line::from(vec![" Play/Pause ".into(), "<P> ".blue().bold()]).centered();

        let sound_controls_block = Block::bordered().title(" Sound Track ").title_bottom(instructions)
            .borders(Borders::ALL).border_set(border::THICK);

        let block = Block::bordered().title(" Raw Sound Wave ").borders(Borders::ALL);

        let progress_bar = Gauge::default().gauge_style(Style::default().fg(self.progress_bar_color))
            .block(sound_controls_block).label("TODO: progress in min:sec").ratio(self.sound_progress);

        let sound_wave = Gauge::default().block(block); // temporary sound_wave object
        // let sound_wave = Canvas::default().block(block);

        // file_selector.render(file_selection_area, buf);
        progress_bar.render(progress_bar_area, buf);
        sound_wave.render(raw_wave_area, buf);
    }
}


impl App {
    fn render_selector(&self, area: Rect, buf: &mut Buffer) {

    }
}