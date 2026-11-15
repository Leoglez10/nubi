//! The app chrome: an inset frame and a fade between screens.
use rune::{El, col, el};

use super::pal;
use crate::State;

/// Space kept clear for the window's own controls and the screen edges.
const INSET: f32 = 9.0;
const SAFE_TOP: f32 = 26.0;

pub fn wrap(s: &State, screen: El) -> El {
    let p = pal();
    let mut root = col().key("app").w_full().h_full().fill(p.bg)
        .padding(INSET)
        .pt(INSET + SAFE_TOP)
        .child(el().key("app-frame").w_full().grow(1.0).radius(16.0).clip_rounded(16.0).child(screen));
    if s.motion.cover > 0.0 {
        // A fade through the background: the old screen fades out, the new one fades in.
        root = root.child(
            el().key("screen-fade").pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .child(el().key("screen-fade-cover").w_full().h_full().fill(p.bg).opacity(s.motion.cover).instant_changes()),
        );
    }
    root
}
