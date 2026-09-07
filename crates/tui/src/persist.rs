//! Versioned local storage. A failed read never authorizes replacing a save.

use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use pioneer_sim::{GameState, JournalEntry};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const SAVE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Save {
    schema_version: u32,
    run_id: String,
    game: GameState,
}

#[derive(Debug, Clone)]
pub struct Storage {
    root: PathBuf,
}

impl Storage {
    pub fn platform() -> Result<Self> {
        let dirs = ProjectDirs::from("", "", "pioneer-trail")
            .context("Cannot find the local application data directory")?;
        Ok(Self::at(dirs.data_dir()))
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load_game(&self) -> Result<Option<GameState>> {
        Ok(self.load_session()?.map(|(_, game)| game))
    }

    pub fn load_session(&self) -> Result<Option<(String, GameState)>> {
        let Some(value) = self.read_json::<serde_json::Value>("save.json")? else {
            return Ok(None);
        };
        // Check the envelope before deserializing state from a future schema.
        let version = value.get("schema_version").and_then(|v| v.as_u64());
        if version != Some(u64::from(SAVE_VERSION)) {
            bail!("Unsupported save version {version:?}; the existing save was preserved");
        }
        let save: Save =
            serde_json::from_value(value).context("The save contains invalid game data")?;
        save.game.validate().context("The saved state failed validation; file preserved")?;
        pioneer_data::validate(&save.game.content)
            .context("The saved content failed validation; file preserved")?;
        anyhow::ensure!(
            !save.run_id.is_empty() && save.run_id.len() <= 100,
            "Invalid saved run identity"
        );
        Ok(Some((save.run_id, save.game)))
    }

    pub fn save_game(&self, game: &GameState) -> Result<()> {
        let run_id = self.load_session()?.map(|(id, _)| id).unwrap_or_else(new_run_id);
        self.save_session(&run_id, game)
    }

    /// The caller supplies the identity created when the player started this run.
    pub fn save_session(&self, run_id: &str, game: &GameState) -> Result<()> {
        self.write_json(
            "save.json",
            &Save { schema_version: SAVE_VERSION, run_id: run_id.to_owned(), game: game.clone() },
        )
    }

    pub fn load_history(&self) -> Result<History> {
        Ok(self.read_json("hall_of_fame.json")?.unwrap_or_default())
    }

    /// A run id is created once at new-game time and survives resuming the save.
    pub fn record_run(&self, entry: RunRecord) -> Result<bool> {
        let mut history = self.load_history()?;
        if history.runs.iter().any(|run| run.run_id == entry.run_id) {
            return Ok(false);
        }
        history.runs.push(entry);
        self.write_json("hall_of_fame.json", &history)?;
        Ok(true)
    }
    pub fn set_epitaph(&self, run_id: &str, epitaph: &str) -> Result<()> {
        anyhow::ensure!(
            epitaph.chars().count() <= 80 && !epitaph.chars().any(char::is_control),
            "Epitaph must be at most 80 printable characters"
        );
        let mut history = self.load_history()?;
        let entry = history
            .runs
            .iter_mut()
            .find(|entry| entry.run_id == run_id)
            .context("Completed journey was not found")?;
        entry.epitaph = epitaph.trim().to_owned();
        self.write_json("hall_of_fame.json", &history)
    }

    pub fn load_settings(&self) -> Result<Settings> {
        let path = self.root.join("settings.toml");
        match fs::read_to_string(&path) {
            Ok(text) => {
                toml::from_str(&text).with_context(|| format!("Invalid {}", path.display()))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(error) => Err(error).with_context(|| format!("Cannot read {}", path.display())),
        }
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        self.atomic_write("settings.toml", toml::to_string_pretty(settings)?.as_bytes())
    }

    fn read_json<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {
        let path = self.root.join(name);
        match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .with_context(|| format!("Invalid {}; file preserved", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).with_context(|| format!("Cannot read {}", path.display())),
        }
    }

    fn write_json(&self, name: &str, value: &impl Serialize) -> Result<()> {
        self.atomic_write(name, &serde_json::to_vec_pretty(value)?)
    }

    fn atomic_write(&self, name: &str, bytes: &[u8]) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        let target = self.root.join(name);
        let mut pending = None;
        for attempt in 0..100 {
            let path = self.root.join(format!(".{name}.{}.{attempt}.tmp", std::process::id()));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    pending = Some((path, file));
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error).context("Cannot create temporary save"),
            }
        }
        let (path, mut file) = pending.context("Too many pending saves in the data directory")?;
        let result = (|| -> Result<()> {
            file.write_all(bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&path, &target)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&path);
        }
        result.with_context(|| format!("Cannot save {}", target.display()))
    }
}

