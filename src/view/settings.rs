//! Ajustes: preferences that apply to every screen. Today, reduced motion.
use rune::color::hex;
use rune::{El, col, el, row, spacer, text};

use super::{SMALL, STRONG, TITLE, pal, plain};
use crate::State;

pub fn view(s: &State) -> El {
    let p = pal();
    let system = s.motion.system_reduced.get();
    let on = s.reduced_motion || system;
    let detail = if system {
        "Tu sistema ya pide movimiento reducido, así que está activo en toda la app."
    } else {
        "Quita las animaciones al cambiar de pantalla y al pulsar botones."
    };
    let motion = row().key("setting-motion").w_full().gap(16.0).center_y().px(20.0).py(16.0)
        .child(col().grow(1.0).gap(4.0)
            .child(text("Reducir movimiento").font(STRONG).color(p.text))
            .child(text(detail).font(rune::tok::TextStyle::new(13.0, 400.0)).color(p.muted).wrap()))
        .child(switch(on, !system));
    let card = col().key("settings-accessibility").w_full().radius(12.0).fill(p.surface).border(1.0, p.border).clip_rounded(12.0)
        .child(text("Accesibilidad").font(rune::tok::TextStyle::new(14.0, 600.0)).color(p.text).w_full().px(20.0).py(14.0)
            .border_sides([0.0, 0.0, 1.0, 0.0], p.border))
        .child(motion);
    let page = col().key("settings").w_full().max_w(720.0).gap(20.0).padding(28.0)
        .child(row().w_full().center_y()
            .child(col().gap(4.0)
                .child(text("Ajustes").font(TITLE).color(p.text))
                .child(text("Se guardan en esta sesión y aplican a todas las pantallas.").font(SMALL).color(p.muted)))
            .child(spacer())
            .child(plain("settings-back", "Volver", |s: &mut State| s.close_settings())))
        .child(card);
    col().w_full().h_full().fill(p.bg).child(row().w_full().justify_center().child(page))
}

/// An on/off switch; `enabled` false shows it locked on (the system decided).
fn switch(on: bool, enabled: bool) -> El {
    let p = pal();
    let track = if on { p.blue } else { hex("#D0D5DD").unwrap_or(p.border) };
    let knob = el().size(18.0, 18.0).radius(9.0).fill(rune::Color::WHITE);
    let mut sw = row().key("motion-toggle").role("switch").label("Reducir movimiento").checked(on)
        .w(40.0).h(22.0).px(2.0).radius(11.0).fill(track).center_y()
        .opacity(if enabled { 1.0 } else { 0.5 });
    sw = if on { sw.justify_end() } else { sw.justify_start() }.child(knob);
    if enabled {
        sw = sw.focusable(true).focus_ring(2.0, 2.0, p.blue).on_press(|s: &mut State| s.reduced_motion = !s.reduced_motion);
    }
    sw
}
