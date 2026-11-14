//! The screens. Light cloud palette (white and blue, orange for rewards); all copy in Spanish.

mod intro;
mod lab;
mod lesson;
mod map;
mod profiles;
mod setup;

use rune::color::hex;
use rune::tok::TextStyle;
use rune::{Color, Cx, El, Easing, Image, LoopSpec, el, text};

use crate::{Screen, State};

#[cfg(test)]
pub use lesson::answer;

pub struct Pal {
    pub bg: Color,
    pub sky: Color,
    pub surface: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub blue: Color,
    pub blue_dark: Color,
    pub blue_light: Color,
    pub orange: Color,
    pub orange_dark: Color,
    pub orange_light: Color,
    pub ink_on_orange: Color,
    pub green: Color,
    pub green_dark: Color,
    pub green_light: Color,
    pub red: Color,
    pub red_dark: Color,
    pub red_light: Color,
    pub locked: Color,
}

pub fn pal() -> Pal {
    let c = |h: &str| hex(h).unwrap_or(Color::BLACK);
    Pal {
        bg: c("#EEF5FC"),
        sky: c("#BFE0FB"),
        surface: c("#FFFFFF"),
        border: c("#D6E4F2"),
        text: c("#0F2A44"),
        muted: c("#51657A"),
        blue: c("#1F6FD1"),
        blue_dark: c("#154F99"),
        blue_light: c("#DCEBFB"),
        orange: c("#FF9900"),
        orange_dark: c("#C27400"),
        orange_light: c("#FFF1DB"),
        ink_on_orange: c("#232F3E"),
        green: c("#22A06B"),
        green_dark: c("#17784F"),
        green_light: c("#E3F6EC"),
        red: c("#D93A3F"),
        red_dark: c("#A3262B"),
        red_light: c("#FDE7E8"),
        locked: c("#E1E9F2"),
    }
}

pub const HERO: TextStyle = TextStyle::new(56.0, 900.0);
pub const TITLE: TextStyle = TextStyle::new(28.0, 800.0);
pub const H2: TextStyle = TextStyle::new(19.0, 800.0);
pub const BODY: TextStyle = TextStyle::new(16.0, 500.0);
pub const STRONG: TextStyle = TextStyle::new(15.0, 700.0);
pub const SMALL: TextStyle = TextStyle::new(13.0, 700.0);
pub const MONO: TextStyle = TextStyle::new(14.0, 400.0).mono();

pub fn view(s: &State, _: &mut Cx<State>) -> El {
    match s.screen {
        Screen::Splash => intro::view(s),
        Screen::Setup => setup::view(s),
        Screen::Profiles => profiles::view(s),
        Screen::Map => map::view(s),
        Screen::Lesson => lesson::view(s),
        Screen::Lab => lab::view(s),
    }
}

/// A chunky game button: solid fill with a darker bottom edge.
pub fn button(key: &str, label: &str, fill: Color, edge: Color, ink: Color, on: impl Fn(&mut State) + 'static) -> El {
    el().key(key.to_string()).role("button").label(label.to_string()).focusable(true)
        .h(50.0).px(24.0).radius(16.0).fill(fill).shadow(4.0, 0.0, edge).center()
        .on_press(on)
        .child(text(label.to_string()).font(STRONG).color(ink))
}

pub fn primary(key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(key, label, p.blue, p.blue_dark, Color::WHITE, on)
}

pub fn reward(key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(key, label, p.orange, p.orange_dark, p.ink_on_orange, on)
}

pub fn good(key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(key, label, p.green, p.green_dark, Color::WHITE, on)
}

pub fn plain(key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(key, label, p.surface, p.border, p.muted, on).border(2.0, p.border)
}

/// A button that cannot be pressed yet.
pub fn disabled(key: &str, label: &str) -> El {
    let p = pal();
    el().key(key.to_string()).role("button").label(label.to_string())
        .h(50.0).px(24.0).radius(16.0).fill(p.locked).center()
        .child(text(label.to_string()).font(STRONG).color(p.muted))
}

/// "✕" back to the map.
pub fn close(on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    el().key("close").role("button").label("Volver al mapa").focusable(true).size(44.0, 44.0).radius(22.0).center()
        .on_press(on)
        .child(text("✕").font(H2).color(p.muted))
}

pub fn stars(n: u8) -> String {
    (0..3).map(|i| if i < n { '★' } else { '☆' }).collect()
}

pub fn plural(n: u32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// An image, or an empty box of the same size until the art is ready (or in tests).
pub fn picture(image: Option<&Image>, w: f32, h: f32) -> El {
    match image {
        Some(img) => el().size(w, h).pixels(img.pixels()),
        None => el().size(w, h),
    }
}

/// Nubi, the mascot, floating gently.
pub fn nubi(s: &State, w: f32) -> El {
    let float = LoopSpec::slide(3.2, [0.0, 0.0], [0.0, -0.06]).easing(Easing::Piecewise(&[[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]]));
    picture(s.art.as_ref().map(|a| &a.nubi), w, w * 0.8).label("Nubi, la mascota").looping(float)
}

/// A speech bubble next to Nubi.
pub fn bubble(content: El) -> El {
    let p = pal();
    el().px(18.0).py(14.0).radius(18.0).fill(p.surface).border(2.0, p.border).child(content)
}

pub fn progress_bar(key: &str, fraction: f32, fill: Color, height: f32) -> El {
    let p = pal();
    el().key(key.to_string()).w_full().h(height).radius(height / 2.0).fill(p.locked)
        .child(el().w_frac(fraction.clamp(0.0, 1.0)).h(height).radius(height / 2.0).fill(fill))
}
