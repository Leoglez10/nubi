//! Teaching and review for a guided class, with a small diagram for the idea.
use labcloud::course::Diagram;
use rune::{Color, El, ScrollSpec, col, el, row, spacer, text};
use super::{BODY, H2, MONO, SMALL, STRONG, TITLE, disabled, pal, plain, primary};
use crate::State;

pub fn view(s: &State) -> El {
    let p = pal();
    let Some(teaching) = s.class_id.as_ref().and_then(|id| s.content.nodes.get(id)).and_then(|n| n.teaching.as_ref()) else { return el() };
    let back = if s.class_return.is_some() {
        plain("class-back", "Volver a mi práctica", |s: &mut State| s.return_to_attempt())
    } else {
        plain("class-back", "Volver al mapa", |s: &mut State| s.to_map())
    };
    let heading = if s.class_review { "Repaso de la clase" } else { "Primero, aprendamos" };
    let mut content = col().key("class-content").w_full().max_w(900.0).gap(14.0).padding(24.0)
        .child(text(format!("Clase {}: {}", teaching.number, teaching.title)).key("class-title").font(TITLE).color(p.text).wrap())
        .child(text(heading).font(H2).color(p.blue))
        .child(text(teaching.explanation.clone()).font(BODY).color(p.text).wrap());
    if let Some(diagram) = teaching.diagram {
        content = content.child(diagram_view(diagram, &teaching.visual));
    } else {
        content = content.child(el().key("class-visual").w_full().padding(18.0).radius(16.0).fill(p.blue_light)
            .child(text(teaching.visual.clone()).font(STRONG).color(p.blue_dark).wrap()));
    }
    content = content
        .child(text("Ejemplo resuelto").font(H2).color(p.text))
        .child(text(teaching.example.clone()).font(BODY).color(p.text).wrap());
    if let Some(diagram) = teaching.diagram {
        content = content.child(example_view(diagram));
    }
    content = content
        .child(text("Para recordar").font(H2).color(p.text))
        .child(review_card(&teaching.review));
    if s.class_return.is_none() {
        let lab = if s.class_lab_ready() { primary("class-lab", "Ir al laboratorio", |s: &mut State| s.start_class_lab()) }
            else { disabled("class-lab", "Completa los juegos primero") };
        content = content.child(row().w_full().gap(12.0).flex_wrap()
            .child(primary("class-games", "Empezar juegos", |s: &mut State| s.start_class_lesson())).child(lab))
            .child(text("Completa los juegos para desbloquear el laboratorio.").font(SMALL).color(p.muted));
    }
    col().key("class").w_full().h_full().fill(p.bg)
        .child(row().w_full().gap(12.0).center_y().padding(16.0).fill(p.surface).child(back).child(spacer())
            .child(text("EXPLICACIÓN · JUEGOS · LABORATORIO · REPASO").font(SMALL).color(p.muted)))
        .child(el().w_full().grow(1.0).scroll(ScrollSpec { stick_bottom: false }).child(content))
}

/// A drawn idea plus its caption, so the explanation is not only words.
fn diagram_view(diagram: Diagram, caption: &str) -> El {
    let p = pal();
    let art = match diagram {
        Diagram::BucketObjects => bucket_objects(),
        Diagram::KeyPrefixes => key_prefixes(),
    };
    col().key("class-diagram").w_full().gap(10.0).padding(18.0).radius(18.0).fill(p.blue_light).border(2.0, p.border)
        .child(art)
        .child(text(caption.to_string()).font(STRONG).color(p.blue_dark).wrap())
}

/// The worked example drawn as what you type and what you get.
fn example_view(diagram: Diagram) -> El {
    let p = pal();
    let (command, result) = match diagram {
        Diagram::BucketObjects => ("aws s3api create-bucket --bucket mi-bucket", empty_bucket()),
        Diagram::KeyPrefixes => ("aws s3 cp enero.txt s3://archivo-nubi/informes/enero.txt", looks_vs_is()),
    };
    col().key("class-example-visual").w_full().gap(10.0).padding(18.0).radius(18.0).fill(p.surface).border(2.0, p.border)
        .child(text("Escribes").font(SMALL).color(p.muted))
        .child(el().w_full().padding(12.0).radius(12.0).fill(p.text)
            .child(text(format!("$ {command}")).font(MONO).color(Color::WHITE).wrap()))
        .child(text("↓  y obtienes").font(SMALL).color(p.muted))
        .child(result)
}

