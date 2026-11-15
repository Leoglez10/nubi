//! Inicio: where the student stands, the class to continue and the module at a glance.
//! Every number comes from saved progress.
use labcloud::grading::{self, Stop, stop_status};
use rune::color::hex;
use rune::{El, col, el, row, spacer, text};

use super::{BODY, H2, SMALL, STRONG, TITLE, nubi, pal, primary, progress_bar, stars};
use crate::State;

pub fn view(s: &State) -> El {
    let p = pal();
    let classes = s.content.classes();
    let total_classes = classes.len();
    let done = classes.iter().filter(|(_, t)| s.progress.completed(&t.lab)).count();
    let name = s.profile.as_ref().map_or("Hola".to_string(), |pr| format!("Hola, {}", pr.name));
    let current = classes.iter().find(|(_, t)| !s.progress.completed(&t.lab)).or(classes.last());

    let mut greeting = row().key("greeting").w_full().gap(16.0).center_y().flex_wrap()
        .child(col().grow(1.0).gap(4.0)
            .child(text(name).font(TITLE).color(p.text))
            .child(text(format!("Llevas {done} de {total_classes} clases del módulo de S3.")).font(BODY).color(p.muted)));
    if let Some((_, t)) = current {
        let tip = if done == total_classes { "¡Terminaste el módulo! Repasa para sacar 3 estrellas.".to_string() } else { format!("Hoy toca {}.", t.title.to_lowercase()) };
        greeting = greeting.child(row().key("map-nubi").gap(12.0).center_y().py(8.0).pl(8.0).pr(16.0).radius(12.0).fill(p.surface).border(1.0, p.border)
            .child(nubi(s, 56.0))
            .child(col().gap(2.0)
                .child(text("Nubi").font(rune::tok::TextStyle::new(12.0, 600.0)).color(p.muted))
                .child(text(tip).font(rune::tok::TextStyle::new(14.0, 500.0)).color(p.text))));
    }

    let stops = s.order().len() as u32;
    let total = s.progress.total_stars();
    let level = grading::level(total);
    let mut kpis = row().key("kpis").w_full().gap(16.0).flex_wrap();
    for (label, value, sub) in [
        ("Clases completadas", format!("{done} / {total_classes}"), "Módulo de S3".to_string()),
        ("Estrellas", total.to_string(), format!("de {} posibles", stops * 3)),
        ("Prácticas", classes.iter().filter(|(n, _)| s.progress.completed(&n.id)).count().to_string(), format!("de {total_classes}")),
        ("Nivel", level.number.to_string(), level.name.to_string()),
    ] {
        kpis = kpis.child(card().grow(1.0).min_w(180.0).gap(6.0).padding(18.0)
            .child(text(label).font(rune::tok::TextStyle::new(13.0, 500.0)).color(p.muted))
            .child(text(value).font(rune::tok::TextStyle::new(28.0, 700.0)).color(p.text))
            .child(text(sub).font(SMALL).color(hex("#667085").unwrap_or(p.muted))));
    }

    let mut page = col().w_full().max_w(1120.0).gap(24.0).padding(28.0).child(greeting).child(kpis);
    if let Some((node, t)) = current {
        let finished = [s.progress.completed(&node.id), s.progress.completed(&t.lab)].iter().filter(|d| **d).count();
        let id = node.id.clone();
        let label = if finished == 0 { "Empezar clase" } else { "Continuar clase" };
        page = page.child(card().key("continue").w_full().gap(10.0).padding(22.0)
            .child(text(if done == total_classes { "REPASA TU ÚLTIMA CLASE" } else { "CONTINÚA DONDE LO DEJASTE" }).font(rune::tok::TextStyle::new(12.0, 600.0)).color(p.blue))
            .child(row().w_full().gap(24.0).center_y().flex_wrap()
                .child(col().grow(1.0).min_w(300.0).gap(8.0)
                    .child(text(format!("Clase {} · {}", t.number, t.title)).font(H2).color(p.text))
                    .child(text(t.explanation.clone()).font(rune::tok::TextStyle::new(14.0, 400.0)).color(p.muted).wrap())
                    .child(row().w_full().gap(12.0).center_y()
                        .child(el().grow(1.0).child(progress_bar("continue-progress", finished as f32 / 2.0, p.blue, 8.0)))
                        .child(text(format!("{finished} de 2 partes")).font(SMALL).color(p.muted))))
                .child(primary("continue-class", label, move |s: &mut State| s.open_class(&id)))));
    }
    page = page.child(module(s));
    super::page_scroll(row().w_full().justify_center().child(page))
}

