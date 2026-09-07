//! Setting-specific scene and menu text for the Talk screen.
//!
//! Pure helpers over [`GameState`]; `app.rs` owns the small cursor/stage state
//! machine (choose a speaker, then a topic) and calls into these to render.
use pioneer_sim::{
    ConversationTopic, GameState, LandmarkKind, RunStatus, SpeakerSetting, SpeakerView,
};

pub(crate) const TOPICS: [(ConversationTopic, &str); 3] = [
    (ConversationTopic::Route, "Ask about the route"),
    (ConversationTopic::Supplies, "Ask about supplies"),
    (ConversationTopic::News, "Ask for news"),
];

/// Menu rows for the speaker-choice stage: one per available named speaker, plus a
/// fallback that preserves the original "listen to camp chatter" quote behavior.
pub(crate) fn speaker_menu(speakers: &[SpeakerView]) -> Vec<String> {
    let mut rows: Vec<String> = speakers
        .iter()
        .map(|speaker| match speaker.setting {
            SpeakerSetting::Fort => format!("Speak with {}", speaker.name),
            SpeakerSetting::Wagon => format!("Speak with {} (neighboring wagon)", speaker.name),
        })
        .collect();
    rows.push("Listen to the camp".into());
    rows
}

pub(crate) fn topic_menu() -> Vec<String> {
    TOPICS.iter().map(|(_, label)| (*label).to_string()).collect()
}

/// The background art file and a one-line setting description, used both for the
/// illustrated scene and for the `--no-art` text description (DESIGN.md §8).
pub(crate) fn setting(game: &GameState, speaker: Option<SpeakerSetting>) -> (String, &'static str) {
    if speaker == Some(SpeakerSetting::Fort) && matches!(game.status, RunStatus::AtLandmark(_)) {
        if let Some(node) = game.current_landmark() {
            if node.kind == LandmarkKind::Fort {
                return (format!("{}.px", node.id), "AT THE FORT");
            }
        }
    }
    if speaker == Some(SpeakerSetting::Wagon) {
        return ("terrain_plains.px".into(), "A NEIGHBORING WAGON");
    }
    ("terrain_plains.px".into(), "CAMP AT NIGHT")
}
