# Changelog

Tất cả những thay đổi nổi bật của dự án **vnchKey** (**fcitx5-vnchkey** & **VnCharKey**) qua từng phiên bản được ghi nhận tại đây.

---

## [0.3.8] - 2026-11-20

### 🚀 Tính Năng Mới (Mặc định TẮT - "Tự Do trên hết")
- **Tự Thêm Khoảng Trắng Sau Dấu Câu (`AutoSpaceAfterPunctuation`)**:
  - Tùy chọn tự động chèn thêm một khoảng trắng sau `.`, `?`, `!` khi ký tự trước đó là chữ cái (không kích hoạt trong code/backtick bypass). Mặc định **TẮT**.
- **Tự Viết Hoa Danh Từ Riêng (`AutoCapitalizeProperNouns`)**:
  - Tùy chọn tự viết hoa ký tự đầu của từ khi khớp danh sách `proper_nouns_list` do người dùng định nghĩa. Danh sách rỗng = không tự viết hoa từ nào. Mặc định **TẮT**.
- **Backspace Xóa Cả Âm Tiết (`BackspaceBySyllable`)**:
  - Khi bật, nhấn `Backspace` trong lúc đang gõ dở một âm tiết sẽ xóa **toàn bộ âm tiết** trong buffer thay vì xóa từng ký tự. Mặc định **TẮT**.
  - Chỉ áp dụng khi IME còn đang giữ buffer âm tiết (chưa commit từ bằng Space/Enter/chuyển focus). Trường hợp reopen từ đã commit vẫn giữ hành vi cũ.
- **Gợi Ý Macro Inline (`MacroPreview`)**:
  - Hiển thị gợi ý expansion khi người dùng gõ trigger macro, giúp xác nhận trước khi bung. Mặc định **TẮT**.

### 🐛 Sửa Lỗi & Tối Ưu Fcitx5 Wayland (`vnchkey_addon.cpp`)
- **Khắc Phục Lag Khi Giữ `BackSpace` Trong Chế Độ Preedit Trên Firefox/Wayland**:
  - Khi bộ đệm từ hiện tại đã rỗng (`!fcitx5_vnchkey_has_active_word(ctx)`), addon gọi `fcitx5_vnchkey_reset(ctx)` và nhả ngay phím `BackSpace` về cho trình duyệt xử lý native thay vì lôi ngược tối đa 16 từ cũ từ `reopen_history` vào Preedit.
  - Giảm cập nhật D-Bus `InputPanel` dư thừa mỗi lần xóa ký tự: chỉ gọi `updateSymbolCandidates(...)` và `updateUserInterface(InputPanel)` khi đang bật `PreeditTooltip`, đang gõ `:shortcode`, hoặc đang có danh sách ứng viên mở.
- **Hiển Thị Tooltip Nổi Trên Google Docs (Kix Canvas / Wayland)**:
  - Khi bật server tooltip (`config_preedit_tooltip_ || isWpsOffice(...)`), addon chủ động xóa rỗng `setClientPreedit(fcitx::Text())` và `setPreedit(fcitx::Text())`, đồng thời dùng duy nhất `setAuxUp(text)` để ép Fcitx5 ClassicUI/Kimpanel luôn vẽ khung nổi 1 dòng duy nhất (không bị nhân đôi từ hay ẩn mất khi Google Docs trả về `cursorRect(0, 0)`).
- **Sửa Lưu/Tải Cấu Hình Cho Tính Năng Mới (`vnchkey-settings`)**:
  - Bổ sung `BackspaceBySyllable` / `MacroPreview` vào `load_config` và `save_config` trong `crates/vnchkey-settings/src/config.rs`, khắc phục lỗi tick setting nhưng restart bị mất.
- **Wiring Fcitx5 Addon Cho Tính Năng Mới (`fcitx5-vnchkey-ffi`, `fcitx5-addon`)**:
  - Thêm FFI setters `fcitx5_vnchkey_set_backspace_by_syllable` / `fcitx5_vnchkey_set_macro_preview`.
  - C++ addon parse/save/apply `BackspaceBySyllable` và `MacroPreview` từ `vnchkey.conf` (chấp nhận cả snake_case cũ).
- **Backspace Xóa Cả Âm Tiết Trong Direct Mode (`fcitx5-addon`)**:
  - `handle_backspace` FFI trả về `bool` (reopen signal). Ở Direct Mode, khi `BackspaceBySyllable` bật và có buffer âm tiết đang gõ, addon xóa toàn bộ âm tiết bằng `deleteSurroundingText` và `filterAndAccept` phím BackSpace vật lý, tránh chỉ xóa 1 ký tự.

