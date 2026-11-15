//! The screens. Light cloud palette (white and blue, orange for rewards); all copy in Spanish.

mod intro;
mod class;
mod motion;
mod lab;
mod lesson;
mod map;
mod profiles;
mod settings;
mod setup;
mod shell;

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
        bg: c("#F7F8FA"),
        sky: c("#EFF4FF"),
        surface: c("#FFFFFF"),
        border: c("#E4E7EC"),
        text: c("#101828"),
        muted: c("#475467"),
        blue: c("#1D4ED8"),
        blue_dark: c("#1E3A8A"),
        blue_light: c("#EFF4FF"),
        orange: c("#F79009"),
        orange_dark: c("#B54708"),
        orange_light: c("#FEF6EE"),
        ink_on_orange: c("#FFFFFF"),
        green: c("#17B26A"),
        green_dark: c("#067647"),
        green_light: c("#ECFDF3"),
        red: c("#D92D20"),
        red_dark: c("#B42318"),
        red_light: c("#FEF3F2"),
        locked: c("#F2F4F7"),
    }
}

pub const HERO: TextStyle = TextStyle::new(40.0, 700.0);
pub const TITLE: TextStyle = TextStyle::new(24.0, 700.0);
pub const H2: TextStyle = TextStyle::new(18.0, 600.0);
pub const BODY: TextStyle = TextStyle::new(15.0, 400.0);
pub const STRONG: TextStyle = TextStyle::new(14.0, 600.0);
pub const SMALL: TextStyle = TextStyle::new(13.0, 500.0);
pub const MONO: TextStyle = TextStyle::new(14.0, 400.0).mono();

pub fn view(s: &State, cx: &mut Cx<State>) -> El {
    s.motion.system_reduced.set(cx.system_reduced_motion());
    cx.reduced_motion(s.reduced_motion || cx.system_reduced_motion());
    let shown = s.motion.displayed(s.screen);
    let screen = match shown {
        Screen::Splash => intro::view(s),
        Screen::Setup => setup::view(s),
        Screen::Profiles => profiles::view(s),
        Screen::Map => shell::wrap(s, shown, map::view(s)),
        Screen::Class => shell::wrap(s, shown, class::view(s)),
        Screen::Lesson => shell::wrap(s, shown, lesson::view(s)),
        Screen::Lab => shell::wrap(s, shown, lab::view(s)),
        Screen::Settings if s.profile.is_some() => shell::wrap(s, shown, settings::view(s)),
        Screen::Settings => settings::view(s),
    };
    motion::wrap(s, screen)
}

/// What every pressable control shares: a hover tint, a small press and a focus ring.
fn pressable(key: &str, label: &str, tint: Color, on: impl Fn(&mut State) + 'static) -> El {
    el().key(key.to_string()).role("button").label(label.to_string()).focusable(true)
        .hover(tint).press_scale(0.98).focus_ring(2.0, 2.0, pal().blue)
        .on_press(on)
}

/// A flat button: solid fill, 1px edge, 8px corners.
pub fn button(key: &str, label: &str, fill: Color, edge: Color, ink: Color, on: impl Fn(&mut State) + 'static) -> El {
    pressable(key, label, Color::BLACK.alpha(0.08), on)
        .h(44.0).px(18.0).radius(8.0).fill(fill).border(1.0, edge).center()
        .child(text(label.to_string()).font(STRONG).color(ink))
}

pub fn primary(key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(key, label, p.blue, p.blue, Color::WHITE, on)
}

pub fn reward(key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(key, label, p.orange_dark, p.orange_dark, p.ink_on_orange, on)
}

pub fn good(key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    button(key, label, p.green_dark, p.green_dark, Color::WHITE, on)
}

pub fn plain(key: &str, label: &str, on: impl Fn(&mut State) + 'static) -> El {
    let p = pal();
    pressable(key, label, p.text.alpha(0.04), on)
        .h(44.0).px(18.0).radius(8.0).fill(p.surface).border(1.0, hex("#D0D5DD").unwrap_or(p.border)).center()
        .child(text(label.to_string()).font(STRONG).color(hex("#344054").unwrap_or(p.text)))
}

