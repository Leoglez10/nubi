//! A lab: objectives on the left, the integrated terminal on the right, and the result with
//! stars, what went wrong and "Perfeccionar".

use labcloud::grading::{self, stars as stars_for};
use labcloud::lab::Kind;
use rune::color::hex;
use rune::{Color, El, Key, KeyEvent, ScrollSpec, TextEdit, Tx, col, el, row, spacer, text, text_input};

use super::{BODY, H2, MONO, SMALL, STRONG, close, pal, plain, plural, primary, reward, stars};
use crate::{Job, State};

struct Term {
    bg: Color,
    bar: Color,
    text: Color,
    prompt: Color,
    ok: Color,
    err: Color,
    info: Color,
    chip_border: Color,
    edge: Color,
}

fn term() -> Term {
    let c = |h: &str| hex(h).unwrap_or(Color::BLACK);
    Term {
        bg: c("#0E1B2A"),
        bar: c("#14263A"),
        text: c("#D6E4F2"),
        prompt: c("#5FB2FF"),
        ok: c("#6EE7A8"),
        err: c("#FF9EA1"),
        info: c("#9FB6CC"),
        chip_border: c("#2B4866"),
        edge: c("#071019"),
    }
}

pub fn view(s: &State) -> El {
    let p = pal();
    let t = term();
    let l = &s.lab;
    let errors = l.session.errors;
    let (chip_bg, chip_ink, chip) = if errors == 0 {
        (p.green_light, p.green_dark, "Sin errores".to_string())
    } else {
        (p.orange_light, hex("#8A5300").unwrap_or(p.text), plural(errors, "error", "errores"))
    };
    let total = s.progress.total_stars();
    let mut header = row().key("lab-header").w_full().gap(16.0).center_y().px(20.0).py(12.0).fill(p.surface).border(2.0, p.border)
        .child(close(|s: &mut State| s.to_map()))
        .child(plain("lab-explanation", "Ver explicación", |s: &mut State| s.revisit_class()))
        .child(
            col().grow(1.0).gap(2.0)
                .child(text("LAB PRÁCTICO").font(SMALL).color(p.muted))
                .child(text(l.title.clone()).key("lab-title").font(H2).color(p.text)),
        );
    if let Some(profile) = &s.profile {
        header = header.child(text(format!("{} · Nivel {} · {} ★", profile.name, grading::level(total).number, total)).font(SMALL).color(p.muted));
    }
    header = header.child(el().key("errors").px(14.0).h(34.0).radius(17.0).fill(chip_bg).center().child(text(chip).font(SMALL).color(chip_ink)));

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
        left = left.child(result(s));
    }

    let mut log = col().key("log").w_full().gap(3.0);
    for (i, line) in l.session.lines.iter().enumerate() {
        let (prefix, color) = match line.kind {
            Kind::Cmd => ("nubi@labcloud $ ", t.prompt),
            Kind::Out => ("", t.text),
            Kind::Err => ("", t.err),
            Kind::Ok => ("", t.ok),
            Kind::Info => ("", t.info),
        };
        log = log.child(text(format!("{prefix}{}", line.text)).key(format!("line-{i}")).font(MONO).color(color).wrap().selectable(true));
    }
    if l.running {
        log = log.child(text("…").key("running").font(MONO).color(t.info));
    }
    let prompt = row().key("prompt").w_full().gap(8.0).center_y()
        .child(text("nubi@labcloud $").font(MONO).color(t.prompt))
        .child(
            text_input(&l.input, MONO).key("input").label("Comando").single_line().grow(1.0)
                .color(t.text).placeholder("escribe un comando y presiona Enter", t.info)
                .on_edit(|s: &mut State| &mut s.lab.input)
                .on_input_key(history_key)
                .on_submit(|s: &mut State, _: &mut Tx| run_input(s)),
        );

    let mut chips = row().key("chips").w_full().gap(8.0).center_y().flex_wrap().child(text("Sugerencias:").font(SMALL).color(t.info));
    let next = l.session.next_objective().and_then(|i| l.objectives.get(i)).map(|(_, hint)| hint.clone());
    let mut suggestions: Vec<(&str, String)> = vec![("pista", "pista".into()), ("aws s3 ls", "aws s3 ls".into())];
    if let Some(next) = next {
        suggestions.insert(0, ("siguiente paso", next));
    }
    for (label, value) in suggestions {
        chips = chips.child(
            el().key(format!("chip-{label}")).role("button").label(label.to_string()).focusable(true)
                .h(34.0).px(12.0).radius(10.0).fill(t.bar).border(1.0, t.chip_border).center()
                .on_press(move |s: &mut State| s.lab.input = TextEdit::new(value.clone()))
                .child(text(label.to_string()).font(MONO).color(t.text)),
        );
    }
    chips = chips.child(spacer()).child(reward("run", "Ejecutar", run_input));

    let terminal = col().key("terminal").grow(1.0).h_full().radius(20.0).clip_rounded(20.0).fill(t.bg).shadow(6.0, 0.0, t.edge)
        .child(
            row().w_full().px(18.0).py(12.0).fill(t.bar).justify_between()
                .child(text("Terminal").font(SMALL).color(t.info))
                .child(text("aws → 127.0.0.1:9297").font(MONO).color(t.info)),
        )
        .child(el().key("scroll").w_full().grow(1.0).padding(18.0).scroll(ScrollSpec { stick_bottom: true }).child(log))
        .child(col().w_full().gap(12.0).px(18.0).py(14.0).child(prompt).child(chips));

    col().w_full().h_full().fill(p.bg)
        .child(header)
        .child(row().key("body").w_full().grow(1.0).gap(24.0).padding(24.0).items_start().child(left).child(terminal))
}

