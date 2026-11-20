# Patch ý tưởng: Tối ưu & tái cấu trúc `telex.rs` + `vni.rs`

> Tài liệu tham khảo only. **Không chỉnh sửa mã nguồn.** Mục tiêu: giảm duplication, giảm cấp phát heap, tăng parity VNI/Telex, tăng an toàn.

---

## 1. Tách helper dùng chung (shared modifiers)

### 1.1. Mục tiêu
- `vni.rs` hiện đang duplicate gần như toàn bộ logic của `telex.rs` (bracket, tone validation, circumflex, horn/breve, smart uo horn, semantic saturation).
- Tạo các helper trong `telex.rs` (hoặc module mới `vietnamese_transform.rs`) để cả Telex và VNI gọi chung.

### 1.2. API đề xuất (pseudo)

```rust
// crates/vnchkey-core/src/telex.rs (hoặc module mới)

pub enum ModifierKind { Circumflex, Horn, Breve, Stroke }

pub enum ToneAction {
    Applied,
    Removed,
    Appended,
    Violation,
}

pub fn apply_tone_to_letters(
    letters: &mut [char],
    tone: usize,           // 0..=5
    tone_style: ToneStyle,
) -> ToneAction { ... }

pub fn apply_circumflex(
    letters: &mut [char],
    tone_style: ToneStyle,
) -> bool { ... } // true nếu biến đổi

pub fn apply_horn_breve(
    letters: &mut [char],
    tone_style: ToneStyle,
    kind: ModifierKind,
) -> bool { ... }

pub fn is_qu_o_exception(letters: &[char], u_idx: usize, o_idx: usize) -> bool { ... }
pub fn is_open_thuo(letters: &[char], u_idx: usize, o_idx: usize) -> bool { ... }
pub fn is_open_huo(letters: &[char], u_idx: usize, o_idx: usize) -> bool { ... }
```

### 1.3. Cách VNI dùng (pseudo)

```rust
// vni.rs::apply_key
'6' => {
    if apply_circumflex(&mut letters, self.tone_style) {
        rebalance_tone(&mut letters, current_tone, self.tone_style);
        return (collect_word(&letters), true, false);
    }
    // toggle off: append '6'
    letters.push('6');
    rebalance_tone(&mut letters, current_tone, self.tone_style);
    return (collect_word(&letters), true, true);
}
```

### 1.4. Lợi ích
- Giảm duplication ~40-50% code.
- Parity VNI/Telex dễ kiểm soát hơn.
- Fix một chỗ áp dụng cho cả hai.

---

## 2. Giảm cấp phát heap trong hot path

### 2.1. `SmallVec` cho `letters`

```rust
// Cargo.toml (vnchkey-core)
[dependencies]
smallvec = "1"   # safe, no unsafe required

// telex.rs / vni.rs
use smallvec::SmallVec;
type Letters = SmallVec<[char; 16]>;

fn apply_key_internal(...) -> (String, bool, bool) {
    let mut letters: Letters = word.chars().collect();
    ...
    (letters.iter().collect(), ...)
}
```

- Âm tiết tiếng Việt thường ≤ 8-10 ký tự → hầu hết không heap alloc.
- Giảm GC pressure (Rust không GC, nhưng allocator overhead).

### 2.2. Helper `collect_word`

```rust
#[inline]
fn collect_word(letters: &[char]) -> String {
    let mut s = String::with_capacity(letters.len());
    s.extend(letters.iter().copied());
    s
}
```

- Thay thế `letters.into_iter().collect()` rải rác để rõ ràng và dễ profile.

### 2.3. Onset/coda validation không alloc

```rust
// Hiện tại
let s: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
matches!(s.as_str(), "b" | "ch" | ...)

// Đề xuất: so sánh trực tiếp trên &[char] lowercase ASCII
pub fn is_valid_vietnamese_onset(chars: &[char]) -> bool {
    // normalize apostrophe nếu cần, rồi match trên slice length + bytes
    match chars.len() {
        0 => true,
        1 => matches!(chars[0].to_ascii_lowercase(), 'b'|'c'|...),
        2 => { /* compare two chars lowercase */ }
        ...
    }
}
```

- Tránh `String` + `to_lowercase()` allocation mỗi lần `can_modify_syllable` gọi.

---

## 3. Tăng an toàn (safety)

### 3.1. Thay `.expect` bằng `if let` / `debug_assert`

```rust
// Hiện tại
let last_v = letters.iter().rposition(...).expect("invariant: valid vowel/digit");

// Đề xuất
let last_v = match letters.iter().rposition(|&c| is_vietnamese_vowel(c)) {
    Some(i) => i,
    None => return false, // hoặc return input nguyên bản
};
```

### 3.2. `unreachable!()` → fallback an toàn

```rust
// Hiện tại
let hat_base = match lower { 'a' => 'â', 'e' => 'ê', 'o' => 'ô', _ => unreachable!() };

// Đề xuất
let hat_base = match lower {
    'a' => 'â',
    'e' => 'ê',
    'o' => 'ô',
    _ => return (collect_word(&letters), false, false), // không panic
};
```

