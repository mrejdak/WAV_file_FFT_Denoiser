use crate::models::wav_file::WavFile;
use crate::models::wav_source::WavSource;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Stylize, Widget, StatefulWidget};
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState};
use ratatui::{DefaultTerminal, Frame};
use rodio::Source;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{env, io, thread};

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
    allow_actions: bool,
    progress_bar_color: Color,
    sound_progress: f64,
    tx: Sender<Event>,
    sink_original: Option<rodio::Sink>,
    sink_denoised: Option<rodio::Sink>,
    start_time: Option<Instant>,
    duration: Option<Duration>,
    ready_to_play: bool,
    label: String,
}

fn play_file(playback_tx: Sender<Event>, path: PathBuf, filename: &String) -> io::Result<()> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink1 = rodio::Sink::try_new(&stream_handle).unwrap();
    let sink2 = rodio::Sink::try_new(&stream_handle).unwrap();

    let full_path = path.clone().join(filename.clone());
    let file_path = full_path.to_str().unwrap();

    let save_path_buf = path.join("denoised").join(filename);
    let save_path = save_path_buf.to_str().unwrap();

    let wav = WavFile::from_wav_file(file_path).unwrap();

    let mut denoised_wav = wav.clone();
    denoised_wav.denoise_data_fft(0.01).expect("denoise panic");
    denoised_wav
        .save_to_file(save_path)
        .expect("save panic");
    let source = WavSource::from_wav_file(&wav);
    let denoised_source = WavSource::from_wav_file(&denoised_wav);

    let total_duration = source.total_duration().unwrap();

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
        .unwrap();

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
        progress_tx.send(Event::SoundProgress(progress)).unwrap();
        progress_tx
            .send(Event::ProgressLabel(
                format_time(start_time.elapsed().as_secs(), total_duration.as_secs()),
                false,
            ))
            .unwrap();
        thread::sleep(Duration::from_millis(100));
    }
    progress_tx
        .send(Event::ProgressLabel(
            "Press <P> to play the sound".to_string(),
            true,
        ))
        .unwrap();
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
        Self {
            files: None,
            path: None,
            selected: 0,
            exit: false,
            allow_actions: false,
            progress_bar_color: Color::Green,
            sound_progress: 0.0,
            tx,
            sink_original: None,
            sink_denoised: None,
            start_time: None,
            duration: None,
            ready_to_play: true,
            label: String::from("Press <P> to play the sound"),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        self.list_wav_files().expect("Couldn't load files panic");
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
                    self.display_progress(start_time, duration);
                }
                Event::ProgressLabel(label, ready_to_play) => {
                    self.label = label;
                    self.ready_to_play = ready_to_play;
                }
            }
        }
        Ok(())
    }

    fn list_wav_files(&mut self) -> io::Result<()> {
        let dir = env::current_dir()?.join("data");
        self.path = Some(dir.clone());

        self.files = Some(
            std::fs::read_dir(dir)?
                .filter_map(|entry| {
                    let path = entry.ok()?.path();
                    if path.extension()?.to_str()? == "wav" {
                        self.allow_actions = true;
                        Some(path.file_name()?.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect(),
        );
        if !self.allow_actions {
            self.files = Some(vec!["<<Couldn't load files>>".to_string()]);
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn display_progress(&mut self, start_time: Instant, duration: Duration) {
        let progress_tx = self.tx.clone();
        thread::spawn(move || {
            load_progress_bar(progress_tx, start_time, duration).expect("progress display panic");
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
        if key_event.is_press() && self.allow_actions {
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
                        thread::spawn(move || {
                            play_file(playback_tx, file_path, &filename).expect("playback panic");
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
                crossterm::event::KeyCode::Down => {
                    self.next()
                }
                crossterm::event::KeyCode::Up => {
                    self.previous()
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_layout = Layout::vertical([
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ]);
        let horizontal_layout =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let [top_area, raw_wave_area, denoised_wave_area] = vertical_layout.areas(area);
        let [file_selection_area, progress_bar_area] = horizontal_layout.areas(top_area);

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
            .as_ref()  // convert Option<&Vec<String>>
            .map(|files| {
                files.iter()
                    .map(|f| ListItem::new(f.as_str()))
                    .collect()
            })
            .unwrap_or_else(|| vec![ListItem::new("<No files found>")]);

        let file_selector = List::new(items).block(controls_block)
            .highlight_style(Style::default().fg(Color::Yellow)).bg(Color::Indexed(017))
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

        let block = Block::bordered()
            .title(" Raw Sound Wave ")
            .borders(Borders::ALL);

        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(self.progress_bar_color))
            .block(sound_controls_block)
            .label(&self.label)
            .ratio(self.sound_progress);

        let sound_wave = Gauge::default().block(block); // temporary sound_wave object
        // let sound_wave = Canvas::default().block(block);

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

        sound_wave.render(raw_wave_area, buf);

    }
}