use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::fx::Glitch;
use tachyonfx::{Effect, EffectRenderer, Interpolation, IntoEffect, Motion, fx};

use crate::app::AppState;

/// Cap on the per-frame delta so a stalled frame doesn't jump an animation.
const MAX_DT: Duration = Duration::from_millis(100);
/// The dim backdrop color used behind popups / screens.
const BG: Color = Color::Indexed(236);

/// Per-screen animation state.
///
/// - `entrance` fires once when a screen mounts (kept afterwards so it isn't rebuilt).
/// - `ambient` loops as long as a screen stays mounted.
/// - `flash` is a one-shot pulse triggered by gameplay events and dropped once done.
/// - `selector_switch` / `test_switch` are transient requests (set from input) whose
///   effects need layout rects, so they are materialized on the next frame in `view`.
pub(crate) struct Fx {
    screen: AppState,
    entrance: Option<Effect>,
    ambient: Option<Effect>,
    flash: Option<Effect>,
    selector_switch: bool,
    test_switch: bool,
    tick: Duration,
}

impl Fx {
    pub(crate) fn new() -> Self {
        Self {
            screen: AppState::Onboarding,
            entrance: None,
            ambient: None,
            flash: None,
            selector_switch: false,
            test_switch: false,
            tick: Duration::from_millis(33),
        }
    }

    pub(crate) fn set_tick(&mut self, dt: Duration) {
        self.tick = dt.min(MAX_DT);
    }

    /// Rebuild entrance/ambient effects whenever the active screen changes.
    pub(crate) fn update(&mut self, state: AppState) {
        if self.screen != state {
            self.screen = state;
            self.entrance = None;
            self.ambient = None;
            self.selector_switch = false;
            self.test_switch = false;
        }
    }

    pub(crate) fn entrance_if_none(&mut self, make: impl FnOnce() -> Effect) {
        if self.entrance.is_none() {
            self.entrance = Some(make());
        }
    }

    pub(crate) fn ambient_if_none(&mut self, make: impl FnOnce() -> Effect) {
        if self.ambient.is_none() {
            self.ambient = Some(make());
        }
    }

    pub(crate) fn flash(&mut self, make: impl FnOnce() -> Effect) {
        self.flash = Some(make());
    }

    /// Request a transition sweep across just the mode/option selector row.
    pub(crate) fn flag_selector_switch(&mut self) {
        self.selector_switch = true;
    }

    /// Request a full transition: selector sweep plus a reveal of the new text.
    pub(crate) fn flag_test_switch(&mut self) {
        self.test_switch = true;
    }

    /// Materialize any pending switch requests into a flash effect. Called each frame
    /// from the typing view, which has the layout rects the effects need.
    pub(crate) fn consume_transitions(&mut self, selector: Rect, para: Rect) {
        if self.selector_switch {
            self.selector_switch = false;
            self.flash = Some(selector_sweep(selector));
        }
        if self.test_switch {
            self.test_switch = false;
            self.flash = Some(test_transition(selector, para));
        }
    }

    pub(crate) fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(effect) = &mut self.flash {
            if effect.running() {
                frame.render_effect(effect, area, self.tick);
            } else {
                self.flash = None;
            }
        }
        if let Some(effect) = &mut self.entrance
            && effect.running()
        {
            frame.render_effect(effect, area, self.tick);
        }
        if let Some(effect) = &mut self.ambient {
            frame.render_effect(effect, area, self.tick);
        }
    }
}

pub(crate) fn typing_entrance(area: Rect) -> Effect {
    fx::fade_from_fg(Color::Black, (450, Interpolation::CubicOut)).with_area(area)
}

pub(crate) fn typing_ambient(area: Rect) -> Effect {
    fx::repeating(fx::ping_pong(fx::lighten_fg(0.035, 1500))).with_area(area)
}

pub(crate) fn onboarding_entrance(popup: Rect) -> Effect {
    fx::coalesce((500, Interpolation::QuadOut)).with_area(popup)
}

pub(crate) fn onboarding_ambient(input: Rect) -> Effect {
    fx::repeating(fx::ping_pong(fx::lighten_fg(0.12, 900))).with_area(input)
}

pub(crate) fn results_entrance(popup: Rect) -> Effect {
    fx::sequence(&[
        fx::sweep_in(Motion::UpToDown, 6, 0, BG, (400, Interpolation::QuadOut)),
        fx::fade_from_fg(Color::Cyan, (300, Interpolation::QuadOut)),
    ])
    .with_area(popup)
}

pub(crate) fn results_ambient(head: Rect) -> Effect {
    let glitch = Glitch::builder()
        .cell_glitch_ratio(0.04)
        .action_ms(80..240)
        .action_start_delay_ms(1200..4000)
        .build()
        .into_effect();
    fx::repeating(glitch).with_area(head)
}

pub(crate) fn scores_entrance(area: Rect) -> Effect {
    fx::fade_from_fg(Color::Black, (350, Interpolation::CubicOut)).with_area(area)
}

pub(crate) fn scores_ambient(summary: Rect) -> Effect {
    fx::repeating(fx::ping_pong(fx::lighten_fg(0.07, 1300))).with_area(summary)
}

/// Quick bright blink as feedback for a mistype.
pub(crate) fn wrong_key_flash() -> Effect {
    fx::lighten_fg(0.15, 120)
}

/// Horizontal sweep across the mode/option selector row after a selection change.
fn selector_sweep(selector: Rect) -> Effect {
    fx::sweep_in(Motion::LeftToRight, 5, 0, BG, (220, Interpolation::QuadOut)).with_area(selector)
}

/// Transition when the test itself changes (mode flip or new word count): the
/// selector row sweeps while the new text reveal-coalesces in.
fn test_transition(selector: Rect, para: Rect) -> Effect {
    fx::parallel(&[
        fx::sweep_in(Motion::LeftToRight, 5, 0, BG, (220, Interpolation::QuadOut))
            .with_area(selector),
        fx::coalesce((400, Interpolation::QuadOut)).with_area(para),
    ])
}
