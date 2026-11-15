//! "¿Quién va a aprender hoy?": local profiles, no passwords.

use nubi::grading;
use rune::color::hex;
use rune::{El, TextEdit, col, el, row, spacer, text, text_input};

use super::{BODY, Mood, SMALL, STRONG, TITLE, disabled, nubi, nubi_face, pal, plain, primary};
use crate::{AVATAR_COLORS, Job, State};

/// A profile's avatar: its color with the first letter of its name.
pub fn avatar(name: &str, color: &str, size: f32) -> El {
    let p = pal();
    let initial: String = name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
    el().size(size, size).radius(size / 2.0).fill(hex(color).unwrap_or(p.sky)).border(1.0, p.text.alpha(0.12)).center()
        .child(text(initial).font(rune::tok::TextStyle::new(size * 0.4, 700.0)).color(p.text))
}

const CARD_W: f32 = 200.0;
const CARD_H: f32 = 196.0;

pub fn view(s: &State) -> El {
    let p = pal();
    let header = row().key("profiles-header").w_full().gap(10.0).center_y()
        .child(nubi_face(s, Mood::Neutral, 34.0))
        .child(col().gap(0.0)
            .child(text("Nubi").font(rune::tok::TextStyle::new(16.0, 700.0)).color(p.text))
            .child(text("Lab Cloud").font(rune::tok::TextStyle::new(12.0, 500.0)).color(p.muted)))
        .child(spacer())
        .child(plain("open-settings", "Ajustes", |s: &mut State| s.open_settings()).h(36.0).px(14.0));
    let welcome = row().w_full().gap(16.0).center_y()
        .child(nubi(s, 72.0))
        .child(col().gap(4.0)
            .child(text("¿Quién va a aprender hoy?").key("profiles-title").font(TITLE).color(p.text))
            .child(text("Elige tu perfil o crea uno nuevo.").font(BODY).color(p.muted)));
    let mut page = col().key("profiles").w_full().max_w(880.0).gap(28.0).padding(32.0).child(header).child(welcome);

    let Some(cards) = &s.profiles else {
        return frame(page.child(text("Cargando perfiles…").font(STRONG).color(p.muted)));
    };

    let mut grid = row().key("cards").w_full().gap(16.0).flex_wrap();
    for card in cards {
        let id = card.profile.id.clone();
        let level = grading::level(card.stars);
        grid = grid.child(
            col().key(format!("profile-{}", card.profile.id)).role("button").label(card.profile.name.clone()).focusable(true)
                .size(CARD_W, CARD_H).padding(20.0).gap(8.0).center().radius(12.0).fill(p.surface).border(1.0, p.border)
                .hover(p.text.alpha(0.03)).press_scale(0.98).focus_ring(2.0, 2.0, p.blue)
                .on_press(move |s: &mut State| s.send(Job::SelectProfile(id.clone())))
                .child(avatar(&card.profile.name, &card.profile.color, 64.0))
                .child(text(card.profile.name.clone()).font(rune::tok::TextStyle::new(16.0, 600.0)).color(p.text).pt(4.0))
                .child(text(format!("Nivel {} · {}", level.number, level.name)).font(SMALL).color(p.muted))
                .child(text(format!("★ {} estrellas", card.stars)).font(SMALL).color(p.orange_dark)),
        );
    }
    let creating = s.creating;
    grid = grid.child(
        col().key("new-profile").role("button").label("Nuevo perfil").focusable(true).checked(creating)
            .size(CARD_W, CARD_H).gap(10.0).center().radius(12.0)
            .fill(if creating { p.blue_light } else { p.bg }).border(1.0, if creating { p.blue } else { hex("#D0D5DD").unwrap_or(p.border) })
            .hover(p.text.alpha(0.03)).press_scale(0.98).focus_ring(2.0, 2.0, p.blue)
            .on_press(|s: &mut State| s.creating = !s.creating)
            .child(el().size(44.0, 44.0).radius(22.0).fill(p.blue_light).center()
                .child(text("+").font(rune::tok::TextStyle::new(24.0, 600.0)).color(p.blue)))
            .child(text("Nuevo perfil").font(STRONG).color(p.blue)),
    );
    page = page.child(grid);

    if creating {
        page = page.child(form(s));
    }
    if let Some(note) = &s.setup_note {
        page = page.child(text(note.clone()).font(SMALL).color(p.red_dark));
    }
    frame(page.child(
        row().key("local-note").w_full().gap(8.0).center_y()
            .child(el().size(6.0, 6.0).radius(3.0).fill(p.green))
            .child(text("Todo se guarda en esta computadora: sin cuentas ni contraseñas.").font(SMALL).color(p.muted).wrap()),
    ))
}

