//! The screens. Light cloud palette (white and blue, orange for rewards); all copy in Spanish.

mod intro;
mod class;
mod motion;
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

pub fn view(s: &State, cx: &mut Cx<State>) -> El {
    s.motion.system_reduced.set(cx.system_reduced_motion());
    cx.reduced_motion(s.reduced_motion || cx.system_reduced_motion());
    let screen = match s.motion.displayed(s.screen) {
        Screen::Splash => intro::view(s),
        Screen::Setup => setup::view(s),
        Screen::Profiles => profiles::view(s),
        Screen::Map => map::view(s),
        Screen::Class => class::view(s),
        Screen::Lesson => lesson::view(s),
        Screen::Lab => lab::view(s),
    };
    motion::wrap(s, screen)
}

/// A chunky game button: solid fill with a darker bottom edge.
pub fn button(s: &State, key: &str, label: &str, fill: Color, edge: Color, ink: Color, on: impl Fn(&mut State) + 'static) -> El {
    let scale = s.motion.hover_scale(key);
    if s.reduced_motion || s.motion.system_reduced.get() { scale.set(1.0); }
    let on_hover = scale.clone();
    el().key(key.to_string()).role("button").label(label.to_string()).focusable(true)
        .h(50.0).px(24.0).radius(16.0).fill(fill).shadow(4.0, 0.0, edge).center()
        .origin(0.5, 1.0).live_scale(scale.smooth(rune::Spring::new(0.16, 0.55)))
        .on_hover_change(move |s: &mut State, hovered, tx| {
            let reduced = s.reduced_motion || s.motion.system_reduced.get();
            on_hover.set(if hovered && !reduced { 1.03 } else { 1.0 });
            tx.live_only();
        })
        .press_scale(0.96)
        .on_press(on)
        .child(text(label.to_string()).font(STRONG).color(ink))
}

pub fn primary(s: &State, key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(s, key, label, p.blue, p.blue_dark, Color::WHITE, on)
}

pub fn reward(s: &State, key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(s, key, label, p.orange, p.orange_dark, p.ink_on_orange, on)
}

pub fn good(s: &State, key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(s, key, label, p.green, p.green_dark, Color::WHITE, on)
}

pub fn plain(s: &State, key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(s, key, label, p.surface, p.border, p.muted, on).border(2.0, p.border)
}

/// A button that cannot be pressed yet.
pub fn disabled(key: &str, label: &str) -> El {
    let p = pal();
    el().key(key.to_string()).role("button").label(label.to_string())
        .h(50.0).px(24.0).radius(16.0).fill(p.locked).center()
        .child(text(label.to_string()).font(STRONG).color(p.muted))
}

