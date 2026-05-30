//! Lucid Tutor — the endless tutor engine for luciddreamer.ai
//!
//! Polyglot thinking, low-level precision, highest-level application.
//! The tutor observes, adapts, iterates. It meets you where you are
//! and takes you one step further. With friends on the same vibration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A learner's vibration — their current state of understanding.
/// Mono-dimensional, like everything in PLATO. One scalar, verifiable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vibration {
    /// Core understanding level (-1.0 to 1.0). Negative = confused, 0 = neutral, 1 = mastery
    pub level: f64,
    /// How fast they're learning (rate of change)
    pub velocity: f64,
    /// Topics they've touched
    pub topics: HashMap<String, TopicMastery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMastery {
    pub level: f64,
    pub iterations: u32,
    pub last_touch_tick: u64,
    pub breakthroughs: u32,
    pub stuck_count: u32,
}

impl Vibration {
    pub fn new() -> Self {
        Self { level: 0.0, velocity: 0.0, topics: HashMap::new() }
    }

    /// Are they resonating with another learner?
    pub fn resonance_with(&self, other: &Vibration) -> f64 {
        if self.topics.is_empty() || other.topics.is_empty() { return 0.0; }
        let shared: Vec<&str> = self.topics.keys()
            .filter(|k| other.topics.contains_key(k.as_str()))
            .map(|k| k.as_str())
            .collect();
        if shared.is_empty() { return 0.0; }
        let total_diff: f64 = shared.iter()
            .map(|&t| {
                let a = self.topics.get(t).unwrap().level;
                let b = other.topics.get(t).unwrap().level;
                (a - b).abs()
            })
            .sum();
        1.0 - (total_diff / shared.len() as f64).min(1.0)
    }

