use std::env;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::core::scorer;
use crate::data::models::{ResumeState, SessionRecord, UserConfig, UserStats};

/// Persistent storage for user stats, session history, and config.
///
/// JSON writes use synced temporary files and atomic replacement. Cache/config
/// reads can fall back to defaults for malformed JSON; appends and statistics
/// migration strictly validate authoritative history before any write.
pub struct ProgressStore {
    base_dir: PathBuf,
}

impl ProgressStore {
    /// Create a new store using the platform-standard data directory
    /// (`~/.local/share/cmdtyper/` on Linux).
    pub fn new() -> Result<Self> {
        let base_dir = env::var("CMDTYPER_USER_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".local")
                    .join("share")
                    .join("cmdtyper")
            });
        Self::from_base_dir(base_dir)
    }

    pub fn load_stats(&self) -> Result<UserStats> {
        self.load_json_or_default(&self.stats_path())
    }

    pub fn save_stats(&self, stats: &UserStats) -> Result<()> {
        self.write_json_atomic(&self.stats_path(), stats)
    }

    pub fn append_record(&self, record: &SessionRecord) -> Result<()> {
        let mut history: Vec<SessionRecord> = match fs::read_to_string(self.history_path()) {
            Ok(contents) => serde_json::from_str(&contents)
                .context("refusing to overwrite invalid practice history")?,
            Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error).context("failed to read practice history"),
        };
        if let Some(existing) = history.iter_mut().find(|existing| existing.id == record.id) {
            *existing = record.clone();
        } else {
            history.push(record.clone());
        }
        self.write_json_atomic(&self.history_path(), &history)
    }

    pub fn save_history(&self, history: &[SessionRecord]) -> Result<()> {
        self.write_json_atomic(&self.history_path(), &history)
    }

    pub fn load_history(&self) -> Result<Vec<SessionRecord>> {
        self.load_json_or_default(&self.history_path())
    }

    /// Rebuild from the authoritative history so interruption between history
    /// and cached-stat writes is recoverable. Original files are backed up once.
    pub fn migrate_stats(&self) -> Result<UserStats> {
        let cached = self.load_stats()?;
        // A corrupt authoritative history is not an empty installation. Refuse
        // migration before creating a baseline or rewriting the valid cache.
        let history: Vec<SessionRecord> = match fs::read_to_string(self.history_path()) {
            Ok(contents) => serde_json::from_str(&contents).context(
                "practice history is invalid; cached statistics and history were preserved",
            )?,
            Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                return Err(error)
                    .context("failed to read practice history for statistics migration");
            }
        };
        if cached.stats_version < scorer::STATS_VERSION {
            for path in [self.stats_path(), self.history_path()] {
                let backup = path.with_extension("json.pre-v2.bak");
                if path.try_exists()? {
                    backup_file_atomic(&path, &backup)?;
                }
            }
        }
        if history.is_empty() && cached.stats_version < scorer::STATS_VERSION {
            if cached.total_sessions == 0
                && cached.daily_stats.is_empty()
                && cached.command_progress.is_empty()
            {
                return Ok(cached);
            }
            let baseline_path = self.base_dir.join("legacy_stats_baseline.json");
            if !baseline_path.exists() {
                self.write_json_atomic(&baseline_path, &cached)?;
            }
        }
        let mut rebuilt = self.stats_for_history(&history)?;
        scorer::merge_command_progress(&mut rebuilt, &cached);
        self.save_stats(&rebuilt)?;
        Ok(rebuilt)
    }

    /// Rebuild current event statistics while retaining an aggregate-only
    /// legacy baseline, when the installation had no old history to replay.
    pub fn stats_for_history(&self, history: &[SessionRecord]) -> Result<UserStats> {
        let mut stats = scorer::rebuild_from_history(history);
        let baseline: UserStats =
            self.load_json_or_default(&self.base_dir.join("legacy_stats_baseline.json"))?;
        scorer::merge_legacy_stats(&mut stats, &baseline);
        Ok(stats)
    }

    pub fn save_legacy_stats_baseline(&self, baseline: &UserStats) -> Result<()> {
        self.write_json_atomic(&self.base_dir.join("legacy_stats_baseline.json"), baseline)
    }

    pub fn load_config(&self) -> Result<UserConfig> {
        self.load_json_or_default(&self.config_path())
    }

    pub fn save_config(&self, config: &UserConfig) -> Result<()> {
        self.write_json_atomic(&self.config_path(), config)
    }

    pub fn load_resume_state(&self) -> Result<ResumeState> {
        self.load_json_or_default(&self.resume_state_path())
    }

    pub fn save_resume_state(&self, state: &ResumeState) -> Result<()> {
        self.write_json_atomic(&self.resume_state_path(), state)
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    // ── internal ──

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

        let mut file = fs::File::create(&temp_path)
            .with_context(|| format!("failed to create {}", temp_path.display()))?;
        file.write_all(&payload)
            .with_context(|| format!("failed to write {}", temp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to sync {}", temp_path.display()))?;
        fs::rename(&temp_path, path).with_context(|| {
            format!(
                "failed to move {} into place for {}",
                temp_path.display(),
                path.display()
            )
        })?;

        fs::File::open(&self.base_dir)?.sync_all()?;
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

    fn resume_state_path(&self) -> PathBuf {
        self.base_dir.join("resume_state.json")
    }
}

/// Publish a complete, durable backup without replacing an earlier backup.
/// Linking a synced same-directory temporary file makes the final name atomic
/// and refuses to clobber a backup another process published in the meantime.
pub(crate) fn backup_file_atomic(source: &Path, backup: &Path) -> Result<()> {
    backup_file_atomic_with(source, backup, |input, output| std::io::copy(input, output))
}

fn backup_file_atomic_with(
    source: &Path,
    backup: &Path,
    copy: impl FnOnce(&mut fs::File, &mut fs::File) -> std::io::Result<u64>,
) -> Result<()> {
    fn existing_backup(backup: &Path) -> Result<bool> {
        match fs::symlink_metadata(backup) {
            Ok(metadata) => {
                ensure!(
                    metadata.is_file(),
                    "backup is not a regular file: {}",
                    backup.display()
                );
                fs::File::open(backup)?.sync_all()?;
                Ok(true)
            }
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => {
                Err(error).with_context(|| format!("failed to inspect backup {}", backup.display()))
            }
        }
    }

    let parent = backup
        .parent()
        .context("backup path has no parent directory")?;
    if existing_backup(backup)? {
        fs::File::open(parent)?.sync_all()?;
        return Ok(());
    }
    let mut input = fs::File::open(source)
        .with_context(|| format!("failed to open backup source {}", source.display()))?;
    let permissions = input.metadata()?.permissions();
    let name = backup.file_name().context("backup path has no file name")?;
    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let (temp_path, mut output) = loop {
        let mut temp_name = name.to_os_string();
        temp_name.push(format!(
            ".tmp-{}-{}",
            rand::random::<u64>(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let temp_path = backup.with_file_name(temp_name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(output) => break (temp_path, output),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "failed to create backup temporary file {}",
                        temp_path.display()
                    )
                });
            }
        }
    };
    let result = (|| -> Result<()> {
        output.set_permissions(permissions)?;
        copy(&mut input, &mut output)
            .with_context(|| format!("failed to copy backup source {}", source.display()))?;
        output.sync_all().with_context(|| {
            format!(
                "failed to sync backup temporary file {}",
                temp_path.display()
            )
        })?;
        match fs::hard_link(&temp_path, backup) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                ensure!(
                    existing_backup(backup)?,
                    "backup disappeared before publication"
                );
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to publish backup {}", backup.display()));
            }
        }
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    drop(output);
    let cleanup = fs::remove_file(&temp_path);
    result?;
    cleanup.with_context(|| {
        format!(
            "failed to remove backup temporary file {}",
            temp_path.display()
        )
    })?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ProgressStore, backup_file_atomic, backup_file_atomic_with};
    use crate::data::models::{Difficulty, PromptStyle, SessionRecord, UserConfig, UserStats};
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

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn stats_and_config_round_trip() {
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

        assert_eq!(store.load_stats().expect("stats reload"), stats);
        assert_eq!(store.load_config().expect("config reload"), config);
        assert!(!store.base_dir().join("stats.json.tmp").exists());
        assert!(!store.base_dir().join("config.json.tmp").exists());

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn append_record_persists_history() {
        let (store, dir) = temp_store();

        store
            .append_record(&sample_record("1"))
            .expect("first append");
        store
            .append_record(&sample_record("2"))
            .expect("second append");

        let history = store.load_history().expect("history reload");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].id, "1");
        assert_eq!(history[1].id, "2");

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn corrupted_json_returns_defaults() {
        let (store, dir) = temp_store();

        fs::write(store.base_dir().join("stats.json"), "{not valid json").expect("write");
        fs::write(store.base_dir().join("history.json"), "{not valid json").expect("write");
        fs::write(store.base_dir().join("config.json"), "{not valid json").expect("write");

        assert_eq!(
            store.load_stats().expect("stats fallback"),
            UserStats::default()
        );
        assert_eq!(
            store.load_history().expect("history fallback"),
            Vec::<SessionRecord>::new()
        );
        assert_eq!(
            store.load_config().expect("config fallback"),
            UserConfig::default()
        );

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn v02_config_fields_round_trip() {
        let (store, dir) = temp_store();
        let config = UserConfig {
            prompt_style: PromptStyle::Minimal,
            prompt_username: "alice".to_string(),
            prompt_hostname: "devbox".to_string(),
            show_path: false,
            ..UserConfig::default()
        };

        store.save_config(&config).expect("save");
        let loaded = store.load_config().expect("load");
        assert_eq!(loaded.prompt_style, PromptStyle::Minimal);
        assert_eq!(loaded.prompt_username, "alice");
        assert_eq!(loaded.prompt_hostname, "devbox");
        assert!(!loaded.show_path);

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn v01_config_json_backward_compat() {
        let (store, dir) = temp_store();

        // Simulate a v0.1 config.json that lacks the new prompt_* fields
        let v01_json = r#"{
            "target_wpm": 45.0,
            "error_flash_ms": 150,
            "show_token_hints": true,
            "adaptive_recommend": true,
            "last_difficulty": "basic",
            "last_category": null
        }"#;
        fs::write(store.base_dir().join("config.json"), v01_json).expect("write");

        let config = store.load_config().expect("load v01 config");
        assert_eq!(config.target_wpm, 45.0);
        // New fields should use defaults
        assert_eq!(config.prompt_style, PromptStyle::Full);
        assert_eq!(config.prompt_username, "user");
        assert_eq!(config.prompt_hostname, "cmdtyper");
        assert!(config.show_path);

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn stable_record_id_replaces_partial_snapshot_atomically() {
        let (store, dir) = temp_store();
        let mut record = sample_record("stable");
        store.append_record(&record).expect("initial snapshot");
        record.finished_at = 400;
        record.wpm = 55.0;
        store.append_record(&record).expect("replacement snapshot");
        store
            .append_record(&record)
            .expect("idempotent replacement");
        let history = store.load_history().expect("reload");
        assert_eq!(history, vec![record]);
        assert!(!dir.join("history.json.tmp").exists());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn legacy_migration_backs_up_and_does_not_invent_character_intervals() {
        use crate::data::models::{CommandProgress, Keystroke};
        let (store, dir) = temp_store();
        let legacy_stats = UserStats {
            total_sessions: 9,
            command_progress: vec![CommandProgress {
                command_id: "archived-command".to_owned(),
                times_practiced: 5,
                mastery: 1.0,
                ..CommandProgress::default()
            }],
            ..UserStats::default()
        };
        store.save_stats(&legacy_stats).unwrap();
        let mut record = sample_record("legacy");
        record.keystrokes = vec![Keystroke {
            expected: 'a',
            actual: 'a',
            correct: false,
            attempts: 3,
            latency_ms: 100,
            timestamp_ms: 100,
        }];
        store.append_record(&record).unwrap();
        let original = fs::read(dir.join("stats.json")).unwrap();
        let first = store.migrate_stats().unwrap();
        assert_eq!(
            fs::read(dir.join("stats.json.pre-v2.bak")).unwrap(),
            original
        );
        assert!(dir.join("history.json.pre-v2.bak").exists());
        assert_eq!(first.stats_version, crate::core::scorer::STATS_VERSION);
        assert_eq!(first.legacy_sessions_count, 1);
        assert_eq!(first.char_stats[0].total_errors, 1);
        assert!(first.char_stats[0].recent_latencies.is_empty());
        assert!(
            first
                .command_progress
                .iter()
                .any(|progress| progress.command_id == "archived-command"
                    && progress.times_practiced == 5)
        );
        assert_eq!(store.migrate_stats().unwrap(), first);
        assert_eq!(
            fs::read(dir.join("stats.json.pre-v2.bak")).unwrap(),
            original
        );
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn startup_rebuild_recovers_history_saved_before_cached_statistics() {
        let (store, dir) = temp_store();
        store.save_stats(&UserStats::default()).unwrap();
        store
            .append_record(&sample_record("written-before-crash"))
            .unwrap();
        let stats = store.migrate_stats().unwrap();
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.applied_record_ids, vec!["written-before-crash"]);
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn appending_refuses_to_overwrite_corrupted_history() {
        let (store, dir) = temp_store();
        let path = dir.join("history.json");
        fs::write(&path, "{broken-but-recoverable").unwrap();
        assert!(store.append_record(&sample_record("new")).is_err());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "{broken-but-recoverable"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn stats_only_legacy_baseline_survives_new_history_and_repeated_rebuilds() {
        let (store, dir) = temp_store();
        let legacy = UserStats {
            total_sessions: 4,
            total_wpm_sessions: 4,
            total_duration_ms: 10_000,
            overall_avg_wpm: 40.0,
            overall_avg_accuracy: 0.9,
            ..UserStats::default()
        };
        store.save_stats(&legacy).unwrap();
        let first = store.migrate_stats().unwrap();
        assert_eq!(first.total_sessions, 4);
        assert_eq!(first.legacy_sessions_count, 4);
        assert!(dir.join("legacy_stats_baseline.json").exists());
        assert_eq!(store.migrate_stats().unwrap(), first);
        store.append_record(&sample_record("new-history")).unwrap();
        let second = store.migrate_stats().unwrap();
        assert_eq!(second.total_sessions, 5);
        assert_eq!(second.total_duration_ms, 10_100);
        assert_eq!(
            store
                .stats_for_history(&store.load_history().unwrap())
                .unwrap(),
            second
        );
        assert_eq!(store.migrate_stats().unwrap(), second);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn migration_refuses_corrupt_v2_history_and_preserves_valid_cached_statistics() {
        use crate::data::models::{CharStat, DailyStat};
        let (store, dir) = temp_store();
        let cached = UserStats {
            stats_version: crate::core::scorer::STATS_VERSION,
            total_sessions: 12,
            total_wpm_sessions: 12,
            total_duration_ms: 45_000,
            overall_avg_wpm: 38.0,
            overall_avg_accuracy: 0.97,
            daily_stats: vec![DailyStat {
                date: "2026-09-06".to_owned(),
                sessions_count: 12,
                total_duration_ms: 45_000,
                ..DailyStat::default()
            }],
            char_stats: vec![CharStat {
                char_key: 'a',
                total_samples: 20,
                total_correct: 19,
                accuracy: 0.95,
                ..CharStat::default()
            }],
            ..UserStats::default()
        };
        store.save_stats(&cached).unwrap();
        let original_cache = fs::read(dir.join("stats.json")).unwrap();
        let corrupt_history = b"[{\"id\":\"recoverable-record\", truncated";
        fs::write(dir.join("history.json"), corrupt_history).unwrap();
        for _ in 0..2 {
            let error = store.migrate_stats().unwrap_err();
            assert!(error.to_string().contains("history is invalid"));
            assert_eq!(fs::read(dir.join("stats.json")).unwrap(), original_cache);
            assert_eq!(fs::read(dir.join("history.json")).unwrap(), corrupt_history);
            assert_eq!(store.load_stats().unwrap(), cached);
            assert!(!dir.join("legacy_stats_baseline.json").exists());
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn atomic_backup_keeps_existing_bytes_and_ignores_interrupted_temporary_files() {
        let (_store, dir) = temp_store();
        let source = dir.join("source.json");
        let backup = dir.join("source.json.pre-v2.bak");
        fs::write(&source, b"original complete source").unwrap();
        let stale = dir.join("source.json.pre-v2.bak.tmp-crashed");
        fs::write(&stale, b"partial").unwrap();
        backup_file_atomic(&source, &backup).unwrap();
        assert_eq!(fs::read(&backup).unwrap(), b"original complete source");
        fs::write(&source, b"new source").unwrap();
        backup_file_atomic(&source, &backup).unwrap();
        assert_eq!(fs::read(&backup).unwrap(), b"original complete source");
        assert_eq!(fs::read(&source).unwrap(), b"new source");
        assert_eq!(fs::read(stale).unwrap(), b"partial");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 3);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn atomic_backup_never_clobbers_a_backup_published_during_copy() {
        let (_store, dir) = temp_store();
        let source = dir.join("source.json");
        let backup = dir.join("source.json.pre-v2.bak");
        fs::write(&source, b"current source").unwrap();
        backup_file_atomic_with(&source, &backup, |input, output| {
            let count = std::io::copy(input, output)?;
            assert!(!backup.exists());
            fs::write(&backup, b"first process backup")?;
            Ok(count)
        })
        .unwrap();
        assert_eq!(fs::read(&backup).unwrap(), b"first process backup");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn atomic_backup_read_and_destination_failures_leave_no_final_or_temporary_backup() {
        let (_store, dir) = temp_store();
        let source = dir.join("source.json");
        let backup = dir.join("source.json.pre-v2.bak");
        // A directory can be opened on Unix but fails when read as a file.
        fs::create_dir(&source).unwrap();
        assert!(backup_file_atomic(&source, &backup).is_err());
        assert!(!backup.exists());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir(&source).unwrap();
        // A missing source fails before a temporary destination is created.
        assert!(backup_file_atomic(&source, &backup).is_err());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        fs::write(&source, b"original").unwrap();
        let unavailable = dir.join("missing-parent/backup.bak");
        assert!(backup_file_atomic(&source, &unavailable).is_err());
        assert!(!unavailable.exists());
        assert_eq!(fs::read(&source).unwrap(), b"original");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn atomic_backup_copy_failure_after_partial_output_never_publishes_partial_bytes() {
        use std::io::{Error, ErrorKind, Write};
        let (_store, dir) = temp_store();
        let source = dir.join("source.json");
        let backup = dir.join("source.json.pre-v2.bak");
        fs::write(&source, b"original complete source").unwrap();
        for kind in [ErrorKind::UnexpectedEof, ErrorKind::WriteZero] {
            let result = backup_file_atomic_with(&source, &backup, |_, output| {
                output.write_all(b"partial")?;
                assert!(!backup.exists());
                Err(Error::new(
                    kind,
                    "injected copy I/O failure after partial output",
                ))
            });
            assert!(result.is_err());
            assert!(!backup.exists());
            assert_eq!(fs::read(&source).unwrap(), b"original complete source");
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        }
        backup_file_atomic(&source, &backup).unwrap();
        assert_eq!(fs::read(&backup).unwrap(), b"original complete source");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_second_migration_backup_preserves_every_original_until_retry() {
        let (store, dir) = temp_store();
        store
            .save_stats(&UserStats {
                total_sessions: 7,
                ..UserStats::default()
            })
            .unwrap();
        store.save_history(&[sample_record("old")]).unwrap();
        let stats = fs::read(dir.join("stats.json")).unwrap();
        let history = fs::read(dir.join("history.json")).unwrap();
        let blocked = dir.join("history.json.pre-v2.bak");
        fs::create_dir(&blocked).unwrap();
        for _ in 0..2 {
            assert!(store.migrate_stats().is_err());
            assert_eq!(fs::read(dir.join("stats.json")).unwrap(), stats);
            assert_eq!(fs::read(dir.join("history.json")).unwrap(), history);
            assert_eq!(fs::read(dir.join("stats.json.pre-v2.bak")).unwrap(), stats);
            assert!(!dir.join("legacy_stats_baseline.json").exists());
        }
        fs::remove_dir(&blocked).unwrap();
        store.migrate_stats().unwrap();
        assert_eq!(fs::read(dir.join("stats.json.pre-v2.bak")).unwrap(), stats);
        assert_eq!(fs::read(&blocked).unwrap(), history);
        assert_eq!(
            store.load_stats().unwrap().stats_version,
            crate::core::scorer::STATS_VERSION
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
