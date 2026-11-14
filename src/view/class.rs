//! A class: the explanation as an article with drawings, and beside it the class's parts and
//! a tip from Nubi.
use labcloud::course::{Diagram, Teaching};
use rune::color::hex;
use rune::{Color, El, ScrollSpec, col, el, row, text};
use super::{BODY, H2, MONO, SMALL, STRONG, TITLE, disabled, nubi, pal, plain, primary, stars};
use crate::State;

pub fn view(s: &State) -> El {
    let Some(id) = s.class_id.clone() else { return el() };
    let Some(teaching) = s.content.nodes.get(&id).and_then(|n| n.teaching.as_ref()) else { return el() };
    let body = row().key("class").w_full().max_w(1180.0).gap(24.0).padding(28.0).items_start().flex_wrap()
        .child(article(s, teaching))
        .child(aside(s, &id, teaching));
    col().w_full().h_full().child(el().w_full().grow(1.0).scroll(ScrollSpec { stick_bottom: false }).child(row().w_full().justify_center().child(body)))
}

fn article(s: &State, teaching: &Teaching) -> El {
    let p = pal();
    let label = if s.class_review { "REPASO" } else { "EXPLICACIÓN" };
    let mut a = col().key("class-content").grow(1.0).min_w(520.0).gap(18.0).padding(32.0).radius(12.0).fill(p.surface).border(1.0, p.border)
        .child(col().gap(6.0)
            .child(text(format!("CLASE {} · {label}", teaching.number)).font(rune::tok::TextStyle::new(12.0, 600.0)).color(p.blue))
            .child(text(teaching.title.clone()).key("class-title").font(TITLE).color(p.text).wrap()))
        .child(text(teaching.explanation.clone()).font(rune::tok::TextStyle::new(16.0, 400.0)).color(hex("#344054").unwrap_or(p.text)).wrap());
    a = a.child(match teaching.diagram {
        Some(d) => diagram_view(d, &teaching.visual),
        None => el().key("class-visual").w_full().padding(16.0).radius(10.0).fill(p.blue_light)
            .child(text(teaching.visual.clone()).font(STRONG).color(p.blue_dark).wrap()),
    });
    a = a.child(text("Ejemplo resuelto").font(H2).color(p.text))
        .child(text(teaching.example.clone()).font(BODY).color(hex("#344054").unwrap_or(p.text)).wrap());
    if let Some(d) = teaching.diagram {
        a = a.child(example_view(d));
    }
    a = a.child(review_card(&teaching.review));

    let mut actions = row().w_full().gap(10.0).pt(6.0).flex_wrap();
    if s.class_return.is_some() {
        actions = actions.child(primary("class-back", "Volver a mi práctica", |s: &mut State| s.return_to_attempt()));
    } else {
        let lab = if s.class_lab_ready() { primary("class-lab", "Ir al laboratorio", |s: &mut State| s.start_class_lab()) }
            else { disabled("class-lab", "Laboratorio: termina la práctica") };
        actions = actions.child(primary("class-games", "Empezar práctica", |s: &mut State| s.start_class_lesson()))
            .child(lab)
            .child(plain("class-back", "Volver al inicio", |s: &mut State| s.to_map()));
    }
    a.child(actions)
}

/// The class's parts, where each stands, and Nubi's tip.
fn aside(s: &State, id: &str, teaching: &Teaching) -> El {
    let p = pal();
    let best = |id: &str| s.progress.nodes.get(id).map(|r| r.best_stars);
    let practice = best(id);
    let lab = best(&teaching.lab);
    let steps = [
        ("Explicación", String::new(), true),
        ("Práctica", practice.map_or("0 / 5".to_string(), stars), practice.is_some()),
        ("Laboratorio", lab.map_or(if s.class_lab_ready() { "Listo".to_string() } else { "Bloqueado".to_string() }, stars), lab.is_some()),
        ("Repaso", String::new(), lab.is_some()),
    ];
    let mut card = col().key("class-steps").w(280.0).gap(4.0).padding(18.0).radius(12.0).fill(p.surface).border(1.0, p.border)
        .child(text("En esta clase").font(rune::tok::TextStyle::new(13.0, 600.0)).color(p.muted).pb(8.0));
    for (i, (label, meta, done)) in steps.into_iter().enumerate() {
        let now = i == 0 && !s.class_review;
        let (dot, ink) = if done && !now { (p.green, Color::WHITE) } else if now { (p.blue, Color::WHITE) } else { (p.locked, p.muted) };
        let mut line = row().w_full().gap(10.0).center_y().px(10.0).py(9.0).radius(8.0)
            .child(el().size(22.0, 22.0).radius(11.0).fill(dot).center()
                .child(text(if done && !now { "✓".to_string() } else { (i + 1).to_string() }).font(rune::tok::TextStyle::new(12.0, 600.0)).color(ink)))
            .child(text(label).font(rune::tok::TextStyle::new(14.0, if now { 600.0 } else { 500.0 })).color(if now { p.blue_dark } else { hex("#344054").unwrap_or(p.text) }).grow(1.0))
            .child(text(meta).font(rune::tok::TextStyle::new(12.0, 500.0)).color(hex("#667085").unwrap_or(p.muted)));
        if now { line = line.fill(p.blue_light); }
        card = card.child(line);
    }
    card.child(row().key("class-nubi").w_full().gap(10.0).items_start().mt(12.0).padding(12.0).radius(10.0).fill(p.bg).border(1.0, p.border)
        .child(nubi(s, 40.0))
        .child(col().grow(1.0).gap(2.0)
            .child(text("Tip de Nubi").font(rune::tok::TextStyle::new(13.0, 600.0)).color(p.text))
            .child(text(teaching.visual.clone()).font(rune::tok::TextStyle::new(13.0, 400.0)).color(hex("#344054").unwrap_or(p.text)).wrap())))
}

