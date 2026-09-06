use ratatui::style::{Color, Modifier, Style};

pub(crate) const FG_TODO: Color = Color::Indexed(244);
pub(crate) const FG_DONE: Color = Color::Indexed(40);
pub(crate) const FG_WARN: Color = Color::Indexed(226);
pub(crate) const FG_ERR: Color = Color::Indexed(210);
pub(crate) const ACCENT: Color = Color::Indexed(51);
pub(crate) const EMPHASIS: Color = Color::Indexed(231);
pub(crate) const MUTED: Color = Color::Indexed(244);
pub(crate) const BORDER: Color = Color::Indexed(245);

pub(crate) fn wpm_tier(wpm: f64) -> Style {
    let color = if wpm >= 60.0 {
        FG_DONE
    } else if wpm >= 40.0 {
        FG_WARN
    } else {
        FG_ERR
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub(crate) fn acc_tier(acc: f64) -> Style {
    let color = if acc >= 95.0 {
        FG_DONE
    } else if acc >= 90.0 {
        FG_WARN
    } else {
        FG_ERR
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub(crate) fn wpm_badge(wpm: f64) -> &'static str {
    if wpm >= 100.0 {
        "spectral"
    } else if wpm >= 80.0 {
        "hyper"
    } else if wpm >= 60.0 {
        "blazing"
    } else if wpm >= 40.0 {
        "smooth"
    } else if wpm >= 25.0 {
        "steady"
    } else {
        "warming up"
    }
}
