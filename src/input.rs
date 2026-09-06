use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, AppState, TestMode};
use crate::words::{TIME_LIMITS, WORD_COUNTS};

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) {
        match self.state {
            AppState::Typing => self.handle_typing_key(key),
            AppState::Results => self.handle_results_key(key),
        }
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        self.extend_if_needed();
        match key.code {
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab if self.start_time.is_none() => {
                self.mode = match self.mode {
                    TestMode::Words => TestMode::Time,
                    TestMode::Time => TestMode::Words,
                };
                self.typed.clear();
                self.delay_queue.clear();
                self.cursor = 0;
                self.generate_text();
            }
            KeyCode::Char('1') if self.start_time.is_none() => self.select_option(0),
            KeyCode::Char('2') if self.start_time.is_none() => self.select_option(1),
            KeyCode::Char('3') if self.start_time.is_none() => self.select_option(2),
            KeyCode::Char('4') if self.start_time.is_none() => self.select_option(3),
            KeyCode::Backspace if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) => {
                self.delete_word();
            }
            KeyCode::Char('w')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.delete_word();
            }
            KeyCode::Char('h')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
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

                if self.cursor == self.text.len() && self.mode == TestMode::Words {
                    self.finish_test();
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

    fn select_option(&mut self, index: usize) {
        match self.mode {
            TestMode::Words => {
                if let Some(&wc) = WORD_COUNTS.get(index) {
                    self.set_word_count(wc);
                }
            }
            TestMode::Time => {
                if let Some(&secs) = TIME_LIMITS.get(index) {
                    self.set_time_limit(secs);
                }
            }
        }
    }

    fn finish_test(&mut self) {
        let elapsed = self
            .start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        self.final_wpm = (self.correct_keystrokes as f64 / 5.0) / (elapsed / 60.0);
        self.wpm_samples.push((elapsed, self.final_wpm));
        self.state = AppState::Results;
    }

    pub(crate) fn check_time_limit(&mut self) {
        if self.mode == TestMode::Time
            && self.state == AppState::Typing
            && let Some(start) = self.start_time
            && start.elapsed().as_secs() >= self.time_limit_secs
        {
            self.finish_test();
        }
    }
}
