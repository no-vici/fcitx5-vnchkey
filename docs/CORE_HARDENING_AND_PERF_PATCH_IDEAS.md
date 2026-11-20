# Patch ý tưởng: Hardening & Performance cho Core/Settings/FFI

> Tài liệu tham khảo only. **Không chỉnh sửa mã nguồn.** Mục tiêu: xử lý các điểm còn lại sau P0–P3 (clippy, I/O atomic, UTF-8 rotation, SmallVec, parity VNI `6`).

---

## 1. Sửa clippy (`-D warnings`) còn sót

### 1.1. `macros.rs` map identity
```rust
// crates/vnchkey-core/src/macros.rs ~76
// Before:
} else { line.split_once(';').map(|p| p) };
// After:
} else { line.split_once(';') };
```

### 1.2. Missing docs
```rust
// crates/vnchkey-core/src/engine/dev_code.rs
/// State machine for Developer Mode (Markdown inline code & fenced blocks).
pub struct DevCodeState { ... }

// crates/vnchkey-core/src/telex.rs
/// Checks if `chars` form a valid Vietnamese onset consonant cluster.
pub fn is_valid_vietnamese_onset(chars: &[char]) -> bool { ... }
```

### 1.3. `vnchkey-dev/src/main.rs`
- Gộp hai nhánh `if` trùng thành một điều kiện.
- Tách type phức tạp thành `type` alias.

---

## 2. I/O robustness

### 2.1. Atomic write cho macros/dictionary/app_modes
```rust
// crates/vnchkey-settings/src/config.rs
impl ConfigManager {
    pub fn save_macros(macros: &[(String, String)]) -> std::io::Result<()> {
        let content = /* build lines */;
        Self::atomic_write(&Self::macro_path(), &content)
    }
    pub fn save_dictionary(words: &[String]) -> std::io::Result<()> {
        let content = words.join("\n") + "\n";
        Self::atomic_write(&Self::dict_path(), &content)
    }
    pub fn save_app_modes(app_modes: &[(String, String)]) -> std::io::Result<()> {
        let content = /* build */;
        Self::atomic_write(&Self::app_modes_path(), &content)?;
        let _ = Self::atomic_write(&Self::legacy_app_modes_path(), &content);
        Ok(())
    }
}
```

### 2.2. Habit log rotation theo dòng (tránh cắt giữa UTF-8)
```rust
// crates/vnchkey-core/src/habit_learner.rs
fn rotate_log_file(&self, path: &Path) {
    let Ok(content) = fs::read_to_string(path) else { return; };
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() { return; }
    let keep = (lines.len() * 7 / 10).max(1);
    let start = lines.len().saturating_sub(keep);
    let new_content = lines[start..].join("\n") + "\n";
    let _ = fs::write(path, new_content);
}
```

### 2.3. Batch I/O (tùy chọn)
- Gom `append_log_entry` + `append_to_dictionary_file` vào queue, flush theo idle timeout hoặc khi promotion xảy ra, để keystroke path không blocking.

---

## 3. Performance: SmallVec cho `letters`

```toml
# crates/vnchkey-core/Cargo.toml
[dependencies]
smallvec = "1"
```

```rust
// telex.rs / vni.rs / engine
use smallvec::SmallVec;
type Letters = SmallVec<[char; 16]>;

pub fn apply_key(&self, word: &str, ch: char, ...) -> (String, bool, bool) {
    let mut letters: Letters = word.chars().collect();
    ...
    (letters.iter().collect(), ...)
}
```

- Âm tiết tiếng Việt thường ≤ 10 ký tự → hầu hết không heap alloc.

## 4. Performance: `extract_last_syllable_index` O(n²) → incremental

```rust
pub fn extract_last_syllable_index(word: &str) -> usize {
    let chars: Vec<char> = word.chars().collect();
    // Tìm start lớn nhất sao cho suffix [start..] là in-progress syllable hợp lệ
    // và prefix trước start không thuộc cùng âm tiết.
    for start in (0..=chars.len()).rev() {
        if start == chars.len() { continue; }
        if is_valid_in_progress_syllable(&chars[start..]) {
            // boundary check: nếu start>0 và chars[start-1] là vowel/consonant liên quan, lùi thêm
            return start;
        }
    }
    0
}
```

- Cần giữ test cũ (Vim commands) để verify boundary.

## 5. An toàn FFI

### 5.1. Xử lý NUL trong commit_text
```rust
// fcitx5-vnchkey-ffi/src/lib.rs::process_key
if !out_commit.is_null() {
    match CString::new(output.commit_text.clone()) {
        Ok(c_str) => *out_commit = c_str.into_raw(),
        Err(_) => {
            // log warning; fallback: commit without NUL (strip)
            let sanitized = output.commit_text.replace('\0', "");
            if let Ok(c_str) = CString::new(sanitized) {
                *out_commit = c_str.into_raw();
            }
        }
    }
}
```

### 5.2. `engine/mod.rs:1195` dùng `if let`
```rust
let split_pos = trimmed_trailing
    .rfind(|c: char| !c.is_alphanumeric())
    .map(|idx| {
        idx + trimmed_trailing[idx..].chars().next().map(|c| c.len_utf8()).unwrap_or(0)
    })
    .unwrap_or(0);
```

## 6. Parity VNI `6`

### 6.1. Quyết định
- Option A (nhất quán với Telex): VNI `6` chỉ promote khi đúng một nguyên âm family `a/e/o` matching; còn lại append `6`.
- Option B (free marking): giữ hành vi hiện tại (promote vowel phải nhất).

### 6.2. Patch Option A (pseudo)
```rust
// vni.rs '6' branch
let matching: Vec<usize> = letters.iter().enumerate()
    .filter_map(|(i, &c)| decompose_vowel(c).and_then(|(b, _, _)| {
        if matches!(vowel_family(b), 'a' | 'e' | 'o') { Some(i) } else { None }
    }))
    .collect();

if matching.len() == 1 {
    let idx = matching[0];
    // promote logic hiện tại
} else {
    // append '6'
    letters.push('6');
    rebalance_tone(&mut letters, current_tone, self.tone_style);
    return (letters.into_iter().collect(), true, true);
}
```

### 6.3. Test parity bổ sung
- TitleCase/ALL-CAPS: `Duong972` vs `Duongw...`? (Telex không có số; so sánh output).
- NFD roundtrip: `feed` output normalize NFC/NFD.

---

## 7. Maintainability

### 7.1. Tách `engine/mod.rs` thêm
- `macro_flow.rs`: xử lý macro expansion/undo.
- `restore_flow.rs`: auto-restore English + collision.
- `selection_rescue.rs`: `apply_accent_to_selection`.

### 7.2. Docs API
- Thêm `///` cho tất cả `pub` struct/fn trong `engine/`, `telex.rs`, `vni.rs`.

---

## 8. Checklist

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] Fuzz `apply_key` 10 phút không panic
- [ ] Parity test VNI/Telex pass
- [ ] Benchmark `process_key` không regression > 5%
- [ ] ASan/UBSan FFI test pass

---

## 9. Ưu tiên

- **P0:** clippy errors + rotate log theo dòng.
- **P1:** atomic write macros/dict/app_modes; SmallVec.
- **P2:** extract_last_syllable_index; parity VNI `6`.
- **P3:** tách module engine; docs API.
path">
/home/codegpt/antigravity/vnchKey/docs/CORE_HARDENING_AND_PERF_PATCH_IDEAS.md