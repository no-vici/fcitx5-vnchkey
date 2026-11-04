# Hướng Dẫn Kỹ Thuật Xây Dựng Bộ Gõ Tiếng Việt Hiệu Năng Cao Trên Linux

Tài liệu này cung cấp bản thiết kế kiến trúc toàn diện và hướng dẫn từng bước để xây dựng một bộ gõ tiếng Việt hiện đại, nhẹ, ổn định và an toàn trên hệ điều hành Linux (tương thích cả Wayland và X11) dựa trên kiến trúc của **vnchKey**.

---

## 1. Tổng Quan Kiến Trúc (Architecture Overview)

Một bộ gõ tiếng Việt hiện đại trên Linux gồm 3 tầng kiến trúc phân lớp rõ ràng:

```
┌────────────────────────────────────────────────────────┐
│             Desktop Applications (Wayland / X11)       │
│        (Alacritty, VSCode, Firefox, Chrome, LibreOffice)│
└───────────────────────────▲────────────────────────────┘
                            │ Wayland text-input-v3 / XIM / DBus
┌───────────────────────────┴────────────────────────────┐
│              Fcitx5 Input Method Framework             │
│   (Quản lý InputContext, StatusArea, Tray, KeyEvents)  │
└───────────────────────────▲────────────────────────────┘
                            │ C++ Module (fcitx5-addon)
┌───────────────────────────┴────────────────────────────┐
│          Fcitx5 C++ Shim Module (fcitx5-vnchkey)       │
│ - Bắt sự kiện bàn phím (KeyEvent)                      │
│ - Kiểm tra SurroundingText Capability                  │
│ - Điều hướng phím BackSpace / CommitString             │
└───────────────────────────▲────────────────────────────┘
                            │ C-ABI (extern "C" Shared Library)
┌───────────────────────────┴────────────────────────────┐
│         Rust Core IME Engine (vnchkey-core / ffi)      │
│ - Telex & VNI State Machine                            │
│ - Micro-Diff LCP Replacement (So khớp tiền tố)         │
│ - Restore English Algorithm                            │
│ - Built-in Vietnamese Dictionary & Chánh Tả Checker    │
└────────────────────────────────────────────────────────┘
```

---

## 2. Vì Sao Chọn Fcitx5 Làm Nền Tảng Thay Vì IBus Hay X11 Hooking Thuần?

1. **Hỗ trợ Wayland gốc (Native Wayland Support)**:
   - Fcitx5 hỗ trợ giao thức `wayland text-input-v1/v2/v3` và `input-method-v1/v2`.
   - Các giải pháp cũ bắt phím cấp thấp (`xdotool`, `XGrabKeyboard`) hoàn toàn tê liệt trên Wayland (Sway, Hyprland, GNOME Wayland, KDE Plasma Wayland).
2. **Khả năng cách ly Context (Per-client State)**:
   - Mỗi cửa sổ ứng dụng (mỗi tab hoặc tiến trình) sở hữu một `InputContext` riêng biệt. Chuyển đổi giữa terminal và trình duyệt không bao giờ bị dính đệm gõ cũ.
3. **Cơ chế SurroundingText**:
   - Cho phép bộ gõ đọc trực tiếp đoạn văn bản xung quanh con trỏ trong các ứng dụng hỗ trợ (GTK/Qt), thực hiện `deleteSurroundingText` mà không cần giả lập bấm phím Backspace.

---

## 3. Thiết Kế Lõi Biến Đổi Âm Tiết Bằng Rust

Lõi `vnchkey-core` được xây dựng hoàn toàn bằng **100% Safe Rust** (`#![forbid(unsafe_code)]`):

### 3.1. Máy Trạng Thái Telex & VNI
- Nhận ký tự thô `char` theo thời gian thực.
- Sử dụng bảng biến đổi ma trận nguyên âm:
  - Phím mũ: `a + a -> â`, `e + e -> ê`, `o + o -> ô`
  - Phím râu / trăng: `w` hoặc số `7`, `8`
  - Phím thanh: `s/f/r/x/j` hoặc số `1/2/3/4/5`
- Tách bạch rõ rệt giữa:
  - Onset glide (`qu-`, `gi-`): Không bỏ dấu nhầm lên bán nguyên âm.
  - Vowel nucleus (Hạt nhân nguyên âm): Nơi tiếp nhận thanh điệu theo chuẩn Mới hoặc Cũ.
- **Sửa dấu linh hoạt (Free marking & In-place modification)**:
  - Cho phép người dùng gõ phím dấu ở bất kỳ vị trí nào trong từ hoặc đổi dấu tự do (ví dụ: `gio72` $\rightarrow$ `giờ`, gõ tiếp `1` $\rightarrow$ `giớ`, gõ `0` $\rightarrow$ xoá dấu thành `giơ`).

