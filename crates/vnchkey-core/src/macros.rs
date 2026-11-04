//! XKey Innovation: Rapid Shorthand Expansion (Gõ Tắt).

use std::collections::HashMap;

/// User shorthand expansion dictionary.
#[derive(Debug, Clone)]
pub struct MacroTable {
    entries: HashMap<String, String>,
}

impl MacroTable {
    /// Creates a macro table with default abbreviations.
    pub fn new() -> Self {
        let mut entries = HashMap::new();
        entries.insert("vn".to_string(), "Việt Nam".to_string());
        entries.insert("ch".to_string(), "Cộng Hòa".to_string());
        entries.insert("td".to_string(), "Tự Do".to_string());
        entries.insert("dc".to_string(), "Dân Chủ".to_string());
        entries.insert("hp".to_string(), "Hạnh Phúc".to_string());
        entries.insert("kd".to_string(), "Kinh Doanh".to_string());
        entries.insert("cntt".to_string(), "Công Nghệ Thông Tin".to_string());

        Self { entries }
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Loads macro definitions from a configuration file (k=v per line).
    pub fn load_from_file(&mut self, path: &str) {
        if let Ok(content) = std::fs::read_to_string(path) {
            self.entries.clear();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    self.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
    }

    /// Expands an abbreviation if present in the dictionary.
    pub fn expand(&self, shortcut: &str) -> Option<String> {
        self.entries.get(shortcut.to_lowercase().as_str()).cloned()
    }

    /// Inserts a new shorthand entry.
    pub fn insert(&mut self, shortcut: String, expansion: String) {
        self.entries.insert(shortcut.to_lowercase(), expansion);
    }
}

impl Default for MacroTable {
    fn default() -> Self {
        Self::new()
    }
}