### ⚡ Tối Ưu Hiệu Năng & An Toàn Lõi (`vnchkey-core`, `fcitx5-vnchkey-ffi`, `vnchkey-settings`)
- **Zero-Allocation Onset & Coda Validation** (`telex.rs`): Viết lại `is_valid_vietnamese_onset` và `is_valid_vietnamese_coda` để so khớp trực tiếp trên `&[char]` qua `norm_onset_char` / `to_ascii_lowercase()`, loại bỏ hoàn toàn cấp phát `String` trên hot path.
- **Atomic Config Write** (`config.rs`): Bổ sung `ConfigManager::atomic_write` (ghi ra file `.tmp` rồi `fs::rename`) cho toàn bộ thao tác lưu `config.conf`, `macros.conf`, `dictionary.conf`, và `app_modes.conf`, tránh hỏng dữ liệu cấu hình khi crash/mất điện đột ngột.
- **Hardening & Robustness**:
  - Thay thế toàn bộ 28 lời gọi `.unwrap()` trong `vni.rs` (6) và `telex.rs` (22) bằng `.expect("invariant: valid vowel/digit")`.
  - Giới hạn độ dài `$CLIPBOARD` tối đa `4096` ký tự trong `macros.rs` để tránh khựng khi clipboard quá lớn.
  - Gộp logic chuyển đổi `InputMethod` (`0/1/2/3`) trong `fcitx5-vnchkey-ffi/src/lib.rs` thành helper `method_from_u32` và `method_to_u32`.
  - Dọn sạch toàn bộ cảnh báo `clippy` và `missing_docs` trên toàn workspace.

### 🏗️ Tái Cấu Trúc & Kiểm Thử (Refactoring & Testing)
- **Tách Module `engine/`**: Chia `engine.rs` (3.980 dòng) thành `engine/mod.rs` (2.347 dòng), `engine/dev_code.rs` (601 dòng), và `engine/tests.rs` (1.033 dòng) để dễ đọc, dễ bảo trì và tăng tốc biên dịch.
- **Bảng Parity Test VNI vs Telex & Fuzzing** (`engine/tests.rs`): Bổ sung `test_vni_telex_parity_table_and_fuzz` đối chiếu 26 cặp từ/vần giữa Telex và VNI (bao gồm chữ thường, `TitleCase`, `ALL-CAPS`) cùng 500 bước fuzzing ngẫu nhiên trên `Telex`, `Vni`, và `Auto`.

---

## [0.3.7] - 2026-11-19

### 🐛 Sửa Lỗi (Bug Fixes)
- **Bảo Toàn Nguyên Âm Đơn Có Dấu Đi Sau `đ` Trong Kiểu Gõ VNI / Auto (`engine.rs`, `vietc_regression_tests.rs`)**:
  - Khắc phục lỗi `alphanumeric_trap` nhận diện nhầm các âm tiết mở hợp lệ có chữ `đ` đi trước nguyên âm đơn mang dấu (`d9a4` → `đã`, `d9e63` → `để`, `d9o1` → `đó`, `d9u3` → `đủ`) là từ lẫn số và ép hoàn nguyên ngược về phím thô khi nhấn `Space`.
  - Đồng bộ trạng thái khởi tạo khi nhập số ở đầu từ trong chế độ `InputMethod::Vni` và mở rộng bộ kiểm thử hồi quy bao phủ cả thứ tự gõ thuận lẫn gõ đảo phím (`da49`, `de639`, `do19`, `du39`).
- **Tối Ưu & Sửa Lỗi Trải Nghiệm Gõ (`engine.rs`, `habit_learner.rs`, `vnchkey_addon.cpp`)**:
  - Tự động mở khóa `english_locked` khi bấm `Backspace` xóa hết ký tự số/đặc biệt khỏi từ đang gõ (ví dụ `chon8` → xóa `8` → `chọn`).
  - Đặt mặc định `HabitPromoteThreshold = 0` (tắt tự động đưa từ vào User Dict) để tránh học nhầm từ gõ sai.
  - Bổ sung tùy chọn **"Cho phép gõ Tiếng Việt trong `backtick` / ```code```"** trên cả 3 hệ điều hành (Linux, macOS, Windows).
  - Bổ sung nhóm từ kỹ thuật lẫn số phổ biến (`icon16`–`icon256`, `base64`, `sha256`, `mp3`, `mp4`, `ipv4`, `ipv6`, `win32`, `arm64`, `html5`, `css3`...) vào `DEFAULT_ENGLISH_WORDS`.
  - Gỡ bỏ `/tmp/vnchkey.sock` và cờ `neovim_bypass_` khỏi Fcitx5 addon để tránh xung đột trạng thái bật/tắt với `Ctrl+Space` của Fcitx5.

