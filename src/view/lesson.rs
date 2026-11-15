//! A lesson: one exercise per screen, a check bar that says right or "try again" with a
//! hint, unlimited retries, and a result with stars and what to review.

use nubi::course::Exercise;
use nubi::grading::{Answer, stars as stars_for};
use nubi::lesson::{Feedback, prompt, shuffled};
use rune::{Color, El, col, el, row, spacer, text, text_input};

use super::{BODY, H2, MONO, Mood, SMALL, STRONG, TITLE, disabled, good, nubi_face, pal, plain, plural, primary, progress_bar, reward, stars};
use crate::{Job, LessonView, State};

/// The student's answer for `ex`, or `None` while it is incomplete.
pub fn answer(ex: &Exercise, l: &LessonView) -> Option<Answer> {
    match ex {
        Exercise::Choice { .. } => l.choice.map(Answer::Choice),
        Exercise::Command { .. } => {
            let t = l.command.text.trim();
            (!t.is_empty()).then(|| Answer::Command(t.to_string()))
        }
        Exercise::Order { steps, .. } => {
            let display = shuffled(steps.len());
            (l.picked.len() == steps.len()).then(|| Answer::Order(l.picked.iter().map(|&d| display[d]).collect()))
        }
        Exercise::Match { pairs, .. } => (l.pairs.len() == pairs.len()).then(|| Answer::Match(l.pairs.clone())),
    }
}

fn current(s: &State) -> Option<&Exercise> {
    s.content.nodes.get(&s.lesson.id)?.exercises.get(s.lesson.session.index)
}

fn check(s: &mut State) {
    let content = s.content.clone();
    let Some(ex) = content.nodes.get(&s.lesson.id).and_then(|n| n.exercises.get(s.lesson.session.index)) else { return };
    if let Some(a) = answer(ex, &s.lesson) {
        s.lesson.session.check(ex, &a);
    }
}

fn next(s: &mut State) {
    s.lesson.session.next();
    s.lesson.clear_answer();
    if s.lesson.session.finished() && !s.lesson.saved {
        s.lesson.saved = true;
        s.send(Job::Finish { id: s.lesson.id.clone(), errors: s.lesson.session.errors });
    }
}

fn retry(s: &mut State) {
    s.lesson.session.retry();
    s.lesson.clear_answer();
}

pub fn view(s: &State) -> El {
    let p = pal();
    let l = &s.lesson;
    let n = (l.session.index + 1).min(l.session.total);
    let top = row().key("lesson-top").w_full().gap(14.0).center_y().px(24.0).py(14.0).border_sides([0.0, 0.0, 1.0, 0.0], p.border)
        .child(text(format!("Pregunta {n} de {}", l.session.total)).font(rune::tok::TextStyle::new(13.0, 600.0)).color(p.muted))
        .child(el().grow(1.0).child(progress_bar("lesson-progress", l.session.progress(), p.blue, 6.0)))
        .child(plain("lesson-explanation", "Ver explicación", |s: &mut State| s.revisit_class()).h(36.0).px(12.0));

    let finished = l.session.finished();
    let body = if finished { result(s) } else { exercise(s) };
    let mut card = col().key("lesson").w_full().max_w(760.0).radius(12.0).fill(p.surface).border(1.0, p.border).clip_rounded(12.0);
    if !finished {
        card = card.child(top);
    }
    card = card.child(el().w_full().px(24.0).py(28.0).child(body));
    if !finished {
        card = card.child(footer(s));
    }
    super::page_scroll(row().w_full().justify_center().items_start().padding(28.0).child(card))
}

fn exercise(s: &State) -> El {
    let p = pal();
    let Some(ex) = current(s) else { return el() };
    let title = match ex {
        Exercise::Choice { .. } => "Elige la respuesta correcta",
        Exercise::Command { .. } => "Escribe el comando",
        Exercise::Order { .. } => "Ordena los pasos",
        Exercise::Match { .. } => "Une las parejas",
    };
    let locked = s.lesson.session.feedback.is_some();
    col().w_full().gap(16.0)
        .child(text(title.to_uppercase()).key("exercise-title").font(rune::tok::TextStyle::new(12.0, 600.0)).color(p.blue))
        .child(text(prompt(ex).to_string()).font(rune::tok::TextStyle::new(20.0, 600.0)).color(p.text).wrap())
        .child(match ex {
            Exercise::Choice { options, .. } => choice(s, options, locked),
            Exercise::Command { .. } => command(s),
            Exercise::Order { steps, .. } => order(s, steps, locked),
            Exercise::Match { pairs, .. } => matching(s, pairs, locked),
        })
}

