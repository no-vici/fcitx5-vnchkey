# vnchKey (Tự Do Edition)

<div align="center">

<img src="assets/logo.png" alt="vnchKey Official Logo" width="128" height="128" />

**Bộ Gõ Tiếng Việt Hiệu Năng Cao Cho Linux (Fcitx5 / Wayland / X11)**  
*Được phát triển bằng Rust kết hợp Native C++ Fcitx5 Module.*  
*Hội tụ tinh hoa xử lý biến âm từ **OpenViet IME**, **OpenKey**, **GoNhanh**, **VietC** và thuật toán Micro-diff từ **Lilypad**.*  
*Mang đậm tinh thần Tự Do và Di sản Cờ Vàng Ba Sọc Đỏ.*

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-GNU%20GPL%203.0-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20(Wayland%20%2F%20X11)-red.svg?style=flat-square&logo=linux)](https://archlinux.org)
[![Zero Telemetry](https://img.shields.io/badge/Telemetry-Zero%20(100%25%20Offline)-success.svg?style=flat-square)](#)
[![AI-Assisted](https://img.shields.io/badge/AI%20Assisted-Google%20Gemini-8E75C2?logo=google-gemini&logoColor=white)](https://gemini.google.com)

</div>

---

*Nếu bạn hông thích biểu tượng của bộ gõ thì hãy bỏ qua và lựa chọn những bộ khác với những tính năng vượt trội tương tự như [fcitx5-lilypad](https://github.com/chiconcota/fcitx5-lilypad) (chiconcota). Xin đừng vì sự khác biệt về ý thức hệ hay quan điểm chánh trị mà report đứa con tinh thần của một coding learner thật sự yêu tự do trên từng ký tự anh ta gõ xuống.*

## 📖 Mục Lục

1. [Tổng Quan Kiến Trúc Thực Tế](#1-tổng-quan-kiến-trúc-thực-tế)
2. [Cấu Trúc Workspace & Phân Tầng Mã Nguồn](#2-cấu-trúc-workspace--phân-tầng-mã-nguồn)
3. [Các Tính Năng & Cơ Chế Hoạt Động Cốt Lõi](#3-các-tính-năng--cơ-chế-hoạt-động-cốt-lõi)
   - [3.1. Thuật Toán Micro-diff Minimal Replacement](#31-thuật-toán-micro-diff-minimal-replacement)
   - [3.2. Chế Độ Xuất Chữ Kép: Direct Mode & Preedit Mode (Đặc Trị Google Docs & Web App)](#32-chế-độ-xuất-chữ-kép-direct-mode--preedit-mode-đặc-trị-google-docs--web-app)
   - [3.3. Tách Âm Tiết Cuối Tối Ưu Cho Neovim / WezTerm / Vim](#33-tách-âm-tiết-cuối-tối-ưu-cho-neovim--wezterm--vim)
   - [3.4. Bộ Kiểm Tra Chánh Tả, Ngữ Âm Học & Địa Danh Tây Nguyên](#34-bộ-kiểm-tra-chánh-tả-ngữ-âm-học--địa-danh-tây-nguyên)
   - [3.5. Tự Động Khôi Phục Tiếng Anh (Restore English)](#35-tự-động-khôi-phục-tiếng-anh-restore-english)
   - [3.6. Chế Độ Gõ Đa Năng: Auto, Telex, VNI](#36-chế-độ-gõ-đa-năng-auto-telex-vni)
   - [3.7. Bảo Toàn Chữ Hoa Thông Minh (Smart Casing)](#37-bảo-toàn-chữ-hoa-thông-minh-smart-casing)
   - [3.8. Bảng Gõ Tắt Mở Rộng Tức Thì & Hoàn Tác Thông Minh (Macro Expansion & Smart Undo)](#38-bảng-gõ-tắt-mở-rộng-tức-thì--hoàn-tác-thông-minh-macro-expansion--smart-undo)
   - [3.9. Bộ Bóc Tách Âm Tiết 5 Thành Phần & Dời Dấu Thanh Động (Next-Gen Syllable Parser)](#39-bộ-bóc-tách-âm-tiết-5-thành-phần--dời-dấu-thanh-động-next-gen-syllable-parser)
   - [3.10. Hoán Đổi Dấu Phụ Bảo Toàn Thanh Điệu (Shape Modifier Swapping — Tinh Hoa VietC)](#310-hoán-đổi-dấu-phụ-bảo-toàn-thanh-điệu-shape-modifier-swapping--tinh-hoa-vietc)
   - [3.11. Bảo Vệ Ngữ Âm Học & Chống Gõ Sai Dấu (Phonetic Tone Protection)](#311-bảo-vệ-ngữ-âm-học--chống-gõ-sai-dấu-phonetic-tone-protection)
   - [3.12. Tổ Hợp Phím Tắt Thông Minh & Hệ Thống "Cứu Chữ / Cứu Dấu"](#312-tổ-hợp-phím-tắt-thông-minh--hệ-thống-cứu-chữ--cứu-dấu)
   - [3.13. Tự Động Học Thói Quen Gõ & Đưa Vào Whitelist (Adaptive Habit Learner — Log 100 MB)](#313-tự-động-học-thói-quen-gõ--đưa-vào-whitelist-adaptive-habit-learner--log-100-mb)
   - [3.14. Chẩn Đoán Hệ Thống `--doctor`, Cố Vấn Electron Wayland & Tự Động Bypass Game/Password](#314-chẩn-đoán-hệ-thống---doctor-cố-vấn-electron-wayland--tự-động-bypass-gamepassword)
   - [3.15. Bộ Lọc & Đặc Trị Thực Chiến Linux / Wayland (Hardened Runtime Fixes)](#315-bộ-lọc--đặc-trị-thực-chiến-linux--wayland-hardened-runtime-fixes)
4. [Giao Diện Cài Đặt Trực Quan (Rust + egui GUI)](#4-giao-diện-cài-đặt-trực-quan-rust--egui-gui)
5. [Hướng Dẫn Cài Đặt Cho Linux, macOS & Windows](#5-hướng-dẫn-cài-đặt-cho-linux-macos--windows)
6. [Lộ Trình Phát Triển & Tài Liệu Kỹ Thuật (Roadmap & Docs)](#6-lộ-trình-phát-triển--tài-liệu-kỹ-thuật-roadmap--docs)
7. [Ý Nghĩa Biểu Tượng & Di Sản Tự Do](#7-ý-nghĩa-biểu-tượng--di-sản-tự-do)
8. [Lời Cảm Ơn / Ghi Chú Phát Triển](#8-lời-cảm-ơn--ghi-chú-phát-triển)
9. [Bản Quyền & Giấy Phép](#9-bản-quyền--giấy-phép)

---

## 1. Tổng Quan Kiến Trúc Thực Tế

**vnchKey** được xây dựng như một module native tích hợp trực tiếp vào framework **Fcitx5** – khung nhập liệu chuẩn mực và phổ biến nhất trên Linux hiện nay. 

Thay vì phải chạy tiến trình daemon ngoài với quyền root để can thiệp kernel, **vnchKey** hoạt động trực tiếp trong không gian người dùng của Fcitx5:
- **Lõi xử lý Tiếng Việt viết bằng Rust (`vnchkey-core`)**: Quản lý toàn bộ State Machine của âm tiết, kiểm tra luật ngữ âm, đặt dấu thanh và khôi phục từ tiếng Anh.
- **Cầu nối C-ABI (`fcitx5-vnchkey-ffi`)**: Cung cấp giao diện hàm extern C an toàn, truyền nhận dữ liệu trực tiếp giữa Rust và C++.
- **Module Fcitx5 Native Addon (`fcitx5-addon`)**: Plugin C++ biên dịch thành `fcitx5-vnchkey-addon.so`, nạp trực tiếp vào Fcitx5 Server, tương thích hoàn hảo với cả Wayland (Sway, Hyprland, GNOME, KDE) và X11.
- **Ứng dụng thiết lập các tính năng (`vnchkey-settings`)**: 100% Rust viết bằng thư viện đồ hoạ `egui` / `eframe`, khởi động tức thì, tinh tế và đồng bộ trực tiếp với file cấu hình Fcitx5.

---

## 2. Cấu Trúc Workspace & Phân Tầng Mã Nguồn

```
fcitx5-vnchkey/
├── Cargo.toml                       # Workspace Root Manifest (LTO, strip, panic=abort)
├── crates/
│   ├── vnchkey-core/                # [Lõi Rust - #![forbid(unsafe_code)]] Xử lý âm vị học & biến đổi dấu Tiếng Việt
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs               # Module exports
│   │   │   ├── engine.rs            # Bộ điều phối trung tâm (Syllable State Machine)
│   │   │   ├── telex.rs             # Biến đổi Telex, quy tắc âm tiết & tách âm tiết cuối
│   │   │   ├── vni.rs               # Biến đổi VNI đa thứ tự phím (1-9, 0 xóa dấu)
│   │   │   ├── syllable_parser.rs   # Bộ bóc tách âm tiết 5 thành phần (Onset-Glide-Nucleus-Coda-Tone)
│   │   │   ├── habit_learner.rs     # [Mới] Tự động học thói quen sửa lỗi & Whitelist (Log 100 MB FIFO)
│   │   │   ├── auto_restore.rs      # Từ điển tiếng Anh kỹ thuật & từ điển ngoại lệ người dùng
│   │   │   ├── casing.rs            # Theo dõi và bảo toàn kiểu chữ hoa/thường
│   │   │   ├── macros.rs            # Bảng từ viết tắt (Gõ tắt) & hoàn tác 2 bước
│   │   │   └── vietnamese_dict.rs   # Bộ từ điển Chánh Tả ~6.750 từ gốc tiếng Việt
│   │   └── tests/
│   │       └── vietc_regression_tests.rs # Bộ kiểm thử hồi quy đối sánh từ VietC
│   │
│   ├── fcitx5-vnchkey-ffi/          # [Tầng C-ABI] Giao tiếp giữa Rust core và C++ / Swift
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs               # C-compatible FFI exported functions
│   │
│   └── vnchkey-settings/            # [Giao diện Cài Đặt & Chẩn Đoán] Viết bằng Rust + egui/eframe
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs              # Điểm khởi chạy GUI và lệnh CLI chẩn đoán (--doctor / -d)
│           ├── app.rs               # Giao diện UI với 10 tab quản lý trực quan (kèm Tab Doctor)
│           ├── doctor.rs            # [Mới] Trình kiểm tra môi trường Fcitx5 & /proc/PID Electron Advisor
│           └── config.rs            # Trình đọc/ghi cấu hình ~/.config/fcitx5/conf/
│
├── fcitx5-addon/                    # [Fcitx5 Native Addon] Module C++ cho Fcitx5
│   ├── CMakeLists.txt
│   └── src/
│       ├── vnchkey_addon.h          # Khai báo lớp VnchkeyEngine kế thừa InputMethodEngineV2
│       └── vnchkey_addon.cpp        # Đăng ký sự kiện phím, StatusArea và Menu tùy chọn
│
├── data/                            # Desktop entries, metadata & icon packs
├── install.sh                       # Script biên dịch và cài đặt tự động 1-Click
└── PKGBUILD                         # Script đóng gói chuẩn cho Arch Linux
```

---

## 3. Các Tính Năng & Cơ Chế Hoạt Động Cốt Lõi

### 3.1. Thuật Toán Micro-diff Minimal Replacement
Được tối ưu từ nguyên lý của `fcitx5-lilypad` (`compareAndSplitStrings`):
- Khi biến đổi âm tiết (ví dụ: `toan` + `s` $\to$ `toán`), thay vì xóa toàn bộ từ rồi gõ lại từ đầu, engine tìm **Tiền Tố Chung Dài Nhất (Longest Common Prefix)**:
  - Chỉ xóa lùi chính xác $1$ ký tự `n` (`backspaces = 1`).
  - Gởi tiếp chuỗi `án` (`commit_text = "án"`).
- **Kết quả**: Triệt tiêu hoàn toàn hiện tượng nhấp nháy chữ và giật lag con trỏ trong editor hiện đại (VS Code, Cursor, Zed, Sublime Text).

---

### 3.2. Chế Độ Xuất Chữ Kép: Direct Mode & Preedit Mode (Đặc Trị Google Docs & Web App)
Giải quyết triệt để vấn đề kinh điển của các bộ gõ trên Linux: **lặp chữ và nhảy con trỏ** trên các ứng dụng nền tảng Canvas hoặc React/Vue (Google Docs, StackEdit, Sider.ai, Slack, Discord...).

- **Chế độ Gõ Trực Tiếp (Direct Mode - Khuyên dùng cho Coding/Terminal/Editor)**:
  - Sử dụng thuật toán Micro-diff: phát phím Backspace giả lập và commit phần đuôi biến đổi ngay tức thì.
  - Tối ưu cho độ trễ cực thấp (gần như bằng $0$), mang lại cảm giác gõ mộc tự nhiên trên các editor chuyên nghiệp (Neovim, VS Code, Cursor, Zed, Alacritty, Kitty).
- **Chế độ Gõ Gạch Chân (Preedit Mode - Khuyên dùng cho Web/Google Docs/Chat)**:
  - Khắc phục tận gốc nguyên nhân gây lặp chữ: **Tuyệt đối không phát phím Backspace giả lập ra ứng dụng bên ngoài**.
  - Từ đang soạn được hiển thị bằng nét gạch chân nội dòng (`setClientPreedit`) và được cam kết nguyên tử một lần duy nhất (`commitString`) khi người dùng bấm phím cách (Space), Enter hoặc dấu phân cách.
  - **Khung chữ nổi (Floating Preedit Tooltip)**: Hiển thị từ đang gõ trong một ô nổi nhỏ ngay sát con trỏ chuột cho các ứng dụng tự vẽ Canvas như Google Docs (nơi trình duyệt không vẽ được nét gạch chân nội dòng). Đi kèm tùy chọn bật/tắt nhanh trên Menu hoặc Cài Đặt nếu người dùng muốn tránh phân tâm.
  - **Tự động nhớ chế độ theo từng ứng dụng (Per-App Mode Memory)**: Tự động kích hoạt Preedit cho các trình duyệt web (`Chrome`, `Firefox`, `Brave`, `Edge`) và ứng dụng chat Electron (`Slack`, `Discord`, `Telegram`), đồng thời tự bật Direct khi quay lại Terminal/IDE.

---

### 3.3. Tách Âm Tiết Cuối, VNI Numeric Guard & Khoảng Lặng Ngắt Từ (Đặc Trị Neovim / Terminal)
Một vấn đề cố hữu của các bộ gõ trên terminal emulator là các phím lệnh Normal mode (như `ci"`, `cw`, `i`, `a`, `jk`, `u`) hoặc các chuỗi số đứng sát trước con trỏ dễ bị kẹt trong bộ đệm từ, gây hiện tượng đột biến chữ (ví dụ gõ `1` sau `height="50"` bị biến thành `heigh="úi"`):
- **Giải pháp bóc tách âm tiết cuối**: Tích hợp hàm `extract_last_syllable_index()`:
  - Khi gõ chuỗi phím số VNI (như `biet61`), engine tự động bóc tách âm tiết Tiếng Việt hợp lệ đứng cuối cùng (`biet`) để áp dụng dấu thành `biết`.
  - Giữ nguyên vẹn văn bản đứng trước, gõ mượt mà ngay cả khi con trỏ đứng sát dấu ngoặc kép `""` hoặc sau các lệnh vim.
  - Tự động khóa khôi phục Tiếng Anh đối với từ gõ phím số VNI khi thoát Insert mode bằng phím tắt `jk`.
- **Bảo vệ Chuỗi Số VNI (VNI Numeric Guard)**:
  - Trong kiểu gõ VNI, các chuỗi số liên tiếp (`501`, `12345`, `width="100"`) được engine bảo đảm giữ nguyên định dạng số tự nhiên nguyên bản (literal digits), tuyệt đối không tự động biến âm thành dấu thanh hay phát xung Backspace làm xáo trộn code.
- **Khoảng Lặng Ngắt Từ Tự Động (Keystroke Idle Timeout)**:
  - Mặc định được đặt là `0ms` (tắt) trên cả 3 hệ điều hành, đảm bảo khi người dùng gõ chậm, dừng tay suy nghĩ hay tìm phím số VNI (như gõ `tieng` rồi đợi vài giây mới gõ `16` -> `tiếng`, hoặc flip word) thì bộ đệm không bao giờ bị dọn ngang chừng.
  - Người dùng Neovim được khuyến nghị sử dụng cơ chế IPC socket / module Lua chính thức (`vnchkey.lua`) để tự động chuyển trạng thái bật/tắt khi ra vào Insert mode vừa nhẹ nhàng vừa chuẩn xác tuyệt đối. Ngoài ra người dùng vẫn có thể tuỳ chỉnh thời gian timeout (từ 0 đến 2000ms) trong bảng Settings nếu muốn.
- **Module Tích Hợp Sẵn Cho Neovim (`vnchkey.lua`)**:
  - Gói sẵn module chính thức tại `data/neovim/lua/vnchkey.lua` (tự động cài vô `/usr/share/nvim/runtime/lua/vnchkey.lua`).
  - Hỗ trợ cơ chế **One-time Load Guard** (ngăn chặn duplicate load), chuyển đổi IME bất đồng bộ qua `vim.uv.spawn` với độ trễ bằng $0$, và hỗ trợ phím thoát siêu tốc `jk` / `<Esc>`.
  - Cách dùng trong Neovim khi cần:
    ```vim
    :lua require('vnchkey').load()
    ```
    *(Module sẽ kích hoạt ngay và hiển thị thông báo; nếu gõ lại lần 2 sẽ hiện cảnh báo đã nạp rồi).*

---

### 3.4. Bộ Kiểm Tra Chánh Tả, Ngữ Âm Học & Địa Danh Tây Nguyên
Tích hợp sẵn danh mục ~6.750 từ đơn gốc Tiếng Việt và bộ luật ngữ âm học nghiêm ngặt:
- **Kiểm soát âm cuối tắc (`-c, -ch, -p, -t`)**: Chỉ cho phép mang thanh Sắc hoặc thanh Nặng. Ngăn chặn các âm tiết vô lý như `bạc` mang dấu hỏi (`bacr`) hoặc dấu huyền (`bacf`).
- **Phát hiện xung đột ngữ âm**: Loại bỏ các kết hợp phụ âm / nguyên âm sai quy tắc như `ce`, `ci`, `cy`, `ge`, `giê`, `nge`, `ngi`.
- **Hỗ trợ địa danh Tây Nguyên & ký tự đặc biệt**: Mở rộng nhận diện và bỏ qua kiểm tra âm học cho các địa danh đặc thù có cụm phụ âm lạ và dấu nháy đơn (`Đắk Lắk`, `Cư M'gar`, `Ea H'leo`, `M'Đrắk`, `Đắk Nông`, `Krông Nô`, `Pleiku`...). Tích hợp bộ chuẩn hóa dấu nháy đơn (curly quotes `’` / `‘` $\to$ `'`).
- **Hỗ trợ từ ngữ Nam Bộ, Sài Gòn xưa & khẩu ngữ**: Tích hợp kho ngữ liệu trích từ bài sưu tập *"Gom góp từ ngữ của miền Nam và Saigon xưa"* (Nguyễn Cao Trường). Công nhận các âm tiết phương ngữ và biến âm tự nhiên (`dzìa`, `dzô`, `dzí`, `gởi`, `chánh`, `quởn`, `quỡn`, `số dzách`, `trớt quớt`, `cà chớn`, `bảnh tỏn`, `quỉ`, `chùm hum`, `mèn đéc ơi`...).
- **Xử lý trọn vẹn bộ ba ngoại lệ vần `uơ`**:
  - `thuo` + râu $\to$ `thuơ` (cho từ `thuở`).
  - `quon` / `quo` + râu $\to$ `quơn` / `quơ` (cho từ `quởn`, `quỡn`, `quơ tay`, chữ `u` sau `q` không bao giờ biến thành `ư`).
  - `huo` + râu $\to$ `huơ` (cho từ `huơ tay`, `huơ dao`).

---

### 3.5. Tự Động Khôi Phục Tiếng Anh (Restore English)
Kế thừa thuật toán từ điển tĩnh từ **GoNhanh**:
- Khi gõ các từ vựng lập trình hoặc Tiếng Anh phổ biến (`text`, `post`, `struct`, `general`, `filter`, `return`, `window`, `user`...):
- Khi vừa hoàn thành từ (bằng phím Space hoặc dấu câu), engine nhận diện từ không phải Tiếng Việt hoặc trùng với từ điển Tiếng Anh và lập tức khôi phục về dạng ký tự thô nguyên bản.

---

### 3.6. Chế Độ Gõ Đa Năng: Auto, Telex, VNI
- **Tự Động (Auto)**: Cho phép gõ đồng thời cả Telex (`s, f, r, x, j, w, aa, ee...`) và VNI (`1-9, 0`) trên cùng một phiên làm việc mà không cần bấm phím chuyển đổi.
- **Telex & Simple Telex**: Hỗ trợ đầy đủ kiểu gõ Telex truyền thống, tùy chọn phím `w` thành `ư` (`w2u`) và uốn nguyên âm muộn (`khong` + `o` $\to$ `không`).
- **VNI**: Hỗ trợ đầy đủ bảng quy ước VNI, đổi dấu nhanh (`gio72` + `1` $\to$ `giớ`), xóa dấu bằng phím `0` (`gio72` + `0` $\to$ `giơ`).

---

### 3.7. Bảo Toàn Chữ Hoa Thông Minh (Smart Casing) & Neo Chữ Cái Đầu (Initial Case Anchor)
- Khi gõ toàn bộ chữ hoa: `VIET` $\to$ `VIỆT` (không bị lỗi `VIệt` hoặc `ViỆt`).
- Khi gõ hoa đầu từ: `Viet` $\to$ `Việt`.
- Khi gõ hoa cụm từ: `CONG HOA` $\to$ `CỘNG HÒA`.
- **Neo chữ cái đầu (Initial Letter Case Anchor)**:
  - Tự động đồng nhất kiểu chữ hoa / thường theo đúng chữ cái đầu tiên:
    - Khi chữ cái đầu là chữ hoa: `ĐƯợc` hoặc `ĐƯỢc` $\to$ tự động chuẩn hóa thành TitleCase **`Được`**.
    - Khi chữ cái đầu là chữ thường: `đƯỢc` hoặc bất kỳ âm nào bị dính phím hoa $\to$ tự động chuẩn hóa thành chữ thường **`được`**.
    - Khi toàn bộ âm tiết là chữ hoa: `ĐƯỢC` $\to$ bảo lưu trọn vẹn toàn bộ chữ hoa **`ĐƯỢC`**.
- Khi mở rộng macro: `vn` $\to$ `Việt Nam`, nhưng nếu gõ `VN` $\to$ tự động mở rộng thành `VIỆT NAM`.

---

### 3.8. Tự Động Bỏ Dấu Đơn Trị & Gõ Dấu Thanh Tiếp Diễn (Unique Syllables Auto-Completion)
- **Tự động bỏ dấu thanh/mũ cho các từ đơn trị**:
  - Khi gõ `khong` $\to$ tự động biến thành **`không`** (bảo toàn định dạng chữ hoa: `Khong` $\to$ `Không`, `KHONG` $\to$ `KHÔNG`).
  - Khi gõ `truong` $\to$ tự động biến thành **`trương`** (`Truong` $\to$ `Trương`, `TRUONG` $\to$ `TRƯƠNG`).
  - Khi gõ `nganh` $\to$ tự động biến thành **`ngành`** (`Nganh` $\to$ `Ngành`, `NGANH` $\to$ `NGÀNH`).
- **Tự do thêm/đổi dấu thanh tiếp nối**:
  - Sau khi `khong` đã tự động thành `không`, người dùng hoàn toàn có thể gõ tiếp phím dấu thanh để chuyển đổi tự nhiên:
    - Trong Telex: gõ thêm `s` $\to$ **`khống`**, `f` $\to$ **`khồng`**, `r` $\to$ **`khổng`**, `x` $\to$ **`khỗng`**, `j` $\to$ **`khộng`**.
    - Trong VNI: gõ thêm `1` $\to$ **`khống`**, `2` $\to$ **`khồng`**, `3` $\to$ **`khổng`**, `4` $\to$ **`khỗng`**, `5` $\to$ **`khộng`**.
    - Tương tự với `truong`: gõ tiếp `f`/`2` $\to$ **`trường`**, `s`/`1` $\to$ **`trướng`**, `r`/`3` $\to$ **`trưởng`**, `j`/`5` $\to$ **`trượng`**.
    - Tương tự với `nganh`: gõ tiếp `j`/`5` $\to$ **`ngạnh`**, `r`/`3` $\to$ **`ngảnh`**.
  - Không bị tính năng khôi phục tiếng Anh (Restore English) hay bộ kiểm tra chánh tả chặn hoặc xóa nhầm.

---

### 3.9. Bảng Gõ Tắt Mở Rộng Tức Thì, Hoàn Tác 2 Bước & Hút Dấu Câu Thông Minh
- Tự động mở rộng từ viết tắt khi gõ phím phân cách (Space, dấu câu): ví dụ `vn` + phím cách $\to$ `Việt Nam `.
- **Hoàn tác Macro 2 bước thông minh bằng `Backspace` (2-Step Macro Undo)**:
  - **Nhấn `Backspace` lần 1**: Chỉ xóa khoảng trắng thừa ở cuối (`Việt Nam ` $\to$ `Việt Nam`), giữ nguyên cụm từ đã bung để bạn đặt dấu câu hoặc gõ nối tiếp mà không cần dùng phím mũi tên trái.
  - **Nhấn `Backspace` lần 2 liên tiếp**: Thu hồi toàn bộ cụm từ đã bung và phục hồi lại nguyên vẹn phím tắt gốc (`Việt Nam` $\to$ `vn`) để sửa hoặc giữ nguyên chữ `vn`.
- **Tự động Hút Dấu Câu Sát Đuôi Macro (Smart Punctuation Snapping)**:
  - Sau khi bung macro bằng phím `Space` (`vn` + `Space` $\to$ `Việt Nam `), nếu bạn gõ ngay dấu câu (`,`, `.`, `;`, `:`, `!`, `?`, `)`, `]`, `}`, `>`), bộ gõ tự động "nuốt" khoảng trắng thừa phía trước để dấu câu bám sát vào từ (`Việt Nam,`), đúng chuẩn quy tắc trình bày văn bản.
- **Biến Động Theo Múi Giờ Địa Phương (`$DATE`, `$TIME`, `$DATETIME`, `$CLIPBOARD`)**:
  - Hỗ trợ chèn nhanh ngày giờ lấy từ đồng hồ hệ thống local của người dùng và nội dung Clipboard hiện tại.
- **Tương thích Đa Định Dạng (Universal Macro Sync)**:
  - Hỗ trợ đọc trực tiếp cả 3 ký tự phân cách `=`, `:`, `;` — cho phép nạp thẳng tệp gõ tắt từ **UniKey** (`ukmacro.txt`), **EVKey**, **Ngó Sen** mà không cần chỉnh sửa định dạng.
- **Cơ chế Chống Lặp Vô Hạn (Infinite Loop Suppression)**:
  - Khi macro vừa được hoàn tác về lại từ viết tắt gốc (ví dụ `vn`), nếu người dùng nhấn tiếp phím `Space` (vì thực sự muốn gõ chữ `vn ` như trong `vn index`), engine sẽ **tạm khóa việc bung macro** cho từ đó.
- **Preset Macro Sài Gòn Xưa Tích Hợp Sẵn (`~/.config/fcitx5/conf/saigon-xua-macro.conf`)**:
  - Tích hợp sẵn bộ gõ tắt hơn 60 thành ngữ và khẩu ngữ Nam Kỳ quen thuộc: `sdz` $\to$ `số dzách`, `tq` $\to$ `trớt quớt`, `bcbc` $\to$ `bá cháy bù chét`, `crct` $\to$ `cà rịch cà tang`, `tlml` $\to$ `tào lao mía lao`, `hxq` $\to$ `hết xí quách`, `hgqn` $\to$ `hổng quởn`, `mdo` $\to$ `mèn đéc ơi`...

---

### 3.9. Bộ Bóc Tách Âm Tiết 5 Thành Phần & Dời Dấu Thanh Động (Next-Gen Syllable Parser)
Khác biệt với các bộ gõ truyền thống dựa trên chuỗi regex hoặc bảng tra cứu tĩnh, **vnchKey** trang bị module ngữ âm học chuyên sâu `syllable_parser.rs`:
- **Kiến trúc bóc tách 5 thành phần chuẩn hóa**:
  $$\text{Syllable} = \text{Onset (Phụ âm đầu)} + \text{Glide (Âm đệm)} + \text{Nucleus (Âm chính)} + \text{Coda (Âm cuối)} + \text{Tone (Thanh điệu)}$$
  - Nhận diện toàn diện các phụ âm ghép chuẩn (`ngh`, `th`, `tr`, `ph`, `nh`, `ch`...) và mở rộng cho các cụm phụ âm bản địa Tây Nguyên (`kr`, `pr`, `dr`, `đr`, `br`, `kl`, `pl`, `bl`).
  - Phân lập chính xác âm đệm `u`, `o` (`quang` $\to$ Onset `q`, Glide `u`, Nucleus `a`, Coda `ng`).
- **Dời Dấu Thanh Động (Dynamic Tone Shifting)**:
  - Khi người dùng gõ thêm nguyên âm hoặc mở rộng từ, dấu thanh tự động dịch chuyển mượt mà đến đúng vị trí âm chính mới mà không làm vỡ từ:
    - `ho` + `s` $\to$ `hó` $\xrightarrow{+\text{a}}$ `hoá` (Kiểu mới Modern) hoặc `hóa` (Kiểu cũ Classic).
    - `thuy` + `r` $\to$ `thủy` $\xrightarrow{+\text{a}}$ `thuyển` $\to$ chuyển dấu chuẩn xác.
- **Hỗ trợ Song Song Chuẩn Dấu Mới & Cổ Điển**:
  - Dấu Mới (Modern): `hóa`, `hòa`, `thủy`, `họa` (dấu đặt trên âm chính).
  - Dấu Cổ Điển (Classic): `hoá`, `hoà`, `thuỷ`, `hoạ`.

---

### 3.10. Hoán Đổi Dấu Phụ Bảo Toàn Thanh Điệu (Shape Modifier Swapping — Tinh Hoa VietC)
Chắt lọc và nâng cấp từ thuật toán hoán đổi dấu phụ của **VietC**:
- Khi người dùng muốn đổi dạng mũ nguyên âm mà từ đã có dấu thanh, không cần phải xóa đi hay bấm hoàn tác:
  - **`â ↔ ă`**: Gõ `w`/`a` (Telex) hoặc `6`/`8` (VNI) trên âm tiết có mũ/móc:
    - `ấm` + `w` $\to$ `ắm`
    - `tấn` + `w` $\to$ `tắn`
    - `mặt` + `a` $\to$ `mật`
    - `tắt` + `a` $\to$ `tất`
  - **`ô ↔ ơ`**: Gõ `w`/`o` (Telex) hoặc `6`/`7` (VNI):
    - `bốn` + `w` $\to$ `bớn`
    - `nhớ` + `o` $\to$ `nhố`
    - `lợn` + `o` $\to$ `lộn`
- **Bảo toàn 100% thanh điệu**: Toàn bộ dấu thanh (Sắc, Huyền, Hỏi, Ngã, Nặng) và âm cuối được giữ nguyên vẹn, chuyển đổi tự nhiên tức thì.

---

### 3.11. Bảo Vệ Ngữ Âm Học & Chống Gõ Sai Dấu (Phonetic Tone Protection)
- Trong ngữ âm học Tiếng Việt, các âm tiết kết thúc bằng âm cuối tắc (**Stop Codas**: `-c`, `-ch`, `-p`, `-t`, `-k`) **chỉ có thể mang thanh Sắc hoặc thanh Nặng**, tuyệt đối không thể đi cùng thanh Huyền, Hỏi, Ngã.
- **Cơ chế bảo vệ của vnchKey**:
  - Khi từ đang có âm cuối tắc và đã mang dấu thanh hợp lệ (ví dụ `được`, `mất`, `bác`, `đẹp`), nếu người dùng vô tình gõ các phím thanh không tương thích (như `f` - Huyền, `r` - Hỏi, `x` - Ngã):
    - Thay vì làm hỏng từ hoặc rớt dấu kỳ quặc (như `được` + `f` $\to$ `đườc` ở một số bộ gõ khác), engine nhận diện sự vi phạm ngữ âm học và **từ chối biến âm**.
    - Ký tự gõ sai được giữ nguyên ở dạng phím thô (`được` + `f` $\to$ `đượcf`), bảo toàn nguyên vẹn từ tiếng Việt trước đó.

---

### 3.12. Tổ Hợp Phím Tắt Thông Minh & Hệ Thống "Cứu Chữ / Cứu Dấu"
- **Lật Từ Tức Thì (`Ctrl + Shift + Z` / Tùy chỉnh tự do — Flip Word)**:
  - Đảo ngược qua lại trạng thái của từ đang gõ giữa Tiếng Việt và chuỗi phím thô (`thể` $\longleftrightarrow$ `theer`, `việt` $\longleftrightarrow$ `vieetj`).
  - **Không xung đột với phím tắt Redo của Editor**: Tổ hợp phím chỉ bắt sự kiện khi trong bộ đệm đang thực sự có từ vừa gõ (`has_active_word()`). Khi bộ đệm đang rỗng, phím tắt được trả thẳng về cho ứng dụng (VS Code, trình duyệt...) thực hiện lệnh Redo bình thường. Trên Windows, người dùng có thể tự do đổi sang `Alt + X`, `Ctrl + Alt + Z` hoặc thu phím bất kỳ.
- **Hoàn Nguyên Từ Thô Bằng `Shift + Esc` & Bảo Toàn `Esc` Cho Neovim**:
  - **Nhấn `Esc` đơn thuần**: Giữ nguyên từ tiếng Việt vừa gõ, chốt bộ đệm và truyền thẳng `Esc` cho ứng dụng — giúp lập trình viên **thoát `INSERT` mode của Neovim ngay lập tức ở lần bấm đầu tiên** mà không bị mất dấu hay biến dạng chữ!
  - **Nhấn `Shift + Esc`**: Hoàn nguyên ngay lập tức từ đang gõ về chuỗi phím thô (`tiếng` $\to$ `tieengs`) và tạm khóa tiếng Anh cho từ đó.
- **Hồi Sinh Từ Trước Sau Phím Space (`Backspace` — Reopen Word)**:
  - Lỡ gõ `tiêng ` (quên bỏ dấu sắc) rồi đã bấm `Space`? Chỉ cần bấm `Backspace` một lần để xóa dấu cách rồi gõ luôn phím `s` $\to$ từ tự động hồi sinh và biến thành `tiếng` mà không cần xóa cả chữ!
- **Cứu Chữ & Cứu Dấu Kiểu Ngó Sen (Word & Accent Rescue)**:
  - **Cứu Chữ tự động khi bật bộ gõ**: Nếu bạn quên bật bộ gõ và lỡ gõ cả câu tiếng Anh thô (`chaof banj, tieengs vieetj`), chỉ cần bấm phím chuyển sang vnchKey — toàn bộ câu vừa gõ sẽ tự động hóa phép thành **`chào bạn, tiếng việt`**! (Nhấn `Backspace` hoặc `Esc` ngay sau đó nếu muốn hoàn tác về chữ thô).
  - **Cứu Dấu khi bôi đen từ**: Dùng chuột hoặc bàn phím bôi đen một từ bất kỳ (ví dụ `tiêng`) rồi bấm phím dấu (`s` hoặc `1`), từ đang bôi đen sẽ lập tức biến thành **`tiếng`**.
- **Tự Động Bỏ Qua Ký Tự Lập Trình (Developer Code Auto-Bypass)**:
  - Tự động nhận diện và khóa tiếng Anh cho các định danh mã nguồn: biến bắt đầu bằng gạch dưới (`_variable`, `__init__`), chuỗi `snake_case` (`user_id`, `parse_string`), chuỗi truy cập thuộc tính qua dấu chấm (`db.collection.findOne`), và `camelCase` / `PascalCase` (`useState`, `ClassName`).
- **Hỗ Trợ Bảng Mã Unicode Tổ Hợp (NFD Composite Charset)**:
  - Cho phép chuyển đổi linh hoạt giữa **Unicode Dựng Sẵn (NFC)** và **Unicode Tổ Hợp (NFD)** ngay trong bảng Cài Đặt để tương thích tuyệt đối với các phần mềm đồ họa XWayland đặc thù.

---

### 3.13. Tự Động Học Thói Quen Gõ & Đưa Vào Whitelist (Adaptive Habit Learner — Log 100 MB)
Tích hợp bộ máy học thích nghi cục bộ (`habit_learner.rs`) ngay trong lõi `vnchkey-core` cho cả 3 hệ điều hành (Linux, Windows, macOS):
- **Tự động phát hiện 4 hành vi sửa lỗi của người dùng**:
  1. **Xóa bằng `Backspace` rồi gõ lại chữ không dấu (`BackspaceRetype`)**: Khi một từ bị bỏ dấu nhầm, người dùng bấm `Backspace` xóa đi rồi gõ lại từ nguyên bản.
  2. **Nhấn đúp phím dấu để khử dấu (`DoubleKeyUndo`)**: Gõ `ss`, `ff`, `rr`, `ww`... để hoàn tác dấu thanh/dấu mũ về chữ ASCII.
  3. **Bấm phím Lật Từ (`FlipToRaw`)**: Dùng `Ctrl + Shift + Z` / `Alt + X` để trả từ tiếng Việt về chuỗi phím gốc.
  4. **Bấm phím Hoàn Nguyên (`EscapeToRaw`)**: Dùng `Esc` / `Shift + Esc` để hủy bỏ dấu vừa gõ.
- **Tự động thăng cấp vào Whitelist (Auto-Promote)**:
  - Khi người dùng sửa cùng một từ **từ 2 lần trở lên**, `vnchkey-core` lập tức nạp từ đó vào bộ nhớ `EnglishAutoRestore` và ghi nối vĩnh viễn vào tệp **Từ Điển Cá Nhân** (`vnchkey-dictionary.conf` / `dictionary.conf`).
  - Từ lần tiếp theo trở đi, mỗi khi gặp lại từ đó, bộ gõ sẽ **cho pass-through nguyên bản 100%** mà bạn không bao giờ phải xóa đi sửa lại nữa.
- **Nhật ký thói quen dung lượng cao (100 MB FIFO Rotation) & Bảo mật tuyệt đối**:
  - Hỗ trợ lưu lịch sử học tập lên tới **100 MB** (`vnchkey-habit.log` / `habit.log`), tự động xoay vòng FIFO giữ lại 70% dữ liệu mới nhất khi đầy.
  - Chỉ ghi nhận các từ chữ cái đơn thuần (2–32 ký tự), tuyệt đối không ghi nhận chuỗi số, mã PIN hay mật khẩu.

---

### 3.14. Chẩn Đoán Hệ Thống `--doctor`, Cố Vấn Electron Wayland & Tự Động Bypass Game/Password
Học hỏi và phát triển từ các công cụ thực chiến trên Wayland:
- **Lệnh CLI Chẩn Đoán Nhanh (`fcitx5-vnchkey-settings --doctor` / `-d`) & Tab "Doctor" trên GUI**:
  - Kiểm tra toàn diện môi trường hiển thị (`XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`, `SWAYSOCK`), các biến môi trường `XMODIFIERS`, `GTK_IM_MODULE`, `QT_IM_MODULE`, trạng thái tiến trình daemon `fcitx5`, và sự hiện diện của các thư viện hệ thống (`fcitx5-vnchkey-addon.so`, `libfcitx5_vnchkey_ffi.so`).
- **Cố Vấn Ứng Dụng Electron / Chromium Trên Wayland (`/proc/<PID>/cmdline` Advisor)**:
  - Tự động quét các ứng dụng Electron/Chromium đang chạy (`VS Code`, `VSCodium`, `Cursor`, `Windsurf`, `Discord`, `Vesktop`, `Slack`, `Obsidian`, `Notion`, `Spotify`, `Chrome`, `Brave`...) và kiểm tra `/proc/<PID>/cmdline`.
  - Cảnh báo ngay nếu ứng dụng đang chạy qua XWayland hoặc thiếu cờ `--enable-wayland-ime` / `--ozone-platform-hint=auto`, đồng thời cung cấp câu lệnh khắc phục chính xác cho từng ứng dụng.
- **Tự Động Bypass Khi Chơi Game & Nhập Mật Khẩu / PIN (Zero-Config Bypass)**:
  - **Bảo vệ Ô Mật Khẩu & Trường Nhạy Cảm (`isSensitiveContext`)**: Tự động xả bộ đệm và ngắt biến đổi tiếng Việt khi con trỏ nằm trong các trường có cờ `CapabilityFlag::Password`, `Sensitive`, `Number`, `Digit`, `Dialable`, hoặc `Alpha`.
  - **Nhận diện Game Tự Động (`isGamingProgram`)**: Tự động chuyển sang chế độ `Bypass` (trả phím `WASD` thô tức thì không độ trễ) khi focus vào Steam (`steam_app_*`), `gamescope`, Lutris, Heroic, Bottles, PrismLauncher, Minecraft, RetroArch, RPCS3, Yuzu, Ryujinx, PCSX2, Dolphin, CS2, Dota 2, Valorant, Genshin Impact, Elden Ring, Cyberpunk 2077, Baldur's Gate 3, Overwatch, Apex Legends, Roblox/Sober, osu!, các tiến trình Unreal/Unity/Godot và các game `.exe` chạy qua Wine/Proton.

---

### 3.15. Bộ Lọc & Đặc Trị Thực Chiến Linux / Wayland (Hardened Runtime Fixes)
Được tôi luyện qua hàng loạt tình huống thực tế trên môi trường Linux Wayland và X11:
- **WPS Office (`hidesClientPreedit`)**: Bộ ứng dụng WPS Office (`wps`, `wpp`, `et`) tự vẽ giao diện nhập liệu riêng. vnchKey tự động phát hiện và áp dụng `hidesClientPreedit = true`, loại bỏ hoàn toàn hiện tượng hiển thị 2 dòng chữ chồng chéo lên nhau.
- **Google Sheets (`deleteAfterChars`)**: Xử lý đặc thù cho bảng tính Google Docs/Sheets, tối ưu hoá chuỗi sự kiện xoá/chèn ký tự (`deleteAfterChars`) để tránh làm xáo trộn ô tính hoặc mất nội dung khi đang chỉnh sửa cell.
- **GNOME Shell Wayland Proxy Guard**: Bổ sung rào chắn bảo vệ khi Fcitx5 giao tiếp với input method proxy của GNOME Shell trên Wayland, ngăn ngừa nguy cơ deadlock, đóng băng con trỏ hoặc crash tiến trình Fcitx5 khi chuyển đổi giữa các cửa sổ ứng dụng.
- **Firefox Direct Mode Polish**: Tinh chỉnh phát phím Backspace vi mô cho Firefox ở chế độ Direct, loại bỏ hiện tượng "nuốt" phím Backspace đầu tiên khi gõ nhanh trên thanh địa chỉ URL (Awesomebar) hoặc biểu mẫu web.
- **Giao diện Cài Đặt (Applications Manager) Full-Height**: Bố cục danh sách ứng dụng hiển thị tràn viền chiều cao, tích hợp nút gỡ bỏ ứng dụng căn giữa kèm icon thùng rác thanh lịch, đồng bộ hai chiều trực quan.

---

## 4. Giao Diện Cài Đặt Trực Quan (Rust + egui GUI)

Cung cấp công cụ cấu hình đồ họa native độc lập **`fcitx5-vnchkey-settings`** được viết 100% bằng Rust (`egui` / `eframe`):
- **Khởi động tức thì trong mili-giây**: Tiêu thụ cực ít tài nguyên, giao diện Dark Theme Catppuccin chuẩn mực.
- **Quản Lý Ứng Dụng Thông Minh (Applications Mode Manager)**:
  - **Bố cục 2 cột hiện đại**: Sidebar bên trái quản lý danh sách ứng dụng kèm ô tìm kiếm thời gian thực và icon danh mục (`🌐 Browser`, `💻 Terminal`, `📝 Editor`, `💬 Chat`, `🎮 Game`...).
  - **Bộ Chọn Ứng Dụng Tự Động (System App Picker)**: Tự động quét toàn bộ ứng dụng hệ thống (`/usr/share/applications`, `~/.local/share/applications`, Flatpak) và các tiến trình người dùng đang chạy (`/proc`). Bạn chỉ cần tìm kiếm và **nhấp đúp chuột (Double-click)** để thêm ngay ứng dụng vào danh sách quy tắc mà không cần nhớ hay gõ tay tên binary. Hỗ trợ tab nhập thủ công cho các script hoặc portable binary riêng biệt.
  - **Lưới Thẻ Chế Độ 1-Click (Mode Cards Grid)**: Trực quan hóa việc chuyển đổi chế độ xuất chữ cho từng ứng dụng với các thẻ bấm lớn nổi bật:
    - **`Direct (Micro-diff)`**: Gõ trực tiếp như UniKey Windows, tối ưu cho Terminal, Neovim, VS Code, WezTerm với độ trễ bằng 0.
    - **`Preedit (Gạch Chân)`**: Gom từ nguyên tử, đặc trị triệt để 100% lỗi nhảy chữ, lặp chữ trên Google Docs, Chrome, Firefox, Slack, Discord.
    - **`Bypass (OFF)`**: Tắt bộ gõ hoàn toàn, chỉ gửi phím tiếng Anh thô cho Game hoặc Terminal đặc thù.
    - **`Default Typing`**: Kế thừa chế độ mặc định toàn cục của hệ thống (Global Default).
- **Hỗ trợ toàn diện 10 Tab chuyên sâu**: **General**, **Typing**, **Applications**, **Macros**, **Dictionary**, **Keymap**, **Appearance**, **Backup**, **Doctor**, **About**.
- **Tự động đồng bộ**: Nạp lại cấu hình tức thì vào Fcitx5 ngay khi bấm Áp Dụng.

```bash
# Mở bảng cài đặt đồ họa từ terminal:
fcitx5-vnchkey-settings
# Hoặc chạy kiểm tra chẩn đoán hệ thống & Electron Wayland qua CLI:
fcitx5-vnchkey-settings --doctor
```

---

## 5. Hướng Dẫn Cài Đặt Cho Linux, macOS & Windows

### 5.1. Cài đặt trên Arch Linux bằng `install.sh` (Khuyên Dùng ⭐)
Script tự động biên dịch thư viện Rust, build module C++ addon, cài đặt vào hệ thống `/usr` và khởi động lại Fcitx5:

```bash
git clone https://github.com/no-vici/fcitx5-vnchkey.git
cd fcitx5-vnchkey
chmod +x install.sh
./install.sh
```

---

### 5.2. Đóng gói chuẩn Arch Linux bằng `makepkg`
Tệp tin `PKGBUILD` được tích hợp sẵn trong repository:

```bash
git clone https://github.com/no-vici/fcitx5-vnchkey.git
cd fcitx5-vnchkey
makepkg -si
```

---

### 5.3. Kích hoạt bộ gõ trên Linux (Fcitx5)
1. Mở công cụ cấu hình Fcitx5: `fcitx5-configtool`.
2. Tìm kiếm và thêm **vnchKey** vào danh sách bộ gõ.
3. Nhấn phím chuyển đổi bộ gõ (mặc định `Ctrl + Space` hoặc `Ctrl + Shift`) để bắt đầu sử dụng.

---

### 5.4. Build & Cài đặt trên macOS (`vnchKey.app`)
Xem hướng dẫn chi tiết từng bước và cách xử lý quyền Trợ năng (Accessibility / TCC) tại **[macos/README.md](macos/README.md)**:

```bash
git clone https://github.com/no-vici/fcitx5-vnchkey.git
cd fcitx5-vnchkey/macos
make clean && make app
killall vnchKey 2>/dev/null || true
cp -R build/vnchKey.app /Applications/
open /Applications/vnchKey.app
```
> **Lưu ý quan trọng trên macOS**: Mỗi khi build lại bản mới, bạn nhớ vào **System Settings > Privacy & Security > Accessibility**, xoá mục `vnchKey` cũ bằng nút `(-)` (hoặc tắt rồi bật lại công tắc) và mở lại `/Applications/vnchKey.app` để macOS cấp quyền bắt phím cho binary mới.

---

### 5.5. Build & Chạy trên Windows (`VnCharKey.exe`)
Phiên bản dành cho Windows sử dụng tên gọi trung tính **VnCharKey** kèm biểu tượng **Nón Lá Việt Nam**. Xem chi tiết tại **[windows/README.md](windows/README.md)**.

---

## 6. Lộ Trình Phát Triển & Tài Liệu Kỹ Thuật (Roadmap & Docs)

- 📌 **[Lộ Trình Phát Triển (Project Roadmap)](docs/ROADMAP.md)**: Theo dõi tiến độ chi tiết của các mốc phát hành **v0.1.0** (Nền tảng kiến trúc), **v0.2.0** (Chế độ Kép, Persistent Context Guard, Adaptive Micro-Pause) và kế hoạch tương lai cho **v0.3.0** (Smart Context Auto-Bypass, Wayland deep integration, AUR/PPA packaging).
- ⚖️ **[Đánh Giá Khách Quan & So Sánh Kiến Trúc (Benchmark & Landscape Analysis)](docs/BENCHMARK_AND_COMPARISON.md)**: Phân tích so sánh toàn diện và trung thực giữa vnchKey và các bộ gõ hàng đầu trên Linux (Bamboo, UniKey, Lilypad, OpenKey), chỉ ra những điểm mạnh thực sự và những hạn chế cần tiếp tục hoàn thiện.
- 🎯 **[Đánh Giá Khách Quan & Tiêu Chí Kiểm Định v0.3.0 (State of the Art Assessment)](docs/STATE_OF_THE_ART_ASSESSMENT.md)**: Báo cáo đánh giá mức độ hoàn thiện của vnchKey so với các bộ gõ đương thời và checklist kịch bản kiểm thử thực tế toàn diện trước khi phát hành v0.3.0.
- 🔬 **[Phân Tích Kỹ Thuật Cơ Chế Non-Preedit UniKey vs Wayland](docs/NON_PREEDIT_WAYLAND_ANALYSIS.md)**: Phân tích chuyên sâu về sự khác biệt kiến trúc xử lý bàn phím giữa Windows và Linux Wayland, nguyên nhân gốc rễ gây lỗi nhảy chữ trên Web DOM và giải pháp toàn diện của vnchKey.
- 📖 **[Nền Tảng Ngôn Ngữ Học & Âm Vị Học Tiếng Việt](docs/VIETNAMESE_LANGUAGE_SYSTEM.md)**: Cẩm nang chuyên sâu về cấu trúc âm tiết (Onset-Glide-Nucleus-Coda-Tone), quy tắc đặt dấu thanh (Mới vs Cũ), ràng buộc âm cuối tắc và bảng ma trận chuyển đổi vần.
- ⚡ **[Kiến Trúc Kỹ Thuật Nhập Liệu & Các Đột Phá Công Nghệ](docs/INPUT_ENGINEERING_AND_INNOVATIONS.md)**: Phân tích cơ chế phát phím Micro-Diff, Select-and-Replace cho Web/Electron, hồi sinh từ sau phím cách, Escape to Undo, nhận diện camelCase và hệ thống Macro động theo múi giờ địa phương.
- 🛡️ **[Kiến Trúc Kiểm Thử Đa Tầng & Báo Cáo Kiểm Định Chất Lượng (Multi-Tier Testing & ASan Guide)](docs/MULTI_TIER_TESTING_AND_VERIFICATION.md)**: Hướng dẫn toàn diện về hệ thống kiểm thử 3 tầng (Property-Based Fuzzing, Mô phỏng bẫy tiếng Anh vs bảo vệ tiếng Việt, kiểm định rò rỉ bộ nhớ C++ AddressSanitizer/UBSan qua FFI) dành cho lập trình viên.

---

## 7. Ý Nghĩa Biểu Tượng & Di Sản Tự Do

Huy hiệu của **vnchKey** kết hợp hai yếu tố văn hóa và công nghệ:
1. **Khối Keycap Cơ 3D Cờ Vàng Ba Sọc Đỏ (Hoàng Long & Ba Sọc Đỏ)**:
   - Thiết kế keycap cơ 3D nổi khối với nền trong suốt (transparent background), 4 góc vát cong tự nhiên chuẩn công thái học (ergonomic keycap profile). Khi đặt lên thanh Waybar, Swaybar hay khay hệ thống KDE/GNOME (16px, 22px, 24px đến HiDPI 256px), biểu tượng hiển thị nổi bật, sáng rõ và sắc nét tuyệt đối.
   - Nền vàng rực rỡ và ba sọc đỏ tượng trưng cho ba miền Bắc — Trung — Nam gắn bó keo sơn, gắn liền với di sản tinh thần **Tự Do — Dân Chủ — Phú Cường**.
2. **Ký Tự `V` Khắc Chìm Trung Tâm (Việt Nam & V-Key)**:
   - Ký tự `V` sắc sảo nằm ngay tâm phím mang ý nghĩa kép: đại diện cho chữ cái thiêng liêng **Việt Nam**, phím tắt bộ gõ **Vietnamese Key**, và biểu tượng chiến thắng tinh thần Tự Do.

---

## 8. Lời Cảm Ơn / Ghi Chú Phát Triển

- Dự án được xây dựng và tối ưu hoá với sự đồng hành của **Google Gemini** trong vai trò AI pair programmer (hỗ trợ phân tích kiến trúc, gỡ lỗi và kiểm thử tự động).
- Chân thành cảm các developers của dự án [Fcitx5](https://github.com/fcitx/fcitx5) và các bộ gõ Tiếng Việt đi trước.

---

## 9. Bản Quyền & Giấy Phép

- Dự án được phát hành dưới hình thức **Mã Nguồn Mở Tự Do 100%** theo các điều khoản của [Giấy Phép GNU General Public License v3.0 (GPLv3)](LICENSE).
- Mọi đóng góp, báo cáo lỗi và thảo luận xin gởi về: [GitHub Repository](https://github.com/no-vici/fcitx5-vnchkey).
