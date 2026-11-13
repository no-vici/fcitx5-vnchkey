//! Configuration manager for Fcitx5 vnchKey Settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Main vnchKey engine and addon settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VnchkeyConfig {
    pub input_method: String,
    pub mode: String,
    pub output_charset: String,
    pub output_mode: String,
    pub remember_app_mode: bool,
    pub preedit_tooltip: bool,
    pub persistent_context_guard: bool,
    pub select_and_replace_web: bool,
    pub enable_word_rescue: bool,
    pub direct_delay_ms: i32,
    pub idle_timeout_ms: u64,
    pub type_w_to_u: String,
    pub bracket_transform: String,
    pub modern_style: bool,
    pub smart_horn: bool,
    pub spell_check: bool,
    pub auto_restore_english: bool,
    pub enable_dictionary: bool,
    pub smart_casing: bool,
    pub free_marking: bool,
    pub fast_consonant_doubling: bool,
    pub quick_start_consonant: bool,
    pub quick_end_consonant: bool,
    pub enable_macro: bool,
    pub capitalize_macro: bool,
    pub esc_restore_raw: bool,
    pub switch_shortcut: String,
    pub flip_shortcut: String,
    pub temp_english_mode: String,
    pub tray_icon_theme: String,
    pub show_indicator_badge: bool,
    pub learn_typing_habits: bool,
    pub habit_promote_threshold: u32,
    pub initial_case_anchor: bool,
}

impl Default for VnchkeyConfig {
    fn default() -> Self {
        Self {
            input_method: "Auto".to_string(),
            mode: "Uinput (Smooth)".to_string(),
            output_charset: "Unicode".to_string(),
            output_mode: "Direct".to_string(),
            remember_app_mode: true,
            preedit_tooltip: true,
            persistent_context_guard: true,
            select_and_replace_web: true,
            enable_word_rescue: true,
            direct_delay_ms: 0,
            idle_timeout_ms: 0,
            type_w_to_u: "Non-Start".to_string(),
            bracket_transform: "Disabled".to_string(),
            modern_style: false,
            smart_horn: true,
            spell_check: true,
            auto_restore_english: true,
            enable_dictionary: true,
            smart_casing: true,
            free_marking: true,
            fast_consonant_doubling: false,
            quick_start_consonant: false,
            quick_end_consonant: false,
            enable_macro: true,
            capitalize_macro: true,
            esc_restore_raw: false,
            switch_shortcut: "Ctrl+Shift".to_string(),
            flip_shortcut: "Ctrl+Shift+Z".to_string(),
            temp_english_mode: "Disabled".to_string(),
            tray_icon_theme: "Default".to_string(),
            show_indicator_badge: true,
            learn_typing_habits: true,
            habit_promote_threshold: 4,
            initial_case_anchor: false,
        }
    }
}

