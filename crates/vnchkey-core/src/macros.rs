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
        entries.insert("hn".to_string(), "hôm nay".to_string());
        entries.insert("hp".to_string(), "Hạnh Phúc".to_string());
        entries.insert("idk".to_string(), "tui hông biết".to_string());
        entries.insert("kbi".to_string(), "không biết".to_string());
        entries.insert("kd".to_string(), "Kinh Doanh".to_string());
        entries.insert("ko".to_string(), "không".to_string());
        entries.insert("kt".to_string(), "kinh tế".to_string());
        entries.insert("mb".to_string(), "Miền Bắc".to_string());
        entries.insert("mn".to_string(), "Miền Nam".to_string());
        entries.insert("mng".to_string(), "mọi người".to_string());
        entries.insert("mst".to_string(), "mã số thuế".to_string());
        entries.insert("mt".to_string(), "Miền Tây".to_string());
        entries.insert("ng".to_string(), "người".to_string());
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
    /// Automatically resolves dynamic date/time variables: $DATE, $TIME, $YEAR, $NOW, $DATETIME.
    pub fn expand(&self, shortcut: &str) -> Option<String> {
        let raw = self.entries.get(shortcut.to_lowercase().as_str())?;
        Some(resolve_dynamic_tokens(raw))
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

/// Resolves dynamic date and time variables ($DATE, $TIME, $YEAR, $NOW, $DATETIME).
pub fn resolve_dynamic_tokens(text: &str) -> String {
    if !text.contains('$') {
        return text.to_string();
    }
    let (year, month, day, hour, min) = get_civil_datetime();
    let date_str = format!("{:02}/{:02}/{:04}", day, month, year);
    let time_str = format!("{:02}:{:02}", hour, min);
    let year_str = format!("{:04}", year);
    let now_str = format!("{} {}", date_str, time_str);

    text.replace("$DATETIME", &now_str)
        .replace("$NOW", &now_str)
        .replace("$DATE", &date_str)
        .replace("$TIME", &time_str)
        .replace("$YEAR", &year_str)
}

/// Calculates civil date and time (UTC+7) from Unix Epoch using Howard Hinnant's algorithm.
fn get_civil_datetime() -> (i32, u32, u32, u32, u32) {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Default to UTC+7 (Vietnam standard time)
    let local_secs = secs + 7 * 3600;
    let sec_in_day = local_secs % 86400;
    let hour = (sec_in_day / 3600) as u32;
    let min = ((sec_in_day % 3600) / 60) as u32;

    let z = (local_secs / 86400) as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1020 + doe / 1461 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d, hour, min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_macro_tokens() {
        let mut table = MacroTable::new();
        table.insert("dt".to_string(), "Hôm nay $DATE lúc $TIME".to_string());
        table.insert("yr".to_string(), "Năm $YEAR".to_string());

        let res = table.expand("dt").unwrap();
        assert!(res.starts_with("Hôm nay "));
        assert!(res.contains(" lúc "));
        assert!(res.contains("/202"));

        let yr = table.expand("yr").unwrap();
        assert!(yr.starts_with("Năm 202"));
    }
}

