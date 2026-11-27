use pioneer_sim::{
    ConversationTopic, GameState, LandmarkKind, RunStatus, SpeakerSetting, SpeakerView,
};

pub(crate) const TOPICS: [(ConversationTopic, &str); 3] = [
    (ConversationTopic::Route, "Ask about the route"),
    (ConversationTopic::Supplies, "Ask about supplies"),
    (ConversationTopic::News, "Ask for news"),
];

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
