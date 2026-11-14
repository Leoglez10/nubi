//! The screens: guided setup and the lab with its integrated terminal.
//! Light cloud palette (white and blue, orange for rewards); all copy in Spanish.

use labcloud::lab::Kind;
use labcloud::setup::{Action, Need, Os, action};
use rune::color::hex;
use rune::tok::TextStyle;
use rune::{Color, Cx, El, Key, KeyEvent, ScrollSpec, TextEdit, Tx, col, el, row, spacer, text, text_input};

use crate::{Job, Screen, State};

struct Pal {
    bg: Color,
    surface: Color,
    border: Color,
    text: Color,
    muted: Color,
    blue: Color,
    blue_dark: Color,
    orange: Color,
    orange_dark: Color,
    ink_on_orange: Color,
    green: Color,
    green_dark: Color,
    green_light: Color,
    term_bg: Color,
    term_bar: Color,
    term_text: Color,
    term_prompt: Color,
    term_ok: Color,
    term_err: Color,
    term_info: Color,
}

fn pal() -> Pal {
    let c = |h: &str| hex(h).unwrap_or(Color::BLACK);
    Pal {
        bg: c("#EEF5FC"),
        surface: c("#FFFFFF"),
        border: c("#D6E4F2"),
        text: c("#0F2A44"),
        muted: c("#51657A"),
        blue: c("#1F6FD1"),
        blue_dark: c("#154F99"),
        orange: c("#FF9900"),
        orange_dark: c("#C27400"),
        ink_on_orange: c("#232F3E"),
        green: c("#22A06B"),
        green_dark: c("#17784F"),
        green_light: c("#E3F6EC"),
        term_bg: c("#0E1B2A"),
        term_bar: c("#14263A"),
        term_text: c("#D6E4F2"),
        term_prompt: c("#5FB2FF"),
        term_ok: c("#6EE7A8"),
        term_err: c("#FF9EA1"),
        term_info: c("#9FB6CC"),
    }
}

const TITLE: TextStyle = TextStyle::new(26.0, 800.0);
const H2: TextStyle = TextStyle::new(18.0, 800.0);
const BODY: TextStyle = TextStyle::new(15.0, 500.0);
const STRONG: TextStyle = TextStyle::new(15.0, 700.0);
const SMALL: TextStyle = TextStyle::new(13.0, 700.0);
const MONO: TextStyle = TextStyle::new(14.0, 400.0).mono();

/// A chunky game button: solid fill with a darker bottom edge.
fn button(key: &str, label: &str, fill: Color, edge: Color, ink: Color, on: impl Fn(&mut State) + 'static) -> El {
    el().key(key.to_string()).role("button").label(label.to_string()).focusable(true)
        .h(48.0).px(22.0).radius(14.0).fill(fill).shadow(4.0, 0.0, edge).center()
        .on_press(on)
        .child(text(label.to_string()).font(STRONG).color(ink))
}

pub fn view(s: &State, _: &mut Cx<State>) -> El {
    match s.screen {
        Screen::Setup => setup(s),
        Screen::Lab => lab(s),
    }
}

fn need_copy(need: Need) -> (&'static str, &'static str) {
    match need {
        Need::Docker => ("Docker", "Ejecuta tu nube local en un contenedor."),
        Need::DockerRunning => ("Docker encendido", "Docker está instalado pero apagado."),
        Need::AwsCli => ("AWS CLI", "Los comandos aws que practicarás en la terminal."),
        Need::Emulator => ("AWS Emulator", "Tu nube local: AWS en tu computadora, sin cuenta ni factura."),
    }
}

fn action_label(need: Need, a: &Action) -> Option<&'static str> {
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