fn card() -> El {
    let p = pal();
    col().radius(12.0).fill(p.surface).border(1.0, p.border)
}

/// One row per class with its two parts and where it stands.
fn module(s: &State) -> El {
    let p = pal();
    let order = s.order();
    let classes = s.content.classes();
    let mut table = card().key("module").w_full().clip_rounded(12.0)
        .child(row().w_full().px(20.0).py(16.0).center_y().border_sides([0.0, 0.0, 1.0, 0.0], p.border)
            .child(text("Módulo 1 · Fundamentos de S3").font(rune::tok::TextStyle::new(16.0, 600.0)).color(p.text))
            .child(spacer())
            .child(text(format!("{} clases", classes.len())).font(SMALL).color(p.muted)));
    for (node, t) in &classes {
        let status = stop_status(&order, &node.id, |id| s.progress.nodes.get(id).map(|r| r.best_stars));
        let lesson = s.progress.nodes.get(&node.id).map_or(0, |r| r.best_stars);
        let lab = s.progress.nodes.get(&t.lab).map_or(0, |r| r.best_stars);
        let (badge, bg, fg, num_bg, num_fg) = if s.progress.completed(&t.lab) {
            ("Completada", p.green_light, p.green_dark, p.green_light, p.green_dark)
        } else if status == Stop::Locked {
            ("Bloqueada", p.locked, p.muted, p.locked, p.muted)
        } else {
            ("En curso", p.blue_light, p.blue_dark, p.blue, rune::Color::WHITE)
        };
        let title = format!("Clase {}: {}", t.number, t.title);
        let detail = if status == Stop::Locked { "Disponible al terminar la clase anterior".to_string() } else { format!("Práctica {} · Laboratorio {}", stars(lesson), stars(lab)) };
        let mut line = row().key(format!("class-{}", t.number)).role("button").label(format!("{title}, {badge}"))
            .w_full().gap(16.0).center_y().px(20.0).py(14.0).border_sides([0.0, 0.0, 1.0, 0.0], hex("#F2F4F7").unwrap_or(p.border))
            .child(el().size(32.0, 32.0).radius(8.0).fill(num_bg).center().child(text(t.number.to_string()).font(STRONG).color(num_fg)))
            .child(col().grow(1.0).gap(2.0)
                .child(text(t.title.clone()).font(STRONG).color(p.text))
                .child(text(detail).font(SMALL).color(hex("#667085").unwrap_or(p.muted))))
            .child(el().w(104.0).h(24.0).radius(12.0).fill(bg).center().child(text(badge).font(rune::tok::TextStyle::new(12.0, 500.0)).color(fg)));
        if status != Stop::Locked {
            let id = node.id.clone();
            line = line.focusable(true).hover(p.text.alpha(0.03)).focus_ring(2.0, -2.0, p.blue)
                .on_press(move |s: &mut State| s.open_class(&id));
        }
        table = table.child(line);
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Screen;

    fn home(record: &[&str]) -> rune::Runtime<State> {
        let mut s = State {
            content: std::sync::Arc::new(labcloud::course::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap()),
            screen: Screen::Map,
            ..State::default()
        };
        for id in record { s.progress.record(id, 0); }
        let mut rt = rune::Runtime::new(s, crate::view::view, (1280.0, 800.0), rune::Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        rt
    }

    fn click(rt: &mut rune::Runtime<State>, key: &str) {
        let (x, y) = rt.find(key).unwrap_or_else(|| panic!("missing {key}"));
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
    }

    #[test]
    fn home_shows_progress_nubi_and_the_class_to_continue() {
        let mut rt = home(&[]);
        for key in ["greeting", "map-nubi", "kpis", "continue", "module", "class-1", "class-2"] {
            assert!(rt.find(key).is_some(), "missing {key}");
        }
        click(&mut rt, "continue-class");
        assert_eq!(rt.state.screen, Screen::Class);
        assert_eq!(rt.state.class_id.as_deref(), Some("s3-buckets"));
        assert!(rt.state.lesson.id.is_empty() && rt.state.lab.id.is_empty(), "a class opens teaching, not a practice");
    }

    #[test]
    fn a_locked_class_row_does_nothing_and_an_open_one_opens_its_class() {
        let mut rt = home(&[]);
        click(&mut rt, "class-2");
        assert_eq!(rt.state.screen, Screen::Map, "class 2 is locked until class 1's lab is done");

        let mut rt = home(&["s3-buckets", "lab-primer-bucket"]);
        click(&mut rt, "class-2");
        assert_eq!(rt.state.class_id.as_deref(), Some("s3-objetos"));
    }
}
