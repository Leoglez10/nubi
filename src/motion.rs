//! Clock-driven screen transitions; navigation remains owned by `State::screen`.
use std::cell::Cell;

use crate::{Screen, State};

/// Cover and reveal durations in seconds. The new screen is swapped in on the next frame and
/// fades in briefly, so a change of tab feels instant without a hard flash.
pub const COVER: f64 = 0.0;
pub const REVEAL: f64 = 0.08;

/// Smoothstep: no visible stop at the ends of the transition.
fn smooth(t: f64) -> f32 {
    let t = t.clamp(0.0, 1.0);
    (t * t * (3.0 - 2.0 * t)) as f32
}

#[derive(Default)]
pub struct Motion {
    displayed: Option<Screen>,
    started: Option<f64>,
    switched: bool,
    pub system_reduced: Cell<bool>,
    pub cover: f32,
}

impl Motion {
    pub fn displayed(&self, fallback: Screen) -> Screen {
        self.displayed.unwrap_or(fallback)
    }
}

pub fn tick(s: &mut State, now: f64) -> bool {
    let m = &mut s.motion;
    let Some(displayed) = m.displayed else {
        m.displayed = Some(s.screen);
        return false;
    };
    if s.reduced_motion || m.system_reduced.get() {
        let changed = displayed != s.screen || m.started.is_some() || m.cover != 0.0;
        m.displayed = Some(s.screen);
        m.started = None;
        m.switched = false;
        m.cover = 0.0;
        return changed;
    }
    if m.started.is_none() && displayed != s.screen {
        m.started = Some(now);
        return true;
    }
    let Some(started) = m.started else { return false };
    let elapsed = (now - started).max(0.0);
    if elapsed < COVER {
        m.cover = smooth(elapsed / COVER);
    } else if !m.switched {
        // Present at least one fully covered frame, even if the clock skipped ahead.
        m.displayed = Some(s.screen);
        m.switched = true;
        m.cover = 1.0;
        m.started = Some(now - COVER);
    } else if elapsed < COVER + REVEAL {
        m.cover = smooth(1.0 - (elapsed - COVER) / REVEAL);
    } else {
        m.cover = 0.0;
        m.started = None;
        m.switched = false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view;
    use rune::{Env, Runtime};

    #[test]
    fn a_screen_change_is_done_within_a_tenth_of_a_second() {
        const { assert!(COVER + REVEAL <= 0.1, "tab changes must feel instant") };
        let mut s = State::default();
        tick(&mut s, 0.0);
        s.screen = Screen::Setup;
        tick(&mut s, 1.0);
        tick(&mut s, 1.0 + 1.0 / 60.0);
        assert_eq!(s.motion.displayed(s.screen), Screen::Setup, "swapped on the next frame");
        tick(&mut s, 1.0 + 1.0 / 60.0 + REVEAL / 2.0);
        assert!(s.motion.cover > 0.0 && s.motion.cover < 1.0, "fading in");
        tick(&mut s, 1.0 + 1.0 / 60.0 + REVEAL + 0.01);
        assert_eq!(s.motion.cover, 0.0);
        assert!(!tick(&mut s, 1.2));
    }

    #[test]
    fn skipped_clock_still_presents_full_cover_before_reveal() {
        let mut s = State::default();
        tick(&mut s, 0.0);
        s.screen = Screen::Setup;
        tick(&mut s, 1.0);
        tick(&mut s, 5.0);
        assert_eq!(s.motion.displayed(s.screen), Screen::Setup);
        assert_eq!(s.motion.cover, 1.0);
        tick(&mut s, 5.0 + REVEAL / 2.0);
        assert!(s.motion.cover > 0.0 && s.motion.cover < 1.0);
        tick(&mut s, 5.0 + REVEAL + 0.01);
        assert_eq!(s.motion.cover, 0.0);
        assert!(!tick(&mut s, 5.5));
    }

    #[test]
    fn reduced_motion_finishes_immediately_for_user_or_system() {
        for system in [false, true] {
            let mut s = State::default();
            tick(&mut s, 0.0);
            s.screen = Screen::Setup;
            tick(&mut s, 1.0);
            if system {
                s.motion.system_reduced.set(true);
            } else {
                s.reduced_motion = true;
            }
            assert!(tick(&mut s, 1.1));
            assert_eq!(s.motion.displayed(s.screen), Screen::Setup);
            assert_eq!(s.motion.cover, 0.0);
            assert!(!tick(&mut s, 1.2));
        }
    }

    #[test]
    fn rapid_navigation_during_reveal_still_lands_on_the_last_screen() {
        let mut s = State { screen: Screen::Profiles, ..State::default() };
        tick(&mut s, 0.0);
        s.screen = Screen::Setup;
        tick(&mut s, 1.0);
        tick(&mut s, 1.01);
        assert_eq!(s.motion.displayed(s.screen), Screen::Setup);
        s.screen = Screen::Map;
        tick(&mut s, 1.02);
        for i in 1..20 { tick(&mut s, 1.02 + i as f64 * 0.02); }
        assert_eq!(s.motion.displayed(s.screen), Screen::Map);
        assert_eq!(s.motion.cover, 0.0);
    }

    #[test]
    fn the_old_screen_stays_for_one_frame_at_most() {
        let mut s = State { screen: Screen::Profiles, ..State::default() };
        tick(&mut s, 0.0);
        s.screen = Screen::Setup;
        assert!(tick(&mut s, 1.0), "navigation starts a transition");
        let mut rt = Runtime::new(s, view::view, (1180.0, 760.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(1.0);
        rt.update(|s, _| { tick(s, 1.0 + 1.0 / 60.0); });
        rt.frame(1.0 + 1.0 / 60.0);
        assert!(rt.find("setup-card").is_some(), "the new screen is up on the next frame");
    }
}