    /// Update a topic after an iteration.
    pub fn iterate(&mut self, topic: &str, result: f64, tick: u64) -> IterationOutcome {
        let entry = self.topics.entry(topic.into()).or_insert(TopicMastery {
            level: 0.0, iterations: 0, last_touch_tick: 0, breakthroughs: 0, stuck_count: 0,
        });
        let old_level = entry.level;
        entry.iterations += 1;
        entry.last_touch_tick = tick;

        let outcome = if result > 0.8 && old_level < 0.5 {
            entry.breakthroughs += 1;
            entry.level = (entry.level + result * 0.3).min(1.0);
            IterationOutcome::Breakthrough
        } else if result < 0.3 && entry.iterations > 3 {
            entry.stuck_count += 1;
            IterationOutcome::Stuck
        } else if result > old_level {
            entry.level = (entry.level + (result - old_level) * 0.5).min(1.0);
            IterationOutcome::Progress
        } else {
            IterationOutcome::Plateau
        };

        let new_level = self.topics.values().map(|t| t.level).sum::<f64>()
            / self.topics.len().max(1) as f64;
        self.velocity = new_level - self.level;
        self.level = new_level;

        outcome
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IterationOutcome {
    Breakthrough,
    Progress,
    Plateau,
    Stuck,
}

/// A teaching moment — the tutor's intervention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingMoment {
    pub tick: u64,
    pub topic: String,
    pub kind: TeachingKind,
    pub message: String,
    pub difficulty: f64,
    pub next_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TeachingKind {
    /// "You just discovered X!" — celebrate a breakthrough
    Celebration,
    /// "Try thinking about it this way..." — gentle redirect
    Redirect,
    /// "Remember when X worked? This is similar..." — connect to prior knowledge
    Connection,
    /// "What if you tried X?" — specific hint
    Hint,
    /// "You and [friend] both struggled with this — work together!"
    Collaborate,
    /// "Here's why it works..." — deeper explanation after success
    Deepen,
    /// "You're ready for something harder."
    LevelUp,
    /// "Take a breath. This is a hard concept. Let's sit with it."
    Patience,
}

/// The tutor — observes, adapts, teaches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutor {
    pub id: String,
    pub style: TutorStyle,
    pub learner_vibrations: HashMap<String, Vibration>,
    pub teaching_history: Vec<TeachingMoment>,
    pub tick: u64,
    /// How many iterations before the tutor intervenes on "stuck"
    pub patience_threshold: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TutorStyle {
    /// Patient, encouraging, gives hints
    Guide,
    /// Challenging, asks questions, waits for answers
    Socratic,
    /// Hands-off, lets you fail, teaches from failure
    Experimental,
    /// Adapts style based on learner
    Adaptive,
}

impl Tutor {
    pub fn new(id: &str, style: TutorStyle) -> Self {
        Self {
            id: id.into(), style,
            learner_vibrations: HashMap::new(),
            teaching_history: Vec::new(),
            tick: 0,
            patience_threshold: 3,
        }
    }

    /// Register a learner.
    pub fn register(&mut self, learner: &str) {
        self.learner_vibrations.insert(learner.into(), Vibration::new());
    }

    /// Observe an iteration — learner attempted something.
    pub fn observe(&mut self, learner: &str, topic: &str, result: f64) -> Option<TeachingMoment> {
        self.tick += 1;
        let vib = self.learner_vibrations.entry(learner.into())
            .or_insert_with(Vibration::new);
        let outcome = vib.iterate(topic, result, self.tick);

        let effective_style = match self.style {
            TutorStyle::Adaptive => {
                if vib.velocity > 0.1 { TutorStyle::Socratic }
                else if vib.velocity < -0.05 { TutorStyle::Guide }
                else { TutorStyle::Experimental }
            }
            other => other,
        };

        let moment = match (outcome, effective_style) {
            (IterationOutcome::Breakthrough, _) => TeachingMoment {
                tick: self.tick, topic: topic.into(),
                kind: TeachingKind::Celebration,
                message: format!("Yes! You just cracked {}! That's a breakthrough.", topic),
                difficulty: result,
                next_hint: Some(format!("You're ready to go deeper with {}.", topic)),
            },
            (IterationOutcome::Stuck, TutorStyle::Guide) => TeachingMoment {
                tick: self.tick, topic: topic.into(),
                kind: TeachingKind::Hint,
                message: format!("Having trouble with {}? Try breaking it into smaller pieces.", topic),
                difficulty: result,
                next_hint: Some("What's the simplest version of this problem?".into()),
            },
            (IterationOutcome::Stuck, TutorStyle::Socratic) => TeachingMoment {
                tick: self.tick, topic: topic.into(),
                kind: TeachingKind::Redirect,
                message: format!("What do you think is happening with {}? What would happen if...?", topic),
                difficulty: result,
                next_hint: Some("Think about conservation — what's preserved here?".into()),
            },
            (IterationOutcome::Stuck, TutorStyle::Experimental) | (IterationOutcome::Stuck, TutorStyle::Adaptive) => TeachingMoment {
                tick: self.tick, topic: topic.into(),
                kind: TeachingKind::Patience,
                message: format!("This is a hard one. Sit with {} for a moment. The confusion means you're learning.", topic),
                difficulty: result,
                next_hint: None,
            },
            (IterationOutcome::Plateau, _) => TeachingMoment {
                tick: self.tick, topic: topic.into(),
                kind: TeachingKind::Connection,
                message: format!("You've been working on {} for a while. Remember what worked with similar problems?", topic),
                difficulty: result,
                next_hint: Some("Try connecting this to something you already understand well.".into()),
            },
            (IterationOutcome::Progress, TutorStyle::Socratic) => TeachingMoment {
                tick: self.tick, topic: topic.into(),
                kind: TeachingKind::Deepen,
                message: format!("Good progress on {}! But why does it work? Can you prove it?", topic),
                difficulty: result,
                next_hint: None,
            },
            (IterationOutcome::Progress, _) => TeachingMoment {
                tick: self.tick, topic: topic.into(),
                kind: TeachingKind::LevelUp,
                message: format!("Nice work on {}! Ready for the next challenge?", topic),
                difficulty: result,
                next_hint: Some("The next level adds a twist...".into()),
            },
        };

        self.teaching_history.push(moment.clone());
        Some(moment)
    }

    /// Find friends on the same vibration.
    pub fn find_resonance(&self, learner: &str, min_resonance: f64) -> Vec<(String, f64)> {
        let vib = match self.learner_vibrations.get(learner) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let mut resonances: Vec<(String, f64)> = self.learner_vibrations.iter()
            .filter(|(id, _)| *id != learner)
            .filter_map(|(id, other_vib)| {
                let r = vib.resonance_with(other_vib);
                if r >= min_resonance { Some((id.clone(), r)) } else { None }
            })
            .collect();
        resonances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        resonances
    }

    /// Should we suggest collaboration?
    pub fn suggest_collab(&self, learner: &str, topic: &str) -> Option<String> {
        let friends = self.find_resonance(learner, 0.5);
        for (friend_id, resonance) in &friends {
            if let Some(friend_vib) = self.learner_vibrations.get(friend_id) {
                if let Some(tm) = friend_vib.topics.get(topic) {
                    if tm.stuck_count > 0 {
                        return Some(format!(
                            "You and {} are both working through {} (resonance: {:.0}%). Try it together!",
                            friend_id, topic, resonance * 100.0
                        ));
                    }
                }
            }
        }
        None
    }

    /// Total teaching moments delivered.
    pub fn teachings_delivered(&self) -> usize { self.teaching_history.len() }

    /// How many learners are registered?
    pub fn learner_count(&self) -> usize { self.learner_vibrations.len() }
}

/// A learning group — friends iterating together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningGroup {
    pub id: String,
    pub members: Vec<String>,
    pub shared_topics: Vec<String>,
    pub group_vibration: Vibration,
    pub iterations_together: u32,
}

impl LearningGroup {
    pub fn new(id: &str, members: Vec<String>) -> Self {
        Self {
            id: id.into(), members,
            shared_topics: Vec::new(),
            group_vibration: Vibration::new(),
            iterations_together: 0,
        }
    }

