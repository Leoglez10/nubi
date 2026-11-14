//! A procedural cloud curtain and an always-available motion preference.
use rune::{Color, El, col, el, row};

use super::{pal, plain};
use crate::State;

pub fn wrap(s: &State, screen: El) -> El {
    let mut root = col().w_full().h_full().child(el().w_full().grow(1.0).child(screen));
    if s.motion.cover > 0.0 {
        let mut edge = col().pin(None, Some(0.0), Some(-60.0), Some(0.0)).w(140.0);
        for _ in 0..8 {
            edge = edge.child(el().w_full().grow(1.0).radius(70.0).fill(Color::WHITE));
        }
        let curtain = el().key("cloud-curtain").pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
            .instant_changes().transition(rune::Transition::none())
            .child(el().key("cloud-cover").w_frac(s.motion.cover).h_full().fill(Color::WHITE).child(edge));
        root = root.child(curtain);
    }
    let label = if s.motion.system_reduced.get() {
        if s.reduced_motion { "Movimiento reducido: activado (sistema)" } else { "Movimiento reducido: sistema" }
    } else if s.reduced_motion {
        "Movimiento reducido: activado"
    } else {
        "Movimiento reducido: desactivado"
    };
    let p = pal();
    root.child(row().w_full().h(64.0).px(12.0).center_y().justify_end().fill(p.surface)
        .child(plain(s, "motion-toggle", label, |s| s.reduced_motion = !s.reduced_motion)))
}
