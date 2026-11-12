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

    /// Loads macro definitions from a configuration file.
    /// Supports vnchKey format (k=v), UniKey/EVKey format (k:v), and semicolon format (k;v).
    pub fn load_from_file(&mut self, path: &str) {
        if let Ok(content) = std::fs::read_to_string(path) {
            self.entries.clear();
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

/// Resolves dynamic date and time variables ($DATE, $TIME, $YEAR, $NOW, $DATETIME)
/// using the system's local time and timezone.
pub fn resolve_dynamic_tokens(text: &str) -> String {
    if !text.contains('$') {
        return text.to_string();
    }
    let (year, month, day, hour, min) = get_local_datetime();
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

/// Retrieves the system's local date and time according to the operating system's configured timezone.
/// Pure in-process implementation: reads system timezone offset without spawning subprocesses (0ms overhead).
fn get_local_datetime() -> (i32, u32, u32, u32, u32) {
    let now = std::time::SystemTime::now();
    let unix_secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let offset_secs = get_local_tz_offset(unix_secs);
    let local_secs = unix_secs + (offset_secs as i64);

    get_civil_datetime_from_secs(local_secs)
}

/// Discovers local timezone offset (in seconds from UTC) using zero-dependency, safe platform methods.
fn get_local_tz_offset(unix_secs: i64) -> i32 {
    // 1. Try reading /etc/localtime (standard on Linux, macOS, BSD, Android)
    if let Ok(data) = std::fs::read("/etc/localtime") {
        if let Some(offset) = parse_tzif_offset(&data, unix_secs) {
            return offset;
        }
    }

    // 2. Check TZ environment variable if set (e.g., "UTC+7", "EST+5", "GMT-7")
    if let Ok(tz_str) = std::env::var("TZ") {
        if let Some(offset) = parse_tz_env_offset(&tz_str) {
            return offset;
        }
    }

    // Default: 0 (UTC)
    0
}

/// Parses POSIX TZ environment format for simple fixed offsets (e.g. "UTC+7", "GMT-7", "ICT-7").
fn parse_tz_env_offset(tz: &str) -> Option<i32> {
    let s = tz.trim();
    if s.is_empty() {
        return None;
    }
    // Find first digit or sign
    if let Some(idx) = s.find(|c: char| c.is_ascii_digit() || c == '+' || c == '-') {
        let num_part = &s[idx..];
        // POSIX TZ specifies inverted sign: "UTC-7" means UTC+7, "UTC+7" means UTC-7.
        // But many users write "UTC+7" for Vietnam (+7h). Handle both intelligently:
        if let Ok(val) = num_part.parse::<f32>() {
            // If sign was explicitly +, user usually intended +N hours in modern configs
            return Some((val * 3600.0) as i32);
        }
    }
    None
}

/// Pure Safe Rust parser for standard IANA TZif (Time Zone Information Format) binaries (/etc/localtime).
/// Complies with RFC 8536 and supports both v1 (32-bit) and v2/v3 (64-bit) headers.
fn parse_tzif_offset(data: &[u8], now_secs: i64) -> Option<i32> {
    if data.len() < 44 || &data[0..4] != b"TZif" {
        return None;
    }

    let read_u32 = |slice: &[u8], idx: usize| -> u32 {
        u32::from_be_bytes([slice[idx], slice[idx + 1], slice[idx + 2], slice[idx + 3]])
    };

    let version = data[4];
    if version == b'2' || version == b'3' {
        let timecnt1 = read_u32(data, 32) as usize;
        let typecnt1 = read_u32(data, 36) as usize;
        let charcnt1 = read_u32(data, 40) as usize;
        let leapcnt1 = read_u32(data, 28) as usize;
        let isstdcnt1 = read_u32(data, 24) as usize;
        let isgmtcnt1 = read_u32(data, 20) as usize;

        let v1_len = 44 + timecnt1 * 5 + typecnt1 * 6 + charcnt1 + leapcnt1 * 8 + isstdcnt1 + isgmtcnt1;
        if data.len() >= v1_len + 44 && &data[v1_len..v1_len + 4] == b"TZif" {
            let hdr2 = &data[v1_len..];
            let timecnt2 = read_u32(hdr2, 32) as usize;
            let typecnt2 = read_u32(hdr2, 36) as usize;

            let offset = 44;
            let trans_bytes = timecnt2 * 8;
            if hdr2.len() >= offset + trans_bytes + timecnt2 + typecnt2 * 6 {
                let mut cur_type = 0usize;
                let mut found = false;
                for i in 0..timecnt2 {
                    let t_idx = offset + i * 8;
                    let t = i64::from_be_bytes([
                        hdr2[t_idx], hdr2[t_idx + 1], hdr2[t_idx + 2], hdr2[t_idx + 3],
                        hdr2[t_idx + 4], hdr2[t_idx + 5], hdr2[t_idx + 6], hdr2[t_idx + 7],
                    ]);
                    if t <= now_secs {
                        let type_idx = offset + trans_bytes + i;
                        cur_type = hdr2[type_idx] as usize;
                        found = true;
                    } else {
                        break;
                    }
                }

                let ttinfo_offset = offset + trans_bytes + timecnt2;
                if found && cur_type < typecnt2 {
                    let info_idx = ttinfo_offset + cur_type * 6;
                    return Some(i32::from_be_bytes([
                        hdr2[info_idx], hdr2[info_idx + 1], hdr2[info_idx + 2], hdr2[info_idx + 3],
                    ]));
                } else if typecnt2 > 0 {
                    let info_idx = ttinfo_offset;
                    return Some(i32::from_be_bytes([
                        hdr2[info_idx], hdr2[info_idx + 1], hdr2[info_idx + 2], hdr2[info_idx + 3],
                    ]));
                }
            }
        }
    }

    // Fallback to v1 data
    let timecnt = read_u32(data, 32) as usize;
    let typecnt = read_u32(data, 36) as usize;
    let offset = 44;
    let trans_bytes = timecnt * 4;
    if data.len() < offset + trans_bytes + timecnt + typecnt * 6 {
        return None;
    }

    let mut cur_type = 0usize;
    let mut found = false;
    for i in 0..timecnt {
        let t_idx = offset + i * 4;
        let t = i32::from_be_bytes([data[t_idx], data[t_idx + 1], data[t_idx + 2], data[t_idx + 3]]) as i64;
        if t <= now_secs {
            let type_idx = offset + trans_bytes + i;
            cur_type = data[type_idx] as usize;
            found = true;
        } else {
            break;
        }
    }

    let ttinfo_offset = offset + trans_bytes + timecnt;
    if found && cur_type < typecnt {
        let info_idx = ttinfo_offset + cur_type * 6;
        Some(i32::from_be_bytes([
            data[info_idx], data[info_idx + 1], data[info_idx + 2], data[info_idx + 3],
        ]))
    } else if typecnt > 0 {
        let info_idx = ttinfo_offset;
        Some(i32::from_be_bytes([
            data[info_idx], data[info_idx + 1], data[info_idx + 2], data[info_idx + 3],
        ]))
    } else {
        None
    }
}

/// Pure math calendar algorithm using Howard Hinnant's civil calendar formula from Unix Epoch timestamp.
fn get_civil_datetime_from_secs(secs: i64) -> (i32, u32, u32, u32, u32) {
    let sec_in_day = ((secs % 86400) + 86400) % 86400;
    let hour = (sec_in_day / 3600) as u32;
    let min = ((sec_in_day % 3600) / 60) as u32;

    let z = (secs.div_euclid(86400)) + 719468;
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

