//! Grading by mastery: no lives, no XP. Answers are right or wrong, a finished stop earns
//! stars from its error count, and the level comes from the total stars.

use crate::course::Exercise;

/// What the student submitted for one exercise. Indices always refer to the order in the
/// content file (the UI maps its shuffled display back to these).
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    Choice(usize),
    Command(String),
    /// Step indices in the order the student placed them.
    Order(Vec<usize>),
    /// `(left, right)` pairs the student joined.
    Match(Vec<(usize, usize)>),
}

pub fn is_correct(exercise: &Exercise, answer: &Answer) -> bool {
    match (exercise, answer) {
        (Exercise::Choice { answer: right, .. }, Answer::Choice(got)) => got == right,
        (Exercise::Command { answers, .. }, Answer::Command(got)) => {
            let got = normalize(got);
            answers.iter().any(|a| normalize(a) == got)
        }
        (Exercise::Order { steps, .. }, Answer::Order(got)) => got.len() == steps.len() && got.iter().enumerate().all(|(i, &s)| i == s),
        (Exercise::Match { pairs, .. }, Answer::Match(got)) => {
            let mut lefts: Vec<usize> = got.iter().map(|&(l, _)| l).collect();
            lefts.sort_unstable();
            lefts.dedup();
            got.len() == pairs.len() && lefts.len() == pairs.len() && got.iter().all(|&(l, r)| l == r && l < pairs.len())
        }
        _ => false,
    }
}

fn normalize(command: &str) -> String {
    command.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 3 stars = perfect (0 errors), 2 = few errors (1–2), 1 = completed.
pub fn stars(errors: u32) -> u8 {
    match errors {
        0 => 3,
        1 | 2 => 2,
        _ => 1,
    }
}

/// Stars needed to reach each level, and its name.
const LEVELS: [(u32, &str); 6] = [
    (0, "Despegando"),
    (5, "Explorador"),
    (15, "Constructor de nubes"),
    (30, "Arquitecto"),
    (50, "Arquitecto senior"),
    (75, "Maestro de la nube"),
];

#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub number: u32,
    pub name: &'static str,
    /// Stars where this level starts.
    pub from: u32,
    /// Stars needed for the next level, or `None` at the top level.
    pub next_at: Option<u32>,
}

pub fn level(total_stars: u32) -> Level {
    let i = LEVELS.iter().rposition(|&(min, _)| total_stars >= min).unwrap_or(0);
    Level { number: i as u32 + 1, name: LEVELS[i].1, from: LEVELS[i].0, next_at: LEVELS.get(i + 1).map(|&(min, _)| min) }
}

/// A stop is open when it is the first one or the stop before it is completed.
pub fn is_unlocked(order: &[&str], id: &str, completed: impl Fn(&str) -> bool) -> bool {
    match order.iter().position(|&o| o == id) {
        Some(0) => true,
        Some(i) => completed(order[i - 1]),
        None => false,
    }
}

/// How a stop looks on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// Completed, with its best stars.
    Done(u8),
    /// Open and not completed yet: the next place to play.
    Current,
    Locked,
}

