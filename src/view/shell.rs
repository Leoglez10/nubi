//! The dashboard frame around the learning screens: a sidebar to move between the current
//! class's parts and a top bar that says where you are.
use labcloud::grading;
use rune::color::hex;
use rune::{El, Image, col, el, row, spacer, text};

use super::profiles::avatar;
use super::{SMALL, STRONG, pal, picture};
use crate::{Screen, State};

const SIDEBAR: f32 = 232.0;

pub fn wrap(s: &State, screen: Screen, content: El) -> El {
    let p = pal();
    let main = col().key("shell-main").grow(1.0).h_full().fill(p.bg)
        .child(top_bar(s, screen))
        // min_h(0) lets long pages scroll inside instead of stretching the frame past the window.
        .child(el().w_full().grow(1.0).min_h(0.0).child(content));
    row().key("shell").w_full().h_full().min_h(0.0).child(sidebar(s, screen)).child(main)
}

fn sidebar(s: &State, screen: Screen) -> El {
    let p = pal();
    let icons = s.art.as_ref();
    let mut bar = col().key("sidebar").w(SIDEBAR).h_full().gap(4.0).px(14.0).py(18.0).fill(p.surface)
        .border_sides([0.0, 1.0, 0.0, 0.0], p.border)
        .child(row().gap(10.0).center_y().px(10.0).pb(14.0)
            .child(picture(icons.map(|a| &a.cloud), 30.0, 24.0))
            .child(text("Lab Cloud").font(rune::tok::TextStyle::new(16.0, 700.0)).color(p.text)))
        .child(text("APRENDER").font(rune::tok::TextStyle::new(11.0, 600.0)).color(hex("#667085").unwrap_or(p.muted)).px(10.0).py(6.0));

    let current = s.current_class();
    let lab_ready = current.as_ref().is_some_and(|(lesson, _)| s.progress.completed(lesson));
    let items: [(&str, &str, Screen, Option<&Image>, bool); 4] = [
        ("nav-home", "Inicio", Screen::Map, icons.map(|a| &a.icon_home), true),
        ("nav-class", "Clase actual", Screen::Class, icons.map(|a| &a.icon_book), current.is_some()),
        ("nav-practice", "Práctica", Screen::Lesson, icons.map(|a| &a.icon_check), current.is_some()),
        ("nav-lab", "Laboratorio", Screen::Lab, icons.map(|a| &a.icon_term), lab_ready || screen == Screen::Lab),
    ];
    for (key, label, target, icon, enabled) in items {
        bar = bar.child(nav_item(key, label, icon, target == screen, enabled, target));
    }
    bar = bar.child(spacer()).child(environment(s))
        .child(super::motion::toggle(s).w_full().h(34.0).px(10.0));
    if let Some(profile) = &s.profile {
        bar = bar.child(
            el().key("switch-profile").role("button").label(format!("Cambiar de perfil ({})", profile.name)).focusable(true)
                .hover(p.text.alpha(0.04)).radius(8.0).focus_ring(2.0, 2.0, p.blue)
                .on_press(|s: &mut State| { s.creating = false; s.screen = Screen::Profiles; })
                .child(row().gap(10.0).center_y().padding(10.0)
                    .child(avatar(&profile.name, &profile.color, 32.0))
                    .child(col().gap(1.0)
                        .child(text(profile.name.clone()).font(STRONG).color(p.text))
                        .child(text("Cambiar de perfil").font(SMALL).color(hex("#667085").unwrap_or(p.muted))))),
        );
    }
    bar
}

fn nav_item(key: &str, label: &str, icon: Option<&Image>, active: bool, enabled: bool, target: Screen) -> El {
    let p = pal();
    let ink = if !enabled { hex("#98A2B3").unwrap_or(p.muted) } else if active { p.text } else { hex("#344054").unwrap_or(p.text) };
    let weight = if active { 600.0 } else { 500.0 };
    let mut item = el().key(key.to_string()).role("button").label(label.to_string()).w_full().h(38.0).px(10.0).radius(8.0)
        .child(row().h_full().gap(10.0).center_y()
            .child(picture(icon, 18.0, 18.0).opacity(if enabled { 1.0 } else { 0.4 }))
            .child(text(label.to_string()).font(rune::tok::TextStyle::new(14.0, weight)).color(ink)));
    if active {
        item = item.fill(hex("#EEF0F4").unwrap_or(p.locked)).checked(true);
    }
    if enabled {
        item = item.focusable(true).hover(p.text.alpha(0.05)).focus_ring(2.0, 2.0, p.blue)
            .on_press(move |s: &mut State| s.go_to(target));
    }
    item
}

