# Changelog

Tất cả những thay đổi nổi bật của dự án **vnchKey** (**fcitx5-vnchkey** & **VnCharKey**) qua từng phiên bản được ghi nhận tại đây.

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