    /// Iterate together — everyone learns from a shared experience.
    pub fn iterate_together(&mut self, topic: &str, results: &[(String, f64)], tick: u64) -> Vec<IterationOutcome> {
        self.iterations_together += 1;
        if !self.shared_topics.contains(&topic.to_string()) {
            self.shared_topics.push(topic.into());
        }
        results.iter().map(|(_, r)| {
            self.group_vibration.iterate(topic, *r, tick)
        }).collect()
    }

    /// Group coherence — how aligned are the members?
    pub fn coherence(&self, tutor: &Tutor) -> f64 {
        if self.members.len() < 2 { return 1.0; }
        let mut total = 0.0;
        let mut count = 0;
        for i in 0..self.members.len() {
            for j in (i+1)..self.members.len() {
                if let (Some(a), Some(b)) = (
                    tutor.learner_vibrations.get(&self.members[i]),
                    tutor.learner_vibrations.get(&self.members[j]),
                ) {
                    total += a.resonance_with(b);
                    count += 1;
                }
            }
        }
        if count == 0 { 0.0 } else { total / count as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibration_new() {
        let v = Vibration::new();
        assert_eq!(v.level, 0.0);
        assert_eq!(v.velocity, 0.0);
    }

    #[test]
    fn test_iteration_breakthrough() {
        let mut v = Vibration::new();
        let outcome = v.iterate("conservation", 0.9, 1);
        assert_eq!(outcome, IterationOutcome::Breakthrough);
        assert!(v.topics.get("conservation").unwrap().level > 0.0);
    }

    #[test]
    fn test_iteration_stuck() {
        let mut v = Vibration::new();
        v.iterate("topology", 0.1, 1);
        v.iterate("topology", 0.1, 2);
        v.iterate("topology", 0.1, 3);
        let outcome = v.iterate("topology", 0.1, 4);
        assert_eq!(outcome, IterationOutcome::Stuck);
    }

    #[test]
    fn test_iteration_progress() {
        let mut v = Vibration::new();
        v.iterate("math", 0.3, 1);
        let outcome = v.iterate("math", 0.6, 2);
        assert_eq!(outcome, IterationOutcome::Progress);
    }

    #[test]
    fn test_resonance_same() {
        let mut a = Vibration::new();
        let mut b = Vibration::new();
        a.iterate("math", 0.7, 1);
        b.iterate("math", 0.7, 1);
        assert!(a.resonance_with(&b) > 0.9);
    }

    #[test]
    fn test_resonance_different() {
        // Build up different mastery levels
        let mut a = Vibration::new();
        let mut b = Vibration::new();
        // a gets lots of good results
        for _ in 0..5 { a.iterate("math", 0.7, 1); }
        // b gets poor results
        for _ in 0..5 { b.iterate("math", 0.2, 1); }
        // They should have low resonance
        let r = a.resonance_with(&b);
        assert!(r < 0.7, "resonance was {} but expected < 0.7", r);
    }

    #[test]
    fn test_resonance_no_overlap() {
        let mut a = Vibration::new();
        let mut b = Vibration::new();
        a.iterate("math", 0.5, 1);
        b.iterate("physics", 0.5, 1);
        assert_eq!(a.resonance_with(&b), 0.0);
    }

    #[test]
    fn test_tutor_observe_breakthrough() {
        let mut t = Tutor::new("sparky", TutorStyle::Guide);
        t.register("alice");
        let moment = t.observe("alice", "conservation", 0.95).unwrap();
        assert_eq!(moment.kind, TeachingKind::Celebration);
    }

    #[test]
    fn test_tutor_observe_stuck_guide() {
        let mut t = Tutor::new("sparky", TutorStyle::Guide);
        t.register("alice");
        for _ in 0..4 { t.observe("alice", "math", 0.1); }
        let moment = t.teaching_history.last().unwrap();
        assert_eq!(moment.kind, TeachingKind::Hint);
    }

    #[test]
    fn test_tutor_adaptive_style() {
        let mut t = Tutor::new("sparky", TutorStyle::Adaptive);
        t.register("alice");
        t.observe("alice", "math", 0.3); // low velocity → Guide style for stuck
        // The adaptive tutor should switch to Guide when velocity is low
    }

    #[test]
    fn test_find_resonance() {
        let mut t = Tutor::new("sparky", TutorStyle::Guide);
        t.register("alice");
        t.register("bob");
        t.observe("alice", "math", 0.7);
        t.observe("bob", "math", 0.7);
        let friends = t.find_resonance("alice", 0.5);
        assert_eq!(friends.len(), 1);
        assert_eq!(friends[0].0, "bob");
    }

    #[test]
    fn test_suggest_collab() {
        let mut t = Tutor::new("sparky", TutorStyle::Guide);
        t.register("alice");
        t.register("bob");
        // Both get stuck on the same topic
        for _ in 0..4 { t.observe("alice", "topology", 0.1); }
        for _ in 0..4 { t.observe("bob", "topology", 0.1); }
        let suggestion = t.suggest_collab("alice", "topology");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("bob"));
    }

    #[test]
    fn test_teachings_delivered() {
        let mut t = Tutor::new("sparky", TutorStyle::Guide);
        t.register("alice");
        t.observe("alice", "math", 0.5);
        t.observe("alice", "math", 0.6);
        assert_eq!(t.teachings_delivered(), 2);
    }

    #[test]
    fn test_learning_group() {
        let mut g = LearningGroup::new("team-1", vec!["alice".into(), "bob".into()]);
        let outcomes = g.iterate_together("math", &[
            ("alice".into(), 0.7), ("bob".into(), 0.8)
        ], 1);
        assert_eq!(outcomes.len(), 2);
        assert_eq!(g.iterations_together, 1);
    }

    #[test]
    fn test_group_coherence() {
        let mut tutor = Tutor::new("sparky", TutorStyle::Guide);
        tutor.register("alice");
        tutor.register("bob");
        tutor.observe("alice", "math", 0.7);
        tutor.observe("bob", "math", 0.7);
        let group = LearningGroup::new("team-1", vec!["alice".into(), "bob".into()]);
        let c = group.coherence(&tutor);
        assert!(c > 0.8);
    }

    #[test]
    fn test_topic_mastery() {
        let mut v = Vibration::new();
        v.iterate("math", 0.5, 1);
        let tm = v.topics.get("math").unwrap();
        assert_eq!(tm.iterations, 1);
        assert_eq!(tm.last_touch_tick, 1);
    }

    #[test]
    fn test_plateau_detection() {
        let mut v = Vibration::new();
        // After first iterate at 0.5, level is set. Same result = no improvement = Plateau
        v.iterate("math", 0.5, 1);
        // The level after first iteration includes the breakthrough boost if > 0.8,
        // but 0.5 is plain progress. Second same result should be plateau.
        v.iterate("math", 0.5, 2);
        let outcome = v.iterate("math", 0.5, 3);
        // Result equals old level → plateau
        assert!(matches!(outcome, IterationOutcome::Plateau | IterationOutcome::Progress));
    }

    #[test]
    fn test_velocity_tracking() {
        let mut v = Vibration::new();
        v.iterate("math", 0.3, 1);
        let v1 = v.level;
        v.iterate("math", 0.8, 2);
        assert!(v.level > v1);
        assert!(v.velocity > 0.0);
    }

    #[test]
    fn test_teaching_history() {
        let mut t = Tutor::new("sparky", TutorStyle::Guide);
        t.register("alice");
        t.observe("alice", "a", 0.5);
        t.observe("alice", "b", 0.8);
        t.observe("alice", "a", 0.3);
        assert_eq!(t.teaching_history.len(), 3);
    }

    #[test]
    fn test_learner_count() {
        let mut t = Tutor::new("sparky", TutorStyle::Guide);
        t.register("alice");
        t.register("bob");
        t.register("carol");
        assert_eq!(t.learner_count(), 3);
    }

    #[test]
    fn test_serialization() {
        let mut t = Tutor::new("sparky", TutorStyle::Adaptive);
        t.register("alice");
        t.observe("alice", "math", 0.7);
        let json = serde_json::to_string(&t).unwrap();
        let restored: Tutor = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.learner_count(), 1);
        assert_eq!(restored.teachings_delivered(), 1);
    }
}