/// "✕" back to the map.
pub fn close(s: &State, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(s, "close", "✕", p.surface, p.border, p.muted, on)
        .label("Volver al mapa").size(44.0, 44.0).px(0.0).radius(22.0)
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

#[cfg(test)]
mod motion_tests {
    use super::*;
    use rune::{Env, Runtime};

    fn button_view(s: &State, cx: &mut Cx<State>) -> El {
        cx.reduced_motion(s.reduced_motion || s.motion.system_reduced.get());
        el().w_full().h_full().center().child(primary(s, "test-button", "Continuar", |s| s.intro_started = true))
    }

    #[test]
    fn hover_and_press_physically_depress_the_button() {
        let mut rt = Runtime::new(State::default(), button_view, (960.0, 680.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        let (x, y) = rt.find("test-button").unwrap();
        let rest = rt.presented_pose("test-button").unwrap().scale();
        rt.pointer_move(x, y);
        rt.frame(0.04);
        let intermediate = rt.presented_pose("test-button").unwrap().scale();
        for _ in 0..40 { rt.frame(1.0 / 60.0); }
        let hover = rt.presented_pose("test-button").unwrap().scale();
        assert!(hover > intermediate && intermediate > rest, "hover must lift gradually: {rest} {intermediate} {hover}");
        rt.pointer_down(x, y);
        for _ in 0..20 { rt.frame(1.0 / 60.0); }
        assert!(rt.presented_pose("test-button").unwrap().scale() < rest, "press sinks below the resting state");
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.intro_started);
    }

    #[test]
    fn keyboard_skips_disabled_and_activates_the_button() {
        fn controls(s: &State, _: &mut Cx<State>) -> El {
            rune::col().w_full().h_full()
                .child(disabled("disabled-test", "No disponible"))
                .child(primary(s, "enabled-test", "Continuar", |s| s.intro_started = true))
        }
        let mut rt = Runtime::new(State::default(), controls, (960.0, 680.0), Env { system_dark: false, scale: 1.0 });
        rt.set_tab_visits_controls(true);
        rt.frame(0.0);
        let (x, y) = rt.find("disabled-test").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        assert!(!rt.state.intro_started);
        rt.key(rune::Key::Tab.into());
        rt.frame(1.0 / 60.0);
        rt.key(rune::Key::Text(" ".into()).into());
        rt.frame(1.0 / 60.0);
        assert!(rt.state.intro_started);
    }

    #[test]
    fn hover_survives_an_unrelated_view_rebuild() {
        let mut rt = Runtime::new(State::default(), button_view, (960.0, 680.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        let (x, y) = rt.find("test-button").unwrap();
        rt.pointer_move(x, y);
        for _ in 0..30 { rt.frame(1.0 / 60.0); }
        rt.update(|s, _| s.busy = Some("Comprobando".into()));
        for _ in 0..30 { rt.frame(1.0 / 60.0); }
        assert!(rt.presented_pose("test-button").unwrap().scale() > 1.001);
    }

    #[test]
    fn reduced_motion_prevents_hover_and_press_travel() {
        for system in [false, true] {
            let s = State { reduced_motion: !system, ..State::default() };
            s.motion.system_reduced.set(system);
            let mut rt = Runtime::new(s, button_view, (960.0, 680.0), Env { system_dark: false, scale: 1.0 });
            rt.frame(0.0);
            let (x, y) = rt.find("test-button").unwrap();
            rt.pointer_move(x, y);
            rt.pointer_down(x, y);
            for _ in 0..30 { rt.frame(1.0 / 60.0); }
            assert_eq!(rt.presented_pose("test-button").unwrap().scale(), 1.0);
            rt.pointer_up(x, y);
            rt.frame(1.0 / 60.0);
            assert!(rt.state.intro_started);
        }
    }

    #[test]
    fn toggle_changes_the_user_preference() {
        let mut rt = Runtime::new(State::default(), view, (960.0, 680.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        let (x, y) = rt.find("motion-toggle").unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.reduced_motion);
    }

    #[test]
    fn curtain_and_reduced_motion_control_are_global() {
        for size in [(960.0, 680.0), (1180.0, 760.0)] {
            let mut s = State { screen: Screen::Profiles, ..State::default() };
            crate::motion::tick(&mut s, 0.0);
            s.screen = Screen::Setup;
            crate::motion::tick(&mut s, 1.0);
            crate::motion::tick(&mut s, 1.15);
            let mut rt = Runtime::new(s, view, size, Env { system_dark: false, scale: 1.0 });
            rt.frame(0.0);
            assert!(rt.find("motion-toggle").is_some());
            assert!(rt.find("cloud-curtain").is_some());
            rt.update(|s, _| { crate::motion::tick(s, 1.23); });
            rt.frame(1.0 / 60.0);
            let (rect, opacity) = rt.presented("cloud-cover").expect("opaque full cover");
            assert_eq!(opacity, 1.0);
            assert!(rt.find("setup-card").is_some());
            // The inset frame keeps the cover inside the 9pt app padding.
            assert!(rect[0] <= 1.0 && rect[1] <= 1.0 && rect[2] >= size.0 - 20.0 && rect[3] >= size.1 - 20.0, "{rect:?} {size:?}");
        }
    }
}