### 3.3. Fuzz test

- Thêm fuzz harness cho `apply_key` với input Unicode tổ hợp (NFD) để đảm bảo không panic.
- Dùng `cargo fuzz` hoặc property-based (proptest) trong `tests/`.

---

## 4. Parity VNI vs Telex

### 4.1. VNI `6` (circumflex) parity

Hiện VNI `6` promote ngay cả khi nhiều nguyên âm matching (loop rev + return sớm), còn Telex chỉ promote khi `matching.len() == 1`.

**Đề xuất:** thống nhất theo Telex: chỉ promote khi đúng một nguyên âm family `a/e/o` matching; còn lại append `6`.

```rust
// pseudo
let matching: Vec<usize> = letters.iter().enumerate()
    .filter_map(|(i,c)| decompose_vowel(*c).and_then(|(b,_,_)|
        if vowel_family(b) == lower { Some(i) } else { None }))
    .collect();
if matching.len() == 1 {
    // promote
} else {
    letters.push('6');
    // toggle? append
}
```

### 4.2. VNI `7` (horn) parity

- Thêm test parity `duong7` (VNI) vs `duongw` (Telex) → cùng kết quả.
- Case `DUong7` TitleCase parity.

### 4.3. Bảng parity test tự động

```rust
// tests/parity_telex_vni.rs
let pairs = [("duong", "duongw", "duong7"), ("tien", "tiene", "tien6"), ...];
for (base, telex_keys, vni_keys) in pairs {
    let t = TelexEngine::new(ToneStyle::Modern).feed(telex_keys).0;
    let v = VniEngine::new(ToneStyle::Modern).feed(vni_keys).0;
    assert_eq!(t, v, "parity failed for {}", base);
}
```

---

## 5. Tối ưu độ phức tạp

### 5.1. `extract_last_syllable_index` O(n²) → incremental

```rust
// Đề xuất: duyệt từ cuối, kiểm tra suffix hợp lệ, dừng sớm
pub fn extract_last_syllable_index(word: &str) -> usize {
    let chars: Vec<char> = word.chars().collect();
    // tìm boundary cuối cùng mà suffix là in-progress syllable hợp lệ
    for start in (0..=chars.len()).rev() {
        if start == chars.len() { continue; }
        if is_valid_in_progress_syllable(&chars[start..]) {
            // ensure prefix before start is not part of same syllable
            return start;
        }
    }
    0
}
```

(Cần kiểm tra kỹ với Vim commands; giữ test cũ.)

### 5.2. `place_tone` giảm clone

```rust
// Build effective_vowels trực tiếp thay vì clone all_vowels rồi filter
let mut effective_vowels: Vec<usize> = letters.iter().enumerate()
    .filter(|(_, c)| is_vietnamese_vowel(**c))
    .map(|(i, _)| i)
    .collect();
// rồi remove glide đầu nếu cần
```

### 5.3. `rebalance_tone` tránh reconstruct khi không cần

- Nếu `tone == 0` và không có tone hiện tại → skip parse.
- Nếu `letters` rỗng → return sớm.

---

## 6. Refactor lớn (tùy chọn, breaking API)

### 6.1. `Modifier` enum + `apply_modifier`

```rust
pub enum Modifier { Circumflex, Horn, Breve, Stroke }

pub fn apply_modifier(
    letters: &mut [char],
    modifier: Modifier,
    tone_style: ToneStyle,
) -> bool { ... }
```

### 6.2. `ToneAction` thay vì bool

- Giúp caller biết chính xác đã apply/remove/append/violation → giảm nhánh `collect()` rải rác.

### 6.3. `Cow<str>` cho `feed`

```rust
pub fn feed(&self, input: &str) -> (Cow<'_, str>, bool) {
    if input.is_empty() { return (Cow::Borrowed(""), false); }
    ...
    if !any_transformed && word == input {
        (Cow::Borrowed(input), false)
    } else {
        (Cow::Owned(word), true)
    }
}
```

- Tránh alloc khi không transform.

---

## 7. Checklist kiểm thử trước khi áp dụng

- [ ] `cargo test --workspace` pass.
- [ ] `cargo clippy --workspace -- -D warnings` pass.
- [ ] Fuzz `apply_key` 10 phút không panic.
- [ ] Parity test VNI/Telex pass.
- [ ] Benchmark `process_key` không regression > 5%.
- [ ] ASan/UBSan FFI test pass.

---

## 8. Ưu tiên

- **P0:** an toàn (expect/unreachable) + parity VNI `6`.
- **P1:** giảm alloc (SmallVec + collect_word + onset/coda no-alloc).
- **P2:** tách helper shared.
- **P3:** refactor lớn (Modifier/ToneAction/Cow) + fuzz.

> Ghi chú: Đây là ý tưởng tham khảo. Khi triển khai, nên làm từng bước nhỏ, mỗi bước có test parity để tránh regression ngữ âm.
path">
/home/codegpt/antigravity/vnchKey/docs/TELEX_VNI_OPTIMIZATION_PATCH_IDEAS.md