fn frame(page: El) -> El {
    super::page_scroll(row().w_full().justify_center().child(page)).fill(pal().bg)
}

fn form(s: &State) -> El {
    let p = pal();
    let mut swatches = row().gap(10.0);
    for (i, (color, name)) in AVATAR_COLORS.iter().enumerate() {
        let on = s.new_color == i;
        swatches = swatches.child(
            el().key(format!("color-{i}")).role("button").label(name.to_string()).checked(on).focusable(true)
                .size(36.0, 36.0).radius(18.0).fill(hex(color).unwrap_or(p.sky))
                .border(if on { 3.0 } else { 1.0 }, if on { p.blue } else { p.text.alpha(0.12) })
                .focus_ring(2.0, 2.0, p.blue)
                .on_press(move |s: &mut State| s.new_color = i),
        );
    }
    let name = s.new_name.text.trim().to_string();
    let create_button = if name.is_empty() {
        disabled("create-profile", "Crear y empezar")
    } else {
        primary("create-profile", "Crear y empezar", create)
    };
    let preview_name = if name.is_empty() { "?" } else { name.as_str() };
    col().key("profile-form").w_full().padding(24.0).gap(16.0).radius(12.0).fill(p.surface).border(1.0, p.border)
        .child(row().w_full().gap(14.0).center_y()
            .child(avatar(preview_name, AVATAR_COLORS[s.new_color.min(AVATAR_COLORS.len() - 1)].0, 48.0))
            .child(col().gap(2.0)
                .child(text("Nuevo perfil").font(rune::tok::TextStyle::new(16.0, 600.0)).color(p.text))
                .child(text("Tu nombre y un color para reconocerte.").font(SMALL).color(p.muted))))
        .child(col().w_full().gap(6.0)
            .child(text("Nombre").font(rune::tok::TextStyle::new(13.0, 600.0)).color(hex("#344054").unwrap_or(p.text)))
            .child(
                el().w_full().h(44.0).px(14.0).radius(8.0).fill(p.surface).border(1.0, hex("#D0D5DD").unwrap_or(p.border)).center_y().child(
                    text_input(&s.new_name, BODY).key("name").label("Tu nombre").single_line().grow(1.0)
                        .color(p.text).placeholder("Por ejemplo, Alex", hex("#98A2B3").unwrap_or(p.muted))
                        .on_edit(|s: &mut State| &mut s.new_name)
                        .on_submit(|s: &mut State, _| create(s)),
                ),
            ))
        .child(col().w_full().gap(6.0)
            .child(text("Color").font(rune::tok::TextStyle::new(13.0, 600.0)).color(hex("#344054").unwrap_or(p.text)))
            .child(swatches))
        .child(row().w_full().child(spacer()).child(create_button))
}

fn create(s: &mut State) {
    let name = s.new_name.text.trim().to_string();
    if name.is_empty() {
        return;
    }
    let color = AVATAR_COLORS[s.new_color.min(AVATAR_COLORS.len() - 1)].0.to_string();
    s.new_name = TextEdit::new("");
    s.send(Job::CreateProfile { name, color });
}
