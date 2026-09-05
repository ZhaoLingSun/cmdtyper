use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use chrono::Utc;
use cmdtyper::app::{App, AppState};
use cmdtyper::data::aliases::CommandAliases;
use cmdtyper::data::models::{Command, CommandProgress, Difficulty, SessionRecord, UserStats};
use cmdtyper::data::progress::ProgressStore;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct Fixture {
    _lock: MutexGuard<'static, ()>,
    root: PathBuf,
    old_user: Option<OsString>,
    old_data: Option<OsString>,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let root = env::temp_dir().join(format!(
            "cmdtyper-alias-{label}-{}",
            Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir_all(root.join("catalog")).unwrap();
        let old_user = env::var_os("CMDTYPER_USER_DIR");
        let old_data = env::var_os("CMDTYPER_DATA_DIR");
        // These tests serialize all environment changes inside this test binary.
        unsafe {
            env::set_var("CMDTYPER_USER_DIR", root.join("user"));
            env::set_var(
                "CMDTYPER_DATA_DIR",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"),
            );
        }
        Self {
            _lock: lock,
            root,
            old_user,
            old_data,
        }
    }

    fn aliases(&self, difficulty: Difficulty) -> CommandAliases {
        fs::write(
            self.root.join("catalog/command_aliases.toml"),
            "[aliases]\nold_a = 'canonical'\nold_b = 'canonical'\n",
        )
        .unwrap();
        CommandAliases::load(
            &self.root.join("catalog"),
            &[Command {
                id: "canonical".to_owned(),
                difficulty,
                ..Command::default()
            }],
        )
        .unwrap()
    }

    fn store(&self) -> ProgressStore {
        ProgressStore::new().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        unsafe {
            match &self.old_user {
                Some(value) => env::set_var("CMDTYPER_USER_DIR", value),
                None => env::remove_var("CMDTYPER_USER_DIR"),
            }
            match &self.old_data {
                Some(value) => env::set_var("CMDTYPER_DATA_DIR", value),
                None => env::remove_var("CMDTYPER_DATA_DIR"),
            }
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn progress(id: &str, count: u32) -> CommandProgress {
    CommandProgress {
        command_id: id.to_owned(),
        times_practiced: count,
        best_accuracy: 1.0,
        best_wpm: 40.0,
        ..CommandProgress::default()
    }
}

fn record(id: &str, command_id: &str) -> SessionRecord {
    SessionRecord {
        id: id.to_owned(),
        command_id: command_id.to_owned(),
        started_at: 100,
        finished_at: 200,
        wpm: 40.0,
        cpm: 200.0,
        accuracy: 1.0,
        difficulty: Difficulty::Beginner,
        ..SessionRecord::default()
    }
}

#[test]
fn merged_alias_mastery_uses_each_canonical_difficulty_target() {
    let fixture = Fixture::new("difficulty");
    for difficulty in [
        Difficulty::Beginner,
        Difficulty::Basic,
        Difficulty::Advanced,
        Difficulty::Practical,
    ] {
        let aliases = fixture.aliases(difficulty);
        let mut stats = UserStats {
            command_progress: vec![progress("old_a", 1), progress("old_b", 2)],
            ..UserStats::default()
        };
        aliases.normalize_progress(&mut stats);
        assert_eq!(stats.command_progress.len(), 1);
        assert_eq!(stats.command_progress[0].command_id, "canonical");
        assert_eq!(stats.command_progress[0].times_practiced, 3);
        assert_eq!(
            stats.command_progress[0].mastery,
            (3.0 / difficulty.target_attempts() as f64).min(1.0)
        );
        let once = stats.clone();
        aliases.normalize_progress(&mut stats);
        assert_eq!(stats, once);
    }
}

#[test]
fn alias_migration_preserves_record_ids_and_is_idempotent() {
    let fixture = Fixture::new("history");
    let aliases = fixture.aliases(Difficulty::Beginner);
    let store = fixture.store();
    let records = vec![
        record("one", "old_a"),
        record("two", "old_b"),
        record("three", "old_b"),
    ];
    store.save_history(&records).unwrap();
    store
        .save_stats(&UserStats {
            total_sessions: 3,
            command_progress: vec![progress("old_a", 1), progress("old_b", 2)],
            ..UserStats::default()
        })
        .unwrap();
    let original = fs::read(store.base_dir().join("history.json")).unwrap();
    aliases.migrate(&store).unwrap();
    let history = store.load_history().unwrap();
    assert_eq!(
        history
            .iter()
            .map(|record| record.id.as_str())
            .collect::<Vec<_>>(),
        vec!["one", "two", "three"]
    );
    assert!(
        history
            .iter()
            .all(|record| record.command_id == "canonical")
    );
    assert_eq!(
        fs::read(store.base_dir().join("history.json.pre-catalog.bak")).unwrap(),
        original
    );
    let first = store.migrate_stats().unwrap();
    assert_eq!(first.total_sessions, 3);
    assert_eq!(first.command_progress[0].times_practiced, 3);
    aliases.migrate(&store).unwrap();
    assert_eq!(store.migrate_stats().unwrap(), first);
}

#[test]
fn catalog_updates_migrate_stats_only_baselines_without_doubling_progress() {
    let fixture = Fixture::new("baseline");
    let store = fixture.store();
    store
        .save_stats(&UserStats {
            total_sessions: 2,
            total_wpm_sessions: 2,
            command_progress: vec![progress("old_a", 2)],
            ..UserStats::default()
        })
        .unwrap();
    store.migrate_stats().unwrap();
    let aliases = fixture.aliases(Difficulty::Beginner);
    for _ in 0..3 {
        aliases.migrate(&store).unwrap();
        let stats = store.migrate_stats().unwrap();
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.command_progress.len(), 1);
        assert_eq!(stats.command_progress[0].command_id, "canonical");
        assert_eq!(stats.command_progress[0].times_practiced, 2);
    }
    assert!(
        store
            .base_dir()
            .join("legacy_stats_baseline.json.pre-catalog.bak")
            .exists()
    );
}

#[test]
fn failed_history_write_does_not_mark_record_applied_in_app() {
    let _fixture = Fixture::new("history-failure");
    let mut app = App::new().unwrap();
    let path = app.progress_store.base_dir().join("history.json");
    fs::write(&path, "{corrupt-source").unwrap();
    app.persist_record(record("not-saved", "canonical"));
    assert!(app.persistence_error.is_some());
    assert!(app.history.is_empty());
    assert!(app.user_stats.applied_record_ids.is_empty());
    assert_eq!(app.user_stats.total_sessions, 0);
    assert_eq!(fs::read_to_string(path).unwrap(), "{corrupt-source");
}

#[test]
fn failed_cached_stat_write_can_retry_same_snapshot_without_recounting() {
    let _fixture = Fixture::new("cache-failure");
    let mut app = App::new().unwrap();
    let stats_path = app.progress_store.base_dir().join("stats.json");
    fs::create_dir(&stats_path).unwrap();
    let record = record("saved-history", "canonical");
    app.persist_record(record.clone());
    assert!(app.persistence_error.is_some());
    assert_eq!(app.history.len(), 1);
    assert_eq!(app.user_stats.total_sessions, 1);
    fs::remove_dir(&stats_path).unwrap();
    app.persist_record(record);
    assert!(app.persistence_error.is_none());
    assert_eq!(app.user_stats.total_sessions, 1);
    assert_eq!(app.progress_store.load_history().unwrap().len(), 1);
    assert_eq!(app.progress_store.load_stats().unwrap().total_sessions, 1);
}

#[test]
fn multiline_partial_and_completed_snapshots_share_one_app_record() {
    let _fixture = Fixture::new("multiline");
    let mut app = App::new().unwrap();
    let command = Command {
        id: "integration-sequence".to_owned(),
        command: "ab\ncd".to_owned(),
        ..Command::default()
    };
    app.typing_commands = vec![command];
    app.typing_index = 0;
    app.state = AppState::Typing;
    app.typing_engine.reset("ab\ncd");
    for code in [
        KeyCode::Char('a'),
        KeyCode::Char('b'),
        KeyCode::Enter,
        KeyCode::Char('c'),
    ] {
        app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
    }
    assert_eq!(app.typing_engine.cursor, 4);
    app.save_active_typing();
    assert_eq!(app.history.len(), 1);
    assert!(!app.history[0].is_completed());
    for _ in 0..3 {
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
    }
    app.save_active_typing();
    assert_eq!(app.history.len(), 1);
    assert_eq!(app.history[0].error_count, 1);
    app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.history.len(), 1);
    assert!(app.history[0].is_completed());
    assert_eq!(app.user_stats.completed_chars, 4);
    assert_eq!(app.user_stats.attempted_positions, 4);
    assert_eq!(app.user_stats.error_positions, 1);
    assert_eq!(app.user_stats.command_progress[0].times_practiced, 1);
}

#[test]
fn failed_history_write_on_enter_keeps_command_until_retry_is_durable() {
    let _fixture = Fixture::new("enter-durability");
    let mut app = App::new().unwrap();
    app.typing_commands = vec![
        Command {
            id: "first-command".to_owned(),
            command: "a".to_owned(),
            ..Command::default()
        },
        Command {
            id: "second-command".to_owned(),
            command: "b".to_owned(),
            ..Command::default()
        },
    ];
    app.typing_index = 0;
    app.state = AppState::Typing;
    app.typing_engine.reset("a");
    let history_path = app.progress_store.base_dir().join("history.json");
    fs::create_dir(&history_path).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(app.persistence_error.is_some());
    assert_eq!(app.state, AppState::Typing);
    assert_eq!(app.typing_index, 0);
    assert!(app.typing_engine.is_complete());
    assert!(app.history.is_empty());
    assert!(app.user_stats.applied_record_ids.is_empty());
    fs::remove_dir(&history_path).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(app.persistence_error.is_none());
    assert_eq!(app.typing_index, 1);
    assert_eq!(app.typing_engine.target, vec!['b']);
    assert_eq!(app.history.len(), 1);
    assert_eq!(app.progress_store.load_history().unwrap().len(), 1);
    assert_eq!(app.user_stats.command_progress[0].times_practiced, 1);
}

#[test]
fn failed_catalog_backup_preserves_history_stats_and_baseline_until_retry() {
    let fixture = Fixture::new("backup-failure");
    let aliases = fixture.aliases(Difficulty::Beginner);
    let store = fixture.store();
    store.save_history(&[record("one", "old_a")]).unwrap();
    let stats = UserStats {
        total_sessions: 1,
        command_progress: vec![progress("old_a", 1)],
        ..UserStats::default()
    };
    store.save_stats(&stats).unwrap();
    store.save_legacy_stats_baseline(&stats).unwrap();
    let names = ["stats.json", "history.json", "legacy_stats_baseline.json"];
    let originals = names.map(|name| fs::read(store.base_dir().join(name)).unwrap());
    let blocked = store.base_dir().join("history.json.pre-catalog.bak");
    fs::create_dir(&blocked).unwrap();
    for _ in 0..2 {
        assert!(aliases.migrate(&store).is_err());
        for (name, bytes) in names.iter().zip(&originals) {
            assert_eq!(fs::read(store.base_dir().join(name)).unwrap(), *bytes);
        }
        assert_eq!(
            fs::read(store.base_dir().join("stats.json.pre-catalog.bak")).unwrap(),
            originals[0],
        );
        assert!(
            !store
                .base_dir()
                .join("legacy_stats_baseline.json.pre-catalog.bak")
                .exists()
        );
    }
    fs::remove_dir(&blocked).unwrap();
    aliases.migrate(&store).unwrap();
    for (name, bytes) in names.iter().zip(&originals) {
        let backup = store
            .base_dir()
            .join(name)
            .with_extension("json.pre-catalog.bak");
        assert_eq!(fs::read(backup).unwrap(), *bytes);
    }
    assert_eq!(store.load_history().unwrap()[0].command_id, "canonical");
    assert_eq!(
        store.load_stats().unwrap().command_progress[0].command_id,
        "canonical"
    );
}

#[test]
fn catalog_migration_preserves_preexisting_backup_bytes() {
    let fixture = Fixture::new("existing-backup");
    let aliases = fixture.aliases(Difficulty::Beginner);
    let store = fixture.store();
    store.save_history(&[record("one", "old_a")]).unwrap();
    let backup = store.base_dir().join("history.json.pre-catalog.bak");
    fs::write(&backup, b"original backup from an earlier migration").unwrap();
    aliases.migrate(&store).unwrap();
    assert_eq!(
        fs::read(&backup).unwrap(),
        b"original backup from an earlier migration"
    );
    assert_eq!(store.load_history().unwrap()[0].command_id, "canonical");
}
