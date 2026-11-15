//! The app chrome: an inset frame, and the short slide-in of a new screen.
use rune::{El, col, el};

use super::pal;
use crate::{Screen, State};

/// Space kept clear for the window's own controls and the screen edges.
const INSET: f32 = 9.0;
const SAFE_TOP: f32 = 26.0;
/// How far a new screen rises while it fades in, in points.
const RISE: f32 = 8.0;

pub fn wrap(s: &State, screen: El) -> El {
    let p = pal();
    // Learning screens animate only their content (see shell); the rest animate whole.
    let framed = matches!(s.motion.displayed(s.screen), Screen::Map | Screen::Class | Screen::Lesson | Screen::Lab)
        || (s.motion.displayed(s.screen) == Screen::Settings && s.profile.is_some());
    let screen = if framed { screen } else { entering(s, screen) };
    col().key("app").w_full().h_full().fill(p.bg)
        .padding(INSET)
        .pt(INSET + SAFE_TOP)
        .child(el().key("app-frame").w_full().grow(1.0).radius(16.0).clip_rounded(16.0).child(screen))
}

/// The new screen fades in while rising a few points into place; the clock drives it.
pub fn entering(s: &State, content: El) -> El {
    let k = s.motion.cover;
    content.opacity(1.0 - k).offset(0.0, RISE * k).instant_changes()
}
