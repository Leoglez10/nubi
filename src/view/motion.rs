//! The app chrome: an inset frame, a procedural cloud curtain and a motion preference.
use rune::{Color, El, col, el, row};

use super::{pal, plain};
use crate::{Screen, State};

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
        // Three soft layers are enough to read as a cloud and cheap to redraw every frame.
        let mut curtain = el().key("cloud-curtain").pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0));
        for (i, radius) in [120.0, 170.0, 230.0].into_iter().enumerate() {
            let size = radius * 2.0;
            curtain = curtain.child(
                el().absolute(-radius + 40.0 + i as f32 * 90.0, -radius * 0.55)
                    .size(size, size).radius(radius).fill(Color::WHITE)
                    .opacity(0.55 + 0.15 * i as f32),
            );
        }
        root = root.child(
            curtain.child(
                el().key("cloud-cover").w_frac(s.motion.cover).h_full().fill(Color::WHITE).instant_changes(),
            ),
        );
    }
    // Learning screens carry the control in their sidebar instead of floating it.
    if matches!(s.motion.displayed(s.screen), Screen::Splash | Screen::Setup | Screen::Profiles) {
        root = root.child(row().w_full().absolute(0.0, 0.0).justify_end().child(el().px(INSET).pt(INSET).child(toggle(s).h(34.0).px(14.0))));
    }
    root
}

pub fn toggle(s: &State) -> El {
    let label = if s.motion.system_reduced.get() {
        if s.reduced_motion { "Movimiento reducido: activado (sistema)" } else { "El sistema pide movimiento reducido" }
    } else if s.reduced_motion {
        "Movimiento reducido: activado"
    } else {
        "Movimiento: normal"
    };
    plain("motion-toggle", label, |s| s.reduced_motion = !s.reduced_motion)
}