fn tile(key: String, label: El, on: bool, enabled: bool, press: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    let t = el().key(key).role("button").focusable(enabled).checked(on).px(16.0).py(14.0).radius(10.0)
        .fill(if on { p.blue_light } else { p.surface })
        .border(1.5, if on { p.blue } else { rune::color::hex("#D0D5DD").unwrap_or(p.border) })
        .child(label);
    if enabled { t.hover(p.text.alpha(0.03)).press_scale(0.99).focus_ring(2.0, 2.0, p.blue).on_press(press) } else { t }
}

fn choice(s: &State, options: &[String], locked: bool) -> El {
    let p = pal();
    let mut grid = col().w_full().gap(10.0);
    for (i, o) in options.iter().enumerate() {
        let label = row().gap(12.0).center_y()
            .child(el().size(26.0, 26.0).radius(6.0).border(1.0, p.border).center().child(text((i + 1).to_string()).font(SMALL).color(p.muted)))
            .child(text(o.clone()).font(MONO).color(p.text).wrap().grow(1.0));
        grid = grid.child(tile(format!("option-{i}"), label, s.lesson.choice == Some(i), !locked, move |s: &mut State| s.lesson.choice = Some(i)).w_full());
    }
    grid
}

fn command(s: &State) -> El {
    let p = pal();
    row().key("command-box").w_full().gap(10.0).center_y().px(18.0).py(14.0).radius(16.0).fill(rune::color::hex("#0E1B2A").unwrap_or(p.text))
        .child(text("$").font(MONO).color(rune::color::hex("#5FB2FF").unwrap_or(p.blue)))
        .child(
            text_input(&s.lesson.command, MONO).key("command").label("Tu comando").single_line().grow(1.0)
                .color(rune::color::hex("#F2F7FC").unwrap_or(Color::WHITE)).placeholder("escribe el comando aquí", rune::color::hex("#9FB6CC").unwrap_or(p.muted))
                .on_edit(|s: &mut State| &mut s.lesson.command)
                .on_submit(|s: &mut State, _| {
                    if s.lesson.session.feedback == Some(Feedback::Right) { next(s) } else if s.lesson.session.feedback.is_none() { check(s) }
                }),
        )
}

fn order(s: &State, steps: &[String], locked: bool) -> El {
    let p = pal();
    let display = shuffled(steps.len());
    let mut chosen = row().key("order-chosen").w_full().min_h(64.0).gap(10.0).flex_wrap().padding(12.0).radius(16.0).border(2.0, p.border);
    for (k, &d) in s.lesson.picked.iter().enumerate() {
        let label = text(format!("{}. {}", k + 1, steps[display[d]])).font(STRONG).color(p.text);
        chosen = chosen.child(tile(format!("picked-{k}"), label, true, !locked, move |s: &mut State| {
            s.lesson.picked.truncate(k);
        }));
    }
    let mut pool = row().w_full().gap(10.0).flex_wrap();
    for (d, &step) in display.iter().enumerate() {
        if s.lesson.picked.contains(&d) {
            continue;
        }
        pool = pool.child(tile(format!("step-{d}"), text(steps[step].clone()).font(STRONG).color(p.text), false, !locked, move |s: &mut State| s.lesson.picked.push(d)));
    }
    col().w_full().gap(14.0)
        .child(text("Tu orden (toca un paso para quitarlo desde ahí):").font(SMALL).color(p.muted))
        .child(chosen)
        .child(pool)
}

fn matching(s: &State, pairs: &[(String, String)], locked: bool) -> El {
    let p = pal();
    let display = shuffled(pairs.len());
    // Pair colors differ from the blue "selected" tile.
    let tints = ["#FFE2B8", "#CFF2DF", "#E7DBFF", "#FDE7E8", "#FFF3B0"];
    let tint = |n: usize| rune::color::hex(tints[n % tints.len()]).unwrap_or(p.blue_light);
    let pair_of_left = |l: usize| s.lesson.pairs.iter().position(|&(a, _)| a == l);
    let pair_of_right = |r: usize| s.lesson.pairs.iter().position(|&(_, b)| b == r);

    let mut left = col().w_frac(0.48).gap(10.0);
    for (i, (concept, _)) in pairs.iter().enumerate() {
        let selected = s.lesson.left == Some(i);
        let mut t = tile(format!("left-{i}"), text(concept.clone()).font(STRONG).color(p.text), selected, !locked, move |s: &mut State| {
            if let Some(k) = s.lesson.pairs.iter().position(|&(a, _)| a == i) {
                s.lesson.pairs.remove(k);
            }
            s.lesson.left = Some(i);
        });
        if let Some(n) = pair_of_left(i) {
            t = t.fill(tint(n));
        }
        left = left.child(t);
    }
    let mut right = col().w_frac(0.48).gap(10.0);
    for (d, &r) in display.iter().enumerate() {
        let mut t = tile(format!("right-{d}"), text(pairs[r].1.clone()).font(BODY).color(p.text).wrap(), false, !locked, move |s: &mut State| {
            let Some(l) = s.lesson.left.take() else { return };
            s.lesson.pairs.retain(|&(a, b)| a != l && b != r);
            s.lesson.pairs.push((l, r));
        });
        if let Some(n) = pair_of_right(r) {
            t = t.fill(tint(n));
        }
        right = right.child(t);
    }
    col().w_full().gap(12.0)
        .child(text("Toca un concepto y luego su significado.").font(SMALL).color(p.muted))
        .child(row().w_full().gap(20.0).items_start().child(left).child(spacer()).child(right))
}

