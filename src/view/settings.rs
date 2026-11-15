//! Ajustes: the signed-in profile (edit or delete it) and preferences that apply everywhere.
use rune::color::hex;
use rune::{El, ScrollSpec, col, el, row, spacer, text, text_input};

use super::profiles::avatar;
use super::{BODY, SMALL, STRONG, TITLE, button, disabled, pal, plain, primary};
use crate::{AVATAR_COLORS, Job, State};

pub fn view(s: &State) -> El {
    let p = pal();
    let mut page = col().key("settings").w_full().max_w(720.0).gap(20.0).padding(28.0)
        .child(row().w_full().center_y()
            .child(col().gap(4.0)
                .child(text("Ajustes").font(TITLE).color(p.text))
                .child(text(if s.profile.is_some() { "Tu perfil y tus preferencias, guardados en esta computadora." } else { "Preferencias para esta sesión." })
                    .font(SMALL).color(p.muted)))
            .child(spacer())
            .child(plain("settings-back", "Volver", |s: &mut State| s.close_settings())));
    if s.profile.is_some() {
        page = page.child(profile_section(s));
    }
    page = page.child(section("settings-accessibility", "Accesibilidad", motion_row(s)));
    if let Some(note) = &s.setup_note {
        page = page.child(text(note.clone()).font(SMALL).color(p.red_dark).wrap());
    }
    if s.profile.is_some() {
        page = page.child(danger_section(s));
    }
    col().w_full().h_full().fill(p.bg)
        .child(el().w_full().grow(1.0).scroll(ScrollSpec { stick_bottom: false }).child(row().w_full().justify_center().child(page)))
}

fn section(key: &str, title: &str, body: El) -> El {
    let p = pal();
    col().key(key.to_string()).w_full().radius(12.0).fill(p.surface).border(1.0, p.border).clip_rounded(12.0)
        .child(text(title.to_string()).font(rune::tok::TextStyle::new(14.0, 600.0)).color(p.text).w_full().px(20.0).py(14.0)
            .border_sides([0.0, 0.0, 1.0, 0.0], p.border))
        .child(body)
}

fn label(t: &str) -> El {
    text(t.to_string()).font(rune::tok::TextStyle::new(13.0, 600.0)).color(hex("#344054").unwrap_or(pal().text))
}

/// Name and color of the signed-in profile, saved with one button.
fn profile_section(s: &State) -> El {
    let p = pal();
    let Some(profile) = &s.profile else { return el() };
    let name = s.edit_name.text.trim().to_string();
    let color = AVATAR_COLORS[s.edit_color.min(AVATAR_COLORS.len() - 1)].0;
    let changed = name != profile.name || color != profile.color;
    let mut swatches = row().gap(10.0);
    for (i, (c, cname)) in AVATAR_COLORS.iter().enumerate() {
        let on = s.edit_color == i;
        swatches = swatches.child(
            el().key(format!("edit-color-{i}")).role("button").label(cname.to_string()).checked(on).focusable(true)
                .size(32.0, 32.0).radius(16.0).fill(hex(c).unwrap_or(p.sky))
                .border(if on { 3.0 } else { 1.0 }, if on { p.blue } else { p.text.alpha(0.12) })
                .focus_ring(2.0, 2.0, p.blue)
                .on_press(move |s: &mut State| s.edit_color = i),
        );
    }
    let save = if name.is_empty() || !changed { disabled("save-profile", "Guardar cambios") } else {
        primary("save-profile", "Guardar cambios", |s: &mut State| {
            let Some(mut profile) = s.profile.clone() else { return };
            profile.name = s.edit_name.text.trim().to_string();
            profile.color = AVATAR_COLORS[s.edit_color.min(AVATAR_COLORS.len() - 1)].0.to_string();
            s.send(Job::UpdateProfile(profile));
        })
    };
    let body = col().w_full().gap(16.0).padding(20.0)
        .child(row().gap(14.0).center_y()
            .child(avatar(if name.is_empty() { "?" } else { &name }, color, 48.0))
            .child(col().gap(2.0)
                .child(text(profile.name.clone()).font(STRONG).color(p.text))
                .child(text("Perfil local").font(SMALL).color(p.muted))))
        .child(col().w_full().gap(6.0).child(label("Nombre"))
            .child(el().w_full().h(44.0).px(14.0).radius(8.0).fill(p.surface).border(1.0, hex("#D0D5DD").unwrap_or(p.border)).center_y()
                .child(text_input(&s.edit_name, BODY).key("edit-name").label("Nombre del perfil").single_line().grow(1.0)
                    .color(p.text).on_edit(|s: &mut State| &mut s.edit_name))))
        .child(col().w_full().gap(6.0).child(label("Color")).child(swatches))
        .child(row().w_full().child(spacer()).child(save));
    section("settings-profile", "Perfil", body)
}

fn motion_row(s: &State) -> El {
    let p = pal();
    let system = s.motion.system_reduced.get();
    let on = s.reduced_motion || system;
    let detail = if system {
        "Tu sistema ya pide movimiento reducido, así que está activo en toda la app."
    } else {
        "Quita las animaciones al cambiar de pantalla y al pulsar botones."
    };
    row().key("setting-motion").w_full().gap(16.0).center_y().px(20.0).py(16.0)
        .child(col().grow(1.0).gap(4.0)
            .child(text("Reducir movimiento").font(STRONG).color(p.text))
            .child(text(detail).font(rune::tok::TextStyle::new(13.0, 400.0)).color(p.muted).wrap()))
        .child(switch(on, !system))
}

/// Deleting asks once more, naming what is lost.
fn danger_section(s: &State) -> El {
    let p = pal();
    let name = s.profile.as_ref().map_or(String::new(), |p| p.name.clone());
    let body = if s.confirm_delete {
        col().key("delete-confirm").w_full().gap(12.0).padding(20.0).fill(p.red_light)
            .child(text(format!("¿Eliminar a {name}? Se borrarán su perfil y todo su progreso. No se puede deshacer.")).font(STRONG).color(p.red_dark).wrap())
            .child(row().gap(10.0)
                .child(plain("cancel-delete", "Cancelar", |s: &mut State| s.confirm_delete = false))
                .child(button("confirm-delete", "Sí, eliminar", p.red_dark, p.red_dark, rune::Color::WHITE, |s: &mut State| {
                    if let Some(profile) = &s.profile { s.send(Job::DeleteProfile(profile.id.clone())); }
                    s.confirm_delete = false;
                })))
    } else {
        row().w_full().gap(16.0).center_y().padding(20.0)
            .child(col().grow(1.0).gap(4.0)
                .child(text("Eliminar perfil").font(STRONG).color(p.text))
                .child(text("Borra este perfil y su progreso de esta computadora.").font(rune::tok::TextStyle::new(13.0, 400.0)).color(p.muted).wrap()))
            .child(plain("delete-profile", "Eliminar perfil", |s: &mut State| s.confirm_delete = true))
    };
    section("settings-danger", "Zona de peligro", body)
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
        sw = sw.focusable(true).focus_ring(2.0, 2.0, p.blue).on_press(|s: &mut State| { let on = !s.reduced_motion; s.set_reduced_motion(on) });
    }
    sw
}
