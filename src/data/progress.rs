use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::data::models::{SessionRecord, UserConfig, UserStats};

pub struct ProgressStore {
    base_dir: PathBuf,
}

impl ProgressStore {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cmdtyper");
        Self::from_base_dir(base_dir)
    }

    pub fn load_stats(&self) -> Result<UserStats> {
        self.load_json_or_default(&self.stats_path())
    }

    pub fn save_stats(&self, stats: &UserStats) -> Result<()> {
        self.write_json_atomic(&self.stats_path(), stats)
    }

    pub fn append_record(&self, record: &SessionRecord) -> Result<()> {
        let mut history = self.load_history()?;
        history.push(record.clone());
        self.write_json_atomic(&self.history_path(), &history)
    }

    pub fn load_history(&self) -> Result<Vec<SessionRecord>> {
        self.load_json_or_default(&self.history_path())
    }

    pub fn load_config(&self) -> Result<UserConfig> {
        self.load_json_or_default(&self.config_path())
    }

    pub fn save_config(&self, config: &UserConfig) -> Result<()> {
        self.write_json_atomic(&self.config_path(), config)
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    fn from_base_dir(base_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&base_dir).with_context(|| {
            format!("failed to create progress directory {}", base_dir.display())
        })?;
        Ok(Self { base_dir })
    }

    fn load_json_or_default<T>(&self, path: &Path) -> Result<T>
    where
        T: DeserializeOwned + Default,
    {
        match fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(value) => Ok(value),
                Err(err) => {
                    eprintln!(
                        "warning: failed to parse {} as JSON: {}; returning default",
                        path.display(),
                        err
                    );
                    Ok(T::default())
                }
            },
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(T::default()),
            Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
        }
    }

    fn write_json_atomic<T>(&self, path: &Path, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        fs::create_dir_all(&self.base_dir)
            .with_context(|| format!("failed to ensure {}", self.base_dir.display()))?;

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("data.json");
        let temp_path = path.with_file_name(format!("{file_name}.tmp"));
        let payload = serde_json::to_vec_pretty(value)
            .with_context(|| format!("failed to serialize {}", path.display()))?;

        fs::write(&temp_path, payload)
            .with_context(|| format!("failed to write {}", temp_path.display()))?;
        fs::rename(&temp_path, path).with_context(|| {
            format!(
                "failed to move {} into place for {}",
                temp_path.display(),
                path.display()
            )
        })?;

        Ok(())
    }

    fn stats_path(&self) -> PathBuf {
        self.base_dir.join("stats.json")
    }

    fn history_path(&self) -> PathBuf {
        self.base_dir.join("history.json")
    }

    fn config_path(&self) -> PathBuf {
        self.base_dir.join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::ProgressStore;
    use crate::data::models::{Difficulty, SessionRecord, UserConfig, UserStats};
    use chrono::Utc;
    use std::fs;
    use std::path::PathBuf;

    fn temp_store() -> (ProgressStore, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "cmdtyper-progress-test-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let store =
            ProgressStore::from_base_dir(dir.clone()).expect("temp store should initialize");
        (store, dir)
    }

    fn sample_record(id: &str) -> SessionRecord {
        SessionRecord {
            id: id.to_string(),
            command_id: "ls-basic".to_string(),
            started_at: 100,
            finished_at: 200,
            wpm: 42.0,
            cpm: 210.0,
            accuracy: 0.95,
            error_count: 1,
            ..SessionRecord::default()
        }
    }

    #[test]
    fn missing_files_return_defaults() {
        let (store, dir) = temp_store();

        assert_eq!(
            store.load_stats().expect("stats should load"),
            UserStats::default()
        );
        assert_eq!(
            store.load_history().expect("history should load"),
            Vec::<SessionRecord>::new()
        );
        assert_eq!(
            store.load_config().expect("config should load"),
            UserConfig::default()
        );

        fs::remove_dir_all(dir).expect("temp dir should be removable");
    }

    #[test]
    fn stats_and_config_round_trip_via_atomic_write() {
        let (store, dir) = temp_store();
        let stats = UserStats {
            total_sessions: 7,
            total_keystrokes: 123,
            ..UserStats::default()
        };
        let config = UserConfig {
            target_wpm: 55.0,
            last_difficulty: Difficulty::Advanced,
            ..UserConfig::default()
        };

        store.save_stats(&stats).expect("stats should save");
        store.save_config(&config).expect("config should save");

        assert_eq!(store.load_stats().expect("stats should reload"), stats);
        assert_eq!(store.load_config().expect("config should reload"), config);
        assert!(!store.base_dir().join("stats.json.tmp").exists());
        assert!(!store.base_dir().join("config.json.tmp").exists());

        fs::remove_dir_all(dir).expect("temp dir should be removable");
    }

    #[test]
    fn append_record_persists_history() {
        let (store, dir) = temp_store();

        store
            .append_record(&sample_record("1"))
            .expect("first record should append");
        store
            .append_record(&sample_record("2"))
            .expect("second record should append");

        let history = store.load_history().expect("history should reload");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].id, "1");
        assert_eq!(history[1].id, "2");
        assert!(!store.base_dir().join("history.json.tmp").exists());

        fs::remove_dir_all(dir).expect("temp dir should be removable");
    }

    #[test]
    fn corrupted_json_returns_defaults() {
        let (store, dir) = temp_store();

        fs::write(store.base_dir().join("stats.json"), "{not valid json")
            .expect("stats fixture should write");
        fs::write(store.base_dir().join("history.json"), "{not valid json")
            .expect("history fixture should write");
        fs::write(store.base_dir().join("config.json"), "{not valid json")
            .expect("config fixture should write");

        assert_eq!(
            store.load_stats().expect("stats should fallback"),
            UserStats::default()
        );
        assert_eq!(
            store.load_history().expect("history should fallback"),
            Vec::<SessionRecord>::new()
        );
        assert_eq!(
            store.load_config().expect("config should fallback"),
            UserConfig::default()
        );

        fs::remove_dir_all(dir).expect("temp dir should be removable");
    }
}
