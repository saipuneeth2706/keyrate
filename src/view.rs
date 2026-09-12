use ratatui::{
    Frame,
    layout::{Constraint, Layout, Offset, Rect},
    style::{Color, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Clear, Dataset, GraphType, Paragraph, Shadow,
    },
};

use crate::app::{App, AppState, TestMode};
use crate::theme::{
    ACCENT, BORDER, EMPHASIS, FG_DONE, FG_ERR, FG_TODO, FG_WARN, MUTED, acc_tier, wpm_badge,
    wpm_tier,
};
use crate::words::{TIME_LIMITS, WORD_COUNTS};

impl App {
    pub(crate) fn view(&self, frame: &mut Frame) {
        let [main_area, footer_space] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(frame.area());
        let [footer_rule, footer_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(footer_space);
        frame.render_widget(
            Block::new()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(MUTED)),
            footer_rule,
        );

        match self.state {
            AppState::Onboarding => self.view_onboarding(frame, main_area),
            AppState::Typing => self.view_typing(frame, main_area, footer_area),
            AppState::Results => {
                self.view_typing(frame, main_area, footer_area);
                self.view_results(frame, main_area);
            }
            AppState::Scores => self.view_scores(frame, main_area, footer_area),
        }
    }

    fn elapsed_secs(&self) -> f64 {
        match self.state {
            AppState::Onboarding | AppState::Scores => 0.0,
            AppState::Results => {
                if self.final_wpm > 0.0 {
                    (self.correct_keystrokes as f64 / 5.0) / (self.final_wpm / 60.0)
                } else {
                    0.0
                }
            }
            AppState::Typing => self
                .start_time
                .map(|t| t.elapsed().as_secs_f64())
                .unwrap_or(0.0),
        }
    }

