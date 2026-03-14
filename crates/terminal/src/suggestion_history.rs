use anyhow::Result;
use chrono::Utc;
use gpui::{App, AppContext as _, Context, Entity, EventEmitter, Global, Task};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Data model ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestionEntry {
    pub command: String,
    /// Unix timestamp of when this command was last used.
    /// `None` for entries saved before this field existed (backward compat).
    #[serde(default)]
    pub last_used: Option<i64>,
}

fn default_max_entries() -> usize {
    10_000
}

/// A flat (context-free) history of commands for autosuggestion.
/// Entries are sorted by recency (most recent first).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestionHistory {
    entries: Vec<SuggestionEntry>,
    #[serde(skip, default = "default_max_entries")]
    max_entries: usize,
}

impl Default for SuggestionHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: default_max_entries(),
        }
    }
}

impl SuggestionHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a command to the history. Deduplicates by command text — moves existing to front.
    pub fn add_command(&mut self, command: String) {
        if command.is_empty() {
            return;
        }
        let now = Utc::now().timestamp();
        // Remove duplicate if exists
        self.entries.retain(|entry| entry.command != command);
        // Insert at front (most recent first)
        self.entries.insert(
            0,
            SuggestionEntry {
                command,
                last_used: Some(now),
            },
        );
        // Enforce max limit
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    /// Find a suggestion for a prefix. Returns the full command text of the most recent match.
    pub fn find_suggestion(&self, prefix: &str) -> Option<&str> {
        if prefix.is_empty() {
            return None;
        }
        self.entries
            .iter()
            .find(|entry| entry.command.starts_with(prefix) && entry.command != prefix)
            .map(|entry| entry.command.as_str())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut history: Self = serde_json::from_str(&content)?;
        history.max_entries = default_max_entries();
        Ok(history)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Remove entries older than `max_age_days`.
    /// Entries with `last_used == None` (pre-upgrade) get stamped with `now` to survive one cycle.
    pub fn purge_expired(&mut self, max_age_days: u64) {
        let now = Utc::now().timestamp();
        let cutoff = now - (max_age_days as i64) * 86400;

        // Backfill legacy entries that have no timestamp
        for entry in &mut self.entries {
            if entry.last_used.is_none() {
                entry.last_used = Some(now);
            }
        }

        self.entries
            .retain(|entry| entry.last_used.unwrap_or(now) >= cutoff);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entries(&self) -> &[SuggestionEntry] {
        &self.entries
    }

    #[cfg(test)]
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }
}

// ── GPUI Entity wrapper ────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum SuggestionHistoryEvent {
    Changed,
}

pub struct GlobalSuggestionHistory(pub Entity<SuggestionHistoryEntity>);
impl Global for GlobalSuggestionHistory {}

pub struct SuggestionHistoryEntity {
    history: SuggestionHistory,
    save_task: Option<Task<()>>,
}

impl EventEmitter<SuggestionHistoryEvent> for SuggestionHistoryEntity {}

impl SuggestionHistoryEntity {
    pub fn init_with_max_age(cx: &mut App, max_age_days: Option<u64>) {
        if cx.try_global::<GlobalSuggestionHistory>().is_some() {
            return;
        }

        let mut history =
            SuggestionHistory::load_from_file(paths::suggestion_history_file()).unwrap_or_else(
                |err| {
                    log::error!("Failed to load suggestion history: {}", err);
                    SuggestionHistory::new()
                },
            );

        let max_age = max_age_days.unwrap_or(7);
        history.purge_expired(max_age);

        let entity = cx.new(|_| Self {
            history,
            save_task: None,
        });

        cx.set_global(GlobalSuggestionHistory(entity));
    }

    pub fn try_global(cx: &App) -> Option<Entity<Self>> {
        cx.try_global::<GlobalSuggestionHistory>()
            .map(|g| g.0.clone())
    }

    pub fn add_command(&mut self, command: String, cx: &mut Context<Self>) {
        self.history.add_command(command);
        self.schedule_save(cx);
    }

