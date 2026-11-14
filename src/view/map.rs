//! The home screen: a horizontal map in the clouds. Units are cloud islands, stops are cloud
//! platforms joined by a dotted flight path, and Nubi waits on the next stop.

use labcloud::course::NodeKind;
use labcloud::grading::{self, Stop, stop_status};
use rune::{Color, El, col, el, row, spacer, text};

use super::profiles::avatar;
use super::{BODY, H2, SMALL, STRONG, TITLE, bubble, disabled, good, nubi, pal, picture, plural, primary, progress_bar, reward, stars};
use crate::{Screen, State};

const STOP_W: f32 = 120.0;
const STOP_H: f32 = 96.0;
const STEP: f32 = 190.0;
const ISLAND_GAP: f32 = 110.0;
const MAP_H: f32 = 430.0;

/// Where stop `i` (of unit `u`) sits on the map.
fn place(i: usize, u: usize) -> (f32, f32) {
    (90.0 + i as f32 * STEP + u as f32 * ISLAND_GAP, if i.is_multiple_of(2) { 230.0 } else { 140.0 })
}

pub fn view(s: &State) -> El {
    let p = pal();
    let order = s.order();
    let status = |id: &str| stop_status(&order, id, |x| s.progress.nodes.get(x).map(|r| r.best_stars));

    // Header: logo, level, profile.
    let total = s.progress.total_stars();
    let level = grading::level(total);
    let (bar, next_text) = match level.next_at {
        Some(next) => (
            (total - level.from) as f32 / (next - level.from) as f32,
            format!("{total} / {next} ★ para el nivel {}", level.number + 1),
        ),
        None => (1.0, format!("{total} ★ · nivel máximo")),
    };
    let mut header = row().key("map-header").w_full().gap(18.0).center_y().px(24.0).py(12.0).fill(p.surface).border(2.0, p.border)
        .child(text("labcloud").font(TITLE).color(p.blue))
        .child(spacer())
        .child(
            row().key("level").gap(12.0).center_y().px(8.0).py(6.0).radius(30.0).fill(p.surface).border(2.0, p.border)
                .child(el().size(40.0, 40.0).radius(20.0).fill(p.blue).center().child(text(level.number.to_string()).font(H2).color(Color::WHITE)))
                .child(
                    col().w(180.0).gap(4.0).padding(4.0)
                        .child(text(format!("Nivel {} · {}", level.number, level.name)).font(STRONG).color(p.text))
                        .child(progress_bar("level-bar", bar, p.orange, 8.0))
                        .child(text(next_text).font(SMALL).color(p.muted)),
                ),
        );
    if let Some(profile) = &s.profile {
        header = header.child(
            el().key("switch-profile").role("button").label(format!("Cambiar de perfil ({})", profile.name)).focusable(true)
                .on_press(|s: &mut State| {
                    s.creating = false;
                    s.screen = Screen::Profiles;
                })
                .child(avatar(&profile.name, &profile.color, 48.0)),
        );
    }

    // The map canvas.
    let art = s.art.as_ref();
    let mut canvas = el().key("map-canvas").h(MAP_H);
    let mut centers: Vec<(f32, f32, bool)> = Vec::new();
    // Stops go on top of the flight path, so they are added last.
    let mut stops: Vec<El> = Vec::new();
    let mut i = 0;
    let mut right_edge = 0.0f32;
    for (u, unit) in s.content.course.units.iter().enumerate() {
        if unit.nodes.is_empty() {
            continue;
        }
        let first = place(i, u).0;
        let last = place(i + unit.nodes.len() - 1, u).0;
        let (left, right) = (first - 60.0, last + STOP_W + 60.0);
        right_edge = right_edge.max(right);
        let open = unit.nodes.iter().any(|id| status(id) != Stop::Locked);
        canvas = canvas
            .child(el().absolute(left, 320.0).size(right - left, 84.0).radius(42.0).fill(p.surface).shadow(12.0, 0.0, p.blue_light))
            .child(
                el().key(format!("island-{}", unit.id)).absolute(left + 16.0, 40.0).px(16.0).py(8.0).radius(14.0)
                    .fill(if open { p.blue } else { p.locked })
                    .shadow(4.0, 0.0, if open { p.blue_dark } else { p.border })
                    .child(text(unit.title.clone()).font(STRONG).color(if open { Color::WHITE } else { p.muted })),
            );
        for id in &unit.nodes {
            let (x, y) = place(i, u);
            let st = status(id);
            centers.push((x + STOP_W / 2.0, y + STOP_H / 2.0 + 8.0, matches!(st, Stop::Done(_))));
            stops.push(stop(s, i, id, x, y, st, art));
            i += 1;
        }
    }
    // "Coming soon" island: honest about what is not built yet.
    let soon = right_edge + ISLAND_GAP;
    canvas = canvas
        .child(el().absolute(soon, 320.0).size(300.0, 84.0).radius(42.0).fill(p.surface).opacity(0.6))
        .child(
            el().key("island-soon").absolute(soon + 16.0, 40.0).px(16.0).py(8.0).radius(14.0).fill(p.locked)
                .child(text("Próximamente: más islas").font(STRONG).color(p.muted)),
        );
    // Flight path: dots between consecutive stops, blue where you have flown.
    for pair in centers.windows(2) {
        let ((x0, y0, done), (x1, y1, _)) = (pair[0], pair[1]);
        for k in 0..5 {
            let t = 0.3 + k as f32 * 0.1;
            let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
            canvas = canvas.child(el().absolute(x - 4.0, y - 4.0).size(8.0, 8.0).radius(4.0).fill(if done { p.blue } else { Color::WHITE }));
        }
    }
    for st in stops {
        canvas = canvas.child(st);
    }
    canvas = canvas.w(soon + 300.0 + 90.0);

    col().w_full().h_full().fill(p.sky)
        .child(header)
        .child(el().key("map-scroll").w_full().grow(1.0).scroll_x().child(canvas))
        .child(el().w_full().px(24.0).pb(24.0).child(detail(s, &order)))
}

