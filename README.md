# vnchKey (Tự Do Edition)

<div align="center">

<img src="assets/logo.png" alt="vnchKey Official Logo" width="128" height="128" />

**Bộ Gõ Tiếng Việt Hiệu Năng Cao, Tuyệt Đối An Toàn Bộ Nhớ Cho Linux (Fcitx5 / Wayland / X11)**  
*Được viết lại hoàn toàn bằng 100% Safe Rust từ dự án fcitx5-lilypad.*  
*Hội tụ tinh hoa công nghệ từ **OpenViet IME**, **GoNhanh** và **XKey**.*  
*Mang đậm tinh thần Tự Do và Di sản Cờ Vàng Ba Sọc Đỏ.*

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-GNU%20GPL%203.0-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20(Wayland%20%2F%20X11)-red.svg?style=flat-square&logo=linux)](https://archlinux.org)
[![Memory Safety](https://img.shields.io/badge/Memory%20Safety-100%25%20Safe%20Rust-brightgreen.svg?style=flat-square)](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html)
[![Zero Telemetry](https://img.shields.io/badge/Telemetry-Zero%20(100%25%20Offline)-success.svg?style=flat-square)](#)

</div>

---

## 📖 Mục Lục Chi Tiết

1. [Tổng Quan & Lý Do Viết Lại Bằng Rust](#1-tổng-quan--lý-do-viết-lại-bằng-rust)
2. [Kiến Trúc Đa Crate (Cargo Multi-Crate Workspace)](#2-kiến-trúc-đa-crate-cargo-multi-crate-workspace)
3. [Cơ Chế Hoạt Động Chuyên Sâu (Deep Technical Mechanisms)](#3-cơ-chế-hoạt-động-chuyên-sâu-deep-technical-mechanisms)
   - [3.1. Direct Clean Commit 100% qua Linux Kernel `/dev/uinput`](#31-direct-clean-commit-100-qua-linux-kernel-devuinput)
   - [3.2. Giao Thức Sentinel Barrier (N+1) & Chống Race Condition](#32-giao-thức-sentinel-barrier-n1--chống-race-condition)
   - [3.3. Cảm Biến Nhịp Gõ IKI (Real-Time Inter-Keystroke Interval EMA)](#33-cảm-biến-nhịp-gõ-iki-real-time-inter-keystroke-interval-ema)
   - [3.4. Chế Độ Tự Động Thông Minh (Adaptive Auto Mode - OpenViet)](#34-chế-độ-tự-động-thông-minh-adaptive-auto-mode---openviet)
   - [3.5. Bộ Lọc Ngữ Cảnh Bảo Vệ URL, Email & Mã Nguồn (Code Guard)](#35-bộ-lọc-ngữ-cảnh-bảo-vệ-url-email--mã-nguồn-code-guard)
   - [3.6. Cơ Chế Xóa Dấu Không Phá Hủy Bằng Phím `z` (Telex) & `0` (VNI)](#36-cơ-chế-xóa-dấu-không-phá-hủy-bằng-phím-z-telex--0-vni)
   - [3.7. Kiểm Soát Âm Cuối Đóng Theo Âm Vị Học (-c, -ch, -p, -t)](#37-kiểm-soát-âm-cuối-đóng-theo-âm-vị-học--c--ch--p--t)
   - [3.8. Xử Lý Nguyên Âm Đôi Phức Tạp (uê, ươ, ưu, uơ) & Uốn Nguyên Âm Muộn](#38-xử-lý-nguyên-âm-đôi-phức-tạp-uê-ươ-ưu-uơ--uốn-nguyên-âm-muộn)
   - [3.9. Phục Hồi Chuỗi Phím Gốc Bằng Phím `ESC` (Instant ESC Revert)](#39-phục-hồi-chuỗi-phím-gốc-bằng-phím-esc-instant-esc-revert)
   - [3.10. Hoàn Tác Gõ Tắt Tức Thì Bằng Phím `Backspace` (Macro Undo)](#310-hoàn-tác-gõ-tắt-tức-thì-bằng-phím-backspace-macro-undo)
   - [3.11. Khôi Phục Tiếng Anh Tự Động (GoNhanh English Auto-Restore)](#311-khôi-phục-tiếng-anh-tự-động-gonhanh-english-auto-restore)
   - [3.12. Bảo Toàn Trạng Thái Chữ Hoa Theo Ký Tự (XKey Smart Casing)](#312-bảo-toàn-trạng-thái-chữ-hoa-theo-ký-tự-xkey-smart-casing)
   - [3.13. Quản Lý Từ Ngoại Lệ Của Người Dùng (Custom Whitelist)](#313-quản-lý-từ-ngoại-lệ-của-người-dùng-custom-whitelist)
   - [3.14. Giới Hạn Thiết Kế: Loại Trừ Gợi Ý Từ & Học Thói Quen](#314-giới-hạn-thiết-kế-loại-trừ-gợi-ý-từ--học-thói-quen)
4. [Bảng So Sánh Kiến Trúc Toàn Diện](#4-bảng-so-sánh-kiến-trúc-toàn-diện)
5. [Hướng Dẫn Cài Đặt Chi Tiết Cho Arch Linux](#5-hướng-dẫn-cài-đặt-chi-tiết-cho-arch-linux)
   - [5.1. Cài đặt các công cụ biên dịch cơ sở](#51-cài-đặt-các-công-cụ-biên-dịch-cơ-sở)
   - [5.2. Thiết lập quyền kernel Linux `/dev/uinput` (Không cần root)](#52-thiết-lập-quyền-kernel-linux-devuinput-không-cần-root)
   - [5.3. Biên dịch và đóng gói (PKGBUILD hoặc Trực tiếp Cargo)](#53-biên-dịch-và-đóng-gói-pkgbuild-hoặc-trực-tiếp-cargo)
6. [Cấu Hình Môi Trường Wayland & Các Compositor](#6-cấu-hình-môi-trường-wayland--các-compositor)
   - [6.1. Biến môi trường hệ thống (`environment.d`)](#61-biến-môi-trường-hệ-thống-environmentd)
   - [6.2. Cấu hình cho Hyprland](#62-cấu-hình-cho-hyprland)
   - [6.3. Cấu hình cho Sway & Wayfire](#63-cấu-hình-cho-sway--wayfire)
   - [6.4. Cấu hình cho KDE Plasma 6 & GNOME 47 (Wayland)](#64-cấu-hình-cho-kde-plasma-6--gnome-47-wayland)
7. [Quản Lý Tiến Trình Daemon Qua Systemd User](#7-quản-lý-tiến-trình-daemon-qua-systemd-user)
8. [Tài Liệu Dòng Lệnh (CLI Options & Configuration)](#8-tài-liệu-dòng-lệnh-cli-options--configuration)
9. [Chẩn Đoán & Khắc Phục Lỗi Toàn Diện (Troubleshooting Matrix)](#9-chẩn-đoán--khắc-phục-lỗi-toàn-diện-troubleshooting-matrix)
10. [Ý Nghĩa Biểu Tượng & Di Sản Tự Do](#10-ý-nghĩa-biểu-tượng--di-sản-tự-do)

---

## 1. Tổng Quan & Lý Do Viết Lại Bằng Rust

Dự án gốc `fcitx5-lilypad` viết bằng C++ đã tiên phong sử dụng `/dev/uinput` để giải quyết hai vấn đề nhức nhối nhất của bộ gõ tiếng Việt trên Linux:
1. **Lỗi Pre-edit String**: Gạch chân nhấp nháy làm gãy menu IntelliSense của VS Code, Cursor, Zed, IntelliJ.
2. **Lỗi Nuốt Chữ & Nhảy Con Trỏ**: Hiện tượng lag bất đồng bộ trên các ứng dụng Electron / Chromium (Discord, Slack, Notion, Chrome, Spotify).

Tuy nhiên, mã nguồn C++ ban đầu tồn tại nhiều hạn chế:
- **Nguy cơ an toàn bộ nhớ**: Xử lý con trỏ thô, mảng ký tự tĩnh dễ tiềm ẩn lỗi tràn bộ đệm (buffer overflow) hoặc segfaults khi người dùng gõ chuỗi phím rác dồn dập.
- **Rò rỉ bộ nhớ (Memory Leaks)**: Khi chạy nền liên tục hàng tuần dưới dạng daemon hệ thống, các cấu trúc cấp phát động không được thu hồi triệt để.
- **Độ trễ tĩnh (Fixed Delays)**: Sử dụng các hàm `usleep()` ngắt quãng cứng ngắc khiến tốc độ gõ bị giới hạn, gây cảm giác nặng nề khi gõ nhanh (>100 WPM).

**vnchKey** được kiến trúc lại hoàn toàn từ đầu bằng **100% Safe Rust**:
- **`#![forbid(unsafe_code)]`**: Toàn bộ crate xử lý ngôn ngữ `vnchkey-core` cấm tuyệt đối mọi khối `unsafe`, đảm bảo an toàn bộ nhớ ở cấp độ ngôn ngữ biên dịch.
- **Zero Heap Allocations Trong Chu Kỳ Gõ (Hot Path)**: Quá trình phân tích âm tiết, biến đổi dấu và kiểm tra quy tắc ngữ âm được thực hiện trên cấu trúc mảng tĩnh kích thước nhỏ hoặc stack, không cấp phát heap động (`no_std` ready).
- **Hội Tụ Tinh Hoa**:
  - Tích hợp các quy tắc gõ hiện đại nhất từ **OpenViet IME** (Firefox Plugin).
  - Tích hợp thuật toán tự động khôi phục từ tiếng Anh khi gõ code từ **GoNhanh** (Rust).
  - Tích hợp mô hình bảo toàn chữ hoa thông minh (Smart Casing) từ **XKey** (Swift/macOS).

---

## 2. Kiến Trúc Đa Crate (Cargo Multi-Crate Workspace)

Hệ thống được tổ chức theo kiến trúc phân tầng độc lập (Modular Monorepo):

```
fcitx5-vnchkey/
├── Cargo.toml                       # Workspace Root Manifest (LTO, strip, panic=abort)
├── crates/
│   ├── vnchkey-core/                # [Tầng 1] Lõi phân tích âm vị học thuần Rust (Safe 100%)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs               # Public APIs & trait Definitions
│   │       ├── types.rs             # EngineConfig, EngineOutput, SyllableBreakdown
│   │       ├── vietnamese.rs        # Ngữ âm học tiếng Việt: nguyên âm, thanh điệu, âm đóng
│   │       ├── telex.rs             # Biến đổi Telex (aa, ee, oo, dd, aw, ow, uw, s, f, r, x, j, z)
│   │       ├── vni.rs               # Biến đổi VNI đa thứ tự phím (1-9, 0 xóa dấu)
│   │       ├── context_guard.rs     # Nhận diện URL, email, mã nguồn (Code Guard)
│   │       ├── whitelist.rs         # Từ điển ngoại lệ người dùng tùy chỉnh
│   │       ├── auto_restore.rs      # Trie từ điển tiếng Anh kỹ thuật (GoNhanh)
│   │       ├── casing.rs            # Vector bảo toàn chữ hoa (XKey Smart Casing)
│   │       ├── macros.rs            # Cây tiền tố mở rộng gõ tắt (Macro Prefix Tree)
│   │       ├── iki.rs               # Cảm biến nhịp gõ thời gian thực (EMA Sensor)
│   │       └── sentinel.rs          # Rào cản đồng bộ hóa Sentinel Barrier (N+1)
│   │
│   ├── vnchkey-uinput/              # [Tầng 2] Trình phát sự kiện Kernel Linux /dev/uinput
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── device.rs            # ioctl(UI_DEV_CREATE), EV_KEY, SYN_REPORT
│   │       └── keymap.rs            # Chuyển đổi Unicode sang Linux Scancodes (evdev)
│   │
│   ├── vnchkey-daemon/              # [Tầng 3] Tiến trình daemon chạy nền độc lập
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs              # CLI parser, vòng lặp sự kiện bất đồng bộ Tokio
│   │       ├── ipc.rs               # Unix Domain Socket cho IPC cấu hình runtime
│   │       └── config.rs            # Tải cấu hình từ ~/.config/vnchkey/config.toml
│   │
│   └── fcitx5-vnchkey-ffi/          # [Tầng 4] Cầu nối C-ABI cho framework Fcitx5
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs               # fcitx_input_method_engine_v2 callbacks
│           └── factory.rs           # Module loader đăng ký vào Fcitx5
└── README.md
```

### Luồng Xử Lý Sự Kiện (Data Flow):

```
[Bàn phím vật lý] 
       │
       ▼
 [Linux evdev] 
       │
       ├─────────────────────────────────┐
       ▼                                 ▼
[Fcitx5 Framework Engine]     [vnchkey-daemon (Stand-alone)]
       │                                 │
       └──────────────┬──────────────────┘
                      ▼
            [vnchkey-core Lõi Rust]
       ┌──────────────────────────────┐
       │ 1. Context Guard (URL/Code)  │
       │ 2. Whitelist Check           │
       │ 3. Adaptive Mode (Telex/VNI) │
       │ 4. Vietnamese Phonology Core │
       │ 5. IKI Sensor (EMA Speed)    │
       │ 6. Smart Casing & Macros     │
       └──────────────┬───────────────┘
                      ▼
         [Sentinel Barrier Protocol]
                      ▼
         [Linux Kernel /dev/uinput]
                      ▼
  [Ứng dụng: VS Code / Chrome / Terminal / Discord]
```

---

## 3. Cơ Chế Hoạt Động Chuyên Sâu (Deep Technical Mechanisms)

### 3.1. Direct Clean Commit 100% qua Linux Kernel `/dev/uinput`
Các bộ gõ truyền thống trên Linux (Unikey, Bamboo, IBus-Bamboo cũ) sử dụng cơ chế *Pre-edit String* (chuỗi gạch chân nhấp nháy dưới con trỏ). Mỗi lần người dùng gõ một phím, bộ gõ gửi thông điệp IBus/Fcitx yêu cầu ứng dụng vẽ chuỗi tạm.
- **Hậu quả trong môi trường hiện đại**: Các ứng dụng Web/Electron (VS Code, Cursor, Discord, Slack, Chrome) có vòng đời DOM phức tạp. Mỗi chuỗi pre-edit kích hoạt đồng thời `compositionstart`, `compositionupdate`, `input` và `selectionchange`. Điều này làm lu mờ gợi ý autocomplete, làm con trỏ nhảy giật và gây lỗi mất chữ khi gõ nhanh.
- **Giải pháp Direct Clean Commit của vnchKey**: 
  1. vnchKey mở trực tiếp thiết bị kernel `/dev/uinput` thông qua `ioctl(fd, UI_DEV_CREATE, &uinput_setup)`.
  2. Khi người dùng nhập một phím, vnchKey phát sự kiện bàn phím phần cứng thật (`EV_KEY`, `KEY_A`, `SYN_REPORT`).
  3. Khi cần sửa dấu (ví dụ: `a` + `s` $\to$ `á`), engine phát chính xác 1 sự kiện phần cứng `KEY_BACKSPACE`, sau đó đẩy trực tiếp ký tự `á` bằng Unicode virtual stroke.
  4. Ứng dụng nhìn nhận toàn bộ quá trình như người dùng đang gõ trên một bàn phím vật lý thực thụ, hoàn toàn không có vệt gạch chân, không gây gián đoạn DOM.

---

### 3.2. Giao Thức Sentinel Barrier (N+1) & Chống Race Condition
Khi gõ với tốc độ cao (>100 WPM), các trình duyệt web và ứng dụng Electron xử lý hàng đợi sự kiện một cách bất đồng bộ qua nhiều tiến trình (Renderer Process $\leftrightarrow$ Browser Process $\leftrightarrow$ GPU Process). Nếu lệnh Backspace và ký tự mới được gửi quá sát nhau, trình duyệt có thể áp dụng ký tự mới *trước khi* xử lý Backspace, gây ra lỗi nhân đôi chữ (ví dụ: gõ `toans` bị biến thành `toaán`).

**Giao Thức Sentinel Barrier (N+1) của vnchKey**:
1. **Pha 1 (Arming Barrier)**: Khi phát hiện cần thay thế một từ $N$ ký tự, engine đóng rào cản ngăn chặn sự kiện phím kế tiếp.
2. **Pha 2 (Backspaces Burst)**: Bắn nhanh $N$ mã quét `KEY_BACKSPACE`.
3. **Pha 3 (Sentinel Fence & Watchdog)**: Chèn một tín hiệu đồng bộ hóa Sentinel. Engine thiết lập một Watchdog Timer an toàn (200ms). Nếu ứng dụng phản hồi kịp thời hoặc sau khoảng thời gian thích ứng của cảm biến IKI, rào cản được mở.
4. **Pha 4 (Release Commit)**: Đẩy toàn bộ chuỗi Unicode đã biến đổi chuẩn xác vào hệ thống.

---

### 3.3. Cảm Biến Nhịp Gõ IKI (Real-Time Inter-Keystroke Interval EMA)
Không sử dụng độ trễ tĩnh (static delay) gây chậm chạp vô cớ, vnchKey đo lường thời gian thực khoảng cách giữa các nhịp gõ phím liên tiếp (Inter-Keystroke Interval - IKI) bằng giải thuật Trung Bình Động Số Mũ (Exponential Moving Average):

$$\Delta_{\text{EMA}} = \alpha \cdot \Delta_{\text{mới}} + (1 - \alpha) \cdot \Delta_{\text{cũ}} \quad (\text{với } \alpha = 0.25)$$

- **Trên Terminal / Alacritty / Neovim**: Tốc độ xử lý của thiết bị đầu cuối cực nhanh $\to$ IKI Sensor giữ độ trễ ở mức tối thiểu $1.5\text{ms}$ (siêu tốc, không có cảm giác trễ).
- **Trên VS Code / Chrome khi gõ dồn dập**: Khi nhịp gõ tăng vọt và hệ số trễ DOM tăng, IKI tự động điều chỉnh độ trễ commit lên $4\text{ms} - 8\text{ms}$ nhằm triệt tiêu 100% hiện tượng rớt phím hoặc nuốt chữ.

---

### 3.4. Chế Độ Tự Động Thông Minh (Adaptive Auto Mode - OpenViet)
Được kế thừa từ tiện ích **OpenViet IME**, chế độ `auto` cho phép người dùng gõ cả **Telex** và **VNI** trên cùng một bàn phím mà không cần bấm phím chuyển đổi thủ công:
- **Nguyên lý phân luồng**:
  - Khi luồng phím chứa các chữ số `1-9`: Được dẫn truyền ngay qua bộ máy biến đổi VNI.
  - Khi luồng phím chứa các chữ cái quy ước Telex (`s, f, r, x, j, w, z` hoặc nhân đôi nguyên âm `aa, ee, oo, dd`): Được dẫn truyền qua bộ máy Telex.
  - Khi gõ xen kẽ (ví dụ: `thue6` rồi gõ `s`): Engine tự động hấp thụ số `6` thành `thuê`, sau đó nhận diện `s` thành dấu sắc $\to$ kết quả ra `thuế`.

---

### 3.5. Bộ Lọc Ngữ Cảnh Bảo Vệ URL, Email & Mã Nguồn (Code Guard)
Một trong những phiền toái lớn nhất khi lập trình hoặc duyệt web là vô tình bị bỏ dấu vào code hoặc đường dẫn. Crate `vnchkey-core` tích hợp bộ nhận diện ngữ cảnh thời gian thực (`context_guard.rs`):
- **Phát hiện URL & Giao Thức**: `http://`, `https://`, `ftp://`, `file://`.
- **Phát hiện Tên Miền Quốc Tế & Việt Nam**: Các hậu tố domain phổ biến như `.vn`, `.com`, `.org`, `.net`, `.io`, `.dev`, `.app`, `.edu`, `.gov`, `.xyz`, `.ai`.
- **Phát hiện Địa Chỉ Email**: Cấu trúc `user@domain.ext`.
- **Phát hiện Định Danh Mã Nguồn**:
  - `snake_case`: `user_id`, `max_value`, `get_user_by_id`.
  - `kebab-case`: `header-title`, `btn-primary`.
  - `dot.property`: `console.log`, `obj.property`, `std::sync::Arc`.
  - `operator & pointer`: `->`, `=>`, `===`, `!==`, `&mut`.
- **Cơ chế**: Khi chuỗi phím đang gõ khớp với mẫu trên, engine tự động đưa chuỗi về dạng ký tự thô nguyên thủy (bỏ qua mọi biến đổi dấu), giúp việc viết code diễn ra hoàn toàn tự nhiên.

---

### 3.6. Cơ Chế Xóa Dấu Không Phá Hủy Bằng Phím `z` (Telex) & `0` (VNI)
Theo chuẩn của OpenViet IME:
- **Khi âm tiết đã có dấu thanh**:
  - Telex: Gõ phím `z` sẽ lập tức bóc tách và xóa thanh điệu (`toán` + `z` $\to$ `toan`, `thuế` + `z` $\to$ `thuê`, `mừng` + `z` $\to$ `mưng`).
  - VNI: Gõ phím `0` sẽ xóa thanh điệu (`toán` + `0` $\to$ `toan`, `thuế` + `0` $\to$ `thuê`).
- **Khi âm tiết chưa có dấu**:
  - Ký tự `z` hoặc chữ số `0` được giữ nguyên vẹn như một ký tự bình thường (`zen`, `zoom`, `kho0`), không gây nuốt chữ của người dùng.

---

### 3.7. Kiểm Soát Âm Cuối Đóng Theo Âm Vị Học (-c, -ch, -p, -t)
Theo ngữ âm học tiếng Việt, các âm tiết kết thúc bằng âm tắc vô thanh (Stop Consonants: `-c, -ch, -p, -t`) chỉ có thể mang một trong hai thanh điệu: **Thanh 1 (Sắc)** hoặc **Thanh 5 (Nặng)**. Các thanh Huyền, Hỏi, Ngã không tồn tại trong cấu trúc âm vị học tiếng Việt chuẩn.
- **Thực thi trong vnchKey**:
  - Gõ `bac` + `s` $\to$ `bác` (Hợp lệ).
  - Gõ `bac` + `j` $\to$ `bạc` (Hợp lệ).
  - Gõ `bac` + `f` (Huyền) hoặc `r` (Hỏi) hoặc `x` (Ngã) $\to$ **Bị chặn**, giữ nguyên chuỗi `bacf`, `bacr`, `bacx` thay vì tạo ra ký tự dị dạng không có trong từ điển tiếng Việt.

---

### 3.8. Xử Lý Nguyên Âm Đôi Phức Tạp (uê, ươ, ưu, uơ) & Uốn Nguyên Âm Muộn
vnchKey xử lý triệt để các trường hợp nguyên âm phức tạp:
- **Cụm `uê` (`thuế`, `huế`, `quế`, `quệ`, `xuề`)**: Trong cụm này, `u` đóng vai trò âm đệm (glide) còn `ê` là âm chính (nucleus) $\to$ dấu thanh luôn luôn được đặt chuẩn xác trên `ê`.
- **Cụm `ươ` (`được`, `nước`, `lướt`)**: Gõ `uo` + `w` hoặc `uo` + `7` sẽ biến đổi đồng thời cả hai nguyên âm thành `ươ`. Dấu thanh được định vị chính xác trên `ơ`.
- **Cụm `ưu` (`cứu`, `cừu`, `hưu`, `lưu`)**: Dấu thanh luôn được đặt trên nguyên âm `ư`.
- **Uốn Nguyên Âm Muộn (Late Vowel Shaping)**: Cho phép gõ phụ âm cuối trước rồi uốn nguyên âm sau mà vẫn ra từ đúng:
  - Gõ `chong` rồi gõ thêm `o` $\to$ tự động nhận diện và uốn thành `chông`.
  - Gõ `kieu` rồi gõ thêm `e` $\to$ tự động chuyển thành `kiêu`.
  - Gõ `dem` rồi gõ thêm `e` $\to$ tự động chuyển thành `đêm`.

---

### 3.9. Phục Hồi Chuỗi Phím Gốc Bằng Phím `ESC` (Instant ESC Revert)
Khi đang gõ một từ tiếng Việt nhưng nhận ra cần giữ nguyên chuỗi phím chữ cái tiếng Anh gốc (ví dụ đang gõ `modem` thành `mô đ...`), người dùng chỉ cần nhấn phím **`ESC`**:
- vnchKey lập tức tính toán độ dài âm tiết hiện tại, phát số lượng `KEY_BACKSPACE` tương ứng và trả về ngay chuỗi phím thô ban đầu (`modem`), không cần phải nhấn Backspace từng chữ.

---

### 3.10. Hoàn Tác Gõ Tắt Tức Thì Bằng Phím `Backspace` (Macro Undo)
Khi người dùng gõ một từ khóa gõ tắt và mở rộng thành công (ví dụ: gõ `vn ` $\to$ nở thành `Việt Nam `):
- Nếu người dùng bấm phím **`Backspace` ngay lập tức**, engine nhận diện đây là hành động muốn hủy bỏ việc gõ tắt, lập tức xóa toàn bộ cụm `Việt Nam ` và phục hồi lại đúng từ khóa gốc `vn`.

---

### 3.11. Khôi Phục Tiếng Anh Tự Động (GoNhanh English Auto-Restore)
Được kế thừa từ thuật toán xuất sắc của bộ gõ **GoNhanh**:
- Bộ nhớ tĩnh tích hợp sẵn một cây Trie chứa hàng trăm từ vựng lập trình và giao tiếp quốc tế thông dụng (`text`, `test`, `post`, `struct`, `match`, `class`, `import`, `export`, `return`, `filter`, `while`, `const`, `let`...).
- Khi người dùng gõ `t-e-x-t`, trong quá trình gõ tạm thời có thể hiển thị `têt`. Nhưng ngay khi nhấn **Phím Cách (Space)**, **Dấu Chấm**, hoặc **Dấu Phẩy**, engine nhận diện từ gốc là một từ tiếng Anh hợp lệ và tự động phát xung xóa lùi khôi phục lại `text `, hoàn toàn tự động và giải phóng người lập trình khỏi phiền toái.

---

### 3.12. Bảo Toàn Trạng Thái Chữ Hoa Theo Ký Tự (XKey Smart Casing)
Kế thừa thuật toán vector nhận diện kiểu chữ hoa từ **XKey**:
- Khi gõ toàn bộ chữ hoa: `VIET` $\to$ `VIỆT` (thay vì các lỗi thường gặp như `VIệt` hoặc `ViỆt`).
- Khi gõ chữ hoa đầu từ: `Viet` $\to$ `Việt`.
- Khi gõ hoa cụm từ: `CONG HOA` $\to$ `CỘNG HÒA`.
- Khi mở rộng macro: `vn` $\to$ `Việt Nam`, nhưng nếu gõ `VN` $\to$ tự động mở rộng hoa toàn bộ thành `VIỆT NAM`.

---

### 3.13. Quản Lý Từ Ngoại Lệ Của Người Dùng (Custom Whitelist)
vnchKey cho phép người dùng định nghĩa danh sách các từ vựng kỹ thuật riêng hoặc tên riêng không bao giờ được phép bỏ dấu:
- Tệp cấu hình lưu trữ tại `~/.config/vnchkey/whitelist.txt`.
- Bất kỳ từ nào nằm trong danh sách này (ví dụ: `neovim`, `hyprland`, `wayland`, `cargo`, `pacman`) sẽ luôn luôn được giữ nguyên phím chữ cái thô.

---

### 3.14. Giới Hạn Thiết Kế: Loại Trừ Gợi Ý Từ & Học Thói Quen
Nhằm đáp ứng yêu cầu cốt lõi về **tính tối giản, hiệu năng cao và an toàn bảo mật tuyệt đối**:
- **Không tích hợp gợi ý từ kế tiếp (Next Word Suggestion)**: Tránh tiêu tốn hàng chục MB RAM cho mô hình n-gram và không làm phân tâm thị giác khi gõ văn bản chuyên sâu.
- **Không học thói quen người dùng (User Habit Learning)**: Loại bỏ hoàn toàn nguy cơ rò rỉ dữ liệu nhạy cảm hoặc mật khẩu vào tệp lưu trữ thói quen. vnchKey hoạt động 100% dự đoán được (deterministic) và không tạo bất kỳ dấu vết telemetry nào trên máy tính.

---

## 4. Bảng So Sánh Kiến Trúc Toàn Diện

| Tiêu Chí Kỹ Thuật | fcitx5-lilypad (Gốc C++) | GoNhanh (macOS/Rust) | OpenViet (Firefox) | vnchKey: Tự Do (Rust) |
| :--- | :--- | :--- | :--- | :--- |
| **Ngôn ngữ lập trình** | C++ / CMake | Rust 100% | JavaScript / WebExtension | **Rust 100% (`#![forbid(unsafe_code)]`)** |
| **Môi trường hoạt động** | Fcitx5 Linux | macOS Native | Trình duyệt Firefox | **Toàn hệ thống Linux (Wayland & X11)** |
| **Cơ chế Commit phím** | `/dev/uinput` | `CGEventPost` (macOS) | DOM Input Events | **Linux Kernel `/dev/uinput` Direct Commit** |
| **An toàn bộ nhớ** | Thủ công (Dễ rò rỉ/segfault) | Compile-time Safe | Garbage Collected | **Compile-time Memory Safety (Tuyệt đối)** |
| **Tiêu thụ RAM** | ~35 MB | ~20 MB | Phụ thuộc trình duyệt | **< 15 MB (Siêu nhẹ, zero-leak)** |
| **Độ trễ commit** | Tĩnh (Fixed `usleep`) | Thích ứng macOS | Bất đồng bộ JS | **Cảm biến EMA IKI Thích ứng (1.5ms - 8ms)** |
| **Giao thức chống nuốt chữ**| Sentinel N+1 cơ bản | Chốt chặn sự kiện | Quản lý DOM | **Sentinel Barrier (N+1) + Watchdog 200ms** |
| **Chế độ Tự Động (Auto)** | Không có | Không có | Có (Mặc định) | **Có (Tích hợp sâu trong lõi Rust)** |
| **Bảo vệ URL, Email & Code**| Không có | Không có | Có | **Có (Lexer nhận diện đa cú pháp)** |
| **Xóa dấu phím `z` & `0`** | Không có | Không có | Có | **Có (`z` Telex, `0` VNI)** |
| **Quy tắc âm cuối tắc (-c,-ch,-p,-t)** | Dễ sai chính tả | Cơ bản | Chính xác | **Kiểm soát âm vị học tuyệt đối** |
| **Hoàn tác gõ tắt bằng Backspace**| Không có | Không có | Có | **Có (Macro Undo Stack)** |
| **Phục hồi phím thô bằng ESC** | Không có | Không có | Có | **Có (Instant Snapshot Rollback)** |
| **Khôi phục tiếng Anh (Auto-Restore)**| Không có | Có (Trie từ điển) | Không có | **Có (Trie từ điển tích hợp)** |
| **Bảo toàn chữ hoa (Smart Casing)**| Cơ bản | Khá | Cơ bản | **Kế thừa vector XKey (Xuất sắc)** |
| **Gợi ý từ / Học thói quen**| Không | Không | Có | **Loại trừ có chủ đích (Bảo mật tối đa)** |

---

## 5. Hướng Dẫn Cài Đặt Chi Tiết Cho Arch Linux

Dành cho người dùng **Arch Linux, EndeavourOS, Manjaro, CachyOS, Garuda, Artix Linux** (hoặc bất kỳ bản phân phối nào dựa trên Arch).

### 5.1. Cài đặt các công cụ biên dịch cơ sở
Mở Terminal và cài đặt các gói công cụ:

```bash
# 1. Cập nhật cơ sở dữ liệu hệ thống và cài đặt base-devel, git, rustup, fcitx5
sudo pacman -Syu --needed base-devel git rustup fcitx5 fcitx5-configtool fcitx5-gtk fcitx5-qt

# 2. Cấu hình toolchain Rust Stable mới nhất
rustup default stable
rustup update
```

---

### 5.2. Thiết lập quyền kernel Linux `/dev/uinput` (Không cần root)
Để vnchKey có thể tạo thiết bị phím ảo và phát sự kiện ở cấp độ nhân Linux mà **không cần chạy tiến trình daemon dưới quyền root** (`sudo`):

```bash
# Bước 1: Thêm tài khoản người dùng hiện tại vào nhóm 'input'
sudo usermod -aG input $USER

# Bước 2: Tạo tệp quy tắc udev cấp quyền đọc/ghi (0660) cho nhóm 'input'
echo 'KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"' | sudo tee /etc/udev/rules.d/99-uinput.rules

# Bước 3: Đảm bảo module uinput được nạp tự động khi khởi động máy
echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf
sudo modprobe uinput

# Bước 4: Nạp lại quy tắc udev ngay lập tức
sudo udevadm control --reload-rules && sudo udevadm trigger
```

> **Lưu ý quan trọng**: Sau khi chạy lệnh `usermod`, bạn cần **đăng xuất và đăng nhập lại** để quyền thành viên trong nhóm `input` có hiệu lực. Bạn có thể kiểm tra bằng lệnh `groups` (phải thấy xuất hiện chữ `input`).

---

### 5.3. Hướng Dẫn Cài Đặt (Nhanh Nhất & Tự Động)

#### Cách 1: Cài đặt tự động 1-Click bằng `install.sh` (Khuyên Dùng ⭐)
Script sẽ tự động biên dịch mã nguồn Rust, build Fcitx5 addon, cài đặt các file cấu hình vào hệ thống, cấp quyền kernel `/dev/uinput` và khởi động lại Fcitx5:

```bash
git clone https://github.com/no-vici/fcitx5-vnchkey.git
cd fcitx5-vnchkey
chmod +x install.sh
./install.sh
```

---

#### Cách 2: Đóng gói chuẩn Arch Linux bằng `makepkg -si`
Tệp tin `PKGBUILD` đã được tích hợp sẵn ngay trong thư mục gốc của repository:

```bash
git clone https://github.com/no-vici/fcitx5-vnchkey.git
cd fcitx5-vnchkey
makepkg -si
```

---

### 5.4. Menu Nhanh Trực Tiếp Trên Fcitx5 & Cửa Sổ Cài Đặt 100% Native Rust

vnchKey tích hợp sâu trực tiếp vào Fcitx5 với hai cơ chế điều khiển hiện đại, **hoàn toàn viết bằng 100% Safe Rust**, không phụ thuộc Python:

#### 1. Menu Nhanh Trên Khay Hệ Thống (Status Area Quick Menu)
Khi vnchKey được kích hoạt, biểu tượng trên thanh tác vụ / khay hệ thống Fcitx5 hiển thị nhãn kiểu gõ hiện tại (`Auto`, `Telex`, `VNI`). Nhấp trực tiếp vào biểu tượng sẽ chuyển nhanh kiểu gõ hoặc mở ngay Menu Nhanh:
* **Kiểu gõ**: Tự Động (Auto Telex & VNI), Telex, VNI.
* **Bảng mã**: Chuẩn Unicode quốc tế duy nhất (tối ưu, tinh gọn, loại bỏ hoàn toàn các bảng mã lỗi thời).
* **Bật/Tắt tức thì trên thanh tác vụ**: Khôi phục tiếng Anh (GoNhanh), Gõ tắt (Macro), Kiểm tra chính tả.
* **Cài đặt Fcitx5 vnchKey...**: Mở cửa sổ cấu hình chi tiết.

#### 2. Cửa Sổ Cài Đặt Fcitx5 vnchKey Settings (100% Native Rust)
Giao diện đồ hoạ được viết bằng Rust native (`egui` / `eframe`), khởi động tức thì trong mili-giây, thiết kế thanh thoát theo chuẩn Catppuccin / Lilypad dark theme với typography rõ nét, tinh tế:
1. Mở từ Application Menu / Tìm kiếm ứng dụng: **Fcitx5 vnchKey Settings**
2. Hoặc chạy lệnh trong terminal (chuẩn theo định dạng `fcitx5-*-settings` như lilypad):
```bash
fcitx5-vnchkey-settings
# Hoặc lệnh rút gọn:
vnchkey-settings
```

Giao diện cài đặt hỗ trợ đầy đủ 10 danh mục:
* **General**: Chọn kiểu gõ (**Auto**, **Telex**, hoặc **VNI**), chế độ (**Uinput Smooth** hoặc **Native Fcitx5 Direct Commit**), bảng mã đầu ra chuẩn **Unicode**, tuỳ chọn gõ phím `w` thành `ư`, chuyển dấu ngoặc vuông `[` thành `ơ` và `]` thành `ư`.
* **Typing**: Quy tắc dấu thanh (Chuẩn mới `oà, uý` / Chuẩn cũ `òa, úy`), Smart Horn tự động thêm râu cho `uo` (`duoc95` -> `được`), GoNhanh tự động khôi phục từ tiếng Anh, XKey Smart Casing, Gõ nhanh phụ âm đầu (`cc` -> `ch`, `gg` -> `gi`).
* **Applications**: Quản lý danh sách ứng dụng bỏ qua (Bypass cho terminal, game, v.v.).
* **Macros**: Bảng quản lý gõ tắt trực quan (Thêm, Xoá, Nhập, Xuất file).
* **Dictionary**: Quản lý từ điển từ ngữ chuyên ngành của người dùng.
* **Keymap & Shortcuts**: Xem bảng quy ước phím gõ VNI/Telex và tuỳ biến phím tắt chuyển đổi bộ gõ.
* **Appearance**: Tuỳ chỉnh biểu tượng khay hệ thống (Default, Breeze Dark, Monochrome White, Minimal) và huy hiệu trạng thái.
* **Backup**: Xuất / Nhập toàn bộ cấu hình ra tệp JSON hoặc khôi phục mặc định gốc.
* **About**: Thông tin phiên bản, bản quyền GPLv3, liên kết báo lỗi, yêu cầu tính năng và xuất nhật ký gỡ lỗi (Export Debug Logs).

#### Cách 2: Biên dịch trực tiếp bằng Cargo
```bash
git clone https://github.com/no-vici/fcitx5-vnchkey.git vnchkey
cd vnchkey

# Biên dịch với cấu hình tối ưu hóa cao nhất (LTO Fat)
cargo build --release

# Cài đặt thủ công thư viện Fcitx5 FFI và file thực thi Daemon
sudo install -m 755 target/release/libfcitx5_vnchkey_ffi.so /usr/lib/fcitx5/libfcitx5-vnchkey.so
sudo install -m 755 target/release/vnchkey-daemon /usr/bin/vnchkey-daemon
```

---

## 6. Cấu Hình Môi Trường Wayland & Các Compositor

### 6.1. Biến môi trường hệ thống (`environment.d`)
Tạo hoặc chỉnh sửa tệp `~/.config/environment.d/im.conf` (áp dụng tự động cho toàn bộ phiên đăng nhập Wayland systemd):

```bash
mkdir -p ~/.config/environment.d

cat << 'EOF' > ~/.config/environment.d/im.conf
GTK_IM_MODULE=fcitx
QT_IM_MODULE=fcitx
XMODIFIERS=@im=fcitx
INPUT_METHOD=fcitx
SDL_IM_MODULE=fcitx
GLFW_IM_MODULE=ibus
EOF
```

---

### 6.2. Cấu hình cho Hyprland
Mở tệp `~/.config/hypr/hyprland.conf` và thêm cấu hình khởi chạy cùng các windowrule cần thiết:

```ini
# ==============================================================================
# vnchKey & Fcitx5 Configuration for Hyprland
# ==============================================================================

# Khởi động Fcitx5 daemon và vnchKey Uinput Daemon
exec-once = fcitx5 -d --replace
exec-once = vnchkey-daemon --method auto --tone modern --auto-restore --smart-casing --url-code-guard

# Thiết lập biến môi trường Wayland trực tiếp trong Hyprland
env = GTK_IM_MODULE,fcitx
env = QT_IM_MODULE,fcitx
env = XMODIFIERS,@im=fcitx
env = INPUT_METHOD,fcitx
env = SDL_IM_MODULE,fcitx
env = GLFW_IM_MODULE,ibus

# Window rules cho các cửa sổ cấu hình của Fcitx5
windowrulev2 = float, class:(org.fcitx.fcitx5-config-qt)
windowrulev2 = size 700 500, class:(org.fcitx.fcitx5-config-qt)
```

---

### 6.3. Cấu hình cho Sway & Wayfire
Thêm vào `~/.config/sway/config`:

```ini
exec fcitx5 -d --replace
exec vnchkey-daemon --method auto --tone modern --auto-restore
```

---

### 6.4. Cấu hình cho KDE Plasma 6 & GNOME 47 (Wayland)
- **KDE Plasma 6**: Mở `System Settings` $\to$ `Input Devices` $\to$ `Virtual Keyboard` $\to$ Chọn **Fcitx 5**.
- **GNOME 47**: Vào `Settings` $\to$ `Keyboard` $\to$ Thêm `Vietnamese (vnchKey)` vào danh sách Input Sources.

---

## 7. Quản Lý Tiến Trình Daemon Qua Systemd User

Sử dụng `systemd --user` để quản lý tiến trình `vnchkey-daemon` chạy ngầm, tự động khởi động cùng phiên đồ họa và tự phục hồi nếu xảy ra sự cố:

```bash
mkdir -p ~/.config/systemd/user/

cat << 'EOF' > ~/.config/systemd/user/vnchkey.service
[Unit]
Description=vnchKey Vietnamese Input Daemon (Tu Do Edition)
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/vnchkey-daemon --method auto --tone modern --auto-restore --smart-casing --url-code-guard
Restart=always
RestartSec=1
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

# Giới hạn tài nguyên an toàn
MemoryMax=64M
CPUQuota=20%

[Install]
WantedBy=graphical-session.target
EOF

# Nạp lại cấu hình và kích hoạt service
systemctl --user daemon-reload
systemctl --user enable --now vnchkey.service
```

Kiểm tra trạng thái hoạt động:
```bash
systemctl --user status vnchkey.service
```

---

## 8. Tài Liệu Dòng Lệnh (CLI Options & Configuration)

`vnchkey-daemon` hỗ trợ các tùy chọn dòng lệnh linh hoạt:

```
vnchkey-daemon [OPTIONS]

CÁC TÙY CHỌN:
  -m, --method <METHOD>            Kiểu gõ mặc định [auto | telex | vni] (Mặc định: auto)
  -t, --tone <STYLE>               Kiểu đặt dấu thanh [modern | classic] (Mặc định: modern)
      --auto-restore               Bật tính năng tự động khôi phục tiếng Anh (GoNhanh)
      --smart-casing               Bật tính năng bảo toàn chữ hoa thông minh (XKey)
      --url-code-guard             Bật bộ lọc chống bỏ dấu vào URL, Email & Code (OpenViet)
      --no-sentinel                Tắt giao thức rào cản an toàn Sentinel Barrier
      --dom-delay <MS>             Thêm độ trễ mô phỏng Web DOM (0 - 60ms) (Mặc định: 0)
  -c, --config <FILE>              Đường dẫn tệp cấu hình TOML (Mặc định: ~/.config/vnchkey/config.toml)
  -d, --daemon                     Chạy tách rời dưới dạng tiến trình nền (Detached daemon)
  -h, --help                       Hiển thị trợ giúp
  -V, --version                    Hiển thị phiên bản
```

### Ví dụ tệp cấu hình `~/.config/vnchkey/config.toml`:

```toml
[engine]
method = "auto"              # "auto", "telex", hoặc "vni"
tone_style = "modern"        # "modern" (hoà, uỷ) hoặc "classic" (hòa, ủy)
auto_restore_english = true  # Khôi phục từ tiếng Anh khi nhấn Space
smart_casing = true          # Bảo toàn VIET -> VIỆT
url_and_code_guard = true    # Bảo vệ https://, user@domain, snake_case
sentinel_enabled = true      # Chống race condition nuốt chữ

[iki]
min_delay_ms = 1.5           # Độ trễ tối thiểu (Terminal mode)
max_delay_ms = 8.0           # Độ trễ tối đa khi lag dồn dập (Web DOM mode)
ema_weight = 0.25            # Trọng số làm mượt nhịp gõ

[hotkeys]
toggle_ime = "DoubleCtrl"    # Nhấp đúp Ctrl (<400ms) để bật/tắt bộ gõ
instant_revert = "Escape"    # Nhấn Escape để khôi phục ký tự thô
```

---

## 9. Chẩn Đoán & Khắc Phục Lỗi Toàn Diện (Troubleshooting Matrix)

### Bảng Chẩn Đoán Sự Cố:

| Triệu Chứng / Lỗi | Nguyên Nhân Gốc Rễ | Cách Khắc Phục Triệt Để |
| :--- | :--- | :--- |
| `Permission denied (os error 13)` khi mở `/dev/uinput` | User chưa có quyền ghi vào thiết bị uinput của kernel | 1. Kiểm tra lệnh `groups` xem có nhóm `input` chưa.<br>2. Chạy `sudo usermod -aG input $USER`.<br>3. Kiểm tra file `/etc/udev/rules.d/99-uinput.rules`.<br>4. Đăng xuất và đăng nhập lại. |
| Không gõ được tiếng Việt trong Chrome / Chromium | Chromium chạy qua XWayland không nhận diện đúng module input | Thêm cờ Wayland khi mở Chrome:<br>`google-chrome-stable --enable-features=UseOzonePlatform --ozone-platform=wayland` |
| Không gõ được trong ứng dụng Flatpak (Discord, Obsidian, Spotify) | Sandbox của Flatpak chặn truy cập socket Fcitx5 | Cấp quyền IPC cho Flatpak:<br>`flatpak override --user --filesystem=xdg-run/im:ro --talk-name=org.fcitx.Fcitx5` |
| Bị mất chữ hoặc giật trên VS Code | VS Code DOM lag xử lý sự kiện Backspace chậm hơn bàn phím | 1. Bật `--url-code-guard` và `--sentinel`.<br>2. Cài đặt extensions VS Code: tắt tính năng `editor.acceptSuggestionOnEnter` nếu bị xung đột. |
| Bộ gõ không tự khởi động khi vào Hyprland | Thiếu lệnh `exec-once` trong cấu hình Hyprland | Mở `~/.config/hypr/hyprland.conf`, thêm:<br>`exec-once = fcitx5 -d --replace && vnchkey-daemon` |
| Bị lỗi gạch chân nhấp nháy pre-edit | Ứng dụng đang dùng bộ gõ khác (IBus hoặc engine cũ) | Chạy `fcitx5-configtool`, xóa tất cả bộ gõ khác, chỉ giữ lại duy nhất **vnchKey**. |

### Kiểm tra hệ thống toàn diện:
```bash
# Kiểm tra biến môi trường Fcitx5
fcitx5-diagnose

# Kiểm tra quyền truy cập tệp thiết bị uinput
ls -la /dev/uinput
# Kết quả hợp lệ: crw-rw---- 1 root input 10, 223 /dev/uinput

# Theo dõi nhật ký sự kiện daemon thời gian thực
journalctl --user -u vnchkey.service -f
```

---

## 10. Ý Nghĩa Biểu Tượng & Di Sản Tự Do

Huy hiệu của **vnchKey** kết hợp hai yếu tố văn hóa và công nghệ:
1. **Nền Tròn Cờ Vàng Ba Sọc Đỏ (Hoàng Long & Ba Sọc Đỏ)**:
   - Màu vàng rực rỡ biểu trưng cho đất nước, con người và sự phồn vinh của dân tộc Việt Nam.
   - Ba sọc đỏ tượng trưng cho ba miền Bắc — Trung — Nam thống nhất keo sơn, gắn liền với di sản tinh thần **Tự Do — Dân Chủ — Phú Cường**.
2. **Khối Phím Cơ 3D Dập Nổi Chữ `vn`**:
   - Khối phím keycap cơ học sắc nét vươn lên từ nền cờ mang ý nghĩa: *Mỗi phím bấm là một tiếng nói tự do, độc lập và bảo tồn trọn vẹn ngôn ngữ mẹ đẻ trong thời đại số.*
   - Biểu tượng được cung cấp dưới dạng ảnh 3D kết xuất độ sâu chân thực (`vnch_key_128.png`) và bản vector toán học SVG siêu nét (`VnchKeyVectorIcon`) không phụ thuộc độ phân giải.

---

## 📄 Bản Quyền & Giấy Phép

- Dự án được phát hành dưới hình thức **Mã Nguồn Mở Tự Do 100%** theo các điều khoản của [Giấy Phép GNU General Public License v3.0 (GPLv3)](LICENSE).
- Tôn chỉ giấy phép GPLv3 bảo đảm rằng **vnchKey** và tất cả các phiên bản phái sinh sẽ luôn luôn tự do, mở mã nguồn vĩnh viễn và không bao giờ bị biến tướng thành phần mềm đóng hay thương mại độc quyền.
- Mọi đóng góp, báo cáo lỗi (issues) và yêu cầu kéo (pull requests) xin gửi về: [GitHub Repository](https://github.com/no-vici/fcitx5-vnchkey).