fn result(s: &State) -> El {
    let p = pal();
    let l = &s.lab;
    let errors = l.session.errors;
    let (title, body) = if errors == 0 {
        ("¡Misión perfecta!".to_string(), "Cero errores. Dominas este lab.".to_string())
    } else {
        ("¡Misión completada!".to_string(), format!("Tuviste {}. Repítelo desde cero para lograr la perfección.", plural(errors, "error", "errores")))
    };
    let mut panel = col().key("result").w_full().gap(10.0).padding(18.0).radius(20.0).fill(p.green_light).border(2.0, hex("#8FD9B5").unwrap_or(p.green))
        .child(text(title).font(H2).color(p.green_dark))
        .child(text(stars(stars_for(errors))).key("stars").font(rune::tok::TextStyle::new(30.0, 800.0)).color(p.orange))
        .child(text(body).font(BODY).color(p.green_dark).wrap());
    if let Some(best) = s.progress.nodes.get(&l.id) {
        panel = panel.child(text(format!("Tu mejor marca: {} · {}", stars(best.best_stars), plural(best.best_errors, "error", "errores"))).font(SMALL).color(p.muted));
    }
    for m in l.session.mistakes.iter().take(3) {
        panel = panel.child(
            col().w_full().gap(2.0).padding(10.0).radius(12.0).fill(hex("#FFF6E8").unwrap_or(p.surface))
                .child(text(m.command.clone()).font(MONO).color(hex("#8A5300").unwrap_or(p.text)).wrap())
                .child(text(m.error.clone()).font(SMALL).color(p.text).wrap()),
        );
    }
    let again = if errors > 0 { reward("perfect", "Perfeccionar", |s: &mut State| s.restart_lab()) } else { primary("again", "Repetir lab", |s: &mut State| s.restart_lab()) };
    panel.child(again).child(plain("class-review", "Repasar la clase", |s: &mut State| s.show_class_review()))
        .child(primary("back-to-map", "Seguir en el mapa", |s: &mut State| s.to_map()))
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
        Key::Up => s.lab.session.history_prev().map(String::from).unwrap_or_else(|| s.lab.input.text.clone()),
        Key::Down => s.lab.session.history_next().unwrap_or_default().to_string(),
        _ => return false,
    };
    s.lab.input = TextEdit::new(line);
    true
}