### ⚡ Tối Ưu Hiệu Năng & Bộ Nhớ (Performance & Zero-Allocation)
- **Từ Điển Chánh Tả Tĩnh + Bloom Filter 2 KB** (`vietnamese_dict_data.rs`, `vietnamese_dict.rs`): Chuyển 6.974 từ tiếng Việt sang mảng `const VIETNAMESE_WORDS_SORTED` đã sắp xếp sẵn kết hợp `VIETNAMESE_BLOOM_FILTER` (2 KB L1 cache) và `binary_search()`, loại bỏ chi phí khởi tạo `OnceLock<HashSet<String>>` trên luồng tra cứu.
- **FxHash & Stack Buffer Lowercasing** (`auto_restore.rs`): Thay SipHash mặc định bằng `FxHasher` / `FastHashSet` thuần Safe Rust (`#![forbid(unsafe_code)]`) và `with_stack_lowercase` dùng buffer `[u8; 64]` trên Stack để triệt tiêu cấp phát Heap cho từ ASCII.

### 🚀 Tính Năng Mới & Cải Tiến Lõi (Core, FFI & UI)
- **Biến Động Macro Mới**: Hỗ trợ `$VN_DATE` (`DD-MM-YYYY`), `$ISO_DATE` (`YYYY-MM-DD`), `$WEEKDAY` (`Thứ Hai` → `Chủ Nhật`), và `$CLIPBOARD` (nối trực tiếp Clipboard OS trên macOS `NSPasteboard` và Windows `Win32 Clipboard API`, không sinh tiến trình con).
- **Bảo Vệ Viết Tắt Chữ Hoa (`acronym_bypass`)**: Tự động giữ nguyên các cụm viết tắt ALL-CAPS 2–5 ký tự (như `IOS`, `VPS`, `FPS`...) khi kết thúc từ nếu không trùng từ điển tiếng Việt.
- **Tự Động Viết Hoa Đầu Câu (`auto_capitalize`)**: Tùy chọn viết hoa chữ cái đầu câu mới sau `.`, `?`, `!`, `\n` (bỏ qua tên miền, số thập phân, từ viết tắt `vd.`, `tp.`).
- **Bổ Sung Dấu Mũ/Móc Thông Minh (`smart_fuzzy_vowel`)**: Tùy chọn tự suy luận vần khép khi gõ Space (ví dụ `truongf ` → `trường `, `chuyenj ` → `chuyện `).
- **Phân Giải Va Chạm Theo Ngữ Cảnh (`prev_word_was_english`)**: Khi từ liền trước vừa được Auto-Restore về tiếng Anh, các từ va chạm ngắn kế tiếp (`is`, `as`, `this`, `has`, `see`) tự động giữ nguyên tiếng Anh.
- **Bộ Đệm Vòng Lùi Nhiều Từ (`reopen_history`)**: Nâng cấp `reopen_state` thành `VecDeque<ReopenWordState>` (tối đa 16 từ) cho phép bấm Backspace lùi qua nhiều từ liên tiếp để sửa dấu.
- **Khử Nhiễu Phím Kép Phần Cứng (`debounce_ms`)**: Lọc phím lặp phần cứng dựa trên `timestamp_ms`.
- **Live Inspector (Debug)**: Thêm `Engine::inspect_state()`, FFI `fcitx5_vnchkey_inspect_state`, và mục mở trực tiếp trên Windows, macOS, Linux (`vnchkey-settings`).
- **Thống Kê Hiệu Suất (`TypingStats`)**: Bổ sung `keystrokes_saved`, `serialize_compact()`, `parse_compact()`, và FFI `fcitx5_vnchkey_get_stats`.
- **Chế Độ Code (`developer_mode` & `DevCodeState` FSM)** (`engine.rs`, FFI, Linux/macOS/Windows): Tự động ngắt bỏ dấu tiếng Việt và ngắt bung Gõ Tắt bên trong Markdown inline code (`` `...` ``) và fenced code block (```` ```...``` ````); inline code tự đóng khi xuống dòng (`\n`).
- **Đà Ngữ Cảnh Song Ngữ & Ngữ Cảnh Toán Học (`in_english_context`, `math_context_words_remaining`)**: Các từ trùng phím Telex (`tree`, `cos`, `must`, `room`, `see`, `this`...) giữ nguyên tiếng Việt khi đứng độc lập (`tree ` → `trê `, `cos ` → `có `) nhưng tự động chuyển sang tiếng Anh khi đi sau từ neo tiếng Anh (`the tree `, `that is a tree `, `he must see this room.`) hoặc biểu thức lượng giác (`sin and cos `, `tan or cos `), và tự reset về tiếng Việt khi gặp dấu kết câu hoặc từ khóa tiếng Việt (`anh`, `em`, `tôi`, `không`).
- **Cải Tiến Lõi Telex & Guard Kỹ Thuật (`telex.rs`, `macros.rs`, `auto_restore.rs`, `engine.rs`)**:
  - Giữ dấu thanh cố định khi kéo dài nguyên âm biểu cảm sau âm tiết có móc `ư`/`ơ` (`uwfaaa ` → `ừaaa `, `trowifiii ` → `trờiiii `) mà vẫn bảo toàn từ tiếng Anh (`owe `, `twee `, `weee `).
  - Bão hòa ngữ nghĩa khi dư phím `w` cuối âm tiết hoàn chỉnh (`dduowcjw` → `đượcw`, `Backspace` → `được`) và khôi phục trọn vẹn từ tiếng Anh chứa nhiều `w` (`awkward `, `downward `).
  - Tự động khôi phục từ không phải tiếng Việt trước chữ số khi bật đồng thời Macro & Auto-Restore (`test123 `, `user2026 `) nhưng vẫn giữ nguyên macro chứa số (`k8s`, `mp3`) qua `MacroTable::has_prefix`.
  - Chặn bung Gõ Tắt bên cạnh dấu phân cách kỹ thuật (`google.com.vn`, `https://vn/dc`, `user@vn.com`, `dc()`, `vn[0]`) và bảo vệ slash commands / đường dẫn Unix (`/compact`, `/aws`, `/os`, `/usr/local/bin`).

