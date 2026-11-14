//! The animated intro: Nubi rises over the clouds, the logo and the local-cloud checks
//! appear one after another, then "Entrar".

use rune::{Color, El, LoopSpec, Transition, col, el, row, text};

use super::{BODY, HERO, STRONG, nubi, pal, picture, primary};
use crate::State;

pub fn view(s: &State) -> El {
    let p = pal();
    let soft = s.art.as_ref().map(|a| &a.cloud_soft);
    let drift = |period: f64, k: f32| LoopSpec::slide(period, [-k, 0.0], [k, 0.0]).easing(rune::Easing::Piecewise(&[[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]]));
    let mut root = el().key("intro").w_full().h_full().fill(p.sky)
        .child(picture(soft, 300.0, 240.0).absolute(-40.0, 470.0).opacity(0.95).looping(drift(14.0, 0.08)))
        .child(picture(soft, 240.0, 192.0).absolute(60.0, 60.0).opacity(0.8).looping(drift(11.0, 0.1)))
        .child(picture(soft, 280.0, 224.0).absolute(900.0, 520.0).opacity(0.95).looping(drift(16.0, 0.07)))
        .child(picture(soft, 200.0, 160.0).absolute(940.0, 90.0).opacity(0.75).looping(drift(12.0, 0.1)));

    // Pieces are added on the second frame so they enter with motion.
    if s.intro_started {
        let mut center = col().key("intro-center").w_full().h_full().center().gap(14.0)
            .child(nubi(s, 200.0).transition(Transition::offset(0.0, 140.0)))
            .child(text("Lab Cloud").key("logo").font(HERO).color(p.text).transition(Transition::opacity()).enter_delay(0.5))
            .child(text("Aprende AWS en tu propia nube").font(BODY).color(p.blue_dark).transition(Transition::opacity()).enter_delay(0.8));

        let checks = [
            ("check-docker", "Docker encontrado", s.setup.map(|r| r.docker_running)),
            ("check-emulator", "Tu nube local (AWS Emulator)", s.setup.map(|r| r.emulator)),
            ("check-progress", "Tu progreso está guardado en esta computadora", s.profiles.as_ref().map(|_| true)),
        ];
        let mut list = col().key("checks").w(440.0).gap(8.0);
        for (i, (key, label, ok)) in checks.into_iter().enumerate() {
            let (mark, color) = match ok {
                None => ("…", p.muted),
                Some(true) => ("✓", p.green_dark),
                Some(false) => ("!", p.orange_dark),
            };
            list = list.child(
                row().key(key).w_full().gap(10.0).center_y().px(14.0).py(10.0).radius(14.0).fill(Color::WHITE.alpha(0.88))
                    .transition(Transition::offset(0.0, 16.0)).enter_delay(1.1 + i as f32 * 0.25)
                    .child(text(mark).font(STRONG).color(color))
                    .child(text(label).font(STRONG).color(p.text)),
            );
        }
        center = center.child(el().h(8.0)).child(list);

        if let Some(report) = s.setup {
            let label = if report.ready() { "Entrar" } else { "Preparar mi computadora" };
            center = center.child(el().h(8.0)).child(
                primary("enter", label, |s: &mut State| s.enter()).transition(Transition::scale(0.8)).enter_delay(1.9),
            );
        }
        root = root.child(center.absolute(0.0, 0.0));
    }
    root
}
