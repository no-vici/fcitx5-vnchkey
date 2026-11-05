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
        entries.insert("bvi".to_string(), "bệnh viện".to_string());
        entries.insert("cbi".to_string(), "chuẩn bị".to_string());
        entries.insert("ck".to_string(), "chuyển khoản".to_string());
        entries.insert("cntt".to_string(), "Công Nghệ Thông Tin".to_string());
        entries.insert("cso".to_string(), "cuộc sống".to_string());
        entries.insert("cv".to_string(), "công việc".to_string());
        entries.insert("cx".to_string(), "cũng".to_string());
        entries.insert("dc".to_string(), "được".to_string());
        entries.insert("dma".to_string(), "đụ má".to_string());
        entries.insert("dmm".to_string(), "đụ má mày".to_string());
        entries.insert("hp".to_string(), "Hạnh Phúc".to_string());
        entries.insert("idk".to_string(), "tui hông biết".to_string());
        entries.insert("kbi".to_string(), "không biết".to_string());
        entries.insert("kd".to_string(), "Kinh Doanh".to_string());
        entries.insert("ko".to_string(), "không".to_string());
        entries.insert("kt".to_string(), "kinh tế".to_string());
        entries.insert("mb".to_string(), "Miền Bắc".to_string());
        entries.insert("md".to_string(), "mất dạy".to_string());
        entries.insert("mn".to_string(), "Miền Nam".to_string());
        entries.insert("mng".to_string(), "mọi người".to_string());
        entries.insert("mst".to_string(), "mã số thuế".to_string());
        entries.insert("mt".to_string(), "Miền Tây".to_string());
        entries.insert("nk".to_string(), "nhập khẩu".to_string());
        entries.insert("nt".to_string(), "nhắn tin".to_string());
        entries.insert("nv".to_string(), "Người Việt".to_string());
        entries.insert("sg".to_string(), "Sài Gòn".to_string());
        entries.insert("std".to_string(), "số điện thoại".to_string());
        entries.insert("stk".to_string(), "số tài khoản".to_string());
        entries.insert("td".to_string(), "Tự Do".to_string());
        entries.insert("tn".to_string(), "tin nhắn".to_string());
        entries.insert("tvi".to_string(), "Tiếng Việt".to_string());
        entries.insert("vc".to_string(), "vợ chồng".to_string());
        entries.insert("vn".to_string(), "Việt Nam".to_string());
        entries.insert("vnch".to_string(), "Việt Nam Cộng Hòa".to_string());
        entries.insert("xk".to_string(), "xuất khẩu".to_string());
        entries.insert("xl".to_string(), "xin lỗi".to_string());
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
