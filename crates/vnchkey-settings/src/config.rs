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
    pub type_w_to_u: String,
    pub bracket_transform: String,
    pub modern_style: bool,
    pub smart_horn: bool,
    pub auto_restore_english: bool,
    pub smart_casing: bool,
    pub free_marking: bool,
    pub fast_consonant_doubling: bool,
    pub enable_macro: bool,
    pub capitalize_macro: bool,
    pub switch_shortcut: String,
    pub temp_english_mode: String,
    pub tray_icon_theme: String,
    pub show_indicator_badge: bool,
}

impl Default for VnchkeyConfig {
    fn default() -> Self {
        Self {
            input_method: "VNI".to_string(),
            mode: "Uinput (Smooth)".to_string(),
            output_charset: "Unicode".to_string(),
            type_w_to_u: "Non-Start".to_string(),
            bracket_transform: "Disabled".to_string(),
            modern_style: true,
            smart_horn: true,
            auto_restore_english: true,
            smart_casing: true,
            free_marking: true,
            fast_consonant_doubling: false,
            enable_macro: true,
            capitalize_macro: true,
            switch_shortcut: "Ctrl+Shift".to_string(),
            temp_english_mode: "Disabled".to_string(),
            tray_icon_theme: "Default".to_string(),
            show_indicator_badge: true,
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

    pub fn dict_path() -> PathBuf {
        Self::config_dir().join("vnchkey-dictionary.conf")
    }

    pub fn app_modes_path() -> PathBuf {
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
            if let Some(v) = map.get("TypeW2U") { cfg.type_w_to_u = v.clone(); }
            if let Some(v) = map.get("BracketTransform") { cfg.bracket_transform = v.clone(); }
            if let Some(v) = map.get("ModernStyle") { cfg.modern_style = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("SmartHorn") { cfg.smart_horn = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("AutoRestoreEnglish") { cfg.auto_restore_english = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("SmartCasing") { cfg.smart_casing = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("FreeMarking") { cfg.free_marking = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("FastConsonantDoubling") { cfg.fast_consonant_doubling = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("EnableMacro") { cfg.enable_macro = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("CapitalizeMacro") { cfg.capitalize_macro = v.eq_ignore_ascii_case("true"); }
            if let Some(v) = map.get("SwitchShortcut") { cfg.switch_shortcut = v.clone(); }
            if let Some(v) = map.get("TempEnglishMode") { cfg.temp_english_mode = v.clone(); }
            if let Some(v) = map.get("TrayIconTheme") { cfg.tray_icon_theme = v.clone(); }
            if let Some(v) = map.get("ShowIndicatorBadge") { cfg.show_indicator_badge = v.eq_ignore_ascii_case("true"); }
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
        lines.push(format!("TypeW2U={}", cfg.type_w_to_u));
        lines.push(format!("BracketTransform={}", cfg.bracket_transform));
        lines.push(format!("ModernStyle={}", if cfg.modern_style { "True" } else { "False" }));
        lines.push(format!("SmartHorn={}", if cfg.smart_horn { "True" } else { "False" }));
        lines.push(format!("AutoRestoreEnglish={}", if cfg.auto_restore_english { "True" } else { "False" }));
        lines.push(format!("SmartCasing={}", if cfg.smart_casing { "True" } else { "False" }));
        lines.push(format!("FreeMarking={}", if cfg.free_marking { "True" } else { "False" }));
        lines.push(format!("FastConsonantDoubling={}", if cfg.fast_consonant_doubling { "True" } else { "False" }));
        lines.push(format!("EnableMacro={}", if cfg.enable_macro { "True" } else { "False" }));
        lines.push(format!("CapitalizeMacro={}", if cfg.capitalize_macro { "True" } else { "False" }));
        lines.push(format!("SwitchShortcut={}", cfg.switch_shortcut));
        lines.push(format!("TempEnglishMode={}", cfg.temp_english_mode));
        lines.push(format!("TrayIconTheme={}", cfg.tray_icon_theme));
        lines.push(format!("ShowIndicatorBadge={}", if cfg.show_indicator_badge { "True" } else { "False" }));

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

        let mut res = Vec::new();
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    res.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
        }
        res
    }

    pub fn save_macros(macros: &[(String, String)]) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        let mut lines = Vec::new();
        lines.push("# vnchKey Macro Replacement Table".to_string());
        for (k, v) in macros {
            if !k.trim().is_empty() {
                lines.push(format!("{}={}", k.trim(), v.trim()));
            }
        }

        fs::write(Self::macro_path(), lines.join("\n") + "\n")?;
        Self::reload_fcitx5();
        Ok(())
    }

    pub fn load_dictionary() -> Vec<String> {
        let path = Self::dict_path();
        if !path.exists() {
            return Vec::new();
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
        res
    }

    pub fn save_dictionary(words: &[String]) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        let mut lines = Vec::new();
        lines.push("# vnchKey Custom User Words Dictionary".to_string());
        for w in words {
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
        if !path.exists() {
            return vec![
                ("alacritty".to_string(), "Bypass (Raw English)".to_string()),
                ("kitty".to_string(), "Bypass (Raw English)".to_string()),
                ("steam".to_string(), "Bypass (Raw English)".to_string()),
                ("code".to_string(), "Direct Commit (Smooth)".to_string()),
            ];
        }

        let mut res = Vec::new();
        if let Ok(content) = fs::read_to_string(&path) {
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

        fs::write(Self::app_modes_path(), lines.join("\n") + "\n")?;
        Self::reload_fcitx5();
        Ok(())
    }

    pub fn reset_to_defaults() {
        let _ = Self::save_config(&VnchkeyConfig::default());
        let _ = fs::remove_file(Self::macro_path());
        let _ = fs::remove_file(Self::dict_path());
        let _ = fs::remove_file(Self::app_modes_path());
        Self::reload_fcitx5();
    }

    pub fn export_backup(path: &Path) -> std::io::Result<()> {
        let backup = BackupData {
            config: Self::load_config(),
            macros: Self::load_macros(),
            dictionary: Self::load_dictionary(),
            app_modes: Self::load_app_modes(),
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

