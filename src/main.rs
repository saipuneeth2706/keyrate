use std::collections::VecDeque;
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use rand::seq::SliceRandom;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    symbols::Marker,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph},
};

const WORD_BANK: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "I",
    "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
    "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
    "or", "an", "will", "my", "one", "all", "would", "there", "their", "what",
    "so", "up", "out", "if", "about", "who", "get", "which", "go", "me",
    "when", "make", "can", "like", "time", "no", "just", "him", "know", "take",
    "people", "into", "year", "your", "good", "some", "could", "them", "see",
    "other", "than", "then", "now", "look", "only", "come", "its", "over",
    "think", "also", "back", "after", "use", "two", "how", "our", "work",
    "first", "well", "way", "even", "new", "want", "because", "any", "these",
    "give", "day", "most", "us", "great", "between", "need", "large", "under",
    "never", "each", "right", "begin", "too", "hand", "high", "keep", "last",
    "long", "much", "where", "while", "world", "still", "own", "small", "off",
    "every", "found", "head", "place", "again", "live", "give", "most", "very",
    "body", "make", "know", "leave", "group", "real", "help", "turn", "move",
    "play", "run", "close", "night", "open", "start", "show", "try", "ask",
    "must", "home", "big", "read", "hand", "port", "spell", "add", "land",
    "here", "must", "hard", "school", "grow", "study", "still", "learn", "plant",
    "cover", "food", "four", "between", "city", "tree", "cross", "farm", "hard",
    "story", "son", "once", "front", "few", "left", "side", "feet", "car",
    "mile", "river", "dark", "surface", "deep", "morning", "simple", "several",
    "toward", "house", "point", "page", "letter", "mother", "answer", "found",
    "study", "still", "learn", "should", "America", "world",
];

const WORD_COUNTS: &[usize] = &[10, 25, 50, 100];

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Typing,
    Results,
}

