use std::collections::VecDeque;
use std::time::Instant;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppState {
    #[default]
    Typing,
    Results,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TestMode {
    #[default]
    Words,
    Time,
}

pub(crate) struct App {
    pub(crate) text: String,
    pub(crate) typed: Vec<Option<char>>,
    pub(crate) delay_queue: VecDeque<char>,
    pub(crate) cursor: usize,
    pub(crate) start_time: Option<Instant>,
    pub(crate) wpm_samples: Vec<(f64, f64)>,
    pub(crate) last_sample_time: f64,
    pub(crate) state: AppState,
    pub(crate) total_keystrokes: usize,
    pub(crate) correct_keystrokes: usize,
    pub(crate) final_wpm: f64,
    pub(crate) should_quit: bool,
    pub(crate) word_count: usize,
    pub(crate) mode: TestMode,
    pub(crate) time_limit_secs: u64,
}

impl App {
    pub(crate) fn new() -> Self {
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
            mode: TestMode::Words,
            time_limit_secs: 30,
        };
        app.generate_text();
        app
    }
}