/// A drawn idea plus its caption, so the explanation is not only words.
fn diagram_view(diagram: Diagram, caption: &str) -> El {
    let p = pal();
    let art = match diagram {
        Diagram::BucketObjects => bucket_objects(),
        Diagram::KeyPrefixes => key_prefixes(),
    };
    col().key("class-diagram").w_full().radius(10.0).border(1.0, p.border).clip_rounded(10.0)
        .child(el().w_full().padding(16.0).fill(p.bg).child(art))
        .child(text(caption.to_string()).font(SMALL).color(p.muted).wrap().w_full().px(16.0).py(10.0)
            .border_sides([1.0, 0.0, 0.0, 0.0], p.border))
}

/// One bucket with three objects inside: the bucket holds keys, not folders.
fn bucket_objects() -> El {
    let p = pal();
    let mut objects = row().gap(10.0).flex_wrap();
    for name in ["foto.jpg", "informe.txt", "video.mp4"] {
        objects = objects.child(
            col().key(format!("diagram-object-{name}")).gap(2.0).px(14.0).py(10.0).radius(8.0)
                .fill(p.surface).border(1.0, p.border)
                .child(text(name).font(MONO).color(p.text))
                .child(text(format!("clave: {name}")).font(rune::tok::TextStyle::new(12.0, 500.0)).color(p.muted)),
        );
    }
    col().key("diagram-bucket").w_full().gap(12.0).padding(16.0).radius(10.0).fill(p.surface).border(1.0, p.blue)
        .child(row().gap(8.0).center_y()
            .child(el().size(28.0, 28.0).radius(6.0).fill(p.blue).center().child(text("S3").font(rune::tok::TextStyle::new(12.0, 700.0)).color(Color::WHITE)))
            .child(text("Bucket: mi-bucket").font(STRONG).color(p.text)))
        .child(objects)
}

/// A flat key read left to right: the prefixes only look like folders.
fn key_prefixes() -> El {
    let p = pal();
    let parts = [("BUCKET", "archivo-nubi", p.blue_light, p.blue_dark), ("PREFIJO", "informes/", p.orange_light, p.orange_dark), ("NOMBRE", "enero.txt", p.surface, p.muted)];
    let mut strip = row().key("diagram-key").w_full().radius(8.0).border(1.0, p.border).clip_rounded(8.0).flex_wrap();
    for (role, part, fill, ink) in parts {
        strip = strip.child(col().grow(1.0).min_w(140.0).gap(4.0).padding(14.0).fill(fill)
            .child(text(role).font(rune::tok::TextStyle::new(11.0, 600.0)).color(ink))
            .child(text(part).font(MONO).color(p.text)));
    }
    col().key("diagram-prefixes").w_full().child(strip)
}

/// The worked example drawn as what you type and what you get.
fn example_view(diagram: Diagram) -> El {
    let p = pal();
    let term = hex("#0C111D").unwrap_or(p.text);
    let (command, output, looks, is): (&str, &str, &[&str], &[&str]) = match diagram {
        Diagram::BucketObjects => ("aws s3api create-bucket --bucket mi-bucket", "{ \"Location\": \"/mi-bucket\" }",
            &["mi-bucket/", "  (vacío)"], &["Bucket creado, listo para objetos"]),
        Diagram::KeyPrefixes => ("aws s3 cp enero.txt s3://archivo-nubi/informes/enero.txt", "upload: ./enero.txt to s3://archivo-nubi/informes/enero.txt",
            &["informes/", "  ├ enero.txt", "  └ febrero.txt"], &["informes/enero.txt", "informes/febrero.txt"]),
    };
    let side = |key: &str, title: &str, lines: &[&str]| {
        let mut c = col().key(key.to_string()).grow(1.0).min_w(220.0).gap(4.0).padding(14.0).radius(10.0).border(1.0, p.border)
            .child(text(title.to_string()).font(rune::tok::TextStyle::new(13.0, 600.0)).color(p.muted).pb(4.0));
        for l in lines { c = c.child(text(l.to_string()).font(MONO).color(p.text)); }
        c
    };
    col().key("class-example-visual").w_full().gap(12.0)
        .child(col().w_full().gap(6.0).padding(16.0).radius(10.0).fill(term)
            .child(row().gap(8.0).child(text("$").font(MONO).color(hex("#7CD4FD").unwrap_or(p.blue))).child(text(command).font(MONO).color(hex("#E4E7EC").unwrap_or(Color::WHITE)).wrap().grow(1.0)))
            .child(text(output).font(MONO).color(hex("#98A2B3").unwrap_or(p.muted)).wrap()))
        .child(row().w_full().gap(12.0).flex_wrap()
            .child(side("example-looks", "Lo que muestra la consola", looks))
            .child(side("example-is", "Lo que S3 guarda", is)))
}

/// The takeaway with a check, easy to spot when revisiting.
fn review_card(review: &str) -> El {
    let p = pal();
    row().key("class-review-visual").w_full().gap(12.0).items_start().padding(14.0).radius(10.0).fill(p.green_light).border(1.0, hex("#ABEFC6").unwrap_or(p.green))
        .child(el().size(22.0, 22.0).radius(11.0).fill(p.green_dark).center().child(text("✓").font(rune::tok::TextStyle::new(12.0, 700.0)).color(Color::WHITE)))
        .child(col().grow(1.0).gap(2.0)
            .child(text("Para recordar").font(STRONG).color(hex("#054F31").unwrap_or(p.green_dark)))
            .child(text(review.to_string()).font(rune::tok::TextStyle::new(14.0, 400.0)).color(hex("#054F31").unwrap_or(p.green_dark)).wrap()))
}
