//! On-Device Adaptive Habit Learner & Auto-Whitelist Engine.
//!
//! Learns from the user's typing corrections (Flip Word, Escape to Raw, Backspace-delete-and-retype,
//! and repeated Telex/VNI double-tap mark undo) and promotes frequently corrected words into the
//! user's custom bypass whitelist so that once a word is learned, it passes through untouched
//! without ever needing to be deleted and retyped again.
//!
//! Includes a configurable high-capacity habit log (default cap: 100 MB with automatic FIFO rotation).

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Default maximum habit log size: 100 MB (100 * 1024 * 1024 bytes).
pub const DEFAULT_MAX_LOG_BYTES: u64 = 100 * 1024 * 1024;

/// Default number of corrections before a word is automatically promoted to the whitelist.
pub const DEFAULT_PROMOTE_THRESHOLD: u32 = 2;

/// Type of user correction event recorded by the adaptive habit learner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionKind {
    /// User pressed Flip Word shortcut (e.g. Ctrl+Shift+Z / Alt+X) to revert accented word to raw.
    FlipToRaw,
    /// User pressed Escape / Shift+Escape to revert accented word to raw.
    EscapeToRaw,
    /// User deleted an accented word via Backspace and immediately retyped it raw/unaccented.
    BackspaceRetype,
    /// User undid a Telex/VNI mark by double-pressing the modifier key (e.g. `w` -> `ư` -> `w`).
    DoubleKeyUndo,
}

impl CorrectionKind {
    /// Short code written to the habit log file.
    pub fn as_str(self) -> &'static str {
        match self {
            CorrectionKind::FlipToRaw => "FLIP_TO_RAW",
            CorrectionKind::EscapeToRaw => "ESC_TO_RAW",
            CorrectionKind::BackspaceRetype => "BACKSPACE_RETYPE",
            CorrectionKind::DoubleKeyUndo => "DOUBLE_KEY_UNDO",
        }
    }
}

/// Adaptive habit learner tracking correction counts and managing persistent whitelist & habit log.
#[derive(Debug, Clone)]
pub struct HabitLearner {
    /// Counts of how many times each lowercase raw token was corrected by the user.
    correction_counts: HashMap<String, u32>,
    /// Set of words already promoted to the whitelist during or prior to this session.
    promoted_words: HashMap<String, bool>,
    /// Optional path to the user dictionary (`vnchkey-dictionary.conf` / `dictionary.conf`)
    /// where newly promoted words are appended immediately.
    dictionary_path: Option<String>,
    /// Optional path to the high-capacity habit log (`vnchkey-habit.log` / `habit.log`).
    log_path: Option<String>,
    /// Maximum size of the habit log file in bytes (default: 100 MB).
    max_log_bytes: u64,
    /// Number of corrections required to promote a word to the whitelist (default: 2).
    promote_threshold: u32,
    /// Enabled flag.
    enabled: bool,
}

impl Default for HabitLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl HabitLearner {
    /// Creates a new `HabitLearner` with default 100 MB log cap and threshold = 2.
    pub fn new() -> Self {
        Self {
            correction_counts: HashMap::new(),
            promoted_words: HashMap::new(),
            dictionary_path: None,
            log_path: None,
            max_log_bytes: DEFAULT_MAX_LOG_BYTES,
            promote_threshold: DEFAULT_PROMOTE_THRESHOLD,
            enabled: true,
        }
    }

    /// Enables or disables adaptive habit learning.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether adaptive habit learning is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the promotion threshold (minimum corrections before auto-whitelisting).
    pub fn set_promote_threshold(&mut self, threshold: u32) {
        self.promote_threshold = threshold.max(1);
    }

    /// Returns the promotion threshold.
    pub fn promote_threshold(&self) -> u32 {
        self.promote_threshold
    }

    /// Sets the maximum habit log size in bytes (e.g. 100 MB).
    pub fn set_max_log_bytes(&mut self, max_bytes: u64) {
        self.max_log_bytes = max_bytes.max(1024 * 1024);
    }

    /// Returns the maximum habit log size in bytes.
    pub fn max_log_bytes(&self) -> u64 {
        self.max_log_bytes
    }

