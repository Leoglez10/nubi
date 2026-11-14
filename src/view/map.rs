//! The course map groups each lesson and lab into one numbered class.
use labcloud::grading::{self, Stop, stop_status};
use rune::{El, col, el, row, spacer, text};
use super::{bubble, nubi, picture};
use super::profiles::avatar;
use super::{BODY, H2, SMALL, STRONG, TITLE, disabled, good, pal, primary, progress_bar, stars};
use crate::{Screen, State};

pub fn view(s: &State) -> El {
    let p = pal();
    let order = s.order();
    let classes = s.content.classes();
    let total = s.progress.total_stars();
    let level = grading::level(total);
    let (fraction, next) = match level.next_at {
        Some(n) => ((total - level.from) as f32 / (n - level.from) as f32, format!("{total} / {n} ★")),
        None => (1.0, format!("{total} ★ · nivel máximo")),
    };
    let mut header = row().key("map-header").w_full().gap(18.0).center_y().px(24.0).py(12.0).fill(p.surface).border(2.0, p.border)
        .child(text("labcloud").font(TITLE).color(p.blue)).child(spacer())
        .child(col().w(180.0).gap(4.0).child(text(format!("Nivel {} · {}", level.number, level.name)).font(STRONG).color(p.text))
            .child(progress_bar("level-bar", fraction, p.orange, 8.0)).child(text(next).font(SMALL).color(p.muted)));
    if let Some(profile) = &s.profile {
        header = header.child(el().key("switch-profile").role("button").label(format!("Cambiar de perfil ({})", profile.name)).focusable(true)
            .on_press(|s: &mut State| { s.creating = false; s.screen = Screen::Profiles; })
            .child(avatar(&profile.name, &profile.color, 48.0)));
    }
    let mut canvas = el().key("map-canvas").w(1100.0).h(372.0)
        .child(el().key("island-fundamentos").absolute(36.0, 296.0).size(620.0, 56.0).radius(28.0).fill(p.surface))
        .child(el().absolute(50.0, 12.0).padding(12.0).radius(14.0).fill(p.blue)
            .child(text("Isla 1 · Fundamentos").font(STRONG).color(rune::Color::WHITE)))
        .child(el().absolute(760.0, 296.0).size(280.0, 56.0).radius(28.0).fill(p.surface).opacity(0.6))
        .child(text("Próximamente: más islas").absolute(770.0, 32.0).font(STRONG).color(p.muted));
    // The dotted route is behind the cloud stops, never drawn over them.
    let mut route = el().key("flight-path").w_full().h_full();
    for i in 0..7 {
        let f = i as f32 / 6.0;
        route = route.child(el().absolute(225.0 + f * 145.0, 213.0 - f * 70.0)
            .size(8.0, 8.0).radius(4.0).fill(p.surface));
    }
    canvas = canvas.child(route);
    let mut detail = el().key("detail");
    let current = classes.iter().position(|(_, teaching)| !s.progress.completed(&teaching.lab)).unwrap_or(classes.len().saturating_sub(1));
    for (position, (node, teaching)) in classes.iter().enumerate() {
        let index = order.iter().position(|id| *id == node.id).unwrap_or(0);
        let selected = order.get(s.selected).is_some_and(|id| *id == node.id || *id == teaching.lab);
        let status = stop_status(&order, &node.id, |id| s.progress.nodes.get(id).map(|r| r.best_stars));
        let lesson_stars = s.progress.nodes.get(&node.id).map_or(0, |r| r.best_stars);
        let lab_stars = s.progress.nodes.get(&teaching.lab).map_or(0, |r| r.best_stars);
        let title = format!("Clase {}: {}", teaching.number, teaching.title);
        let id = node.id.clone();
        let action = move |s: &mut State| { s.selected = index; s.open_class(&id); };
        let cta = if status == Stop::Locked { disabled(&format!("class-cta-{}", teaching.number), "Bloqueada") }
            else if lesson_stars > 0 { primary(&format!("class-cta-{}", teaching.number), "Abrir clase", action) }
            else { good(&format!("class-cta-{}", teaching.number), "Empezar clase", action) };
        let x = 90.0 + position as f32 * 300.0;
        let y = if position % 2 == 0 { 170.0 } else { 100.0 };
        let cloud = s.art.as_ref().map(|art| if status == Stop::Locked { &art.cloud_locked } else { &art.cloud });
        let stop = el().key(format!("class-{}", teaching.number)).role("button").label(title.clone()).focusable(true)
            .absolute(x, y).size(120.0, 96.0).press_scale(0.96)
            .on_press(move |s: &mut State| s.selected = index)
            .child(picture(cloud, 120.0, 96.0))
            .child(el().absolute(40.0, 31.0).size(44.0, 44.0).radius(22.0)
                .fill(if status == Stop::Locked { p.locked } else { p.blue }).center()
                .child(text(teaching.number.to_string()).font(H2).color(if status == Stop::Locked { p.muted } else { p.surface })));
        canvas = canvas.child(stop).child(text(format!("Clase {}", teaching.number)).absolute(x + 22.0, y + 100.0).font(STRONG).color(p.text));
        if position == current {
            canvas = canvas.child(row().key("map-nubi").absolute(x - 10.0, y - 90.0).gap(6.0)
                .child(nubi(s, 90.0)).child(bubble(text("¡Sigamos!").font(STRONG).color(p.blue_dark))));
        }
        if selected {
            detail = row().key("detail").w_full().gap(20.0).padding(18.0).center_y().radius(20.0).fill(p.surface)
                .child(col().grow(1.0).gap(6.0)
                    .child(text(title).font(H2).color(p.text).wrap())
                    .child(text(if status == Stop::Locked { "Completa la clase anterior para desbloquearla" } else { "Explicación → juegos → laboratorio → repaso" }).font(BODY).color(p.muted).wrap())
                    .child(text(format!("Juegos {} · Lab {}", stars(lesson_stars), stars(lab_stars))).font(STRONG).color(p.orange_dark)))
                .child(cta);
        }
    }
    col().w_full().h_full().fill(p.sky).child(header)
        .child(el().key("map-scroll").w_full().grow(1.0).scroll_x().child(canvas))
        .child(el().w_full().px(24.0).pb(16.0).child(detail))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classes_keep_the_horizontal_cloud_map_and_mascot() {
        let s = State {
            content: std::sync::Arc::new(labcloud::course::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap()),
            ..State::default()
        };
        let mut rt = rune::Runtime::new(s, crate::view::view, (1180.0, 760.0), rune::Env { system_dark: false, scale: 1.0 });
        rt.update(|s, _| s.screen = Screen::Map);
        rt.frame(0.0);
        for key in ["map-scroll", "island-fundamentos", "flight-path", "map-nubi", "class-1", "class-2", "detail"] {
            assert!(rt.find(key).is_some(), "missing {key}");
        }
        let (x, y) = rt.find("class-cta-1").unwrap();
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.screen, Screen::Class);
        assert!(rt.state.lesson.id.is_empty() && rt.state.lab.id.is_empty(), "a class opens teaching, not a practice");
    }
}
