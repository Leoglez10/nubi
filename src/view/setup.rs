//! Guided setup: what is missing on this machine and the one-click fix for each.

use labcloud::setup::{Action, Need, Os, action};
use rune::{Color, El, col, el, row, spacer, text};

use super::{BODY, SMALL, STRONG, TITLE, pal, plain, primary};
use crate::{Job, State};

fn copy(need: Need) -> (&'static str, &'static str) {
    match need {
        Need::Docker => ("Docker", "Ejecuta tu nube local en un contenedor."),
        Need::DockerRunning => ("Docker encendido", "Docker está instalado pero apagado."),
        Need::AwsCli => ("AWS CLI", "Los comandos aws que practicarás en la terminal."),
        Need::Emulator => ("AWS Emulator", "Tu nube local: AWS en tu computadora, sin cuenta ni factura."),
    }
}

fn label(need: Need, a: &Action) -> Option<&'static str> {
    Some(match (need, a) {
        (_, Action::StartEmulator) => "Iniciar emulador",
        (_, Action::Manual(_)) => return None,
        (Need::DockerRunning, _) => "Abrir Docker",
        (Need::Docker, Action::Run { .. }) => "Instalar Docker",
        (Need::Docker, Action::OpenUrl(_)) => "Descargar Docker",
        (Need::AwsCli, Action::Run { .. }) => "Instalar AWS CLI",
        (Need::AwsCli, Action::OpenUrl(_)) => "Cómo instalar AWS CLI",
        (Need::Emulator, _) => "Iniciar emulador",
    })
}

pub fn view(s: &State) -> El {
    let p = pal();
    let mut card = col().key("setup-card").w_full().max_w(660.0).padding(28.0).gap(16.0)
        .radius(24.0).fill(p.surface).border(2.0, p.border)
        .child(text("Preparemos tu nube local").key("setup-title").font(TITLE).color(p.text))
        .child(text("Lab Cloud necesita estas herramientas en tu computadora. Te ayudamos a instalarlas.").font(BODY).color(p.muted).wrap());

    match &s.setup {
        None => card = card.child(text("Revisando tu computadora…").key("probing").font(STRONG).color(p.blue)),
        Some(report) => {
            let needs = report.needs();
            let items = [
                (Need::Docker, report.docker),
                (Need::DockerRunning, report.docker_running),
                (Need::AwsCli, report.aws_cli),
                (Need::Emulator, report.emulator),
            ];
            for (need, ok) in items {
                if need == Need::DockerRunning && !report.docker {
                    continue; // installing Docker comes first
                }
                let (name, why) = copy(need);
                let fix = action(need, Os::current(), s.has_brew);
                // Starting the emulator needs a running Docker first.
                let blocked = need == Need::Emulator && needs.iter().any(|n| matches!(n, Need::Docker | Need::DockerRunning));
                let mut item = row().key(format!("need-{name}")).w_full().gap(14.0).center_y().padding(14.0).radius(16.0)
                    .fill(if ok { p.green_light } else { p.bg })
                    .child(
                        el().size(30.0, 30.0).radius(15.0).fill(if ok { p.green } else { p.border }).center()
                            .child(text(if ok { "✓" } else { "·" }).font(STRONG).color(Color::WHITE)),
                    )
                    .child(
                        col().grow(1.0).gap(2.0)
                            .child(text(name).font(STRONG).color(p.text))
                            .child(text(if ok { "Listo" } else { why }).font(SMALL).color(p.muted).wrap()),
                    );
                if !ok && !blocked && s.busy.is_none() {
                    if let Some(l) = label(need, &fix) {
                        item = item.child(primary(&format!("fix-{name}"), l, move |s: &mut State| s.send(Job::Fix(need))));
                    } else if let Action::Manual(how) = fix {
                        item = item.child(text(how).font(SMALL).color(p.text).wrap().w(220.0));
                    }
                }
                card = card.child(item);
            }
        }
    }
    if let Some(busy) = &s.busy {
        card = card.child(text(busy.clone()).key("busy").font(STRONG).color(p.blue).wrap());
    }
    if let Some(note) = &s.setup_note {
        card = card.child(text(note.clone()).key("note").font(SMALL).color(p.muted).wrap());
    }
    if s.busy.is_none() && s.setup.is_some() {
        card = card.child(row().child(spacer()).child(plain("recheck", "Comprobar de nuevo", |s: &mut State| {
            s.setup = None;
            s.send(Job::Probe);
        })));
    }
    col().w_full().h_full().fill(p.bg).padding(32.0).center().child(card)
}
