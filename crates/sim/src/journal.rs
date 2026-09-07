//! Durable, factual milestones for a single journey.
use serde::{Deserialize, Serialize};
const MAX_ENTRIES: usize = 200;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeathCause {
    RiverCrossing,
    Rafting,
    Ailments(Vec<String>),
    Exhaustion,
    Starvation,
}
impl DeathCause {
    pub fn description(&self) -> String {
        match self {
            Self::RiverCrossing => "a river crossing".into(),
            Self::Rafting => "the Columbia rafting".into(),
            Self::Ailments(ailments) => ailments
                .iter()
                .map(|ailment| ailment.replace('_', " "))
                .collect::<Vec<_>>()
                .join(" and "),
            Self::Exhaustion => "exhaustion".into(),
            Self::Starvation => "starvation".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalKind {
    Departed,
    Landmark { landmark_id: String, name: String },
    Recovered { name: String, ailment: String },
    Relationship { left: String, right: String, affinity: i16 },
    Birth { mother: String, child: String },
    Marriage { left: String, right: String },
    LeftParty { name: String },
    LetterAccepted { recipient: String, destination: String, reward_cents: i64 },
    LetterDelivered { recipient: String, destination: String, reward_cents: i64 },
    Death { name: String, cause: DeathCause },
    Arrived,
    Failed,
}
impl JournalKind {
    fn critical(&self) -> bool {
        matches!(
            self,
            Self::Departed
                | Self::Landmark { .. }
                | Self::Death { .. }
                | Self::Arrived
                | Self::Failed
        )
    }
    pub fn text(&self) -> String {
        match self {
            Self::Departed => "The party departed.".into(),
            Self::Landmark { name, .. } => format!("Reached {name}."),
            Self::Recovered { name, ailment } => {
                format!("{name} recovered from {}.", ailment.replace('_', " "))
            }
            Self::Relationship { left, right, affinity } => format!(
                "{left} and {right} are {}.",
                if *affinity >= 20 {
                    "close"
                } else if *affinity <= -20 {
                    "distant"
                } else {
                    "getting acquainted"
                }
            ),
            Self::Birth { mother, child } => format!("{mother} gave birth to {child}."),
            Self::Marriage { left, right } => format!("{left} and {right} married."),
            Self::LeftParty { name } => format!("{name} left the party."),
            Self::LetterAccepted { recipient, destination, reward_cents } => format!(
                "Accepted a sealed letter for {recipient}, bound for {destination} (${:.2}).",
                *reward_cents as f64 / 100.0
            ),
            Self::LetterDelivered { recipient, destination, reward_cents } => format!(
                "Delivered the sealed letter to {recipient} at {destination}; earned ${:.2}.",
                *reward_cents as f64 / 100.0
            ),
            Self::Death { name, cause } => format!("{name} died from {}.", cause.description()),
            Self::Arrived => "The party arrived at journey's end.".into(),
            Self::Failed => "The journey ended on the trail.".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalEntry {
    pub sequence: u32,
    pub day: u32,
    pub miles: u32,
    pub kind: JournalKind,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Journal {
    #[serde(default)]
    pub entries: Vec<JournalEntry>,
    #[serde(default)]
    next_sequence: u32,
}
impl Journal {
    pub fn record(&mut self, day: u32, miles: u32, kind: JournalKind) {
        if self.entries.len() >= MAX_ENTRIES {
            if let Some(index) = self.entries.iter().position(|entry| !entry.kind.critical()) {
                self.entries.remove(index);
            } else if !kind.critical() {
                return;
            }
        }
        self.entries.push(JournalEntry { sequence: self.next_sequence, day, miles, kind });
        self.next_sequence = self.next_sequence.saturating_add(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_critical_facts_when_trimming() {
        let mut journal = Journal::default();
        journal.record(0, 0, JournalKind::Departed);
        for day in 1..=MAX_ENTRIES as u32 {
            journal.record(
                day,
                day,
                JournalKind::Recovered { name: "Ada".into(), ailment: "fever".into() },
            );
        }
        assert!(matches!(
            journal.entries.first().map(|entry| &entry.kind),
            Some(JournalKind::Departed)
        ));
        assert_eq!(journal.entries.len(), MAX_ENTRIES);
    }
}
