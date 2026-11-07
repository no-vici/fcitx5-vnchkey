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
   - [3.12. Phím Tắt Lật Từ Tức Thì (Flip Word — Ctrl + Shift + Z)](#312-phím-tắt-lật-từ-tức-thì-flip-word--ctrl--shift--z)
   - [3.13. Bộ Lọc & Đặc Trị Thực Chiến Linux / Wayland (Hardened Runtime Fixes)](#313-bộ-lọc--đặc-trị-thực-chiến-linux--wayland-hardened-runtime-fixes)
4. [Giao Diện Cài Đặt Trực Quan (Rust + egui GUI)](#4-giao-diện-cài-đặt-trực-quan-rust--egui-gui)
5. [Hướng Dẫn Cài Đặt Cho Arch Linux](#5-hướng-dẫn-cài-đặt-cho-arch-linux)
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
│   ├── vnchkey-core/                # [Lõi Rust] Xử lý âm vị học & biến đổi dấu Tiếng Việt
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs               # Module exports
│   │   │   ├── engine.rs            # Bộ điều phối trung tâm (Syllable State Machine)
│   │   │   ├── telex.rs             # Biến đổi Telex, quy tắc âm tiết & tách âm tiết cuối
│   │   │   ├── vni.rs               # Biến đổi VNI đa thứ tự phím (1-9, 0 xóa dấu)
│   │   │   ├── syllable_parser.rs   # [Mới] Bộ bóc tách âm tiết 5 thành phần (Onset-Glide-Nucleus-Coda-Tone)
│   │   │   ├── auto_restore.rs      # Trie từ điển tiếng Anh kỹ thuật (GoNhanh)
│   │   │   ├── casing.rs            # Theo dõi và bảo toàn kiểu chữ hoa/thường
│   │   │   ├── macros.rs            # Bảng từ viết tắt (Gõ tắt) & hoàn tác
│   │   │   └── vietnamese_dict.rs   # Bộ từ điển Chánh Tả ~6.750 từ gốc tiếng Việt
│   │   └── tests/
│   │       └── vietc_regression_tests.rs # [Mới] Bộ kiểm thử hồi quy đối sánh từ VietC
│   │
│   ├── fcitx5-vnchkey-ffi/          # [Tầng C-ABI] Giao tiếp giữa Rust core và C++
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs               # C-compatible FFI exported functions
│   │
│   └── vnchkey-settings/            # [Giao diện Cài Đặt] Viết bằng Rust + egui/eframe
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs              # Điểm khởi chạy ứng dụng fcitx5-vnchkey-settings
│           ├── app.rs               # Giao diện UI với 10 tab quản lý trực quan
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
  - Tối ưu tuyệt đối cho độ trễ bằng $0$, mang lại cảm giác gõ mộc tự nhiên trên các editor chuyên nghiệp (Neovim, VS Code, Cursor, Zed, Alacritty, Kitty).
- **Chế độ Gõ Gạch Chân (Preedit Mode - Khuyên dùng cho Web/Google Docs/Chat)**:
  - Khắc phục tận gốc nguyên nhân gây lặp chữ: **Tuyệt đối không phát phím Backspace giả lập ra ứng dụng bên ngoài**.
  - Từ đang soạn được hiển thị bằng nét gạch chân nội dòng (`setClientPreedit`) và được cam kết nguyên tử một lần duy nhất (`commitString`) khi người dùng bấm phím cách (Space), Enter hoặc dấu phân cách.
  - **Khung chữ nổi (Floating Preedit Tooltip)**: Hiển thị từ đang gõ trong một ô nổi nhỏ ngay sát con trỏ chuột cho các ứng dụng tự vẽ Canvas như Google Docs (nơi trình duyệt không vẽ được nét gạch chân nội dòng). Đi kèm tùy chọn bật/tắt nhanh trên Menu hoặc Cài Đặt nếu người dùng muốn tránh phân tâm.
  - **Tự động nhớ chế độ theo từng ứng dụng (Per-App Mode Memory)**: Tự động kích hoạt Preedit cho các trình duyệt web (`Chrome`, `Firefox`, `Brave`, `Edge`) và ứng dụng chat Electron (`Slack`, `Discord`, `Telegram`), đồng thời tự bật Direct khi quay lại Terminal/IDE.

---

### 3.3. Tách Âm Tiết Cuối Tối Ưu Cho Neovim / WezTerm / Vim
Một vấn đề cố hữu của các bộ gõ trên terminal emulator là các phím lệnh Normal mode (như `ci"`, `cw`, `i`, `a`, `jk`) hoặc các ký tự đứng sát trước con trỏ dễ bị kẹt trong bộ đệm từ, phá vỡ việc bỏ dấu VNI:
- **Giải pháp của vnchKey**: Tích hợp hàm `extract_last_syllable_index()`:
  - Khi gõ chuỗi phím số VNI (như `biet61`), engine tự động bóc tách âm tiết Tiếng Việt hợp lệ đứng cuối cùng (`biet`) để áp dụng dấu thành `biết`.
  - Giữ nguyên vẹn văn bản đứng trước, gõ mượt mà ngay cả khi con trỏ đứng sát dấu ngoặc kép `""` hoặc sau các lệnh vim.
  - Tự động khóa khôi phục Tiếng Anh đối với từ gõ phím số VNI khi thoát Insert mode bằng phím tắt `jk`.

---

### 3.4. Bộ Kiểm Tra Chánh Tả, Ngữ Âm Học & Địa Danh Tây Nguyên
Tích hợp sẵn danh mục ~6.750 từ đơn gốc Tiếng Việt và bộ luật ngữ âm học nghiêm ngặt:
- **Kiểm soát âm cuối tắc (`-c, -ch, -p, -t`)**: Chỉ cho phép mang thanh Sắc hoặc thanh Nặng. Ngăn chặn các âm tiết vô lý như `bạc` mang dấu hỏi (`bacr`) hoặc dấu huyền (`bacf`).
- **Phát hiện xung đột ngữ âm**: Loại bỏ các kết hợp phụ âm / nguyên âm sai quy tắc như `ce`, `ci`, `cy`, `ge`, `giê`, `nge`, `ngi`.
- **Hỗ trợ địa danh Tây Nguyên & ký tự đặc biệt**: Mở rộng nhận diện và bỏ qua kiểm tra âm học cho các địa danh đặc thù có cụm phụ âm lạ và dấu nháy đơn (`Đắk Lắk`, `Cư M'gar`, `Ea H'leo`, `M'Đrắk`, `Đắk Nông`, `Krông Nô`, `Pleiku`...). Tích hợp bộ chuẩn hóa dấu nháy đơn (curly quotes `’` / `‘` $\to$ `'`).
- **Hỗ trợ từ ngữ Nam Bộ & khẩu ngữ**: Công nhận các âm tiết phương ngữ và biến âm tự nhiên (`dzìa`, `dzô`, `dzí`, `gởi`, `chánh`...).

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

### 3.7. Bảo Toàn Chữ Hoa Thông Minh (Smart Casing)
- Khi gõ toàn bộ chữ hoa: `VIET` $\to$ `VIỆT` (không bị lỗi `VIệt` hoặc `ViỆt`).
- Khi gõ hoa đầu từ: `Viet` $\to$ `Việt`.
- Khi gõ hoa cụm từ: `CONG HOA` $\to$ `CỘNG HÒA`.
- Khi mở rộng macro: `vn` $\to$ `Việt Nam`, nhưng nếu gõ `VN` $\to$ tự động mở rộng thành `VIỆT NAM`.

---

### 3.8. Bảng Gõ Tắt Mở Rộng Tức Thì & Hoàn Tác Thông Minh (Macro Expansion & Smart Undo)
- Tự động mở rộng từ viết tắt khi gõ phím phân cách (Space, dấu câu): ví dụ `vn` + phím cách $\to$ `Việt Nam `.
- **Hoàn tác Macro một chạm (One-Step Macro Revert on Backspace)**:
  - Nếu vừa bung macro mà người dùng nhấn ngay phím `Backspace`, engine tự động tính toán và gửi số lượng Backspace chính xác để thu hồi toàn bộ cụm từ đã bung cùng dấu cách, phục hồi lại nguyên vẹn chuỗi viết tắt gốc (`Việt Nam ` $\xrightarrow{\text{Backspace}}$ `vn`).
- **Cơ chế Chống Lặp Vô Hạn (Infinite Loop Suppression)**:
  - Điểm cải tiến vượt trội so với các bộ gõ truyền thống: Khi macro vừa được hoàn tác về lại từ viết tắt gốc (ví dụ `vn`), nếu người dùng nhấn tiếp phím `Space` (vì thực sự muốn gõ chữ `vn ` như trong `vn index`), engine sẽ **tạm khóa việc bung macro** cho từ đó. Phím Space đi qua tự nhiên mà không bao giờ bị bẫy trong vòng lặp vô tận (`vn` $\to$ `Việt Nam ` $\to$ `vn` $\to$ `Việt Nam `).
  - Trạng thái tạm khóa tự động giải phóng ngay khi chuyển sang từ tiếp theo, bảo đảm các lần gõ tắt sau đó vẫn hoạt động trơn tru 100%.

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

### 3.12. Phím Tắt Lật Từ Tức Thì (Flip Word — `Ctrl + Shift + Z`)
- Khi đang gõ mã lệnh hoặc tài liệu song ngữ, việc vô tình gõ một từ tiếng Anh thành tiếng Việt (hoặc ngược lại) xảy ra rất thường xuyên.
- **Tính năng Flip Word**:
  - Nhấn tổ hợp phím **`Ctrl + Shift + Z`** (hoặc phím tắt cấu hình trong Settings) để đảo ngược trạng thái của từ vừa gõ:
    - `thể` $\longleftrightarrow$ `theer`
    - `công` $\longleftrightarrow$ `coong`
    - `việt` $\longleftrightarrow$ `vieetj`
  - Giúp lập trình viên và người viết lách sửa nhanh lỗi gõ nhầm mà không cần nhấn Backspace nhiều lần rồi gõ lại từ đầu.

---

### 3.13. Bộ Lọc & Đặc Trị Thực Chiến Linux / Wayland (Hardened Runtime Fixes)
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
- **Hỗ trợ toàn diện 9 Tab chuyên sâu**: **General**, **Typing**, **Applications**, **Macros**, **Dictionary**, **Keymap**, **Appearance**, **Backup**, **About**.
- **Tự động đồng bộ**: Nạp lại cấu hình tức thì vào Fcitx5 ngay khi bấm Áp Dụng.

```bash
# Mở bảng cài đặt từ terminal:
fcitx5-vnchkey-settings
# Hoặc lệnh tắt:
vnchkey-settings
```

---

## 5. Hướng Dẫn Cài Đặt Cho Arch Linux

### 5.1. Cài đặt tự động bằng `install.sh` (Khuyên Dùng ⭐)
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

### 5.3. Kích hoạt bộ gõ
1. Mở công cụ cấu hình Fcitx5: `fcitx5-configtool`.
2. Tìm kiếm và thêm **vnchKey** vào danh sách bộ gõ.
3. Nhấn phím chuyển đổi bộ gõ (mặc định `Ctrl + Space` hoặc `Ctrl + Shift`) để bắt đầu sử dụng.

---

## 6. Lộ Trình Phát Triển & Tài Liệu Kỹ Thuật (Roadmap & Docs)

- 📌 **[Lộ Trình Phát Triển (Project Roadmap)](docs/ROADMAP.md)**: Theo dõi tiến độ chi tiết của các mốc phát hành **v0.1.0** (Nền tảng kiến trúc), **v0.2.0** (Chế độ Kép, Persistent Context Guard, Adaptive Micro-Pause) và kế hoạch tương lai cho **v0.3.0** (Smart Context Auto-Bypass, Wayland deep integration, AUR/PPA packaging).
- ⚖️ **[Đánh Giá Khách Quan & So Sánh Kiến Trúc (Benchmark & Landscape Analysis)](docs/BENCHMARK_AND_COMPARISON.md)**: Phân tích so sánh toàn diện và trung thực giữa vnchKey và các bộ gõ hàng đầu trên Linux (Bamboo, UniKey, Lilypad, OpenKey), chỉ ra những điểm mạnh thực sự và những hạn chế cần tiếp tục hoàn thiện.
- 🎯 **[Đánh Giá Khách Quan & Tiêu Chí Kiểm Định v0.3.0 (State of the Art Assessment)](docs/STATE_OF_THE_ART_ASSESSMENT.md)**: Báo cáo đánh giá mức độ hoàn thiện của vnchKey so với các bộ gõ đương thời và checklist kịch bản kiểm thử thực tế toàn diện trước khi phát hành v0.3.0.
- 🔬 **[Phân Tích Kỹ Thuật Cơ Chế Non-Preedit UniKey vs Wayland](docs/NON_PREEDIT_WAYLAND_ANALYSIS.md)**: Phân tích chuyên sâu về sự khác biệt kiến trúc xử lý bàn phím giữa Windows và Linux Wayland, nguyên nhân gốc rễ gây lỗi nhảy chữ trên Web DOM và giải pháp toàn diện của vnchKey.

---

## 7. Ý Nghĩa Biểu Tượng & Di Sản Tự Do

Huy hiệu của **vnchKey** kết hợp hai yếu tố văn hóa và công nghệ:
1. **Nền Tròn Cờ Vàng Ba Sọc Đỏ (Hoàng Long & Ba Sọc Đỏ)**:
   - Màu vàng biểu trưng cho đất nước, con người và sự phồn vinh của dân tộc Việt Nam.
   - Ba sọc đỏ tượng trưng cho ba miền Bắc — Trung — Nam gắn bó keo sơn, gắn liền với di sản tinh thần **Tự Do — Dân Chủ — Phú Cường**.
2. **Khối Phím Cơ 3D Dập Nổi Chữ `vn`**:
   - Mang thông điệp: *Mỗi phím bấm là một tiếng nói tự do, độc lập và bảo tồn trọn vẹn ngôn ngữ mẹ đẻ trong thời đại số.*

---

## 8. Lời Cảm Ơn / Ghi Chú Phát Triển

- Dự án được xây dựng và tối ưu hoá với sự đồng hành của **Google Gemini** trong vai trò AI pair programmer (hỗ trợ phân tích kiến trúc, gỡ lỗi và kiểm thử tự động).
- Chân thành cảm các developers của dự án [Fcitx5](https://github.com/fcitx/fcitx5) và các bộ gõ Tiếng Việt đi trước.

---

## 9. Bản Quyền & Giấy Phép

- Dự án được phát hành dưới hình thức **Mã Nguồn Mở Tự Do 100%** theo các điều khoản của [Giấy Phép GNU General Public License v3.0 (GPLv3)](LICENSE).
- Mọi đóng góp, báo cáo lỗi và thảo luận xin gởi về: [GitHub Repository](https://github.com/no-vici/fcitx5-vnchkey).