fn stop(s: &State, i: usize, id: &str, x: f32, y: f32, st: Stop, art: Option<&crate::Art>) -> El {
    let p = pal();
    let node = &s.content.nodes[id];
    let cloud = art.map(|a| match st {
        Stop::Done(_) => &a.cloud,
        Stop::Current => &a.cloud_current,
        Stop::Locked => &a.cloud_locked,
    });
    let (badge, mark, ink) = match st {
        Stop::Done(3) => (p.green, "✓", Color::WHITE),
        Stop::Done(_) => (p.blue, "✓", Color::WHITE),
        Stop::Current => (p.orange, "▶", Color::WHITE),
        Stop::Locked => (p.locked, "·", p.muted),
    };
    let selected = s.selected == i;
    let label = match st {
        Stop::Done(n) => format!("{}: {}", node.title, plural(u32::from(n), "estrella", "estrellas")),
        Stop::Current => format!("{}: siguiente", node.title),
        Stop::Locked => format!("{}: bloqueada", node.title),
    };
    let mut col_ = col().absolute(x - 30.0, y).w(STOP_W + 60.0).gap(4.0).center()
        .child(
            el().key(format!("stop-{id}")).role("button").label(label).focusable(true).size(STOP_W, STOP_H)
                .on_press(move |s: &mut State| s.selected = i)
                .child(picture(cloud, STOP_W, STOP_H).absolute(0.0, 0.0))
                .child(el().absolute(40.0, 40.0).size(40.0, 40.0).radius(20.0).fill(badge).center().child(text(mark).font(STRONG).color(ink))),
        )
        .child(
            el().px(10.0).py(4.0).radius(10.0).fill(if selected { p.blue } else { Color::TRANSPARENT })
                .child(text(short(&node.title)).font(STRONG).color(if selected { Color::WHITE } else if st == Stop::Locked { p.muted } else { p.text })),
        );
    if let Stop::Done(n) = st {
        col_ = col_.child(text(stars(n)).font(H2).color(p.orange));
    }
    if st == Stop::Current {
        col_ = col_.child(
            row().absolute(-10.0, -96.0).gap(4.0).items_start()
                .child(nubi(s, 100.0))
                .child(bubble(text("¡Sigamos!").font(STRONG).color(p.orange_dark))),
        );
    }
    col_
}

/// "S3: buckets y objetos" → "S3"; "Lab: tu primer bucket…" → "Lab".
fn short(title: &str) -> String {
    title.split(':').next().unwrap_or(title).trim().to_string()
}

fn detail(s: &State, order: &[&str]) -> El {
    let p = pal();
    let Some(id) = order.get(s.selected).copied() else { return el() };
    let node = &s.content.nodes[id];
    let st = stop_status(order, id, |x| s.progress.nodes.get(x).map(|r| r.best_stars));
    let best = s.progress.nodes.get(id);
    let kind = match node.kind {
        NodeKind::Lesson => "LECCIÓN",
        NodeKind::Lab => "LAB PRÁCTICO",
        NodeKind::Challenge => "DESAFÍO",
    };
    let (message, best_text) = match (st, best) {
        (Stop::Done(3), _) => ("¡Perfecta! Sin errores. Puedes repasarla cuando quieras.".to_string(), "Mejor: 0 errores".to_string()),
        (Stop::Done(_), Some(b)) => (
            format!("Completada con {}. Repítela para lograr la perfección.", plural(b.best_errors, "error", "errores")),
            format!("Mejor: {}", plural(b.best_errors, "error", "errores")),
        ),
        (Stop::Current, _) => ("Tu siguiente parada. Intentos ilimitados, sin penalizaciones.".to_string(), "Aún sin intentar".to_string()),
        _ => ("Se desbloquea al completar la parada anterior.".to_string(), "Bloqueada".to_string()),
    };
    let open = |s: &mut State| {
        let id = s.order()[s.selected].to_string();
        s.open_stop(&id);
    };
    let cta = match st {
        Stop::Done(3) => primary("cta", "Repasar", open),
        Stop::Done(_) => reward("cta", "Perfeccionar", open),
        Stop::Current => good("cta", "Empezar", open),
        Stop::Locked => disabled("cta", "Bloqueada"),
    };
    let shown = match st {
        Stop::Done(n) => n,
        _ => 0,
    };
    row().key("detail").w_full().gap(20.0).center_y().padding(22.0).radius(24.0).fill(p.surface).border(2.0, p.border).shadow(6.0, 0.0, p.border)
        .child(
            col().grow(1.0).gap(4.0)
                .child(text(kind).font(SMALL).color(p.muted))
                .child(text(node.title.clone()).key("detail-title").font(TITLE).color(p.text))
                .child(text(message).font(BODY).color(p.muted).wrap()),
        )
        .child(
            col().gap(2.0).center()
                .child(text(stars(shown)).font(TITLE).color(p.orange))
                .child(text(best_text).font(SMALL).color(p.muted)),
        )
        .child(cta)
}