    pub fn find_suggestion(&self, prefix: &str) -> Option<&str> {
        self.history.find_suggestion(prefix)
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.history.clear();
        self.schedule_save(cx);
        cx.emit(SuggestionHistoryEvent::Changed);
    }

    fn schedule_save(&mut self, cx: &mut Context<Self>) {
        let history = self.history.clone();
        self.save_task = Some(cx.spawn(async move |_, _| {
            if let Err(err) = history.save_to_file(paths::suggestion_history_file()) {
                log::error!("Failed to save suggestion history: {}", err);
            }
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recency_ordering() {
        let mut history = SuggestionHistory::new();
        history.add_command("show interfaces".to_string());
        history.add_command("show version".to_string());
        history.add_command("show ip route".to_string());

        // Most recent first
        assert_eq!(history.entries()[0].command, "show ip route");
        assert_eq!(history.entries()[1].command, "show version");
        assert_eq!(history.entries()[2].command, "show interfaces");

        // Suggestion returns most recent match
        assert_eq!(
            history.find_suggestion("show"),
            Some("show ip route")
        );
        assert_eq!(
            history.find_suggestion("show v"),
            Some("show version")
        );
        assert_eq!(
            history.find_suggestion("show i"),
            Some("show ip route")
        );
    }

    #[test]
    fn test_deduplication() {
        let mut history = SuggestionHistory::new();
        history.add_command("ls -la".to_string());
        history.add_command("pwd".to_string());
        history.add_command("ls -la".to_string());

        // Should have only 2 entries (deduplicated)
        assert_eq!(history.entries().len(), 2);
        // Re-added command should be at front
        assert_eq!(history.entries()[0].command, "ls -la");
        assert_eq!(history.entries()[1].command, "pwd");
    }

    #[test]
    fn test_max_limit() {
        let mut history = SuggestionHistory::new().with_max_entries(3);
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_command("cmd3".to_string());
        history.add_command("cmd4".to_string());

        assert_eq!(history.entries().len(), 3);
        // Oldest should have been dropped
        assert_eq!(history.entries()[0].command, "cmd4");
        assert_eq!(history.entries()[1].command, "cmd3");
        assert_eq!(history.entries()[2].command, "cmd2");
    }

    #[test]
    fn test_purge_expired() {
        let mut history = SuggestionHistory::new();
        let now = Utc::now().timestamp();

        // Add an old entry manually
        history.entries.push(SuggestionEntry {
            command: "old_command".to_string(),
            last_used: Some(now - 86400 * 10), // 10 days ago
        });
        // Add a recent entry
        history.entries.push(SuggestionEntry {
            command: "recent_command".to_string(),
            last_used: Some(now - 86400 * 2), // 2 days ago
        });

        history.purge_expired(7); // 7-day max age

        assert_eq!(history.entries().len(), 1);
        assert_eq!(history.entries()[0].command, "recent_command");
    }

    #[test]
    fn test_empty_prefix() {
        let mut history = SuggestionHistory::new();
        history.add_command("test".to_string());
        assert_eq!(history.find_suggestion(""), None);
    }

    #[test]
    fn test_exact_prefix_no_match() {
        let mut history = SuggestionHistory::new();
        history.add_command("ls".to_string());
        // Exact match should not suggest itself
        assert_eq!(history.find_suggestion("ls"), None);
        // But prefix should work
        history.add_command("ls -la".to_string());
        assert_eq!(history.find_suggestion("ls"), Some("ls -la"));
    }

    #[test]
    fn test_clear() {
        let mut history = SuggestionHistory::new();
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.clear();
        assert!(history.entries().is_empty());
        assert_eq!(history.find_suggestion("cmd"), None);
    }

    #[test]
    fn test_legacy_entries_get_stamped() {
        let mut history = SuggestionHistory::new();
        history.entries.push(SuggestionEntry {
            command: "legacy".to_string(),
            last_used: None,
        });

        history.purge_expired(7);

        // Legacy entry should survive (gets stamped with now)
        assert_eq!(history.entries().len(), 1);
        assert!(history.entries()[0].last_used.is_some());
    }
}