    fn live_wpm(&self) -> f64 {
        match self.state {
            AppState::Onboarding | AppState::Scores => 0.0,
            AppState::Results => self.final_wpm,
            AppState::Typing => {
                let elapsed = self.elapsed_secs();
                if elapsed > 0.0 {
                    (self.correct_keystrokes as f64 / 5.0) / (elapsed / 60.0)
                } else {
                    0.0
                }
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

        let key_style = |active: bool| {
            if active {
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            }
        };
        let mode_style = |active: bool| {
            if active {
                Style::default()
                    .fg(Color::Black)
                    .bg(ACCENT)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            }
        };
        let option_style = |active: bool| {
            if active {
                Style::default()
                    .fg(Color::Black)
                    .bg(FG_DONE)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            }
        };

        let mut selector: Vec<Span> = vec![
            Span::styled(
                " Mode ",
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled("[", Style::default().dim()),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(EMPHASIS)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled("]  ", Style::default().dim()),
        ];

        let (words_active, time_active) = match self.mode {
            TestMode::Words => (true, false),
            TestMode::Time => (false, true),
        };
        selector.push(Span::styled(" Words ", mode_style(words_active)));
        selector.push(Span::styled(" | ", Style::default().dim()));
        selector.push(Span::styled(" Time ", mode_style(time_active)));
        selector.push(Span::styled("   ", Style::default().dim()));

        let options: Vec<u64> = match self.mode {
            TestMode::Words => WORD_COUNTS.iter().map(|&w| w as u64).collect(),
            TestMode::Time => TIME_LIMITS.to_vec(),
        };
        let current: u64 = match self.mode {
            TestMode::Words => self.word_count as u64,
            TestMode::Time => self.time_limit_secs,
        };

        for (i, option) in options.iter().enumerate() {
            let active = *option == current;
            let key = format!("[{}] ", i + 1);
            selector.push(Span::styled(key, key_style(active)));
            selector.push(Span::styled(format!(" {option} "), option_style(active)));
            selector.push(Span::styled(" ", Style::default().dim()));
        }
        frame.render_widget(Paragraph::new(Line::from(selector)), selector_area);

        let mut title = vec![Span::styled(
            " keyrate ",
            Style::default()
                .fg(ACCENT)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )];
        if !self.name.is_empty() {
            title.push(Span::styled("·", Style::default().dim()));
            title.push(Span::styled(
                format!(" {} ", self.name),
                Style::default().fg(EMPHASIS),
            ));
        }
        if let Some(city) = &self.city {
            title.push(Span::styled("·", Style::default().dim()));
            title.push(Span::styled(
                format!(" {city} "),
                Style::default().fg(MUTED),
            ));
        }
        let title = Line::from(title);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER))
            .title(title)
            .title_bottom(self.status_line());
        let inner = block.inner(text_area);
        frame.render_widget(block, text_area);

        let padding = 4u16;
        let paragraph_lines = self
            .text
            .len()
            .div_ceil(inner.width.saturating_sub(padding * 2).max(1) as usize);
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
                        Some(ch) if ch == c => Style::default().bold().fg(FG_DONE),
                        Some(_) => Style::default()
                            .fg(FG_ERR)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                        None => Style::default().fg(FG_TODO),
                    }
                } else {
                    Style::default().fg(FG_TODO)
                };
                let style = if i == self.cursor && self.state == AppState::Typing {
                    style.fg(EMPHASIS).add_modifier(
                        ratatui::style::Modifier::REVERSED | ratatui::style::Modifier::BOLD,
                    )
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
            " Esc ".bold().fg(ACCENT),
            "quit ".dim(),
            " bs ".bold().fg(ACCENT),
            "correct ".dim(),
            " Ctrl+s ".bold().fg(ACCENT),
            "scores ".dim(),
            " Alt+bs ".bold().fg(ACCENT),
            "Ctrl+bs ".bold().fg(ACCENT),
            "Ctrl+w ".bold().fg(ACCENT),
            "word ".dim(),
        ]);
        frame.render_widget(Paragraph::new(footer), footer_area);
    }

    fn status_line(&self) -> Line<'static> {
        let elapsed = self.elapsed_secs();
        let mut spans: Vec<Span> = Vec::new();

        match self.mode {
            TestMode::Time => {
                let limit = self.time_limit_secs as f64;
                let remaining = (limit - elapsed).max(0.0);
                let secs = remaining.ceil() as u64;

                const BAR_W: usize = 12;
                let fill = if limit > 0.0 {
                    ((elapsed / limit) * BAR_W as f64).round() as usize
                } else {
                    0
                };
                let fill = fill.min(BAR_W);
                let bar: String = "█".repeat(fill) + &"░".repeat(BAR_W - fill);

                let bar_style = if remaining > 10.0 {
                    Style::default().fg(FG_DONE)
                } else if remaining > 5.0 {
                    Style::default().fg(FG_WARN)
                } else {
                    Style::default().fg(FG_ERR)
                };

                spans.push(Span::styled(" time left ", Style::default().dim()));
                spans.push(Span::styled(
                    format!("{secs:>3}s "),
                    Style::default()
                        .fg(EMPHASIS)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ));
                spans.push(Span::styled(bar, bar_style));
            }
            TestMode::Words => {
                spans.push(Span::styled(" time ", Style::default().dim()));
                spans.push(Span::styled(
                    format!("{}s ", elapsed.floor() as u64),
                    Style::default()
                        .fg(EMPHASIS)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ));
            }
        }

        let acc = if self.total_keystrokes > 0 {
            (self.correct_keystrokes as f64 / self.total_keystrokes as f64) * 100.0
        } else {
            0.0
        };
        let err = self.total_keystrokes - self.correct_keystrokes;

        spans.push(Span::styled(" · ", Style::default().dim()));
        spans.push(Span::styled(" wpm ", Style::default().dim()));
        spans.push(Span::styled(
            format!("{:.0}", self.live_wpm()),
            Style::default()
                .fg(EMPHASIS)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ));
        spans.push(Span::styled(" · acc ", Style::default().dim()));
        spans.push(Span::styled(
            format!("{:.0}%", acc),
            Style::default()
                .fg(EMPHASIS)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ));
        spans.push(Span::styled(" · err ", Style::default().dim()));
        spans.push(Span::styled(
            format!("{err}"),
            Style::default()
                .fg(EMPHASIS)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ));

        Line::from(spans)
    }

    fn view_onboarding(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().style(Style::default().bg(Color::Indexed(236))),
            area,
        );

        let popup = centered_rect(50, 25, area);
        frame.render_widget(Clear, popup);

        let title = Line::from(vec![Span::styled(
            " keyrate ",
            Style::default()
                .fg(ACCENT)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER))
            .shadow(Shadow::default().offset(Offset::new(1, 1)))
            .title(title);
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        if inner.width < 10 || inner.height < 5 {
            return;
        }

        let [prompt_area, input_area, hint_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(inner);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                " enter name ",
                Style::default().fg(MUTED),
            ))),
            prompt_area,
        );

        let input_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER));
        let input_inner = input_block.inner(input_area);
        frame.render_widget(input_block, input_area);

        let input_line = Line::from(vec![
            if self.draft_name.is_empty() {
                Span::styled(" ", Style::default().fg(MUTED))
            } else {
                Span::styled(self.draft_name.clone(), Style::default().fg(EMPHASIS))
            },
            Span::styled(
                "█",
                Style::default()
                    .fg(FG_DONE)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(input_line),
            Rect {
                y: input_inner.y,
                x: input_inner.x + 1,
                height: input_inner.height,
                width: input_inner.width.saturating_sub(2),
            },
        );

        let hint = Line::from(vec![
            " Enter ".bold().fg(ACCENT),
            "save ".dim(),
            " Backspace ".bold().fg(ACCENT),
            "delete ".dim(),
            " Esc ".bold().fg(ACCENT),
            "quit ".dim(),
        ]);
        frame.render_widget(Paragraph::new(hint), hint_area);
    }

    fn view_scores(&self, frame: &mut Frame, area: Rect, _footer_area: Rect) {
        let [selector_area, summary_area, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        let pill = |active: bool| {
            if active {
                Style::default()
                    .fg(Color::Black)
                    .bg(ACCENT)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            }
        };
        let (words_active, time_active) = match self.scores_mode {
            TestMode::Words => (true, false),
            TestMode::Time => (false, true),
        };
        let selector = Line::from(vec![
            Span::styled(
                " scores ",
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled("[s]", Style::default().dim()),
            Span::styled("  ", Style::default().dim()),
            Span::styled(" Words ", pill(words_active)),
            Span::styled(" | ", Style::default().dim()),
            Span::styled(" Time ", pill(time_active)),
        ]);
        frame.render_widget(Paragraph::new(selector), selector_area);

        let records = crate::scores::load(self.scores_mode);
        if !records.is_empty() {
            let best = records
                .iter()
                .map(|r| r.wpm)
                .fold(f64::NEG_INFINITY, f64::max);
            let avg_wpm = records.iter().map(|r| r.wpm).sum::<f64>() / records.len() as f64;
            let avg_acc = records.iter().map(|r| r.accuracy).sum::<f64>() / records.len() as f64;
            let summary = Line::from(vec![
                Span::styled(format!(" best {:.0} wpm ", best), wpm_tier(best)),
                Span::styled("·", Style::default().dim()),
                Span::styled(
                    format!(" avg {:.0} wpm ", avg_wpm),
                    Style::default()
                        .fg(EMPHASIS)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::styled("·", Style::default().dim()),
                Span::styled(
                    format!(" avg acc {:.0}% ", avg_acc),
                    Style::default()
                        .fg(EMPHASIS)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
            ]);
            frame.render_widget(Paragraph::new(summary), summary_area);
        } else {
            let summary = Line::from(Span::styled(
                " no scores logged yet — finish a test to see it here ",
                Style::default().dim(),
            ));
            frame.render_widget(Paragraph::new(summary), summary_area);
        }

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER));
        let inner = block.inner(list_area);
        frame.render_widget(block, list_area);

        if inner.width < 30 || inner.height < 3 {
            return;
        }

        if records.is_empty() {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " nothing here yet ",
                    Style::default().dim(),
                )))
                .centered(),
                inner,
            );
            return;
        }

        let mut lines: Vec<Line> = Vec::new();
        let header = Line::from(vec![
            Span::styled(format!("{:<3}", "#"), Style::default().dim()),
            Span::styled(format!("{wpm:>5}  ", wpm = "wpm"), Style::default().dim()),
            Span::styled(format!("{acc:>4}  ", acc = "acc"), Style::default().dim()),
            Span::styled(format!("{err:>3}  ", err = "err"), Style::default().dim()),
            Span::styled(format!("cons{:<1}  ", ""), Style::default().dim()),
            Span::styled(format!("{cfg:<4}", cfg = "cfg"), Style::default().dim()),
            Span::styled(format!("{date:<12}", date = "date"), Style::default().dim()),
        ]);
        lines.push(header);

        for (i, r) in records.iter().rev().take(10).enumerate() {
            let err_style = if r.errors > 0 {
                Style::default()
                    .fg(FG_ERR)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default()
                    .fg(FG_DONE)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            };
            let cons = if r.consistency < 0.0 {
                "—".to_string()
            } else {
                format!("{:.0}%", r.consistency)
            };
            let cfg = match self.scores_mode {
                TestMode::Time => format!("{}s", r.option),
                TestMode::Words => format!("{}w", r.option),
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{:<3}", i + 1), Style::default().dim()),
                Span::styled(format!("{:>5}  ", r.wpm.round() as u64), wpm_tier(r.wpm)),
                Span::styled(
                    format!("{:>4}%  ", r.accuracy.round() as u64),
                    acc_tier(r.accuracy),
                ),
                Span::styled(format!("{:>3}  ", r.errors), err_style),
                Span::styled(
                    format!("{cons:>5}  "),
                    if r.consistency < 0.0 {
                        Style::default().dim()
                    } else {
                        acc_tier(r.consistency)
                    },
                ),
                Span::styled(
                    format!("{cfg:<4}"),
                    Style::default()
                        .fg(EMPHASIS)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::styled(format_date(r.timestamp), Style::default().fg(MUTED)),
            ]));
        }

        frame.render_widget(Paragraph::new(lines), inner);
    }

    fn view_results(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().style(Style::default().bg(Color::Indexed(236))),
            area,
        );

        let popup = centered_rect(60, 60, area);
        frame.render_widget(Clear, popup);

        let mode_str = match self.mode {
            TestMode::Words => format!("{} words", self.word_count),
            TestMode::Time => format!("{}s", self.time_limit_secs),
        };
        let title = Line::from(vec![
            Span::styled(
                " results ",
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled("·", Style::default().dim()),
            Span::styled(
                format!(" {mode_str} "),
                Style::default()
                    .fg(EMPHASIS)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled("·", Style::default().dim()),
            Span::styled(" keyrate", Style::default().dim()),
        ]);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER))
            .shadow(Shadow::default().offset(Offset::new(1, 1)))
            .title(title);
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        if inner.width < 10 || inner.height < 8 {
            return;
        }

        let accuracy = if self.total_keystrokes > 0 {
            (self.correct_keystrokes as f64 / self.total_keystrokes as f64) * 100.0
        } else {
            0.0
        };
        let errors = self.total_keystrokes - self.correct_keystrokes;
        let elapsed = self.elapsed_secs();

        let [head_area, stats_area, chart_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let wpm_text = format!("{} wpm", self.final_wpm.round() as u64);
        let badge = format!(" · {} ", wpm_badge(self.final_wpm));
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(wpm_text, wpm_tier(self.final_wpm)),
                Span::styled(badge, Style::default().dim()),
            ]))
            .centered(),
            Rect {
                y: head_area.y,
                height: 1,
                ..head_area
            },
        );

        let acc_text = format!("{}% accuracy", accuracy.round() as u64);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(acc_text, acc_tier(accuracy)))).centered(),
            Rect {
                y: head_area.y + 1,
                height: 1,
                ..head_area
            },
        );

        let consistency = self.consistency();

        let cons_text = if consistency < 0.0 {
            "—".to_string()
        } else {
            format!("{}%", consistency.round() as u64)
        };
        let cons_style = if consistency < 0.0 {
            Style::default().dim()
        } else {
            acc_tier(consistency)
        };
        let err_style = if errors > 0 {
            Style::default()
                .fg(FG_ERR)
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default()
                .fg(FG_DONE)
                .add_modifier(ratatui::style::Modifier::BOLD)
        };

        let chips: Vec<(&str, String, Style)> = vec![
            (
                "wpm",
                format!("{}", self.final_wpm.round() as u64),
                wpm_tier(self.final_wpm),
            ),
            (
                "acc",
                format!("{}%", accuracy.round() as u64),
                acc_tier(accuracy),
            ),
            ("err", format!("{errors}"), err_style),
            (
                "time",
                format!("{}s", elapsed.round() as u64),
                Style::default()
                    .fg(EMPHASIS)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            ("cons", cons_text, cons_style),
        ];

        let cols: [Rect; 5] = Layout::horizontal([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
        ])
        .areas(stats_area);

        for ((label, value, style), col) in chips.iter().zip(cols.iter()) {
            let chip_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER))
                .title(format!(" {label} "));
            let chip_inner = chip_block.inner(*col);
            frame.render_widget(chip_block, *col);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(value.as_str(), *style))).centered(),
                chip_inner,
            );
        }

        if self.wpm_samples.len() >= 2 {
            let chart_area = Rect {
                height: chart_area.height.saturating_sub(1),
                ..chart_area
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

                let data: Vec<(f64, f64)> = self.wpm_samples.iter().map(|&(t, w)| (t, w)).collect();

                let dataset = Dataset::default()
                    .name("wpm")
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(FG_DONE))
                    .data(&data);

                let max_time = self.wpm_samples.last().map_or(1.0, |&(t, _)| t).max(1.0);

                let time_labels: Vec<ratatui::text::Span> = (0..=4)
                    .map(|i| {
                        let val = max_time * (i as f64 / 4.0);
                        Span::raw(format!("{}s", val.round() as u64))
                    })
                    .collect();

                let wpm_labels: Vec<ratatui::text::Span> = (0..=4)
                    .map(|i| {
                        let val =
                            wpm_range.start + (wpm_range.end - wpm_range.start) * (i as f64 / 4.0);
                        Span::raw(format!("{}", val.round() as u64))
                    })
                    .collect();

                let chart = Chart::new(vec![dataset])
                    .x_axis(
                        Axis::default()
                            .title("time")
                            .style(Style::default().fg(BORDER))
                            .labels(time_labels)
                            .bounds([0.0, max_time]),
                    )
                    .y_axis(
                        Axis::default()
                            .title("wpm")
                            .style(Style::default().fg(BORDER))
                            .labels(wpm_labels)
                            .bounds([wpm_range.start, wpm_range.end]),
                    );

                frame.render_widget(chart, chart_area);
            }
        }

        let footer_line = Line::from(vec![
            " q ".bold().fg(ACCENT),
            "Start again with same words ".dim(),
            " r ".bold().fg(ACCENT),
            "Start again with new words".dim(),
            " Esc ".bold().fg(ACCENT),
            "Quit app ".dim(),
        ]);
        frame.render_widget(
            Paragraph::new(footer_line),
            Rect {
                y: popup.y + popup.height - 1,
                height: 1,
                x: popup.x,
                width: popup.width,
            },
        );
    }
}

fn format_date(ts: u64) -> String {
    let days = (ts / 86_400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!("{} {} {}", MONTHS[(m - 1) as usize], d, y)
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