pub fn stop_status(order: &[&str], id: &str, stars_of: impl Fn(&str) -> Option<u8>) -> Stop {
    match stars_of(id) {
        Some(stars) if stars > 0 => Stop::Done(stars),
        _ if is_unlocked(order, id, |prev| stars_of(prev).is_some_and(|s| s > 0)) => Stop::Current,
        _ => Stop::Locked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_shows_done_current_and_locked_stops() {
        let order = ["a", "b", "c"];
        let stars = |id: &str| match id {
            "a" => Some(2),
            _ => None,
        };
        assert_eq!(stop_status(&order, "a", stars), Stop::Done(2));
        assert_eq!(stop_status(&order, "b", stars), Stop::Current);
        assert_eq!(stop_status(&order, "c", stars), Stop::Locked);
        assert_eq!(stop_status(&order, "a", |_| None), Stop::Current, "the first stop is open from the start");
    }

    fn choice() -> Exercise {
        Exercise::Choice { prompt: "?".into(), options: vec!["a".into(), "b".into()], answer: 1, hint: "h".into() }
    }

    #[test]
    fn a_choice_is_right_only_with_its_answer() {
        assert!(is_correct(&choice(), &Answer::Choice(1)));
        assert!(!is_correct(&choice(), &Answer::Choice(0)));
        assert!(!is_correct(&choice(), &Answer::Command("b".into())), "wrong answer kind");
    }

    #[test]
    fn a_command_ignores_extra_whitespace_but_not_other_differences() {
        let ex = Exercise::Command {
            prompt: "?".into(),
            answers: vec!["aws s3 cp hola.txt s3://fotos/".into(), "aws s3 cp hola.txt s3://fotos".into()],
            hint: "h".into(),
        };
        assert!(is_correct(&ex, &Answer::Command("  aws  s3 cp   hola.txt s3://fotos/ ".into())));
        assert!(is_correct(&ex, &Answer::Command("aws s3 cp hola.txt s3://fotos".into())), "any listed answer");
        assert!(!is_correct(&ex, &Answer::Command("aws s3 cp hola.txt s3://otro/".into())));
        assert!(!is_correct(&ex, &Answer::Command("AWS S3 CP hola.txt s3://fotos/".into())), "commands are case sensitive");
    }

    #[test]
    fn an_order_is_right_only_in_the_file_order() {
        let ex = Exercise::Order { prompt: "?".into(), steps: vec!["1".into(), "2".into(), "3".into()], hint: "h".into() };
        assert!(is_correct(&ex, &Answer::Order(vec![0, 1, 2])));
        assert!(!is_correct(&ex, &Answer::Order(vec![1, 0, 2])));
        assert!(!is_correct(&ex, &Answer::Order(vec![0, 1])), "every step must be placed");
    }

    #[test]
    fn a_match_is_right_only_when_every_pair_is_joined_correctly() {
        let pairs = vec![("a".into(), "1".into()), ("b".into(), "2".into())];
        let ex = Exercise::Match { prompt: "?".into(), pairs, hint: "h".into() };
        assert!(is_correct(&ex, &Answer::Match(vec![(1, 1), (0, 0)])), "order of joining does not matter");
        assert!(!is_correct(&ex, &Answer::Match(vec![(0, 1), (1, 0)])));
        assert!(!is_correct(&ex, &Answer::Match(vec![(0, 0)])), "every pair must be joined");
        assert!(!is_correct(&ex, &Answer::Match(vec![(0, 0), (0, 0)])), "no duplicates");
    }

    #[test]
    fn stars_reward_perfection_without_punishing() {
        assert_eq!(stars(0), 3);
        assert_eq!(stars(1), 2);
        assert_eq!(stars(2), 2);
        assert_eq!(stars(3), 1);
        assert_eq!(stars(40), 1, "finishing always earns at least one star");
    }

    #[test]
    fn the_level_comes_from_total_stars() {
        assert_eq!(level(0), Level { number: 1, name: "Despegando", from: 0, next_at: Some(5) });
        assert_eq!(level(10), Level { number: 2, name: "Explorador", from: 5, next_at: Some(15) });
        assert_eq!(level(15).number, 3);
        assert_eq!(level(1000).next_at, None);
    }

    #[test]
    fn a_stop_opens_when_the_one_before_is_completed() {
        let order = ["a", "b", "c"];
        let done = |id: &str| id == "a";
        assert!(is_unlocked(&order, "a", done), "the first stop is always open");
        assert!(is_unlocked(&order, "b", done));
        assert!(!is_unlocked(&order, "c", done));
        assert!(!is_unlocked(&order, "zzz", done), "unknown stops are closed");
    }
}
