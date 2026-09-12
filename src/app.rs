use std::collections::VecDeque;
use std::time::Instant;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppState {
    #[default]
    Onboarding,
    Typing,
    Results,
    Scores,
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
    pub(crate) name: String,
    pub(crate) draft_name: String,
    pub(crate) scores_mode: TestMode,
}

impl App {
    pub(crate) fn consistency(&self) -> f64 {
        if self.wpm_samples.len() < 2 {
            return -1.0;
        }
        let mean =
            self.wpm_samples.iter().map(|(_, w)| *w).sum::<f64>() / self.wpm_samples.len() as f64;
        let var = self
            .wpm_samples
            .iter()
            .map(|(_, w)| (w - mean).powi(2))
            .sum::<f64>()
            / self.wpm_samples.len() as f64;
        let sd = var.sqrt();
        if mean > 0.0 {
            (1.0 - sd / mean).clamp(0.0, 1.0) * 100.0
        } else {
            0.0
        }
    }

    pub(crate) fn new() -> Self {
        let config = crate::config::load();
        let mut app = Self {
            text: String::new(),
            typed: Vec::new(),
            delay_queue: VecDeque::new(),
            cursor: 0,
            start_time: None,
            wpm_samples: Vec::new(),
            last_sample_time: 0.0,
            state: if config.is_some() {
                AppState::Typing
            } else {
                AppState::Onboarding
            },
            total_keystrokes: 0,
            correct_keystrokes: 0,
            final_wpm: 0.0,
            should_quit: false,
            word_count: 25,
            mode: TestMode::Words,
            time_limit_secs: 30,
            name: config
                .and_then(|c| {
                    let name = c.name.trim().to_string();
                    if name.is_empty() { None } else { Some(name) }
                })
                .unwrap_or_default(),
            draft_name: String::new(),
            scores_mode: TestMode::Words,
        };
        app.generate_text();
        app
    }
}
