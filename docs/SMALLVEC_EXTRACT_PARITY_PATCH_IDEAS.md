# Patch ý tưởng: SmallVec, extract_last_syllable_index, Parity VNI `6`

> Tài liệu tham khảo only. **Không chỉnh sửa mã nguồn.** Mục tiêu: 3 tối ưu còn lại (perf + correctness) sau khi P0–P3 và atomic write đã hoàn tất.

---

## 1. SmallVec + Benchmark `process_key`

### 1.1. Dependency
```toml
# crates/vnchkey-core/Cargo.toml
[dependencies]
smallvec = "1"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "process_key"
harness = false
```

### 1.2. Thay `Vec<char>` → `SmallVec<[char; 16]>`
Áp dụng tại `telex.rs`, `vni.rs`, `engine/mod.rs`:
```rust
use smallvec::SmallVec;
type Letters = SmallVec<[char; 16]>;

let mut letters: Letters = word.chars().collect();
// helpers nhận &[char] (Deref) → không đổi chữ ký
(letters.iter().collect(), ...)   // thay .into_iter()
```
- `chars: Vec<char>` trong `extract_last_syllable_index` / `is_valid_elongated_vietnamese_word` cũng đổi sang `SmallVec` để đồng bộ.

### 1.3. Benchmark harness
```rust
// crates/vnchkey-core/benches/process_key.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vnchkey_core::{Engine, EngineConfig, InputMethod};

fn bench_telex(c: &mut Criterion) {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Telex, ..EngineConfig::default()
    });
    c.bench_function("telex_tieengs", |b| b.iter(|| {
        engine.reset();
        for ch in "tieengs ".chars() { let _ = engine.process_key(ch, 0.0); }
        black_box(engine.current_word())
    }));
}
criterion_group!(benches, bench_telex);
criterion_main!(benches);
```
**Chạy:** `cargo bench -p vnchkey-core` (baseline: `--save-baseline before`).

---

## 2. Tối ưu `extract_last_syllable_index` (O(n²) → incremental)

### 2.1. Ý tưởng
Duyệt từ cuối trong cửa sổ 8 ký tự (âm tiết tiếng Việt ≤ 8 char), kiểm tra suffix hợp lệ:
```rust
pub fn extract_last_syllable_index(word: &str) -> usize {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    if n == 0 { return 0; }
    let lower_bound = n.saturating_sub(8);
    for start in (lower_bound..n).rev() {
        if is_valid_in_progress_syllable(&chars[start..]) {
            return start;
        }
    }
    n.saturating_sub(1)
}
```

### 2.2. Verify
- Giữ nguyên test Neovim/VNI numeric guard (`tests.rs`, `nightmare_edge_cases`).
- `cargo test --workspace extract_last_syllable`.

---

## 3. Parity VNI `6`

### 3.1. Quyết định: Option A (nhất quán với Telex)
VNI `6` chỉ promote khi đúng một nguyên âm family `a/e/o` matching; còn lại append `6`.

### 3.2. Patch (pseudo) trong `vni.rs`
```rust
'6' => {
    let matching: Vec<usize> = letters.iter().enumerate()
        .filter_map(|(i, &c)| decompose_vowel(c).and_then(|(b, _, _)| {
            if matches!(vowel_family(b), 'a' | 'e' | 'o') { Some(i) } else { None }
        }))
        .collect();

    if matching.len() == 1 {
        let idx = matching[0];
        let (base, _, is_up) = decompose_vowel(letters[idx]).expect("invariant");
        let hat_base = match vowel_family(base) {
            'a' => 'â', 'e' => 'ê', 'o' => 'ô', _ => unreachable!(),
        };
        if base == hat_base {
            letters[idx] = compose_vowel(vowel_family(base), 0, is_up);
            letters.push('6');
            rebalance_tone(&mut letters, current_tone, self.tone_style);
            return (letters.into_iter().collect(), true, true);
        } else {
            letters[idx] = compose_vowel(hat_base, 0, is_up);
            rebalance_tone(&mut letters, current_tone, self.tone_style);
            return (letters.into_iter().collect(), true, false);
        }
    } else {
        letters.push('6');
        rebalance_tone(&mut letters, current_tone, self.tone_style);
        return (letters.into_iter().collect(), true, true);
    }
}
```

### 3.3. Test parity bổ sung
```rust
// engine/tests.rs
let parity_cases: &[(&str, &str, &str)] = &[
    // ... existing ...
    ("toan6", "toan6", "toân"),
];
```
(Điều chỉnh expected theo thực tế trước khi commit.)

---

## 4. Checklist

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] Fuzz `apply_key` không panic
- [ ] Parity test pass
- [ ] Benchmark không regression > 5%
- [ ] ASan/UBSan FFI pass

## 5. Ưu tiên

1. SmallVec (1.1–1.2) → test.
2. Benchmark baseline (1.3).
3. extract_last_syllable_index (2.1–2.2) → test Neovim/VNI.
4. Parity VNI `6` (3.2–3.3) → test parity.
5. So sánh benchmark trước/sau.