/// Backup data container for full JSON export/import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub config: VnchkeyConfig,
    pub macros: Vec<(String, String)>,
    pub dictionary: Vec<String>,
    pub app_modes: Vec<(String, String)>,
    #[serde(default)]
    pub habit_log: Option<String>,
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn config_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let mut p = PathBuf::from(home);
            p.push(".config/fcitx5/conf");
            p
        } else {
            PathBuf::from("/tmp")
        }
    }

    pub fn conf_path() -> PathBuf {
        Self::config_dir().join("vnchkey.conf")
    }

    pub fn macro_path() -> PathBuf {
        Self::config_dir().join("vnchkey-macro-table.conf")
    }

    pub fn saigon_macro_path() -> PathBuf {
        Self::config_dir().join("saigon-xua-macro.conf")
    }

    pub const SAIGON_MACRO_CONTENT: &'static str = include_str!("../../../presets/saigon-xua-macro.conf");

    pub fn ensure_saigon_macro_preset() {
        let path = Self::saigon_macro_path();
        if !path.exists() {
            let _ = fs::create_dir_all(Self::config_dir());
            let _ = fs::write(&path, Self::SAIGON_MACRO_CONTENT);
        }
    }

    pub fn dict_path() -> PathBuf {
        Self::config_dir().join("vnchkey-dictionary.conf")
    }

    pub fn habit_log_path() -> PathBuf {
        Self::config_dir().join("vnchkey-habit.log")
    }

    pub fn app_modes_path() -> PathBuf {
        Self::config_dir().join("vnchkey-apps.conf")
    }

    pub fn legacy_app_modes_path() -> PathBuf {
        Self::config_dir().join("vnchkey-app-modes.conf")
    }

    pub fn load_config() -> VnchkeyConfig {
        let mut cfg = VnchkeyConfig::default();
        let path = Self::conf_path();
        if !path.exists() {
            return cfg;
        }

        if let Ok(content) = fs::read_to_string(&path) {
            let mut map = HashMap::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim().to_string();
                    let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    map.insert(k, v);
                }
            }

            if let Some(v) = map.get("InputMethod") { cfg.input_method = v.clone(); }
            if let Some(v) = map.get("Mode") { cfg.mode = v.clone(); }
            if let Some(v) = map.get("OutputCharset") { cfg.output_charset = v.clone(); }
            if let Some(v) = map.get("OutputMode") { cfg.output_mode = v.clone(); }
            if let Some(v) = map.get("RememberAppMode") { cfg.remember_app_mode = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("PreeditTooltip") { cfg.preedit_tooltip = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("PersistentContextGuard") { cfg.persistent_context_guard = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("SelectAndReplaceWeb") { cfg.select_and_replace_web = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("EnableWordRescue") { cfg.enable_word_rescue = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("DirectDelayMs") { cfg.direct_delay_ms = v.parse().unwrap_or(0); }
            if let Some(v) = map.get("IdleTimeoutMs") { cfg.idle_timeout_ms = v.parse().unwrap_or(0); }
            if let Some(v) = map.get("TypeW2U") { cfg.type_w_to_u = v.clone(); }
            if let Some(v) = map.get("BracketTransform") { cfg.bracket_transform = v.clone(); }
            if let Some(v) = map.get("ModernStyle") { cfg.modern_style = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("SmartHorn") { cfg.smart_horn = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("SpellCheck") { cfg.spell_check = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("AutoRestoreEnglish") { cfg.auto_restore_english = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("EnableDictionary") { cfg.enable_dictionary = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("SmartCasing") { cfg.smart_casing = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("FreeMarking") { cfg.free_marking = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("FastConsonantDoubling") { cfg.fast_consonant_doubling = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("QuickTelex") { cfg.fast_consonant_doubling = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("QuickStartConsonant") { cfg.quick_start_consonant = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("QuickEndConsonant") { cfg.quick_end_consonant = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("EnableMacro") { cfg.enable_macro = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("CapitalizeMacro") { cfg.capitalize_macro = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("EscRestoreRaw") { cfg.esc_restore_raw = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("SwitchShortcut") { cfg.switch_shortcut = v.clone(); }
            if let Some(v) = map.get("FlipShortcut") { cfg.flip_shortcut = v.clone(); }
            if let Some(v) = map.get("TempEnglishMode") { cfg.temp_english_mode = v.clone(); }
            if let Some(v) = map.get("TrayIconTheme") { cfg.tray_icon_theme = v.clone(); }
            if let Some(v) = map.get("ShowIndicatorBadge") { cfg.show_indicator_badge = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("LearnTypingHabits") { cfg.learn_typing_habits = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("HabitPromoteThreshold") { cfg.habit_promote_threshold = v.parse().unwrap_or(4).max(1); }
            if let Some(v) = map.get("InitialCaseAnchor") { cfg.initial_case_anchor = v.eq_ignore_ascii_case("true"); }
        }

        cfg
    }

    pub fn save_config(cfg: &VnchkeyConfig) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        let mut lines = Vec::new();
        lines.push("# vnchKey Configuration File".to_string());
        lines.push("# Generated by Native Fcitx5 vnchKey Settings (100% Rust)".to_string());
        lines.push(String::new());
        lines.push(format!("InputMethod={}", cfg.input_method));
        lines.push(format!("Mode={}", cfg.mode));
        lines.push(format!("OutputCharset={}", cfg.output_charset));
        lines.push(format!("OutputMode={}", cfg.output_mode));
        lines.push(format!("RememberAppMode={}", if cfg.remember_app_mode { "True" } else { "False" }));
        lines.push(format!("PreeditTooltip={}", if cfg.preedit_tooltip { "True" } else { "False" }));
        lines.push(format!("PersistentContextGuard={}", if cfg.persistent_context_guard { "True" } else { "False" }));
        lines.push(format!("SelectAndReplaceWeb={}", if cfg.select_and_replace_web { "True" } else { "False" }));
        lines.push(format!("EnableWordRescue={}", if cfg.enable_word_rescue { "True" } else { "False" }));
        lines.push(format!("DirectDelayMs={}", cfg.direct_delay_ms));
        lines.push(format!("IdleTimeoutMs={}", cfg.idle_timeout_ms));
        lines.push(format!("TypeW2U={}", cfg.type_w_to_u));
        lines.push(format!("BracketTransform={}", cfg.bracket_transform));
        lines.push(format!("ModernStyle={}", if cfg.modern_style { "True" } else { "False" }));
        lines.push(format!("SmartHorn={}", if cfg.smart_horn { "True" } else { "False" }));
        lines.push(format!("SpellCheck={}", if cfg.spell_check { "True" } else { "False" }));
        lines.push(format!("AutoRestoreEnglish={}", if cfg.auto_restore_english { "True" } else { "False" }));
        lines.push(format!("EnableDictionary={}", if cfg.enable_dictionary { "True" } else { "False" }));
        lines.push(format!("SmartCasing={}", if cfg.smart_casing { "True" } else { "False" }));
        lines.push(format!("InitialCaseAnchor={}", if cfg.initial_case_anchor { "True" } else { "False" }));
        lines.push(format!("FreeMarking={}", if cfg.free_marking { "True" } else { "False" }));
        lines.push(format!("FastConsonantDoubling={}", if cfg.fast_consonant_doubling { "True" } else { "False" }));
        lines.push(format!("QuickTelex={}", if cfg.fast_consonant_doubling { "True" } else { "False" }));
        lines.push(format!("QuickStartConsonant={}", if cfg.quick_start_consonant { "True" } else { "False" }));
        lines.push(format!("QuickEndConsonant={}", if cfg.quick_end_consonant { "True" } else { "False" }));
        lines.push(format!("EnableMacro={}", if cfg.enable_macro { "True" } else { "False" }));
        lines.push(format!("CapitalizeMacro={}", if cfg.capitalize_macro { "True" } else { "False" }));
        lines.push(format!("EscRestoreRaw={}", if cfg.esc_restore_raw { "True" } else { "False" }));
        lines.push(format!("SwitchShortcut={}", cfg.switch_shortcut));
        lines.push(format!("FlipShortcut={}", cfg.flip_shortcut));
        lines.push(format!("TempEnglishMode={}", cfg.temp_english_mode));
        lines.push(format!("TrayIconTheme={}", cfg.tray_icon_theme));
        lines.push(format!("ShowIndicatorBadge={}", if cfg.show_indicator_badge { "True" } else { "False" }));
        lines.push(format!("LearnTypingHabits={}", if cfg.learn_typing_habits { "True" } else { "False" }));
        lines.push(format!("HabitPromoteThreshold={}", cfg.habit_promote_threshold));

        fs::write(Self::conf_path(), lines.join("\n") + "\n")?;
        Self::reload_fcitx5();
        Ok(())
    }

    pub fn load_macros() -> Vec<(String, String)> {
        let path = Self::macro_path();
        if !path.exists() {
            return vec![
                ("vn".to_string(), "Việt Nam".to_string()),
                ("ch".to_string(), "Cộng Hòa".to_string()),
                ("td".to_string(), "Tự Do".to_string()),
                ("dc".to_string(), "được".to_string()),
                ("ko".to_string(), "không".to_string()),
                ("ng".to_string(), "người".to_string()),
                ("hn".to_string(), "hôm nay".to_string()),
            ];
        }
        Self::load_macros_from_file(&path)
    }

    pub fn load_macros_from_file(path: &Path) -> Vec<(String, String)> {
        let mut res = Vec::new();
        let mut seen = std::collections::HashSet::new();
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let pair = if let Some(p) = line.split_once('=') {
                    Some(p)
                } else if let Some(p) = line.split_once(':') {
                    Some(p)
                } else if let Some(p) = line.split_once(';') {
                    Some(p)
                } else {
                    None
                };
                if let Some((k, v)) = pair {
                    let key = k.trim().to_string();
                    let norm = key.to_lowercase();
                    if !norm.is_empty() && !seen.contains(&norm) {
                        seen.insert(norm);
                        res.push((key, v.trim().to_string()));
                    }
                }
            }
        }
        res
    }

    /// Loại bỏ các macro trùng lặp phím tắt (không phân biệt hoa/thường), giữ lại định nghĩa đầu tiên.
    pub fn deduplicate_macros(macros: &mut Vec<(String, String)>) -> usize {
        let mut seen = std::collections::HashSet::new();
        let before = macros.len();
        let mut deduped = Vec::new();
        for (k, v) in macros.drain(..) {
            let norm = k.trim().to_lowercase();
            if !norm.is_empty() {
                if !seen.contains(&norm) {
                    seen.insert(norm);
                    deduped.push((k, v));
                }
            } else {
                // Giữ lại hàng trống nếu người dùng vừa mới bấm thêm
                deduped.push((k, v));
            }
        }
        let removed = before - deduped.len();
        *macros = deduped;
        removed
    }

    pub fn save_macros(macros: &[(String, String)]) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        let mut lines = Vec::new();
        lines.push("# vnchKey Macro Replacement Table".to_string());
        let mut seen = std::collections::HashSet::new();
        for (k, v) in macros {
            let key = k.trim();
            let norm = key.to_lowercase();
            if !norm.is_empty() && !seen.contains(&norm) {
                seen.insert(norm);
                lines.push(format!("{}={}", key, v.trim()));
            }
        }

        fs::write(Self::macro_path(), lines.join("\n") + "\n")?;
        Self::reload_fcitx5();
        Ok(())
    }

    pub fn default_dictionary() -> Vec<String> {
        let mut words: Vec<String> = vnchkey_core::DEFAULT_ENGLISH_WORDS
            .iter()
            .map(|w| w.trim().to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();
        words.sort();
        words.dedup();
        words
    }

    pub fn load_dictionary() -> Vec<String> {
        let path = Self::dict_path();
        if !path.exists() {
            return Self::default_dictionary();
        }

        let mut res = Vec::new();
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    res.push(line.to_string());
                }
            }
        }
        if res.is_empty() {
            return Self::default_dictionary();
        }
        res
    }

    pub fn save_dictionary(words: &[String]) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        // Merge with existing on-disk dictionary to preserve newly learned words from HabitLearner
        let mut merged = Self::load_dictionary();
        for w in words {
            let trimmed = w.trim();
            if !trimmed.is_empty() && !merged.iter().any(|m| m.eq_ignore_ascii_case(trimmed)) {
                merged.push(trimmed.to_string());
            }
        }
        merged.sort_by_key(|a| a.to_lowercase());
        merged.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

        let mut lines = Vec::new();
        lines.push("# vnchKey Custom User Words Dictionary".to_string());
        for w in &merged {
            let trimmed = w.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }

        fs::write(Self::dict_path(), lines.join("\n") + "\n")?;
        Self::reload_fcitx5();
        Ok(())
    }

    pub fn load_app_modes() -> Vec<(String, String)> {
        let path = Self::app_modes_path();
        let legacy_path = Self::legacy_app_modes_path();
        let target_path = if path.exists() {
            path
        } else if legacy_path.exists() {
            legacy_path
        } else {
            return vec![
                ("alacritty".to_string(), "Bypass (Raw English)".to_string()),
                ("kitty".to_string(), "Bypass (Raw English)".to_string()),
                ("steam".to_string(), "Bypass (Raw English)".to_string()),
                ("code".to_string(), "Direct".to_string()),
            ];
        };

        let mut res = Vec::new();
        if let Ok(content) = fs::read_to_string(&target_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((app, mode)) = line.split_once('=') {
                    res.push((app.trim().to_string(), mode.trim().to_string()));
                }
            }
        }
        res
    }

    pub fn save_app_modes(app_modes: &[(String, String)]) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        let mut lines = Vec::new();
        lines.push("# vnchKey Per-Application Modes".to_string());
        for (app, mode) in app_modes {
            if !app.trim().is_empty() {
                lines.push(format!("{}={}", app.trim(), mode.trim()));
            }
        }

        let content = lines.join("\n") + "\n";
        fs::write(Self::app_modes_path(), &content)?;
        let _ = fs::write(Self::legacy_app_modes_path(), &content);
        Self::reload_fcitx5();
        Ok(())
    }

    pub fn reset_to_defaults() {
        let _ = Self::save_config(&VnchkeyConfig::default());
        let _ = fs::remove_file(Self::macro_path());
        let _ = fs::remove_file(Self::dict_path());
        let _ = fs::remove_file(Self::app_modes_path());
        let _ = fs::remove_file(Self::legacy_app_modes_path());
        Self::reload_fcitx5();
    }

    pub fn export_backup(path: &Path) -> std::io::Result<()> {
        let habit_log_content = fs::read_to_string(Self::habit_log_path()).ok();
        let backup = BackupData {
            config: Self::load_config(),
            macros: Self::load_macros(),
            dictionary: Self::load_dictionary(),
            app_modes: Self::load_app_modes(),
            habit_log: habit_log_content,
        };
        let json = serde_json::to_string_pretty(&backup).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    pub fn import_backup(path: &Path) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        let backup: BackupData = serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Self::save_config(&backup.config)?;
        Self::save_macros(&backup.macros)?;
        Self::save_dictionary(&backup.dictionary)?;
        Self::save_app_modes(&backup.app_modes)?;
        if let Some(log_data) = backup.habit_log {
            let log_p = Self::habit_log_path();
            let _ = fs::write(log_p, log_data);
        }
        Ok(())
    }

    pub fn reload_fcitx5() {
        // 1. fcitx5-remote -r
        let _ = Command::new("fcitx5-remote").arg("-r").output();

        // 2. D-Bus reload method
        let _ = Command::new("gdbus")
            .args([
                "call", "--session",
                "--dest", "org.fcitx.Fcitx5",
                "--object-path", "/controller",
                "--method", "org.fcitx.Fcitx.Controller1.ReloadAddonConfig",
                "vnchkey"
            ])
            .output();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_macros() {
        let mut macros = vec![
            ("vn".to_string(), "Việt Nam".to_string()),
            ("ch".to_string(), "Cộng Hòa".to_string()),
            ("VN".to_string(), "Vạn Năng".to_string()),
            ("ch ".to_string(), "Cộng Hòa Xã Hội".to_string()),
            ("td".to_string(), "Tự Do".to_string()),
        ];
        let removed = ConfigManager::deduplicate_macros(&mut macros);
        assert_eq!(removed, 2);
        assert_eq!(macros.len(), 3);
        assert_eq!(macros[0], ("vn".to_string(), "Việt Nam".to_string()));
        assert_eq!(macros[1], ("ch".to_string(), "Cộng Hòa".to_string()));
        assert_eq!(macros[2], ("td".to_string(), "Tự Do".to_string()));
    }
}