pub fn new_run_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{nanos:x}-{:x}-{:x}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct History {
    pub runs: Vec<RunRecord>,
}

impl History {
    pub fn leaders(&self) -> Vec<&RunRecord> {
        let mut runs: Vec<_> = self.runs.iter().filter(|run| run.arrived).collect();
        runs.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.run_id.cmp(&b.run_id)));
        runs.truncate(20);
        runs
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub leader: String,
    pub seed: u64,
    pub trail: String,
    pub era: u16,
    #[serde(default)]
    pub ended_on: String,
    pub occupation: String,
    pub score: u32,
    pub survivors: usize,
    pub days: u32,
    pub miles: u32,
    pub arrived: bool,
    pub epitaph: String,
    pub cause: String,
    /// A completed journey keeps its factual record after `save.json` is replaced.
    #[serde(default)]
    pub journal: Vec<JournalEntry>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    #[default]
    Truecolor,
    Indexed,
    Basic,
    Mono,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Speed {
    Instant,
    Fast,
    #[default]
    Normal,
    Slow,
}

impl Speed {
    pub fn milliseconds(self) -> u64 {
        match self {
            Self::Instant => 0,
            Self::Fast => 300,
            Self::Normal => 600,
            Self::Slow => 1200,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub color: ColorMode,
    pub speed: Speed,
    pub bell: bool,
    pub no_art: bool,
    pub reduced_motion: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "pioneer-persist-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn save_preserves_random_stream_position() {
        let temp = Temp::new();
        let store = Storage::at(&temp.0);
        let mut captured = None;
        crate::headless::journey(
            &pioneer_data::load().unwrap(),
            23,
            &crate::headless::RunConfig::default(),
            |game, _| {
                if captured.is_none() && game.day >= 7 {
                    captured = Some(game.clone());
                }
            },
        )
        .unwrap();
        let mut game = captured.unwrap();
        store.save_game(&game).unwrap();
        let mut loaded = store.load_game().unwrap().unwrap();
        use pioneer_sim::{Command, CrossMethod, RunStatus};
        for _ in 0..20 {
            let command = if let Some(id) = &game.pending_event {
                let event = game.content.events.iter().find(|e| &e.id == id).unwrap();
                let choice = event.choices.iter().find(|c| game.choice_available(c)).unwrap();
                Command::Respond { event_id: id.clone(), choice_id: choice.id.clone() }
            } else {
                match game.status {
                    RunStatus::AwaitingRiver(_) => {
                        Command::CrossRiver { method: CrossMethod::Caulk }
                    }
                    RunStatus::AwaitingFork(_) => Command::ChooseRoute {
                        route_id: game.current_landmark().unwrap().routes[0].id.clone(),
                    },
                    _ => Command::Continue,
                }
            };
            assert_eq!(game.apply(command.clone()), loaded.apply(command));
        }
        assert_eq!(serde_json::to_value(game).unwrap(), serde_json::to_value(loaded).unwrap());
    }

    #[test]
    fn corrupt_and_future_saves_are_preserved() {
        let temp = Temp::new();
        let store = Storage::at(&temp.0);
        assert!(store.load_game().unwrap().is_none());
        for bytes in [b"broken".as_slice(), br#"{"schema_version":999,"game":{}}"#] {
            fs::write(temp.0.join("save.json"), bytes).unwrap();
            assert!(store.load_game().is_err());
            assert_eq!(fs::read(temp.0.join("save.json")).unwrap(), bytes);
        }
    }

    #[test]
    fn forged_active_letter_save_is_rejected_and_preserved() {
        let temp = Temp::new();
        let store = Storage::at(&temp.0);
        let mut game = GameState::with_content(44, pioneer_data::load().unwrap());
        game.apply(pioneer_sim::Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["Ada".into(), "Ben".into(), "Clara".into(), "Dora".into(), "Eli".into()],
            departure_month: 3,
        });
        game.current_node_id = Some("fort_kearney".into());
        game.status = pioneer_sim::RunStatus::AtLandmark("fort_kearney".into());
        game.apply(pioneer_sim::Command::AcceptLetter { letter_id: "platt_note".into() });
        store.save_session("letter-forgery", &game).unwrap();

        let path = temp.0.join("save.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value["game"]["active_letter"]["reward_cents"] = serde_json::json!(99_999);
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let forged = fs::read(&path).unwrap();

        assert!(store.load_session().is_err());
        assert_eq!(fs::read(path).unwrap(), forged);
    }

    #[test]
    fn terminal_saves_cannot_resume_into_mandatory_events() {
        let temp = Temp::new();
        let store = Storage::at(&temp.0);
        let mut game = GameState::with_content(41, pioneer_data::load().unwrap());
        game.apply(pioneer_sim::Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "banker".into(),
            party: vec!["Ada".into(), "Ben".into(), "Clara".into(), "Dora".into(), "Eli".into()],
            departure_month: 3,
        });
        game.status = pioneer_sim::RunStatus::Failed;
        assert!(game.validate().is_ok());
        game.pending_event = Some(game.content.events[0].id.clone());
        store.save_session("terminal-corrupt", &game).unwrap();
        let before = fs::read(temp.0.join("save.json")).unwrap();
        assert!(store.load_session().is_err());
        assert_eq!(before, fs::read(temp.0.join("save.json")).unwrap());
    }