### 3.2. Thuật Toán Thay Thế Tối Thiểu (Micro-Diff LCP Replacement)
Thay vì xóa toàn bộ từ cũ (e.g., bấm 5 lần Backspace rồi gõ lại cả từ), bộ gõ dùng thuật toán **Longest Common Prefix**:

```rust
pub fn compare_and_split_strings(old_word: &str, new_word: &str) -> (usize, String) {
    let old_chars: Vec<char> = old_word.chars().collect();
    let new_chars: Vec<char> = new_word.chars().collect();

    let mut common_len = 0;
    let min_len = old_chars.len().min(new_chars.len());
    while common_len < min_len && old_chars[common_len] == new_chars[common_len] {
        common_len += 1;
    }

    let backspaces = old_chars.len() - common_len;
    let commit_text: String = new_chars[common_len..].iter().collect();
    (backspaces, commit_text)
}
```
**Ví dụ thực tế**:
- Gõ `k - h - o - n - g`: Tất cả các phím này được cho qua tự nhiên (`consumed = false`, `backspaces = 0`). Ứng dụng tự in ra màn hình mà bộ gõ không cần can thiệp.
- Gõ tiếp `o` để tạo thành `không`:
  - `old_word = "khong"`
  - `new_word = "không"`
  - Tiền tố chung là `"kh"` (độ dài 2 ký tự).
  - Kết quả: Chỉ xóa đúng 3 ký tự (`"ong"`) và chèn vào `"ông"`.
  - **Hiệu quả**: Loại bỏ 100% hiện tượng chớp nháy chữ trên màn hình.

---

## 4. Cơ Chế Tích Hợp C++ Addon Module Với Fcitx5

Thư viện C++ Addon liên kết động với Rust thông qua C-ABI:

```cpp
void VnchkeyEngine::keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) {
    // 1. Bỏ qua phím nhả ra (KeyRelease) và phím chức năng đơn độc
    if (keyEvent.isRelease() || keyEvent.rawKey().isModifier()) return;

    // 2. Bỏ qua các tổ hợp phím Ctrl/Alt/Super và các phím điều hướng con trỏ
    if (states.test(fcitx::KeyState::Ctrl) || keyEvent.key().isCursorMove()) {
        fcitx5_vnchkey_reset(ctx);
        return;
    }

    // 3. Xử lý Backspace vật lý
    if (sym == FcitxKey_BackSpace) {
        fcitx5_vnchkey_handle_backspace(ctx);
        return;
    }

    // 4. Gửi ký tự sang lõi Rust
    size_t out_backspaces = 0;
    char* out_commit = nullptr;
    int consumed = fcitx5_vnchkey_process_key(ctx, ch, timestamp, &out_backspaces, &out_commit);

    if (consumed) {
        if (out_backspaces > 0) {
            // Ưu tiên dùng SurroundingText nếu ứng dụng hỗ trợ
            if (ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
                ic->surroundingText().isValid()) {
                ic->deleteSurroundingText(-static_cast<int>(out_backspaces), out_backspaces);
            } else {
                // Fallback: phát xung Backspace vi mô
                for (size_t i = 0; i < out_backspaces; ++i) {
                    ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                }
            }
        }
        if (out_commit && out_commit[0] != '\0') {
            ic->commitString(out_commit);
        }
        keyEvent.filterAndAccept();
    }
}
```

---

## 5. Quy Trình Đóng Gói & Cài Đặt Chuẩn Linux

1. **Biên dịch Release Rust**:
   ```bash
   cargo build --release
   ```
2. **Biên dịch C++ Addon với CMake & Ninja**:
   ```bash
   mkdir -p fcitx5-addon/build && cd fcitx5-addon/build
   cmake -GNinja -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release ..
   ninja
   ```
3. **Cài đặt các tệp metadata vào hệ thống**:
   - `libfcitx5_vnchkey_ffi.so` $\rightarrow$ `/usr/lib/`
   - `fcitx5-vnchkey-addon.so` $\rightarrow$ `/usr/lib/fcitx5/`
   - `vnchkey.conf` (Addon) $\rightarrow$ `/usr/share/fcitx5/addon/vnchkey.conf`
   - `vnchkey.conf` (InputMethod) $\rightarrow$ `/usr/share/fcitx5/inputmethod/vnchkey.conf`
   - `fcitx5-vnchkey-settings` $\rightarrow$ `/usr/bin/fcitx5-vnchkey-settings`
4. **Khởi động lại Fcitx5**:
   ```bash
   fcitx5 -r -d
   ```
Sau khi cài đặt, bộ gõ sẽ xuất hiện tức thì trong khay hệ thống mà không cần khởi động lại máy tính.