/// What the machine has ready, from the last setup probe.
fn environment(s: &State) -> El {
    let p = pal();
    let r = s.setup.unwrap_or_default();
    let mut card = col().key("environment").w_full().gap(8.0).padding(12.0).radius(10.0).border(1.0, p.border)
        .child(text("Entorno local").font(rune::tok::TextStyle::new(12.0, 600.0)).color(p.muted));
    for (name, ok) in [("Docker", r.docker && r.docker_running), ("AWS CLI", r.aws_cli), ("Emulador S3", r.emulator)] {
        card = card.child(row().w_full().gap(8.0).center_y()
            .child(el().size(8.0, 8.0).radius(4.0).fill(if ok { p.green } else { p.red }))
            .child(text(name).font(rune::tok::TextStyle::new(13.0, 500.0)).color(p.text).grow(1.0))
            .child(text(if ok { "Listo" } else { "Falta" }).font(rune::tok::TextStyle::new(12.0, 500.0)).color(if ok { p.green_dark } else { p.red_dark })));
    }
    card
}

fn top_bar(s: &State, screen: Screen) -> El {
    let p = pal();
    let class = s.shown_class().map(|(number, title)| format!("Clase {number} · {title}"));
    let crumb = match (screen, class) {
        (Screen::Class, Some(c)) => c,
        (Screen::Lesson, Some(c)) => format!("{c} · Práctica"),
        (Screen::Lab, Some(c)) => format!("{c} · Laboratorio"),
        _ => "Inicio".to_string(),
    };
    let total = s.progress.total_stars();
    let level = grading::level(total);
    row().key("top-bar").w_full().h(56.0).gap(10.0).center_y().px(28.0).fill(p.surface)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.border)
        .child(text("Fundamentos de S3").font(rune::tok::TextStyle::new(14.0, 500.0)).color(p.muted))
        .child(text("/").font(SMALL).color(hex("#98A2B3").unwrap_or(p.muted)))
        .child(text(crumb).key("crumb").font(STRONG).color(p.text))
        .child(spacer())
        .child(el().key("stars").h(32.0).px(12.0).radius(16.0).border(1.0, p.border).center()
            .child(text(format!("★ {total} · Nivel {} · {}", level.number, level.name)).font(rune::tok::TextStyle::new(13.0, 500.0)).color(p.text)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    fn app(screen: Screen) -> Runtime<State> {
        let s = State {
            content: std::sync::Arc::new(labcloud::course::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap()),
            screen,
            ..State::default()
        };
        let mut rt = Runtime::new(s, crate::view::view, (1280.0, 800.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        rt
    }

    fn press(rt: &mut Runtime<State>, key: &str) {
        let (x, y) = rt.find(key).unwrap_or_else(|| panic!("missing {key}"));
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
    }

    #[test]
    fn learning_screens_share_the_frame_and_setup_screens_do_not() {
        for screen in [Screen::Map, Screen::Class, Screen::Lesson, Screen::Lab] {
            let rt = app(screen);
            for key in ["sidebar", "top-bar", "nav-home", "environment", "stars"] {
                assert!(rt.find(key).is_some(), "{screen:?}: missing {key}");
            }
        }
        assert!(app(Screen::Profiles).find("sidebar").is_none());
    }

    #[test]
    fn sidebar_resumes_the_practice_instead_of_restarting_it() {
        let mut rt = app(Screen::Map);
        press(&mut rt, "nav-practice");
        assert_eq!(rt.state.screen, Screen::Lesson);
        assert_eq!(rt.state.lesson.id, "s3-buckets");
        rt.update(|s, _| s.lesson.session.index = 2);
        press(&mut rt, "nav-class");
        assert_eq!(rt.state.screen, Screen::Class);
        press(&mut rt, "nav-practice");
        assert_eq!(rt.state.screen, Screen::Lesson);
        assert_eq!(rt.state.lesson.session.index, 2, "going back to the practice keeps the attempt");
        press(&mut rt, "nav-home");
        assert_eq!(rt.state.screen, Screen::Map);
        assert!(rt.find("crumb").is_some());
    }

    #[test]
    fn the_lab_entry_waits_for_the_practice() {
        let mut rt = app(Screen::Map);
        press(&mut rt, "nav-lab");
        assert_eq!(rt.state.screen, Screen::Map, "locked until the class practice is done");
        rt.update(|s, _| { s.progress.record("s3-buckets", 0); });
        rt.frame(1.0 / 60.0);
        press(&mut rt, "nav-lab");
        assert_eq!(rt.state.screen, Screen::Lab);
        assert_eq!(rt.state.lab.id, "lab-primer-bucket");
    }
}