    #[test]
    fn saves_reject_forged_route_history_without_replacing_it() {
        let temp = Temp::new();
        let store = Storage::at(&temp.0);
        let mut game = GameState::with_content(41, pioneer_data::load().unwrap());
        game.apply(pioneer_sim::Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "banker".into(),
            party: vec!["Ada".into(), "Ben".into(), "Clara".into(), "Dora".into(), "Eli".into()],
            departure_month: 3,
        });
        game.visited_landmarks.push(pioneer_sim::route_record::Visit {
            landmark_id: "willamette".into(),
            day: 0,
            mile: 0,
        });
        store.save_session("forged-route", &game).unwrap();
        let before = fs::read(temp.0.join("save.json")).unwrap();
        assert!(store.load_session().is_err());
        assert_eq!(before, fs::read(temp.0.join("save.json")).unwrap());
    }

    #[test]
    fn stale_temporary_file_does_not_prevent_replacing_save() {
        let temp = Temp::new();
        let store = Storage::at(&temp.0);
        fs::write(temp.0.join(format!(".save.json.{}.0.tmp", std::process::id())), b"partial")
            .unwrap();
        store.save_game(&GameState::new(3)).unwrap();
        assert!(store.load_game().unwrap().is_some());
        store.save_game(&GameState::new(7)).unwrap();
        assert_eq!(store.load_game().unwrap().unwrap().rng.seed(), 7);
    }

    #[test]
    fn settings_round_trip_and_partial_defaults() {
        let temp = Temp::new();
        let store = Storage::at(&temp.0);
        let settings = Settings {
            no_art: true,
            color: ColorMode::Mono,
            reduced_motion: true,
            ..Settings::default()
        };
        store.save_settings(&settings).unwrap();
        assert_eq!(store.load_settings().unwrap(), settings);
        fs::write(temp.0.join("settings.toml"), "bell = true").unwrap();
        assert!(store.load_settings().unwrap().bell);
        assert_eq!(store.load_settings().unwrap().speed, Speed::Normal);
        assert!(!store.load_settings().unwrap().reduced_motion);
    }

    #[test]
    fn old_save_and_history_default_the_journal() {
        let mut game = serde_json::to_value(GameState::new(7)).unwrap();
        game.as_object_mut().unwrap().remove("journal");
        let restored: GameState = serde_json::from_value(game).unwrap();
        assert!(restored.journal.entries.is_empty());

        let legacy = r#"{"runs":[{"run_id":"old","leader":"Ada","seed":7,"trail":"oregon","era":1848,"occupation":"farmer","score":0,"survivors":0,"days":10,"miles":100,"arrived":false,"epitaph":"","cause":"Unknown"}]}"#;
        let history: History = serde_json::from_str(legacy).unwrap();
        assert!(history.runs[0].journal.is_empty());
    }

    #[test]
    fn resume_keeps_identity_and_records_completion_once() {
        let temp = Temp::new();
        let store = Storage::at(&temp.0);
        let id = new_run_id();
        store.save_session(&id, &GameState::new(42)).unwrap();
        assert_eq!(store.load_session().unwrap().unwrap().0, id);
        let run = RunRecord {
            run_id: id.clone(),
            leader: "Sarah".into(),
            seed: 42,
            trail: "oregon".into(),
            era: 1848,
            ended_on: "1848-08-28".into(),
            occupation: "farmer".into(),
            score: 1200,
            survivors: 3,
            days: 150,
            miles: 2040,
            arrived: true,
            epitaph: String::new(),
            cause: String::new(),
            journal: Vec::new(),
        };
        assert!(store.record_run(run.clone()).unwrap());
        assert!(!store.record_run(run).unwrap());
        store.set_epitaph(&id, "We kept the fire until morning.").unwrap();
        assert_eq!(
            store.load_history().unwrap().runs[0].epitaph,
            "We kept the fire until morning."
        );
        assert_eq!(store.load_history().unwrap().leaders().len(), 1);
        let second_id = new_run_id();
        assert_ne!(second_id, store.load_session().unwrap().unwrap().0);
    }
}