    /// Sets the user dictionary path and loads existing words so we don't duplicate entries.
    pub fn set_dictionary_path(&mut self, path: &str) {
        self.dictionary_path = Some(path.to_string());
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let clean = line.trim().to_lowercase();
                if !clean.is_empty() && !clean.starts_with('#') {
                    self.promoted_words.insert(clean, true);
                }
            }
        }
    }

    /// Configures the habit log file path and replays historical correction counts from disk.
    pub fn set_log_path(&mut self, path: &str) {
        self.log_path = Some(path.to_string());
        self.load_counts_from_log(path);
    }

    /// Reads historical correction events from the habit log file to warm up in-memory counters.
    fn load_counts_from_log(&mut self, path: &str) {
        let Ok(content) = fs::read_to_string(path) else {
            return;
        };
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Format: timestamp_ms\tKIND\traw_word\taccented_word\tcount\tSTATUS
            let mut parts = line.split('\t');
            let _ts = parts.next();
            let _kind = parts.next();
            if let Some(raw) = parts.next() {
                let clean = raw.trim().to_lowercase();
                if clean.len() >= 2 {
                    let entry = self.correction_counts.entry(clean.clone()).or_insert(0);
                    *entry = entry.saturating_add(1);
                    if *entry >= self.promote_threshold {
                        self.promoted_words.insert(clean, true);
                    }
                }
            }
        }
    }

    /// Returns true if `raw` has been learned/promoted into the adaptive whitelist.
    #[inline]
    pub fn is_learned_word(&self, raw: &str) -> bool {
        if !self.enabled {
            return false;
        }
        let clean = raw.trim().to_lowercase();
        self.promoted_words.contains_key(&clean)
    }

    /// Returns the current correction count for a raw word.
    pub fn correction_count(&self, raw: &str) -> u32 {
        let clean = raw.trim().to_lowercase();
        self.correction_counts.get(&clean).copied().unwrap_or(0)
    }

    /// Records a user correction event.
    /// Returns `true` if this word has reached the threshold and is now promoted to the whitelist!
    pub fn record_correction(
        &mut self,
        raw_word: &str,
        accented_word: &str,
        kind: CorrectionKind,
        timestamp_ms: f64,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        let clean = raw_word.trim().to_lowercase();
        // Only learn alphabetic tokens of length 2..=32 (never log numbers/PINs or single letters)
        if clean.len() < 2
            || clean.len() > 32
            || !clean.chars().all(|c| c.is_ascii_alphabetic())
        {
            return false;
        }

        let count = {
            let entry = self.correction_counts.entry(clean.clone()).or_insert(0);
            *entry = entry.saturating_add(1);
            *entry
        };

        let newly_promoted = if count >= self.promote_threshold && !self.promoted_words.contains_key(&clean) {
            self.promoted_words.insert(clean.clone(), true);
            self.append_to_dictionary_file(&clean);
            true
        } else {
            false
        };

        self.append_log_entry(&clean, accented_word, kind, count, newly_promoted, timestamp_ms);

        newly_promoted || self.promoted_words.contains_key(&clean)
    }

    /// Appends a newly promoted word directly to the user dictionary file on disk.
    fn append_to_dictionary_file(&self, clean_word: &str) {
        let Some(ref dict_path) = self.dictionary_path else {
            return;
        };
        let path = Path::new(dict_path);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Check if already in file to avoid duplicates
        if let Ok(existing) = fs::read_to_string(path) {
            if existing
                .lines()
                .any(|l| l.trim().eq_ignore_ascii_case(clean_word))
            {
                return;
            }
        }

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{}", clean_word);
        }
    }

    /// Appends a structured event to the habit log file, rotating (FIFO 50% trim) if it exceeds `max_log_bytes` (100 MB).
    fn append_log_entry(
        &self,
        clean_raw: &str,
        accented: &str,
        kind: CorrectionKind,
        count: u32,
        promoted: bool,
        timestamp_ms: f64,
    ) {
        let Some(ref log_path) = self.log_path else {
            return;
        };
        let path = Path::new(log_path);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Check log file size against max_log_bytes (default 100 MB)
        if let Ok(meta) = fs::metadata(path) {
            if meta.len() >= self.max_log_bytes {
                self.rotate_log_file(path);
            }
        }

        let status = if promoted {
            "WHITELISTED"
        } else if self.promoted_words.contains_key(clean_raw) {
            "ACTIVE_WHITELIST"
        } else {
            "LEARNING"
        };

        let ts = if timestamp_ms > 0.0 {
            timestamp_ms as u64
        } else {
            0
        };

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(
                file,
                "{}\t{}\t{}\t{}\tcount={}\t{}",
                ts,
                kind.as_str(),
                clean_raw,
                accented.trim(),
                count,
                status
            );
        }
    }

    /// FIFO rotation when the log reaches `max_log_bytes` (100 MB): keeps the newest 70% of lines.
    fn rotate_log_file(&self, path: &Path) {
        if let Ok(bytes) = fs::read(path) {
            let keep_from = bytes.len() * 3 / 10; // Drop oldest 30%, keep newest 70%
            if let Some(nl_offset) = bytes[keep_from..].iter().position(|&b| b == b'\n') {
                let start = keep_from + nl_offset + 1;
                let _ = fs::write(path, &bytes[start..]);
            }
        }
    }
}