/// The new bucket: it exists, but holds nothing yet.
fn empty_bucket() -> El {
    let p = pal();
    col().w_full().gap(6.0).padding(16.0).radius(16.0).fill(p.blue_light).border(3.0, p.blue).center()
        .child(text("mi-bucket").font(H2).color(p.text))
        .child(text("vacío · listo para guardar objetos").font(SMALL).color(p.blue_dark))
}

/// What the console shows (folders) next to what S3 stores (flat keys).
fn looks_vs_is() -> El {
    let p = pal();
    let card = |title: &str, lines: &[&str], fill: Color| {
        let mut c = col().grow(1.0).min_w(240.0).gap(6.0).padding(14.0).radius(14.0).fill(fill).border(2.0, p.border)
            .child(text(title.to_string()).font(STRONG).color(p.text));
        for line in lines {
            c = c.child(text(line.to_string()).font(MONO).color(p.text));
        }
        c
    };
    row().w_full().gap(12.0).flex_wrap()
        .child(card("Lo que parece", &["📁 informes/", "    enero.txt", "    febrero.txt"], p.orange_light))
        .child(card("Lo que S3 guarda", &["informes/enero.txt", "informes/febrero.txt"], p.blue_light))
}

/// The takeaway as a card with a check, easy to spot when revisiting.
fn review_card(review: &str) -> El {
    let p = pal();
    row().key("class-review-visual").w_full().gap(14.0).center_y().padding(16.0).radius(16.0).fill(p.green_light).border(2.0, p.green)
        .child(el().size(36.0, 36.0).radius(18.0).fill(p.green).center().child(text("✓").font(H2).color(Color::WHITE)))
        .child(text(review.to_string()).font(BODY).color(p.text).wrap().grow(1.0))
}

/// One bucket with three objects inside: the bucket holds keys, not folders.
fn bucket_objects() -> El {
    let p = pal();
    let mut objects = row().gap(10.0).flex_wrap().center();
    for (name, key) in [("foto.jpg", "foto.jpg"), ("informe.txt", "informe.txt"), ("video.mp4", "video.mp4")] {
        objects = objects.child(
            col().key(format!("diagram-object-{name}")).gap(2.0).px(14.0).py(10.0).radius(12.0)
                .fill(p.surface).border(2.0, p.blue).center()
                .child(text(name).font(STRONG).color(p.text))
                .child(text(format!("clave: {key}")).font(SMALL).color(p.muted)),
        );
    }
    col().key("diagram-bucket").w_full().gap(10.0).padding(16.0).radius(16.0).fill(p.surface).border(3.0, p.blue)
        .child(row().gap(8.0).center_y()
            .child(el().size(34.0, 34.0).radius(8.0).fill(p.blue).center().child(text("S3").font(SMALL).color(Color::WHITE)))
            .child(text("Bucket: mi-bucket").font(H2).color(p.text)))
        .child(objects)
}

/// A flat key read left to right: the prefixes only look like folders.
fn key_prefixes() -> El {
    let p = pal();
    let parts = [("archivo-nubi/", "bucket"), ("informes/", "prefijo"), ("enero.txt", "clave del objeto")];
    let mut rows = col().key("diagram-prefixes").w_full().gap(8.0);
    for (part, role) in parts {
        rows = rows.child(
            row().w_full().gap(12.0).center_y().padding(12.0).radius(12.0).fill(p.surface).border(2.0, p.border)
                .child(text(part).font(rune::tok::TextStyle::new(18.0, 700.0).mono()).color(p.text).grow(1.0))
                .child(el().px(10.0).py(4.0).radius(10.0).fill(p.blue_light).child(text(role).font(SMALL).color(p.blue_dark))),
        );
    }
    col().key("diagram-key").w_full().gap(10.0).child(rows)
}