fn setup(s: &State) -> El {
    let p = pal();
    let mut card = col().key("setup-card").w_full().max_w(640.0).padding(28.0).gap(16.0)
        .radius(24.0).fill(p.surface).border(2.0, p.border)
        .child(text("Preparemos tu nube local").key("setup-title").font(TITLE).color(p.text))
        .child(text("Lab Cloud necesita estas herramientas en tu computadora. Te ayudamos a instalarlas.").font(BODY).color(p.muted).wrap());

    match &s.setup {
        None => card = card.child(text("Revisando tu computadora…").key("probing").font(STRONG).color(p.blue)),
        Some(report) => {
            let needs = report.needs();
            let items = [
                (Need::Docker, report.docker),
                (Need::DockerRunning, report.docker_running || !report.docker),
                (Need::AwsCli, report.aws_cli),
                (Need::Emulator, report.emulator),
            ];
            for (need, ok) in items {
                if need == Need::DockerRunning && !report.docker {
                    continue; // installing Docker comes first
                }
                let (name, why) = need_copy(need);
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
                    if let Some(label) = action_label(need, &fix) {
                        let edge = p.blue_dark;
                        item = item.child(button(&format!("fix-{name}"), label, p.blue, edge, Color::WHITE, move |s: &mut State| s.send(Job::Fix(need))));
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
        card = card.child(row().child(spacer()).child(button("recheck", "Comprobar de nuevo", p.surface, p.border, p.text, |s: &mut State| {
            s.setup = None;
            s.send(Job::Probe);
        })));
    }
    col().w_full().h_full().fill(p.bg).padding(32.0).center().child(card)
}

fn lab(s: &State) -> El {
    let p = pal();
    let l = &s.lab;
    let errors = l.session.errors;
    let (chip_bg, chip_ink, chip) = if errors == 0 {
        (p.green_light, p.green_dark, "Sin errores".to_string())
    } else {
        (hex("#FFF1DB").unwrap_or(p.bg), hex("#8A5300").unwrap_or(p.text), format!("{errors} {}", if errors == 1 { "error" } else { "errores" }))
    };
    let header = row().key("lab-header").w_full().gap(16.0).center_y().px(24.0).py(14.0).fill(p.surface).border(2.0, p.border)
        .child(
            col().grow(1.0).gap(2.0)
                .child(text("LAB PRÁCTICO").font(SMALL).color(p.muted))
                .child(text(l.title.clone()).key("lab-title").font(H2).color(p.text)),
        )
        .child(text(format!("{} · Nivel {} · {} ★", s.profile_name, s.level.number, s.total_stars)).font(SMALL).color(p.muted))
        .child(el().key("errors").px(14.0).h(34.0).radius(17.0).fill(chip_bg).center().child(text(chip).font(SMALL).color(chip_ink)));

    // Objectives
    let mut goals = col().key("objectives").w_full().gap(10.0).padding(20.0).radius(20.0).fill(p.surface).border(2.0, p.border)
        .child(text("Objetivos").font(H2).color(p.text));
    for (i, (txt, _)) in l.objectives.iter().enumerate() {
        let done = l.session.done.get(i).copied().unwrap_or(false);
        goals = goals.child(
            row().key(format!("goal-{i}")).w_full().gap(12.0).center_y().padding(12.0).radius(14.0)
                .fill(if done { p.green_light } else { p.bg })
                .child(
                    el().size(30.0, 30.0).radius(15.0).fill(if done { p.green } else { p.border }).center()
                        .child(text(if done { "✓".to_string() } else { (i + 1).to_string() }).font(SMALL).color(if done { Color::WHITE } else { p.muted })),
                )
                .child(text(txt.clone()).font(STRONG).color(p.text).wrap().grow(1.0)),
        );
    }
    let mut left = col().key("left").w(360.0).gap(16.0).child(goals);
    if l.session.complete() {
        left = left.child(result_panel(s, &p));
    }

    // Terminal
    let mut log = col().key("log").w_full().gap(3.0);
    for (i, line) in l.session.lines.iter().enumerate() {
        let (prefix, color) = match line.kind {
            Kind::Cmd => ("nubi@labcloud $ ", p.term_prompt),
            Kind::Out => ("", p.term_text),
            Kind::Err => ("", p.term_err),
            Kind::Ok => ("", p.term_ok),
            Kind::Info => ("", p.term_info),
        };
        log = log.child(text(format!("{prefix}{}", line.text)).key(format!("line-{i}")).font(MONO).color(color).wrap().selectable(true));
    }
    if l.running {
        log = log.child(text("…").key("running").font(MONO).color(p.term_info));
    }
    let prompt = row().key("prompt").w_full().gap(8.0).center_y()
        .child(text("nubi@labcloud $").font(MONO).color(p.term_prompt))
        .child(
            text_input(&l.input, MONO).key("input").label("Comando").single_line().grow(1.0)
                .color(p.term_text).placeholder("escribe un comando y presiona Enter", p.term_info)
                .on_edit(|s: &mut State| &mut s.lab.input)
                .on_input_key(history_key)
                .on_submit(submit),
        );

    let mut chips = row().key("chips").w_full().gap(8.0).center_y().flex_wrap()
        .child(text("Sugerencias:").font(SMALL).color(p.term_info));
    let next = l.session.next_objective().and_then(|i| l.objectives.get(i)).map(|(_, hint)| hint.clone());
    let mut suggestions: Vec<(&str, String)> = vec![("pista", "pista".into()), ("aws s3 ls", "aws s3 ls".into())];
    if let Some(next) = next {
        suggestions.insert(0, ("siguiente paso", next));
    }
    for (label, value) in suggestions {
        chips = chips.child(
            el().key(format!("chip-{label}")).role("button").label(label.to_string()).focusable(true)
                .h(34.0).px(12.0).radius(10.0).fill(p.term_bar).border(1.0, hex("#2B4866").unwrap_or(p.term_bar)).center()
                .on_press(move |s: &mut State| s.lab.input = TextEdit::new(value.clone()))
                .child(text(label.to_string()).font(MONO).color(p.term_text)),
        );
    }
    chips = chips.child(spacer()).child(button("run", "Ejecutar", p.orange, p.orange_dark, p.ink_on_orange, |s: &mut State| run_input(s)));

    let terminal = col().key("terminal").grow(1.0).h_full().radius(20.0).clip_rounded(20.0).fill(p.term_bg).shadow(6.0, 0.0, hex("#071019").unwrap_or(Color::BLACK))
        .child(
            row().w_full().px(18.0).py(12.0).fill(p.term_bar).justify_between()
                .child(text("Terminal").font(SMALL).color(p.term_info))
                .child(text("aws → 127.0.0.1:9297").font(MONO).color(p.term_info)),
        )
        .child(el().key("scroll").w_full().grow(1.0).padding(18.0).scroll(ScrollSpec { stick_bottom: true }).child(log))
        .child(col().w_full().gap(12.0).px(18.0).py(14.0).child(prompt).child(chips));

    col().w_full().h_full().fill(p.bg)
        .child(header)
        .child(row().key("body").w_full().grow(1.0).gap(24.0).padding(24.0).items_start().child(left).child(terminal))
}

fn result_panel(s: &State, p: &Pal) -> El {
    let l = &s.lab;
    let errors = l.session.errors;
    let stars = labcloud::grading::stars(errors);
    let star_text: String = (0..3).map(|i| if i < stars { '★' } else { '☆' }).collect();
    let (title, body) = if errors == 0 {
        ("¡Misión perfecta!".to_string(), "Cero errores. Dominas este lab.".to_string())
    } else {
        (
            "¡Misión completada!".to_string(),
            format!("Tuviste {errors} {}. Repítelo desde cero para lograr la perfección.", if errors == 1 { "error" } else { "errores" }),
        )
    };
    let mut panel = col().key("result").w_full().gap(10.0).padding(18.0).radius(20.0).fill(p.green_light).border(2.0, hex("#8FD9B5").unwrap_or(p.green))
        .child(text(title).font(H2).color(p.green_dark))
        .child(text(star_text).key("stars").font(TextStyle::new(30.0, 800.0)).color(p.orange))
        .child(text(body).font(BODY).color(p.green_dark).wrap());
    if let Some(best) = &l.best {
        panel = panel.child(text(format!("Tu mejor marca: {} ★ · {} {}", best.best_stars, best.best_errors, if best.best_errors == 1 { "error" } else { "errores" })).font(SMALL).color(p.muted));
    }
    for m in l.session.mistakes.iter().take(3) {
        panel = panel.child(
            col().w_full().gap(2.0).padding(10.0).radius(12.0).fill(hex("#FFF6E8").unwrap_or(p.surface))
                .child(text(m.command.clone()).font(MONO).color(hex("#8A5300").unwrap_or(p.text)).wrap())
                .child(text(m.error.clone()).font(SMALL).color(p.text).wrap()),
        );
    }
    if errors > 0 {
        panel = panel.child(button("perfect", "Perfeccionar", p.orange, p.orange_dark, p.ink_on_orange, |s: &mut State| s.restart_lab()));
    } else {
        panel = panel.child(button("again", "Repetir lab", p.blue, p.blue_dark, Color::WHITE, |s: &mut State| s.restart_lab()));
    }
    panel
}

fn submit(s: &mut State, _: &mut Tx) {
    run_input(s);
}

fn run_input(s: &mut State) {
    if s.lab.running {
        return;
    }
    let line = s.lab.input.text.clone();
    s.lab.input = TextEdit::new("");
    let hints: Vec<String> = s.lab.objectives.iter().map(|(_, h)| h.clone()).collect();
    if let Some(args) = s.lab.session.submit(&line, &hints) {
        s.lab.running = true;
        s.send(Job::Run(args));
    }
}

fn history_key(s: &mut State, ev: &KeyEvent, _: &mut Tx) -> bool {
    let line = match ev.key {
        Key::Up => s.lab.session.history_prev().map(String::from).or_else(|| Some(s.lab.input.text.clone())),
        Key::Down => Some(s.lab.session.history_next().unwrap_or_default().to_string()),
        _ => return false,
    };
    if let Some(line) = line {
        s.lab.input = TextEdit::new(line);
    }
    true
}
