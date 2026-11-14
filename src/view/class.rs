//! Teaching and review for a guided class.
use rune::{El, ScrollSpec, col, el, row, spacer, text};
use super::{BODY, H2, SMALL, STRONG, TITLE, disabled, pal, plain, primary};
use crate::State;

pub fn view(s: &State) -> El {
    let p = pal();
    let Some(teaching) = s.class_id.as_ref().and_then(|id| s.content.nodes.get(id)).and_then(|n| n.teaching.as_ref()) else { return el() };
    let back = if s.class_return.is_some() {
        plain(s, "class-back", "Volver a mi práctica", |s: &mut State| s.return_to_attempt())
    } else {
        plain(s, "class-back", "Volver al mapa", |s: &mut State| s.to_map())
    };
    let heading = if s.class_review { "Repaso de la clase" } else { "Primero, aprendamos" };
    let mut content = col().key("class-content").w_full().max_w(900.0).gap(14.0).padding(24.0)
        .child(text(format!("Clase {}: {}", teaching.number, teaching.title)).key("class-title").font(TITLE).color(p.text).wrap())
        .child(text(heading).font(H2).color(p.blue))
        .child(text(teaching.explanation.clone()).font(BODY).color(p.text).wrap())
        .child(el().key("class-visual").w_full().padding(18.0).radius(16.0).fill(p.blue_light)
            .child(text(teaching.visual.clone()).font(STRONG).color(p.blue_dark).wrap()))
        .child(text("Ejemplo resuelto").font(H2).color(p.text))
        .child(text(teaching.example.clone()).font(BODY).color(p.text).wrap())
        .child(text("Para recordar").font(H2).color(p.text))
        .child(text(teaching.review.clone()).font(BODY).color(p.text).wrap());
    if s.class_return.is_none() {
        let lab = if s.class_lab_ready() { primary(s, "class-lab", "Ir al laboratorio", |s: &mut State| s.start_class_lab()) }
            else { disabled("class-lab", "Completa los juegos primero") };
        content = content.child(row().w_full().gap(12.0).flex_wrap()
            .child(primary(s, "class-games", "Empezar juegos", |s: &mut State| s.start_class_lesson())).child(lab))
            .child(text("Completa los juegos para desbloquear el laboratorio.").font(SMALL).color(p.muted));
    }
    col().key("class").w_full().h_full().fill(p.bg)
        .child(row().w_full().gap(12.0).center_y().padding(16.0).fill(p.surface).child(back).child(spacer())
            .child(text("EXPLICACIÓN · JUEGOS · LABORATORIO · REPASO").font(SMALL).color(p.muted)))
        .child(el().w_full().grow(1.0).scroll(ScrollSpec { stick_bottom: false }).child(content))
}