struct App {
    text: String,
    typed: Vec<Option<char>>,
    delay_queue: VecDeque<char>,
    cursor: usize,
    start_time: Option<Instant>,
    wpm_samples: Vec<(f64, f64)>,
    last_sample_time: f64,
    state: AppState,
    total_keystrokes: usize,
    correct_keystrokes: usize,
    final_wpm: f64,
    should_quit: bool,
    word_count: usize,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| app.view(frame))?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key);
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            text: String::new(),
            typed: Vec::new(),
            delay_queue: VecDeque::new(),
            cursor: 0,
            start_time: None,
            wpm_samples: Vec::new(),
            last_sample_time: 0.0,
            state: AppState::Typing,
            total_keystrokes: 0,
            correct_keystrokes: 0,
            final_wpm: 0.0,
            should_quit: false,
            word_count: 25,
        };
        app.generate_text();
        app
    }

    fn generate_text(&mut self) {
        let mut rng = rand::thread_rng();
        let words: Vec<&str> = WORD_BANK.choose_multiple(&mut rng, self.word_count).copied().collect();
        self.text = words.join(" ");
    }

    fn restart(&mut self) {
        let wc = self.word_count;
        *self = Self::new();
        self.word_count = wc;
        self.generate_text();
    }

    fn set_word_count(&mut self, count: usize) {
        if self.start_time.is_some() {
            return;
        }
        self.word_count = count;
        self.typed.clear();
        self.delay_queue.clear();
        self.cursor = 0;
        self.generate_text();
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match self.state {
            AppState::Typing => self.handle_typing_key(key),
            AppState::Results => self.handle_results_key(key),
        }
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') if self.start_time.is_none() => self.set_word_count(10),
            KeyCode::Char('2') if self.start_time.is_none() => self.set_word_count(25),
            KeyCode::Char('3') if self.start_time.is_none() => self.set_word_count(50),
            KeyCode::Char('4') if self.start_time.is_none() => self.set_word_count(100),
            KeyCode::Backspace
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) =>
            {
                self.delete_word();
            }
            KeyCode::Char('h')
                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.delete_word();
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.typed.pop();
                    self.delay_queue.pop_back();
                    if self.cursor == 0 {
                        self.start_time = None;
                    }
                }
            }
            KeyCode::Char(c) => {
                self.total_keystrokes += 1;
                let expected = self.text.chars().nth(self.cursor).unwrap_or('\0');

                if c == expected {
                    self.correct_keystrokes += 1;
                    self.typed.push(Some(c));
                    self.delay_queue.push_back(c);
                } else {
                    self.typed.push(Some(c));
                    self.delay_queue.push_back(c);
                }

                self.cursor += 1;

                if self.cursor == self.text.len() {
                    let elapsed = self.start_time.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
                    self.final_wpm = (self.correct_keystrokes as f64 / 5.0) / (elapsed / 60.0);
                    self.wpm_samples.push((elapsed, self.final_wpm));
                    self.state = AppState::Results;
                } else if let Some(start) = self.start_time {
                    let now = start.elapsed().as_secs_f64();
                    if now - self.last_sample_time >= 1.0 {
                        let wpm = (self.correct_keystrokes as f64 / 5.0) / (now / 60.0);
                        self.wpm_samples.push((now, wpm));
                        self.last_sample_time = now;
                    }
                }

                if self.start_time.is_none() {
                    self.start_time = Some(Instant::now());
                }
            }
            _ => {}
        }
    }

    fn handle_results_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r') => self.restart(),
            KeyCode::Char('q') => {
                self.typed.clear();
                self.delay_queue.clear();
                self.cursor = 0;
                self.start_time = None;
                self.wpm_samples.clear();
                self.last_sample_time = 0.0;
                self.total_keystrokes = 0;
                self.correct_keystrokes = 0;
                self.final_wpm = 0.0;
                self.state = AppState::Typing;
            }
            KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    fn delete_word(&mut self) {
        let mut removed = 0;
        while self.cursor > 0 {
            let ch = self.text.chars().nth(self.cursor - 1).unwrap_or(' ');
            if ch == ' ' && removed > 0 {
                break;
            }
            self.cursor -= 1;
            self.typed.pop();
            self.delay_queue.pop_back();
            removed += 1;
        }
        if self.cursor == 0 {
            self.start_time = None;
        }
    }

    fn view(&self, frame: &mut Frame) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

        match self.state {
            AppState::Typing => self.view_typing(frame, main_area, footer_area),
            AppState::Results => {
                self.view_typing(frame, main_area, footer_area);
                self.view_results(frame, main_area);
            }
        }
    }

    fn view_typing(&self, frame: &mut Frame, area: Rect, footer_area: Rect) {
        let [selector_area, text_area, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(0),
        ])
        .areas(area);

        let selector: Vec<Span> = WORD_COUNTS
            .iter()
            .enumerate()
            .flat_map(|(i, &wc)| {
                let key = format!(" [{}] ", i + 1);
                let label = if wc == self.word_count {
                    format!("{}*", wc)
                } else {
                    format!("{} ", wc)
                };
                let key_style = if wc == self.word_count {
                    Style::default().fg(Color::Rgb(205, 214, 244)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(108, 112, 134))
                };
                let label_style = if wc == self.word_count {
                    Style::default().fg(Color::Rgb(166, 227, 161)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(108, 112, 134))
                };
                vec![
                    Span::styled(key, key_style),
                    Span::styled(label, label_style),
                ]
            })
            .collect();
        frame.render_widget(Paragraph::new(Line::from(selector)), selector_area);

        let block = Block::bordered().title("keyrate");
        let inner = block.inner(text_area);
        frame.render_widget(block, text_area);

        let padding = 4u16;
        let paragraph_lines = self.text.len().div_ceil(inner.width.saturating_sub(padding * 2).max(1) as usize);
        let vertical_pad = inner.height.saturating_sub(paragraph_lines as u16 + 2) / 2;
        let para_area = Rect {
            x: inner.x + padding,
            y: inner.y + vertical_pad,
            width: inner.width.saturating_sub(padding * 2),
            height: inner.height.saturating_sub(vertical_pad),
        };

        let width = para_area.width as usize;
        if width == 0 {
            return;
        }

        let paragraph_text: Vec<Span> = self
            .text
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let style = if i < self.typed.len() {
                    match self.typed[i] {
                        Some(ch) if ch == c => Style::default().bold().fg(Color::Rgb(166, 227, 161)),
                        Some(_) => Style::default().fg(Color::Rgb(243, 139, 168)).add_modifier(Modifier::BOLD),
                        None => Style::default().fg(Color::Rgb(108, 112, 134)),
                    }
                } else {
                    Style::default().fg(Color::Rgb(108, 112, 134))
                };
                let style = if i == self.cursor && self.state == AppState::Typing {
                    style.fg(Color::Rgb(205, 214, 244)).add_modifier(Modifier::REVERSED | Modifier::BOLD)
                } else {
                    style
                };
                Span::styled(c.to_string(), style)
            })
            .collect();

        let mut lines: Vec<Line> = Vec::new();
        let mut i = 0;
        while i < paragraph_text.len() {
            let end = (i + width).min(paragraph_text.len());
            lines.push(Line::from(paragraph_text[i..end].to_vec()));
            i = end;
        }
        if lines.is_empty() {
            lines.push(Line::from(""));
        }

        frame.render_widget(Paragraph::new(lines), para_area);

        let footer = Line::from(vec![
            " Esc ".bold().cyan(),
            "quit ".dim(),
            " bs ".bold().cyan(),
            "correct ".dim(),
            " Alt+bs ".bold().cyan(),
            "Ctrl+bs ".bold().cyan(),
            "word ".dim(),
        ]);
        frame.render_widget(Paragraph::new(footer), footer_area);
    }

    fn view_results(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Clear;

        let popup = centered_rect(60, 60, area);
        frame.render_widget(Clear, popup);

        let block = Block::bordered().title("results");
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        if inner.width < 10 || inner.height < 6 {
            return;
        }

        let accuracy = if self.total_keystrokes > 0 {
            (self.correct_keystrokes as f64 / self.total_keystrokes as f64) * 100.0
        } else {
            0.0
        };

        let wpm_text = format!("{} wpm", self.final_wpm.round() as u64);
        let acc_text = format!("{}% accuracy", accuracy.round() as u64);

        let wpm_style = if self.final_wpm >= 60.0 {
            Style::default().fg(Color::Rgb(166, 227, 161)).bold()
        } else if self.final_wpm >= 40.0 {
            Style::default().fg(Color::Rgb(249, 226, 175)).bold()
        } else {
            Style::default().fg(Color::Rgb(243, 139, 168)).bold()
        };

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(&wpm_text, wpm_style))).centered(),
            Rect { y: inner.y, height: 2, ..inner },
        );

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(&acc_text, Style::default().dim())).centered()),
            Rect { y: inner.y + 2, height: 1, ..inner },
        );

        if self.wpm_samples.len() >= 2 {
            let chart_area = Rect {
                y: inner.y + 4,
                height: inner.height.saturating_sub(6),
                ..inner
            };

            if chart_area.height >= 3 && chart_area.width >= 10 {
                let wpm_values: Vec<f64> = self.wpm_samples.iter().map(|(_, w)| *w).collect();
                let min_wpm = wpm_values.iter().copied().fold(f64::INFINITY, f64::min);
                let max_wpm = wpm_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let wpm_range = if max_wpm - min_wpm < 10.0 {
                    (min_wpm - 5.0).max(0.0)..(max_wpm + 5.0)
                } else {
                    min_wpm..max_wpm
                };

                let data: Vec<(f64, f64)> =
                    self.wpm_samples.iter().map(|&(t, w)| (t, w)).collect();

                let dataset = Dataset::default()
                    .name("wpm")
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .data(&data);

                let time_labels: Vec<ratatui::text::Span> =
                    (0..=4).map(|i| Span::raw(format!("{}s", i))).collect();

                let wpm_labels: Vec<ratatui::text::Span> = (0..=4)
                    .map(|i| {
                        let val = wpm_range.start
                            + (wpm_range.end - wpm_range.start) * (i as f64 / 4.0);
                        Span::raw(format!("{}", val.round() as u64))
                    })
                    .collect();

                let chart = Chart::new(vec![dataset])
                    .x_axis(
                        Axis::default().title("time").labels(time_labels).bounds([
                            0.0,
                            self.wpm_samples.last().map_or(1.0, |&(t, _)| t).max(1.0),
                        ]),
                    )
                    .y_axis(
                        Axis::default().title("wpm").labels(wpm_labels).bounds([
                            wpm_range.start,
                            wpm_range.end,
                        ]),
                    );

                frame.render_widget(chart, chart_area);
            }
        }

        let footer_line = Line::from(vec![
            " q ".bold().cyan(),
            "again ".dim(),
            " r ".bold().cyan(),
            "gimme new words ".dim(),
            " Esc ".bold().cyan(),
            "bye bye ".dim(),
        ]);
        frame.render_widget(
            Paragraph::new(footer_line),
            Rect { y: popup.y + popup.height - 1, height: 1, x: popup.x, width: popup.width },
        );
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, center, _] = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .areas(area);

    let [_, center, _] = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .areas(center);

    center
}
