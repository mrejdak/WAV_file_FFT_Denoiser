use crate::models::wav_file::WavFile;
use crate::models::wav_source::WavSource;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, StatefulWidget, Stylize, Widget};
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState};
use ratatui::{DefaultTerminal, Frame};
use rodio::Source;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{env, fs, io, thread};

pub(crate) enum Event {
    Input(crossterm::event::KeyEvent),
    // FileSelected(WavFile),
    SoundProgress(f64),
    SinksReady(rodio::Sink, rodio::Sink, Instant, Duration),
    ProgressLabel(String, bool),
}

pub struct App {
    files: Option<Vec<String>>,
    path: Option<PathBuf>,
    selected: usize,
    exit: bool,
    progress_bar_color: Color,
    sound_progress: f64,
    threshold: f64,
    tx: Sender<Event>,
    sink_original: Option<rodio::Sink>,
    sink_denoised: Option<rodio::Sink>,
    start_time: Option<Instant>,
    duration: Option<Duration>,
    ready_to_play: bool,
    label: String,
}

fn play_file(
    playback_tx: Sender<Event>,
    path: PathBuf,
    filename: &String,
    threshold: f64,
) -> io::Result<()> {
    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let sink1 = rodio::Sink::try_new(&stream_handle)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let sink2 = rodio::Sink::try_new(&stream_handle)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let full_path = path.join(filename);
    let file_path = full_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid file path"))?;

    let save_path = path
        .join("denoised")
        .join(filename)
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid save path"))?
        .to_string();

    let wav = WavFile::from_wav_file(file_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Error loading WAV: {:?}", e)))?;

    let mut denoised_wav = wav.clone();
    denoised_wav
        .denoise_data_fft(threshold)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Denoise failed: {:?}", e)))?;
    denoised_wav
        .save_to_file(&save_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Save failed: {:?}", e)))?;

    let source = WavSource::from_wav_file(&wav);
    let denoised_source = WavSource::from_wav_file(&denoised_wav);

    let total_duration = source
        .total_duration()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get total duration"))?;

    sink1.append(source);
    sink2.append(denoised_source);
    sink1.set_volume(1.0);
    sink2.set_volume(0.0);

    playback_tx
        .send(Event::SinksReady(
            sink1,
            sink2,
            Instant::now(),
            total_duration,
        ))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    thread::sleep(Duration::from_secs(total_duration.as_secs()));

    Ok(())
}

fn format_time(current: u64, total: u64) -> String {
    let format = |t: u64| {
        let minutes = t / 60;
        let seconds = t % 60;
        format!("{:02}:{:02}", minutes, seconds)
    };
    format!("{}/{}", format(current), format(total))
}

fn load_progress_bar(
    progress_tx: Sender<Event>,
    start_time: Instant,
    total_duration: Duration,
) -> io::Result<()> {
    let mut progress = 0.0;
    while progress < 1.0 {
        progress = (start_time.elapsed().as_secs_f64() / total_duration.as_secs_f64()).min(1.0);
        progress_tx
            .send(Event::SoundProgress(progress))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        progress_tx
            .send(Event::ProgressLabel(
                format_time(start_time.elapsed().as_secs(), total_duration.as_secs()),
                false,
            ))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        thread::sleep(Duration::from_millis(100));
    }
    progress_tx
        .send(Event::ProgressLabel(
            "Press <P> to play the sound".to_string(),
            true,
        ))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

pub(crate) fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read() {
            Ok(crossterm::event::Event::Key(key_event)) => {
                if let Err(e) = tx.send(Event::Input(key_event)) {
                    eprintln!("Error sending key event: {:?}", e);
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input event: {:?}", e);
            }
        }
    }
}

impl App {
    pub fn new(tx: Sender<Event>) -> App {
        Self {
            files: None,
            path: None,
            selected: 0,
            exit: false,
            progress_bar_color: Color::Green,
            sound_progress: 0.0,
            threshold: 0.01,
            tx,
            sink_original: None,
            sink_denoised: None,
            start_time: None,
            duration: None,
            ready_to_play: false,
            label: String::from("Press <P> to play the sound"),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        self.ensure_directories_exists()?;
        self.list_wav_files()?;

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match rx.recv() {
                Ok(Event::Input(key_event)) => self.handle_key_event(key_event)?,
                Ok(Event::SoundProgress(progress)) => self.sound_progress = progress,
                Ok(Event::SinksReady(sink_orig, sink_denoised, start_time, duration)) => {
                    self.sink_original = Some(sink_orig);
                    self.sink_denoised = Some(sink_denoised);
                    self.start_time = Some(start_time);
                    self.duration = Some(duration);
                    self.display_progress(start_time, duration);
                }
                Ok(Event::ProgressLabel(label, ready_to_play)) => {
                    self.label = label;
                    self.ready_to_play = ready_to_play;
                }
                Err(e) => {
                    eprintln!("Event receive error: {:?}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    fn ensure_directories_exists(&mut self) -> io::Result<()> {
        let current_dir = env::current_dir().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to get current directory: {}", e),
            )
        })?;

        let data_dir = current_dir.join("data");
        let denoised_dir = data_dir.join("denoised");

        fs::create_dir_all(&denoised_dir).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create 'data/denoised' directory: {}", e),
            )
        })?;

        self.path = Some(data_dir);
        Ok(())
    }

    fn list_wav_files(&mut self) -> io::Result<()> {
        let data_path = self
            .path
            .clone()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Data path not set"))?;

        let entries = fs::read_dir(&data_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read directory '{}': {}", data_path.display(), e),
            )
        })?;

        let mut files: Vec<String> = vec![];

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wav") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    files.push(name.to_string());
                    self.ready_to_play = true;
                }
            }
        }

        if files.is_empty() {
            files.push("<<Couldn't load any \".wav\" files; \nensure they are located in the\n\n\\data\\\n\ndirectory>>".to_string());
        }

        self.files = Some(files);
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn display_progress(&mut self, start_time: Instant, duration: Duration) {
        let progress_tx = self.tx.clone();
        thread::spawn(move || {
            if let Err(e) = load_progress_bar(progress_tx, start_time, duration) {
                eprintln!("Progress bar error: {:?}", e);
            }
        });
    }

    fn next(&mut self) {
        if let Some(files) = &self.files {
            if self.selected + 1 < files.len() {
                self.selected += 1;
            }
        }
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn selected_file(&self) -> Option<&String> {
        self.files.as_ref()?.get(self.selected)
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.is_press() {
            match key_event.code {
                crossterm::event::KeyCode::Char('q') => self.exit = true,
                crossterm::event::KeyCode::Char('p') => {
                    if self.ready_to_play {
                        self.ready_to_play = false;
                        self.sound_progress = 0.0;
                        self.progress_bar_color = Color::Green;
                        self.sink_original = None;
                        self.sink_denoised = None;
                        self.label = String::from("Denoising...");
                        let playback_tx = self.tx.clone(); // need to play file in a thread
                        let file_path = self.path.clone().unwrap();
                        let filename = self.selected_file().unwrap().clone();
                        let threshold = self.threshold.clone();
                        thread::spawn(move || {
                            if let Err(e) = play_file(playback_tx, file_path, &filename, threshold)
                            {
                                eprintln!("Playback thread error: {:?}", e);
                            }
                        });
                    }
                }
                crossterm::event::KeyCode::Char('c') => {
                    if let (Some(orig), Some(denoised)) = (&self.sink_original, &self.sink_denoised)
                    {
                        if orig.volume() > 0.0 {
                            orig.set_volume(0.0);
                            denoised.set_volume(1.0);
                            self.progress_bar_color = Color::Red;
                        } else {
                            orig.set_volume(1.0);
                            denoised.set_volume(0.0);
                            self.progress_bar_color = Color::Green;
                        }
                    }
                }
                crossterm::event::KeyCode::Down => self.next(),
                crossterm::event::KeyCode::Up => self.previous(),
                crossterm::event::KeyCode::Left => {
                    self.threshold = (self.threshold - 0.01).max(0.0);
                }
                crossterm::event::KeyCode::Right => {
                    self.threshold = (self.threshold + 0.01).min(0.1);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let horizontal_layout =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let [file_selection_area, right_side_area] = horizontal_layout.areas(area);
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(70), Constraint::Percentage(30)]);
        let [progress_bar_area, threshold_area] = vertical_layout.areas(right_side_area);
        let controls = Line::from(vec![
            " Change File ".into(),
            "<Up/Down>".red().bold(),
            " Play ".into(),
            "<P>".red().bold(),
            " Quit ".into(),
            "<Q> ".red().bold(),
        ])
        .bold()
        .centered();

        let controls_block = Block::bordered()
            .title(" Select WAV File ")
            .title_bottom(controls)
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let items: Vec<ListItem> = self
            .files
            .as_ref() // convert Option<&Vec<String>>
            .map(|files| files.iter().map(|f| ListItem::new(f.as_str())).collect())
            .unwrap_or_else(|| vec![ListItem::new("<No files found>")]);

        let file_selector = List::new(items)
            .block(controls_block)
            .highlight_style(Style::default().fg(Color::Yellow))
            .bg(Color::Indexed(017))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));

        let instructions = Line::from(vec![
            " Change to original/denoised ".into(),
            " <C> ".blue().bold(),
        ])
        .centered();

        let sound_controls_block = Block::bordered()
            .title(" Sound Track ")
            .title_bottom(instructions)
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(self.progress_bar_color))
            .block(sound_controls_block)
            .label(&self.label)
            .ratio(self.sound_progress);

        let threshold_instructions = Line::from(vec![
            " +0.01 / -0.01 ".into(),
            " <Left>/<Right> ".blue().bold(),
        ])
        .centered();

        let threshold_control_block = Block::bordered()
            .title(" Threshold ")
            .title_bottom(threshold_instructions)
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let threshold_bar = Gauge::default()
            .gauge_style(Style::default().fg(Color::LightBlue))
            .block(threshold_control_block)
            .label(Span::raw(format!("Threshold: {:.2}", self.threshold)))
            .ratio(self.threshold * 10.0);

        StatefulWidget::render(&file_selector, file_selection_area, buf, &mut state);

        progress_bar.render(
            Rect {
                x: progress_bar_area.left() + 3,
                y: (progress_bar_area.bottom() - progress_bar_area.top() - 3) / 2,
                width: progress_bar_area.width - 3,
                height: 3,
            },
            buf,
        );

        threshold_bar.render(threshold_area, buf)
    }
}