fn footer(s: &State) -> El {
    let p = pal();
    let ready = current(s).and_then(|ex| answer(ex, &s.lesson)).is_some();
    let (fill, mood, content) = match &s.lesson.session.feedback {
        None => (
            p.bg,
            Mood::Neutral,
            row().w_full().gap(16.0).center_y()
                .child(text(if ready { "¿Lista tu respuesta? Compruébala." } else { "Elige una respuesta para comprobar." }).font(rune::tok::TextStyle::new(14.0, 400.0)).color(p.muted).grow(1.0))
                .child(if ready { primary("check", "Comprobar", check) } else { disabled("check", "Comprobar") }),
        ),
        Some(Feedback::Right) => (
            p.green_light,
            Mood::Happy,
            row().key("feedback-right").w_full().gap(16.0).center_y()
                .child(text("¡Correcto!").font(H2).color(p.green_dark).grow(1.0))
                .child(good("continue", "Continuar", next)),
        ),
        Some(Feedback::Wrong { hint }) => (
            p.red_light,
            Mood::Worried,
            row().key("feedback-wrong").w_full().gap(16.0).center_y()
                .child(
                    col().grow(1.0).gap(4.0)
                        .child(text("Casi. Inténtalo otra vez, sin límite.").font(STRONG).color(p.red_dark))
                        .child(text(format!("Pista: {hint}")).font(rune::tok::TextStyle::new(14.0, 400.0)).color(p.red_dark).wrap()),
                )
                .child(super::button("retry", "Reintentar", p.red_dark, p.red_dark, Color::WHITE, retry)),
        ),
    };
    row().key("lesson-footer").w_full().gap(14.0).center_y().fill(fill).border_sides([1.0, 0.0, 0.0, 0.0], p.border).px(24.0).py(16.0)
        .child(nubi_face(s, mood, 44.0).key("lesson-nubi"))
        .child(content.grow(1.0))
}

fn result(s: &State) -> El {
    let p = pal();
    let l = &s.lesson.session;
    let n = stars_for(l.errors);
    let title = if l.errors == 0 { "¡Lección perfecta!" } else { "¡Lección completada!" };
    let mut col_ = col().key("lesson-result").w_full().gap(16.0).center()
        .child(nubi_face(s, if l.errors == 0 { Mood::Happy } else { Mood::Neutral }, 120.0))
        .child(text(title).font(TITLE).color(p.text))
        .child(text(stars(n)).key("lesson-stars").font(rune::tok::TextStyle::new(56.0, 800.0)).color(p.orange))
        .child(text(if l.errors == 0 { "Cero errores: dominas esta lección.".to_string() } else { format!("Tuviste {}. 3 estrellas = cero errores.", plural(l.errors, "error", "errores")) }).font(BODY).color(p.muted));
    if !l.mistakes.is_empty() {
        let mut review = col().w_full().gap(8.0).padding(18.0).radius(10.0).fill(p.orange_light).border(1.0, rune::color::hex("#F9DBAF").unwrap_or(p.orange))
            .child(text("Para repasar").font(H2).color(p.text));
        for m in &l.mistakes {
            review = review.child(text(format!("• {m}")).font(STRONG).color(p.text).wrap());
        }
        col_ = col_.child(review);
    }
    let again = |s: &mut State| {
        let id = s.lesson.id.clone();
        s.open_stop(&id);
    };
    let mut buttons = row().gap(14.0);
    buttons = if l.errors > 0 { buttons.child(reward("perfect", "Perfeccionar", again)) } else { buttons.child(plain("again", "Repetir", again)) };
    if s.content.nodes.get(&s.lesson.id).is_some_and(|n| n.teaching.is_some()) {
        buttons = buttons.child(primary("lesson-to-lab", "Ir al laboratorio", |s: &mut State| s.lesson_to_lab()));
    }
    col_.child(buttons.child(plain("back-to-map", "Volver al inicio", |s: &mut State| s.to_map())))
        .child(text(if s.profile.is_some() { "Tu resultado quedó guardado." } else { "" }).font(SMALL).color(p.muted))
}
