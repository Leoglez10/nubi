//! One lesson attempt: one exercise at a time, unlimited retries, every wrong check counted
//! as an error (for the stars) and remembered (to review), never punished.

use crate::course::Exercise;
use crate::grading::{Answer, is_correct};

#[derive(Debug, Clone, PartialEq)]
pub enum Feedback {
    Right,
    Wrong { hint: String },
}

#[derive(Debug, Default)]
pub struct LessonSession {
    pub index: usize,
    pub total: usize,
    pub errors: u32,
    /// Prompts of the exercises the student missed at least once, in order.
    pub mistakes: Vec<String>,
    pub feedback: Option<Feedback>,
}

impl LessonSession {
    pub fn new(total: usize) -> LessonSession {
        LessonSession { total, ..LessonSession::default() }
    }

    /// Grades the current exercise.
    pub fn check(&mut self, exercise: &Exercise, answer: &Answer) -> &Feedback {
        if self.feedback != Some(Feedback::Right) {
            let feedback = if is_correct(exercise, answer) {
                Feedback::Right
            } else {
                self.errors += 1;
                let prompt = prompt(exercise).to_string();
                if !self.mistakes.contains(&prompt) {
                    self.mistakes.push(prompt);
                }
                Feedback::Wrong { hint: hint(exercise).to_string() }
            };
            self.feedback = Some(feedback);
        }
        self.feedback.as_ref().expect("just set")
    }

    /// After a wrong answer: back to answering the same exercise.
    pub fn retry(&mut self) {
        if matches!(self.feedback, Some(Feedback::Wrong { .. })) {
            self.feedback = None;
        }
    }

    /// After a right answer: on to the next exercise.
    pub fn next(&mut self) {
        if self.feedback == Some(Feedback::Right) {
            self.index += 1;
            self.feedback = None;
        }
    }

    pub fn finished(&self) -> bool {
        self.index >= self.total
    }

    /// 0.0 to 1.0, for the progress bar.
    pub fn progress(&self) -> f32 {
        if self.total == 0 { 1.0 } else { self.index.min(self.total) as f32 / self.total as f32 }
    }
}

/// Display order for items the student must arrange: never the answer order.
// ponytail: a rotation, not random; enough that the answer isn't shown as-is.
pub fn shuffled(n: usize) -> Vec<usize> {
    (0..n).map(|i| (i + 1) % n).collect()
}

pub fn prompt(exercise: &Exercise) -> &str {
    match exercise {
        Exercise::Choice { prompt, .. } | Exercise::Command { prompt, .. } | Exercise::Order { prompt, .. } | Exercise::Match { prompt, .. } => prompt,
    }
}

fn hint(exercise: &Exercise) -> &str {
    match exercise {
        Exercise::Choice { hint, .. } | Exercise::Command { hint, .. } | Exercise::Order { hint, .. } | Exercise::Match { hint, .. } => hint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(prompt: &str) -> Exercise {
        Exercise::Choice { prompt: prompt.into(), options: vec!["a".into(), "b".into()], answer: 1, hint: "piensa en b".into() }
    }

    #[test]
    fn a_wrong_answer_counts_an_error_shows_the_hint_and_allows_retrying() {
        let ex = choice("¿Cuál?");
        let mut l = LessonSession::new(2);
        assert_eq!(l.check(&ex, &Answer::Choice(0)), &Feedback::Wrong { hint: "piensa en b".into() });
        assert_eq!(l.errors, 1);
        l.retry();
        assert_eq!(l.feedback, None);
        l.check(&ex, &Answer::Choice(0));
        assert_eq!(l.errors, 2, "every wrong check counts");
        assert_eq!(l.mistakes, vec!["¿Cuál?".to_string()], "but the exercise is listed once");
        l.retry();
        assert_eq!(l.check(&ex, &Answer::Choice(1)), &Feedback::Right);
        assert_eq!(l.index, 0, "stays until the student moves on");
    }

    #[test]
    fn moving_on_needs_a_right_answer_and_ends_the_lesson() {
        let ex = choice("?");
        let mut l = LessonSession::new(2);
        l.next();
        assert_eq!(l.index, 0, "no skipping past an unanswered exercise");
        l.check(&ex, &Answer::Choice(1));
        l.next();
        assert_eq!((l.index, l.feedback.clone()), (1, None));
        assert_eq!(l.progress(), 0.5);
        l.check(&ex, &Answer::Choice(1));
        l.next();
        assert!(l.finished());
        assert_eq!(l.progress(), 1.0);
    }

    #[test]
    fn checking_again_after_right_changes_nothing() {
        let ex = choice("?");
        let mut l = LessonSession::new(1);
        l.check(&ex, &Answer::Choice(1));
        l.check(&ex, &Answer::Choice(0));
        assert_eq!((l.errors, l.feedback.clone()), (0, Some(Feedback::Right)));
    }

    #[test]
    fn shuffled_items_never_appear_in_answer_order() {
        for n in 2..8 {
            let order = shuffled(n);
            let mut sorted = order.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (0..n).collect::<Vec<_>>(), "a permutation of {n}");
            assert_ne!(order, sorted, "not the answer order for {n}");
        }
        assert_eq!(shuffled(1), vec![0]);
        assert!(shuffled(0).is_empty());
    }
}
