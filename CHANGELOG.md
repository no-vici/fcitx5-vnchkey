# Changelog

Tất cả những thay đổi nổi bật của dự án **vnchKey** (**fcitx5-vnchkey** & **VnCharKey**) qua từng phiên bản được ghi nhận tại đây.

---

## [0.3.3] - 2026-11-12

### 🚀 Tính Năng Mới & Cải Tiến Đột Phá
- **Nhập Ký Tự Toán Học & Emoji Nhanh Qua Tiền Tố (Inline Symbol & Emoji Picker)**:
  - Hỗ trợ gõ shortcode bắt đầu bằng dấu hai chấm (ví dụ: `:smile:` ➔ 😄, `:arrow:` ➔ ➔, `:alpha:` ➔ α, `:pi:` ➔ π, `:check:` ➔ ✓, `:sum:` ➔ ∑).
  - Tự động thay thế khi kết thúc bằng dấu `:` hoặc phím cách (Space).
  - Hỗ trợ Undo hai bước khi nhấn Backspace để quay lại mã shortcode ban đầu.
  - Tích hợp khung chọn nhanh (candidate popup) trên Fcitx5 và Windows.
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
- Khắc phục triệt để hiện tượng Shift-bleed casing (ví dụ: `Duong92` ➔ `Đường`).
- Bổ sung tùy chọn phím tắt Đảo Từ (Flip Word) vào Fcitx5 Settings (`Ctrl+Shift+Z`, `Alt+X`, `Ctrl+Alt+Z`, hoặc Tắt).
- Cải thiện System Doctor kiểm tra biến môi trường Qt/Wayland.