/// A button that cannot be pressed yet.
pub fn disabled(key: &str, label: &str) -> El {
    let p = pal();
    el().key(key.to_string()).role("button").label(label.to_string())
        .h(44.0).px(18.0).radius(8.0).fill(p.locked).border(1.0, p.border).center()
        .child(text(label.to_string()).font(STRONG).color(hex("#667085").unwrap_or(p.muted)))
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

/// How Nubi reacts to an answer.
#[derive(Clone, Copy, PartialEq)]
pub enum Mood { Neutral, Happy, Worried }

/// Nubi's face for a mood, still (it sits beside text the student is reading).
pub fn nubi_face(s: &State, mood: Mood, w: f32) -> El {
    let img = s.art.as_ref().map(|a| match mood { Mood::Neutral => &a.nubi, Mood::Happy => &a.nubi_happy, Mood::Worried => &a.nubi_worried });
    picture(img, w, w * 0.8).label("Nubi")
}

/// A vertical page that scrolls when the window is short, fading its edge so more content
/// reads as more content even while the system hides scroll bars at rest.
pub fn page_scroll(content: El) -> El {
    rune::col().w_full().h_full()
        .child(el().key("page-scroll").w_full().grow(1.0).min_h(0.0).scroll(rune::ScrollSpec { stick_bottom: false }).scroll_edge_fade(24.0)
            .child(content))
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
        el().w_full().h_full().center().child(primary("test-button", "Continuar", |s| s.intro_started = true))
    }

    #[test]
    fn hover_tints_without_moving_and_press_sinks() {
        let mut rt = Runtime::new(State::default(), button_view, (960.0, 680.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        let (x, y) = rt.find("test-button").unwrap();
        rt.pointer_move(x, y);
        for _ in 0..20 { rt.frame(1.0 / 60.0); }
        assert_eq!(rt.presented_pose("test-button").unwrap().scale(), 1.0, "a dashboard button tints on hover, it does not grow");
        rt.pointer_down(x, y);
        for _ in 0..20 { rt.frame(1.0 / 60.0); }
        assert!(rt.presented_pose("test-button").unwrap().scale() < 1.0, "press sinks below the resting state");
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.intro_started);
    }

    #[test]
    fn keyboard_skips_disabled_and_activates_the_button() {
        fn controls(_: &State, _: &mut Cx<State>) -> El {
            rune::col().w_full().h_full()
                .child(disabled("disabled-test", "No disponible"))
                .child(primary("enabled-test", "Continuar", |s| s.intro_started = true))
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

    fn click(rt: &mut Runtime<State>, key: &str) {
        let (x, y) = rt.find(key).unwrap_or_else(|| panic!("missing {key}"));
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
    }

    #[test]
    fn reduced_motion_lives_in_settings_and_returns_where_you_were() {
        let s = State { screen: Screen::Profiles, profiles: Some(Vec::new()), ..State::default() };
        let mut rt = Runtime::new(s, view, (1180.0, 760.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        assert!(rt.find("motion-toggle").is_none(), "no floating motion button");
        click(&mut rt, "open-settings");
        assert_eq!(rt.state.screen, Screen::Settings);
        click(&mut rt, "motion-toggle");
        assert!(rt.state.reduced_motion);
        click(&mut rt, "motion-toggle");
        assert!(!rt.state.reduced_motion);
        click(&mut rt, "settings-back");
        assert_eq!(rt.state.screen, Screen::Profiles);
    }

    #[test]
    fn settings_is_in_the_sidebar_once_you_have_a_profile() {
        let s = State {
            screen: Screen::Map,
            profile: Some(labcloud::progress::Profile { id: "alex".into(), name: "Alex".into(), color: "#BFE0FB".into(), ..Default::default() }),
            ..State::default()
        };
        let mut rt = Runtime::new(s, view, (1280.0, 800.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        assert!(rt.find("motion-toggle").is_none());
        assert!(rt.find("nav-settings").is_none(), "account options stay folded until asked");
        click(&mut rt, "account-menu");
        click(&mut rt, "nav-settings");
        assert_eq!(rt.state.screen, Screen::Settings);
        assert!(rt.find("sidebar").is_some() && rt.find("motion-toggle").is_some());
        click(&mut rt, "settings-back");
        assert_eq!(rt.state.screen, Screen::Map);
    }

    fn signed_in(screen: Screen) -> (Runtime<State>, std::sync::mpsc::Receiver<crate::Job>) {
        let (tx, rx) = std::sync::mpsc::channel();
        let s = State {
            screen,
            jobs: Some(tx),
            profile: Some(labcloud::progress::Profile { id: "alex".into(), name: "Alex".into(), color: "#BFE0FB".into(), ..Default::default() }),
            ..State::default()
        };
        let mut rt = Runtime::new(s, view, (1280.0, 1200.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        (rt, rx)
    }

    #[test]
    fn the_account_menu_switches_profile_or_signs_out() {
        let (mut rt, _rx) = signed_in(Screen::Map);
        click(&mut rt, "account");
        assert!(rt.state.account_open, "the profile row opens the account menu");
        click(&mut rt, "account-switch");
        assert_eq!(rt.state.screen, Screen::Profiles);
        assert!(rt.state.profile.is_some(), "switching keeps you signed in until you pick another");

        let (mut rt, rx) = signed_in(Screen::Map);
        rt.update(|s, _| { s.progress.record("s3-buckets", 0); });
        click(&mut rt, "account-menu");
        click(&mut rt, "account-signout");
        assert_eq!(rt.state.screen, Screen::Profiles);
        assert!(rt.state.profile.is_none() && rt.state.progress.total_stars() == 0, "nothing of the profile stays in memory");
        assert!(rx.try_iter().any(|j| matches!(j, crate::Job::SignOut)), "the worker stops saving to it");
    }

    #[test]
    fn settings_edits_the_profile_and_saves_motion_with_it() {
        let (mut rt, rx) = signed_in(Screen::Map);
        rt.update(|s, _| s.open_settings());
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.edit_name.text, "Alex");
        rt.update(|s, _| { s.edit_name = rune::TextEdit::new("Alexa"); s.edit_color = 1; });
        rt.frame(1.0 / 60.0);
        click(&mut rt, "save-profile");
        click(&mut rt, "motion-toggle");
        let jobs: Vec<_> = rx.try_iter().collect();
        assert!(jobs.iter().any(|j| matches!(j, crate::Job::UpdateProfile(p) if p.name == "Alexa" && p.color != "#BFE0FB")));
        assert!(jobs.iter().any(|j| matches!(j, crate::Job::UpdateProfile(p) if p.reduced_motion)), "motion is saved with the profile");
        assert!(rt.state.reduced_motion);
    }

    #[test]
    fn deleting_a_profile_asks_first() {
        let (mut rt, rx) = signed_in(Screen::Map);
        rt.update(|s, _| s.open_settings());
        rt.frame(1.0 / 60.0);
        click(&mut rt, "delete-profile");
        assert!(rt.state.confirm_delete);
        click(&mut rt, "cancel-delete");
        assert!(!rt.state.confirm_delete);
        assert!(!rx.try_iter().any(|j| matches!(j, crate::Job::DeleteProfile(_))), "cancel deletes nothing");
        click(&mut rt, "delete-profile");
        click(&mut rt, "confirm-delete");
        assert!(rx.try_iter().any(|j| matches!(j, crate::Job::DeleteProfile(id) if id == "alex")));
    }

    #[test]
    fn a_tab_change_slides_in_the_content_and_keeps_the_sidebar_still() {
        let (mut rt, _rx) = signed_in(Screen::Map);
        rt.update(|s, _| { crate::motion::tick(s, 0.0); s.open_settings(); crate::motion::tick(s, 1.0); crate::motion::tick(s, 1.0 + 1.0 / 60.0); });
        rt.frame(1.0 / 60.0);
        let (sidebar, sidebar_opacity) = rt.presented("sidebar").expect("sidebar");
        let (start, start_opacity) = rt.presented("shell-content").expect("content");
        assert_eq!(sidebar_opacity, 1.0, "the sidebar never fades");
        assert!(start_opacity < 0.05, "the new content starts transparent: {start_opacity}");
        rt.update(|s, _| { crate::motion::tick(s, 1.0 + 1.0 / 60.0 + crate::motion::REVEAL + 0.01); });
        rt.frame(2.0 / 60.0);
        let (sidebar_end, _) = rt.presented("sidebar").unwrap();
        let (end, end_opacity) = rt.presented("shell-content").unwrap();
        assert_eq!(end_opacity, 1.0);
        assert!(start[1] - end[1] > 6.0, "the content rises into place: {start:?} -> {end:?}");
        assert_eq!(sidebar, sidebar_end, "the sidebar does not move");
    }

    #[test]
    fn screens_without_a_sidebar_slide_in_whole() {
        let mut s = State { screen: Screen::Profiles, ..State::default() };
        crate::motion::tick(&mut s, 0.0);
        s.screen = Screen::Setup;
        crate::motion::tick(&mut s, 1.0);
        crate::motion::tick(&mut s, 1.0 + 1.0 / 60.0);
        let mut rt = Runtime::new(s, view, (1180.0, 760.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        assert!(rt.find("setup-card").is_some(), "the new screen is already up");
        let (_, opacity) = rt.presented("app-frame").unwrap();
        assert_eq!(opacity, 1.0, "the frame itself stays; only its screen fades in");
    }
}
