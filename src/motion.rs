//! Clock-driven screen transitions; navigation remains owned by `State::screen`.
use std::cell::Cell;

use crate::{Screen, State};

/// Cover and reveal durations in seconds. Short enough to feel immediate.
const COVER: f64 = 0.22;
const REVEAL: f64 = 0.26;

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
    fn skipped_clock_still_presents_full_cover_before_reveal() {
        let mut s = State::default();
        tick(&mut s, 0.0);
        s.screen = Screen::Setup;
        tick(&mut s, 1.0);
        tick(&mut s, 5.0);
        assert_eq!(s.motion.displayed(s.screen), Screen::Setup);
        assert_eq!(s.motion.cover, 1.0);
        tick(&mut s, 5.15);
        assert!(s.motion.cover > 0.0 && s.motion.cover < 1.0);
        tick(&mut s, 5.31);
        assert_eq!(s.motion.cover, 0.0);
        assert!(!tick(&mut s, 5.32));
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
    fn rapid_navigation_during_reveal_waits_for_another_cover() {
        let mut s = State { screen: Screen::Profiles, ..State::default() };
        tick(&mut s, 0.0);
        s.screen = Screen::Setup;
        tick(&mut s, 1.0);
        tick(&mut s, 1.31);
        assert_eq!(s.motion.displayed(s.screen), Screen::Setup);
        s.screen = Screen::Map;
        tick(&mut s, 1.4);
        assert_eq!(s.motion.displayed(s.screen), Screen::Setup);
        tick(&mut s, 1.61);
        tick(&mut s, 1.62);
        tick(&mut s, 1.93);
        assert_eq!(s.motion.displayed(s.screen), Screen::Map);
    }

    #[test]
    fn navigation_covers_before_switching_screens() {
        let mut s = State { screen: Screen::Profiles, ..State::default() };
        tick(&mut s, 0.0);
        s.screen = Screen::Setup;
        assert!(tick(&mut s, 1.0), "navigation starts a transition");
        let mut rt = Runtime::new(s, view::view, (1180.0, 760.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(1.0);
        assert!(rt.find("profiles").is_some(), "old screen remains while covering");
        rt.update(|s, _| { tick(s, 1.15); });
        rt.frame(1.15);
        assert!(rt.find("profiles").is_some());
        rt.update(|s, _| { tick(s, 1.4); });
        rt.frame(1.4);
        assert!(rt.find("setup-card").is_some(), "new screen appears once covered");
    }
}
