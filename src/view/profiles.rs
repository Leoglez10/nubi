//! "¿Quién va a aprender hoy?": local profiles, no passwords.

use labcloud::grading;
use rune::color::hex;
use rune::{El, TextEdit, col, el, row, text, text_input};

use super::{BODY, H2, SMALL, STRONG, TITLE, disabled, pal, primary};
use crate::{AVATAR_COLORS, Job, State};

/// A profile's avatar: its color with the first letter of its name.
pub fn avatar(name: &str, color: &str, size: f32) -> El {
    let p = pal();
    let initial: String = name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
    el().size(size, size).radius(size / 2.0).fill(hex(color).unwrap_or(p.sky)).border(2.0, p.text).center()
        .child(text(initial).font(rune::tok::TextStyle::new(size * 0.42, 800.0)).color(p.text))
}

pub fn view(s: &State) -> El {
    let p = pal();
    let mut page = col().key("profiles").w_full().h_full().fill(p.bg).padding(40.0).gap(28.0).center()
        .child(text("labcloud").font(H2).color(p.blue))
        .child(text("¿Quién va a aprender hoy?").key("profiles-title").font(TITLE).color(p.text));

    let Some(cards) = &s.profiles else {
        return page.child(text("Cargando perfiles…").font(STRONG).color(p.muted));
    };

    let mut grid = row().key("cards").gap(20.0).flex_wrap().center();
    for card in cards {
        let id = card.profile.id.clone();
        let level = grading::level(card.stars);
        grid = grid.child(
            col().key(format!("profile-{}", card.profile.id)).role("button").label(card.profile.name.clone()).focusable(true)
                .w(190.0).padding(22.0).gap(10.0).center().radius(24.0).fill(p.surface).border(2.0, p.border).shadow(6.0, 0.0, p.border)
                .on_press(move |s: &mut State| s.send(Job::SelectProfile(id.clone())))
                .child(avatar(&card.profile.name, &card.profile.color, 84.0))
                .child(text(card.profile.name.clone()).font(H2).color(p.text))
                .child(text(format!("Nivel {} · {}", level.number, level.name)).font(SMALL).color(p.muted))
                .child(text(format!("★ {} estrellas", card.stars)).font(SMALL).color(p.orange_dark)),
        );
    }
    grid = grid.child(
        col().key("new-profile").role("button").label("Nuevo perfil").focusable(true)
            .w(190.0).h(238.0).gap(10.0).center().radius(24.0).border(3.0, hex("#8CC0F5").unwrap_or(p.blue))
            .on_press(|s: &mut State| s.creating = !s.creating)
            .child(text("+").font(rune::tok::TextStyle::new(44.0, 800.0)).color(p.blue_dark))
            .child(text("Nuevo perfil").font(H2).color(p.blue_dark)),
    );
    page = page.child(grid);

    if s.creating {
        page = page.child(form(s));
    }
    if let Some(note) = &s.setup_note {
        page = page.child(text(note.clone()).font(SMALL).color(p.red_dark));
    }
    page.child(
        row().key("local-note").w_full().max_w(620.0).gap(14.0).center_y().padding(18.0).radius(20.0)
            .fill(p.blue_light).border(2.0, hex("#8CC0F5").unwrap_or(p.blue))
            .child(
                col().grow(1.0).gap(4.0)
                    .child(text("Todo se guarda en esta computadora").font(STRONG).color(p.blue_dark))
                    .child(text("Sin cuentas ni contraseñas. Tu progreso vive en una carpeta local.").font(SMALL).color(p.text).wrap()),
            ),
    )
}

fn form(s: &State) -> El {
    let p = pal();
    let mut swatches = row().gap(12.0);
    for (i, (color, name)) in AVATAR_COLORS.iter().enumerate() {
        let on = s.new_color == i;
        swatches = swatches.child(
            el().key(format!("color-{i}")).role("button").label(name.to_string()).checked(on).focusable(true)
                .size(52.0, 52.0).radius(26.0).fill(hex(color).unwrap_or(p.sky))
                .border(if on { 4.0 } else { 2.0 }, if on { p.blue } else { p.border })
                .on_press(move |s: &mut State| s.new_color = i),
        );
    }
    let name = s.new_name.text.trim().to_string();
    let create_button = if name.is_empty() {
        disabled("create-profile", "Crear y empezar")
    } else {
        primary(s, "create-profile", "Crear y empezar", create)
    };
    col().key("profile-form").w_full().max_w(560.0).padding(24.0).gap(14.0).radius(24.0).fill(p.surface).border(2.0, p.border)
        .child(text("¿Cómo te llamas?").font(STRONG).color(p.text))
        .child(
            el().w_full().h(52.0).px(16.0).radius(14.0).fill(p.bg).border(2.0, p.border).center_y().child(
                text_input(&s.new_name, BODY).key("name").label("Tu nombre").single_line().grow(1.0)
                    .color(p.text).placeholder("Tu nombre", p.muted)
                    .on_edit(|s: &mut State| &mut s.new_name)
                    .on_submit(|s: &mut State, _| create(s)),
            ),
        )
        .child(text("Elige el color de tu avatar").font(STRONG).color(p.text))
        .child(swatches)
        .child(create_button)
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