---

## [0.3.6] - 2026-11-17

### 🚀 Tính Năng Mới & Đột Phá Kiến Trúc (Architectural Innovations)
- **Bộ Chuyển Đổi HOA / thường / Title Case Siêu Tốc (Rapid Case Converter)**:
  - Bổ sung phím tắt chuyển đổi kiểu chữ nhanh theo chu kỳ: `chữ thường` $\rightarrow$ `CHỮ HOA` $\rightarrow$ `Viết Hoa Đầu Từ (Title Case)`.
  - Phím tắt mặc định `Ctrl + Shift + U` (tự do thu nhận và tùy biến phím tắt trên cả Linux, Windows và macOS).
  - Tự động nhận diện ngữ cảnh: chuyển đổi từ đang gõ dở trong bộ đệm (`has_active_word()`) hoặc đoạn văn bản được bôi đen bằng chuột/bàn phím qua `surroundingText().selectedText()`.
- **Đồng Bộ Bộ Gõ Không Độ Trễ Cho Neovim / Terminal Qua Unix Domain Socket IPC Trên Linux**:
  - Tích hợp listener Unix Domain Socket non-blocking tại `/tmp/vnchkey.sock` trực tiếp vào Event Loop của Fcitx5 (`fcitx::EventSourceIO`), siết quyền bảo mật cục bộ `0600`.
  - Phục vụ module Neovim ([data/neovim/lua/vnchkey.lua](file:///home/codegpt/antigravity/vnchKey/data/neovim/lua/vnchkey.lua)) chuyển đổi chế độ gõ tức thì ($< 2\text{ ms}$) qua `vim.uv.new_pipe()` mà không làm mất biểu tượng cờ vàng trên khay hệ thống, không trôi phím chế độ Normal.
- **Bảo Vệ Chuỗi Mã Hóa & Token Kỹ Thuật (Technical Token Protection)**:
  - Tự động phát hiện và khóa tiếng Anh (`english_locked = true`) cho mã màu CSS Hex (`#f97316`, `#ef4444`), đơn vị đo lường lập trình (`10px`, `2rem`, `50vh`, `100vw`), và định danh kỹ thuật pha trộn chữ - số (`v1s`, `sha256`, `utf8`, `x86_64`).
- **Thanh Tra Thói Quen Gõ Trực Quan (Habit Inspector GUI)**:
  - Bổ sung bảng kiểm soát tiến trình tự học từ trực tiếp trong tab Từ Điển (`fcitx5-vnchkey-settings`).
  - Đọc từ nhật ký `vnchkey-habit.log`, hiển thị tiến trình dạng thanh (`count / threshold`) và cung cấp nút `+ Học ngay` để đẩy thẳng từ vào danh sách whitelist.
- **Bổ Sung Các Bộ Preset Gõ Tắt Chuyên Biệt Mới**:
  - Tích hợp sẵn 2 preset chọn lọc: **Lập Trình & Git** (`presets/dev-git-macro.conf`) và **Toán Học & Khoa Học** (`presets/math-science-macro.conf`).
- **Tái Cấu Trúc Toàn Diện VNI & Telex Sang Đại Số `VietnameseGlyph`**:
  - Loại bỏ hoàn toàn các bảng `match` phân rã/tái cấu trúc nguyên âm cồng kềnh trong `telex.rs` và `vni.rs`.
  - Toàn bộ thao tác tách/nạp nguyên âm, hoán đổi dấu thanh được vận hành qua đại số $O(1)$ trong `glyph.rs`.
- **Mô Hình Đại Số Dấu Thanh & NFD (`VietnameseGlyph`)**:
  - Tích hợp module `glyph.rs` mô hình hóa ký tự tiếng Việt dạng đại số:
    $$\text{Ký tự tiếng Việt} = \text{Base (ASCII)} \times \text{Quality (Mũ/Móc/Trăng/Stroke)} \times \text{Tone (Thanh điệu)}$$
  - Thực hiện các phép biến đổi, hoán đổi thanh điệu (`with_tone`), hoán đổi chất âm (`with_quality`) và khôi phục ký tự gốc (`base_ascii`) trong thời gian $O(1)$ thuần túy mà không tốn chi phí cấp phát bộ nhớ động (Zero Heap Allocation).
  - Kết hợp mô hình lai (Hybrid Architecture): bảo toàn tốc độ cực đại của bảng ánh xạ trực tiếp trong cache L1 khi gõ lướt, kết hợp cùng tính chặt chẽ của mô hình đại số NFD khi xử lý các tổ hợp dấu phức tạp.
- **Làm Sạch Toàn Diện Từ Điển Tiếng Anh Mặc Định & Mở Rộng Kho Ngoại Lệ Va Chạm**:
  - Rà soát và loại bỏ triệt để các từ tiếng Anh va chạm với tiếng Việt ra khỏi `DEFAULT_ENGLISH_WORDS`: `as`, `is`, `os`, `us`, `boss`, `chaos`, `less`, `max`, `mix`, `sex`, `six`, `reset`, `var`.
  - Chuyển toàn bộ các từ này vào `VIETNAMESE_COLLISION_EXCEPTIONS` để bảo vệ tối đa chữ tiếng Việt có nghĩa (`á`, `í`, `ó`, `ú`, `bú`, `cháo`, `mã`, `mĩ`, `sẽ`, `sĩ`, `rết`, `vả`) khi nhấn phím Space.
- **Mở Rộng Bộ Kiểm Thử Hồi Quy**:
  - Bổ sung unit tests cho module `glyph` kiểm tra tính đối xứng phân rã $\leftrightarrow$ tái cấu trúc (roundtrip decompose / compose) cho mọi nguyên âm mang dấu.
  - Mở rộng kịch bản kiểm thử trong `english_trap_simulation_tests.rs` bao phủ toàn bộ các từ va chạm mới được bảo vệ.

---

## [0.3.5] - 2026-11-15

### 🐛 Sửa Lỗi Quan Trọng (Bug Fixes)
- **Bảo Vệ Nguyên Âm Đơn Tiếng Việt Khi Nhấn Space (Single Vowel Collision Protection)**:
  - Khắc phục hiện tượng khi gõ Telex `as` $\to$ `á` nhưng sau khi bấm phím `Space` thì `á` bị hoàn nguyên ngược về phím thô (`as `) trên Windows và Linux.
  - Tách các từ nguyên âm đơn 2 ký tự (`as`, `is`, `os`, `us`) ra khỏi danh sách từ tiếng Anh mặc định (`DEFAULT_ENGLISH_WORDS`) và chuyển sang danh mục bảo vệ va chạm tiếng Việt (`VIETNAMESE_COLLISION_EXCEPTIONS`).
  - Đảm bảo các nguyên âm đơn mang dấu tiếng Việt hợp lệ (`á`, `í`, `ó`, `ú`, `é`, `É`, `ù`...) được giữ nguyên vẹn 100% khi kết thúc từ bằng dấu cách hoặc phím dấu câu.
- **Khắc Phục Lỗi Xóa Từ Trong User Dictionary Trên GUI (User Dictionary Persistence)**:
  - Sửa lỗi trong hàm lưu từ điển người dùng (`ConfigManager::save_dictionary`): loại bỏ việc tự động nạp và merge ngược lại với từ điển mặc định / file cũ trên đĩa.
  - Khi người dùng xóa một từ (ví dụ: `as`) trong Settings GUI và lưu, file cấu hình `~/.config/fcitx5/conf/vnchkey-dictionary.conf` sẽ cập nhật chính xác và không còn bị phục hồi lại từ đã xóa sau khi khởi động lại fcitx5.
- **Bổ Sung Bộ Kiểm Thử Hồi Quy Toàn Diện**:
  - Bổ sung test cases bao phủ toàn bộ các nguyên âm đơn (`as `, `is `, `os `, `us `, `es `, `Es `, `uf `) trong `tests/english_trap_simulation_tests.rs` và `tests/vietc_regression_tests.rs`.

---

## [0.3.4] - 2026-11-14

### 🚀 Tính Năng Mới & Đột Phá Ngữ Âm Học (Linguistic Innovations)
- **Cơ Chế Gõ Dấu Trung Gian Tự Do (Intermediate Two-Stroke Accent Mechanics)**:
  - Cho phép người dùng gõ đảo thứ tự dấu tự do trong các từ tiếng Việt bắt buộc mang 2 dấu (dấu mũ/móc và dấu thanh), ví dụ: gõ thanh trước mũ/móc sau hoặc ngược lại:
    - VNI: `duong2` $\to$ `duong2`, `duong7` $\to$ `dương`, `duong72` hoặc `duong27` $\to$ `dường`.
    - Telex: `duowngf` $\to$ `dường`, `dduowngf` $\to$ `đường`, `tuowngf` $\to$ `tường`, `nuowcs` $\to$ `nước`.
  - **Tự động hoàn nguyên Raw Text khi nhấn Space**: Xử lý triệt để bẫy cụm âm tiết trung gian (`duong2`, `tuong2`, `nuoc1`, `tiens`, `Huyèn`, `chuyén`): hiển thị dấu trung gian trực quan trong quá trình soạn thảo, nhưng nếu nhấn `Space` mà người dùng chưa bỏ dấu thứ 2 để tạo nên từ có nghĩa tiếng Việt thì tự động hoàn nguyên về phím thô (`raw text`).
  - Bảo toàn ngoại lệ tốc ký từ "được" (như `duoc95`, `duoc59` $\to$ `được` trong VNI, `dduocwj` $\to$ `được` trong Telex).
- **Hoàn Thiện Tự Động Khôi Phục Tiếng Anh (English Auto-Restore Hardening)**:
  - Khắc phục triệt để hiện tượng gõ Telex nuốt chữ cái lặp khi gặp phím undo (`error` $\to$ `eror`, `mirror` $\to$ `miror`, `sorry` $\to$ `sory`): khi nhấn `Space`, bộ gõ nhận diện từ điển tiếng Anh kỹ thuật và tự động trả lại đúng nguyên bản (`error `, `mirror `).
- **Mở Rộng Bộ Kiểm Thử Hồi Quy & Ngữ Âm Toàn Diện**:
  - Bổ sung cặp nhị trùng âm `('y', 'u')` (trong `khuỷu`) vào danh sách vần hợp lệ của Telex Engine, tránh việc bị nhận nhầm là tiếng Anh lỗi chính tả.
  - Cập nhật thêm các từ hợp lệ vào từ điển chuẩn: `uồn`, `toãn`, `toạn`, `nguệch`, `ngoạc`, cùng bộ âm tiết phương ngữ miền Nam và chính tả trước 1975.
  - **Khắc phục bẫy vần `oeo` / `eo` trong Telex**: sửa lỗi `ngoefo` bị biến âm nhầm `oo` $\to$ `ô`, cho phép gõ chuẩn xác `ngoèo`, `khoèo`, `mèo`.
  - **Phân biệt hoa/thường cho ký tự toán học & Symbol**: tìm kiếm ưu tiên khớp chính xác hoa/thường trước (`:Omega:` $\to$ $\Omega$, `:omega:` $\to$ $\omega$, `:Gamma:` $\to$ $\Gamma$, `:gamma:` $\to$ $\gamma$).
  - **Bảo toàn chữ hoa sau dấu nháy đơn (`harmonize_syllable_case`)**: giữ nguyên dạng viết hoa chuẩn của các địa danh và tên riêng như `M'Đrắk`, `H'Leo` khi bật `initial_case_anchor`.
  - Bổ sung bộ kiểm thử hồi quy nâng cao `nightmare_edge_cases` và `v034_coverage_tests` bao phủ hơn 100 kịch bản quái đản thực tế: gõ ẩu sai thứ tự, kẹt nút bàn phím cơ, bẫy nguyên âm 3, bẫy trùng từ tiếng Anh và cú pháp mã nguồn/URL.

### 🎨 Tinh Chỉnh Giao Diện & Đồng Bộ Đa Nền Tảng
- **Chuẩn Hóa Thông Điệp Giới Thiệu (About Dialogs)**:
  - Đồng bộ thông điệp trên Linux, Windows và macOS: *"Bộ gõ Tiếng Việt hiện đại và gọn nhẹ"*.
  - Sửa chuẩn chính tả mô tả tính năng: *"Tự động khôi phục Tiếng Anh, Smart Casing, hồi sinh từ"*.
  - Bổ sung terminal Ghostty, Rio vào danh sách các ứng dụng dòng lệnh mặc định không dùng Preedit để đảm bảo tốc độ phản hồi tức thì.

---

## [0.3.3] - 2026-11-12

### 🚀 Tính Năng Mới & Cải Tiến Đột Phá
- **Trao Quyền Tự Do Tuyệt Đối Cho Người Dùng (User-Driven Autonomy & Pure Core)**:
  - **Dọn dẹp Core Engine tinh gọn**: Loại bỏ các cơ chế tự đoán từ kiểu áp đặt (opinionated feature creep như tự động đổi `khong` $\to$ `không`, `truong` $\to$ `trương`, `nganh` $\to$ `ngành`), giữ Core chuẩn xác, trung lập và tốc độ tối đa.
  - **Tùy chọn Đồng Nhất Chữ Hoa Đầu Từ (Initial Case Anchor: `ĐƯợc`/`ĐƯỢc` ➔ `Được`, `đƯỢc` ➔ `được`)**:
    - Chuyển thành cờ cấu hình tùy chọn người dùng trên cả 3 nền tảng (Linux Settings GUI, Windows Menu, macOS Menu & Dashboard).
    - **Mặc định TẮT (`false`)** để tôn trọng 100% cách gõ của người dùng và các token lập trình (`iOS`, `eBay`, `parseS`). Người dùng có thể bật lên bất cứ lúc nào nếu muốn tự động hóa giải hiện tượng nhả phím Shift chậm trên bàn phím cơ.
  - **Nâng Cấp Toàn Diện Bộ Học Thói Quen (Adaptive Habit Learner)**:
    - Bổ sung công tắc **Bật/Tắt** toàn quyền cho tính năng tự học thói quen gõ phím.
    - Cho phép người dùng **tự cấu hình số lần xóa/sửa sai (Promotion Threshold)** từ 2 đến 10 lần (mặc định nâng lên **4 lần** để tránh tai nạn gõ nhầm).
    - Đồng bộ xóa hai chiều tức thì (**Instant Unlearn**): các từ tự học được đưa vào Từ Điển Cá Nhân, khi người dùng xóa từ khỏi từ điển thì bộ gõ lập tức hủy học và khôi phục khả năng gõ dấu tiếng Việt bình thường ngay tức thì.
- **Nhập Ký Tự Toán Học & Emoji Nhanh Qua Tiền Tố (Inline Symbol & Emoji Picker)**:
  - Hỗ trợ gõ shortcode bắt đầu bằng dấu hai chấm (ví dụ: `:smile:` ➔ 😄, `:arrow:` ➔ ➔, `:alpha:` ➔ α, `:pi:` ➔ π, `:check:` ➔ ✓, `:sum:` ➔ ∑).
  - Tự động thay thế trực tiếp ngay khi kết thúc bằng dấu `:` hoặc phím cách (Space).
  - Hỗ trợ Undo hai bước khi nhấn Backspace để quay lại mã shortcode ban đầu.
  - Danh mục Emoji & Symbol được chọn lọc trung lập, chuyên dụng cho tài liệu kỹ thuật, toán học và biểu cảm thường nhật; tuyệt đối không chứa biểu tượng chính trị.
- **Tốc Ký Toàn Diện Trên Linux (Quick Consonants)**:
  - Kết nối đầy đủ 3 chế độ gõ nhanh phụ âm trên GUI Settings (`vnchkey-settings`) sang Fcitx5 Addon và Rust FFI:
    - Phụ âm đôi (`cc` ➔ `ch`, `gg` ➔ `gi`, `kk` ➔ `kh`, `nn` ➔ `ng`, `pp` ➔ `ph`, `tt` ➔ `th`).
    - Phụ âm đầu (`f` ➔ `ph`, `j` ➔ `gi`, `w` ➔ `qu`).
    - Phụ âm cuối (`g` ➔ `ng`, `h` ➔ `nh`, `k` ➔ `ch`).
- **Lưu Cấu Hình Di Động 100% (Windows Portable Mode)**:
  - Tự động lưu và khôi phục toàn bộ cấu hình (`method`, `charset`, `tone_style`, `spell_check`, `auto_restore`, `esc_restore`, `quick_*`,...) vào `config.conf` ngay cùng thư mục với `VnCharKey.exe`.
  - Khởi động lại ứng dụng vẫn giữ nguyên 100% kiểu gõ và các tùy chọn người dùng đã chọn.

### ⚡ Tối Ưu Hiệu Năng & An Toàn Tiến Trình
- **Macro Siêu Tốc Trong Luồng Gõ Phím (Zero Subprocess Overhead)**:
  - Loại bỏ hoàn toàn lệnh `Command::new("date")` khi mở rộng `$DATE`, `$TIME`, `$NOW`, `$DATETIME`.
  - Thay bằng giải thuật thuần Safe Rust đọc trực tiếp IANA TZif chuẩn (`/etc/localtime`) và công thức Howard Hinnant từ `SystemTime`. Giảm độ trễ từ 5–15ms xuống ~0.02ms, hoàn toàn an toàn trong sandbox.
- **Khử Deadlock Fcitx5 Addon Khi Mở Cài Đặt**:
  - Loại bỏ lệnh `fork()` + `execlp()` thủ công trong tiến trình đa luồng Fcitx5 C++; chuyển sang sử dụng API chuẩn `fcitx::startProcess()`.
- **Bảo Mật Socket IPC (macOS)**:
  - Chuyển quyền socket `/tmp/vnchkey.sock` sang `0o600`, ngăn chặn truy cập trái phép trên các hệ thống dùng chung nhiều tài khoản.

### 🐛 Sửa Lỗi (Bug Fixes)
- **Harden Adaptive Habit Learner**:
  - Siết chặt điều kiện học `BackspaceRetype`: chỉ ghi nhận khi người dùng xóa sạch toàn bộ từ về rỗng (`current_word.is_empty()`), loại bỏ hoàn toàn việc học nhầm khi người dùng chỉ xóa vài ký tự để sửa chính tả tiếng Việt.
- **Bảo Vệ Từ Điển Cá Nhân Khi Lưu Cấu Hình**:
  - GUI Settings tự động nạp và hợp nhất (merge & dedup) từ điển trên đĩa trước khi ghi xuống file `vnchkey-dictionary.conf`, không làm mất từ do Habit Learner tự động thu nạp.
- **Sửa Lỗi Format Delimiter Cho Unicode Tổ Hợp (NFD)**:
  - Ký tự kết thúc từ hiện tại được định dạng chuẩn qua `format_commit` theo đúng bảng mã đã chọn.
- **Win32 Pure Registry API**:
  - Thay thế hoàn toàn `reg.exe` bằng các Win32 Registry APIs thuần (`RegOpenKeyExW`, `RegQueryValueExW`, `RegSetValueExW`) để đọc/ghi thiết lập khởi động cùng Windows không phụ thuộc công cụ ngoài.

---

## [0.3.2] - 2026-11-12

### 🐛 Sửa Lỗi & Cải Tiến
- **Khắc phục hiện tượng Shift-bleed Casing**: Sửa lỗi gõ VNI TitleCase `Duong92` / `DUong92` bị thành `ĐƯờng`, đảm bảo luôn ra chuẩn `Đường`.
- **Tùy chọn Phím Tắt Đảo Từ (Flip Word)**: Bổ sung cấu hình phím tắt cho Linux Fcitx5 Settings (`Ctrl+Shift+Z`, `Alt+X`, `Ctrl+Alt+Z`, hoặc Tắt) đồng bộ với Windows.
- **Hệ Thống Chẩn Đoán System Doctor**: Cải tiến giao diện kiểm tra chẩn đoán biến môi trường Wayland/Qt (`QT_IM_MODULE`, `XMODIFIERS`).

---

## [0.3.1] - 2026-11-10

### 🚀 Tính Năng Mới
- **VnCharKey Cho Windows**: Phát hành phiên bản chính thức đầu tiên dành cho Windows dưới dạng file portable độc lập (`VnCharKey.exe`) với biểu tượng nón lá đặc trưng.
- **Tự Động Đóng Gói CI/CD**: Thiết lập GitHub Actions tự động build và đóng gói file zip cho Windows.

---

## [0.3.0] - 2026-11-09

### 🚀 Tính Năng Mới & Đột Phá
- **Keystroke Idle Timeout**: Tự động giải phóng từ đang gõ dở sau một khoảng thời gian chờ (configurable idle timeout).
- **Tích Hợp Chuyên Sâu Cho Neovim**: Hỗ trợ Esc thoát lệnh Neovim ngay lập tức và chế độ hoàn nguyên chữ thô với `Shift+Esc`.
- **Học Tập Thói Quen (Adaptive Habit Learner)**: Giới hạn file log 100MB và tự động thăng cấp từ tiếng Anh thường dùng vào danh sách bỏ qua (Whitelist).

