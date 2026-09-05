use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::data::{
    models::{Command, CommandProgress, UserStats},
    progress::{ProgressStore, backup_file_atomic},
};

#[derive(Default, Deserialize)]
pub struct CommandAliases {
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
    #[serde(skip)]
    canonical_targets: BTreeMap<String, u32>,
}

impl CommandAliases {
    pub fn load(data_dir: &Path, commands: &[Command]) -> Result<Self> {
        let path = data_dir.join("command_aliases.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let mut aliases: Self = toml::from_str(&fs::read_to_string(path)?)?;
        let canonical: HashSet<_> = commands.iter().map(|command| command.id.as_str()).collect();
        for (old, new) in &aliases.aliases {
            ensure!(
                !old.is_empty() && old != new && canonical.contains(new.as_str()),
                "invalid command alias {old} -> {new}"
            );
            ensure!(
                !canonical.contains(old.as_str()),
                "alias shadows canonical command {old}"
            );
        }
        aliases.canonical_targets = commands
            .iter()
            .map(|command| (command.id.clone(), command.difficulty.target_attempts()))
            .collect();
        Ok(aliases)
    }

    pub fn resolve<'a>(&'a self, id: &'a str) -> &'a str {
        self.aliases.get(id).map(String::as_str).unwrap_or(id)
    }

    pub fn migrate(&self, store: &ProgressStore) -> Result<()> {
        if self.aliases.is_empty() {
            return Ok(());
        }
        let mut history = store.load_history()?;
        let mut history_changed = false;
        for record in &mut history {
            if let Some(id) = self.aliases.get(&record.command_id) {
                record.command_id = id.clone();
                history_changed = true;
            }
        }
        let mut stats = store.load_stats()?;
        let stats_changed = self.has_alias_progress(&stats);
        let baseline_path = store.base_dir().join("legacy_stats_baseline.json");
        let mut baseline: Option<UserStats> = if baseline_path.exists() {
            Some(serde_json::from_str(&fs::read_to_string(&baseline_path)?)?)
        } else {
            None
        };
        let baseline_changed = baseline
            .as_ref()
            .is_some_and(|stats| self.has_alias_progress(stats));
        if !history_changed && !stats_changed && !baseline_changed {
            return Ok(());
        }
        for name in ["stats.json", "history.json", "legacy_stats_baseline.json"] {
            let source = store.base_dir().join(name);
            let backup = source.with_extension("json.pre-catalog.bak");
            if source.try_exists()? {
                backup_file_atomic(&source, &backup)?;
            }
        }
        // History is authoritative, and the stats-only baseline must be migrated
        // before cached stats are replayed to avoid resurrecting old IDs.
        if history_changed {
            store.save_history(&history)?;
        }
        if baseline_changed {
            if let Some(baseline) = baseline.as_mut() {
                self.normalize_progress(baseline);
                store.save_legacy_stats_baseline(baseline)?;
            }
        }
        if stats_changed {
            self.normalize_progress(&mut stats);
            store.save_stats(&stats)?;
        }
        Ok(())
    }

    fn has_alias_progress(&self, stats: &UserStats) -> bool {
        stats
            .command_progress
            .iter()
            .any(|progress| self.aliases.contains_key(&progress.command_id))
    }

    pub fn normalize_progress(&self, stats: &mut UserStats) {
        let mut combined: BTreeMap<String, CommandProgress> = BTreeMap::new();
        for mut progress in std::mem::take(&mut stats.command_progress) {
            progress.command_id = self.resolve(&progress.command_id).to_owned();
            if let Some(existing) = combined.get_mut(&progress.command_id) {
                existing.times_practiced = existing
                    .times_practiced
                    .saturating_add(progress.times_practiced);
                existing.best_wpm = existing.best_wpm.max(progress.best_wpm);
                existing.best_accuracy = existing.best_accuracy.max(progress.best_accuracy);
                existing.last_practiced = existing.last_practiced.max(progress.last_practiced);
                existing.mastery = existing.mastery.max(progress.mastery);
            } else {
                combined.insert(progress.command_id.clone(), progress);
            }
        }
        for progress in combined.values_mut() {
            if let Some(target) = self.canonical_targets.get(&progress.command_id) {
                progress.mastery = crate::core::scorer::compute_mastery(
                    progress.best_accuracy,
                    progress.times_practiced,
                    *target,
                );
            }
        }
        stats.command_progress = combined.into_values().collect();
    }
}
