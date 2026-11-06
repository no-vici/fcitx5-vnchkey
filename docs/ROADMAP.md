# Lộ Trình Phát Triển & Cột Mốc Dự Án vnchKey (Project Roadmap)

> **Dự án**: vnchKey (Tự Do Edition)  
> **Mục tiêu**: Xây dựng bộ gõ tiếng Việt hiệu năng cao, ổn định tuyệt đối, tôn trọng tự do và mang lại trải nghiệm gõ tốt nhất trên hệ điều hành Linux (Fcitx5 / Wayland / X11).

---

## 📌 Tổng Quan Tiến Độ

```
[v0.1.0: Lõi Rust & Kiến Trúc Fcitx5]  ──(Hoàn tất)──► [v0.2.0: Đột Phá Trải Nghiệm Gõ] ──(Hoàn tất)──► [v0.3.0: Tối Ưu Thực Chiến Linux]
  • Safe Rust Core Engine                                • Chế độ Kép: Direct & Preedit                     • GNOME Wayland gnome-shell Guard
  • Micro-diff LCP Algorithm                             • Floating Preedit Tooltip                         • Allowlist WPS Office (hidesPreedit)
  • Ngữ âm học ~6.750 từ gốc                             • Per-App Mode Memory                              • Dọn dẹp Sheets (deleteAfterChars)
  • Native Rust Settings GUI (egui)                      • Địa danh Tây Nguyên & Dấu nháy '                 • BlindWrites Auto-fallback (Cursor)
                                                         • Persistent Context Guard (Java IDEs/Wine)        • Phím tắt Lật Từ (Ctrl+Shift+Z)
                                                         • Adaptive Micro-Pause Engine (Direct on Web)      • Re-open âm tiết sau Backspace
                                                         • System App Picker & Mode Grid
```

---

## ✅ Cột Mốc Đã Hoàn Thành (Shipped & Active)

### Phiên bản 0.1.0: Nền Móng Kiến Trúc & Hiệu Năng Cốt Lõi
- [x] **Lõi Xử Lý Thuần Rust (`vnchkey-core`)**: Quản lý State Machine biến âm âm tiết tiếng Việt, không phụ thuộc daemon ngoài, an toàn bộ nhớ tuyệt đối.
- [x] **Thuật Toán Micro-diff Minimal Replacement**:
  - Dựa trên nguyên lý Tiền Tố Chung Dài Nhất (Longest Common Prefix).
  - Giảm tới 70% số lượng phím Backspace gửi ra ứng dụng, triệt tiêu hiện tượng giật lag con trỏ trong các editor chuyên nghiệp (Neovim, VS Code, Zed, Sublime Text).
- [x] **Bộ Kiểm Tra Chánh Tả & Ngữ Âm Học Khắt Khe**:
  - Tích hợp sẵn ~6.750 từ đơn gốc tiếng Việt chuẩn (Hunspell + OpenKey + OpenViet).
  - Kiểm soát âm cuối tắc (`-c, -ch, -p, -t`) chỉ cho phép mang thanh Sắc hoặc Nặng.
  - Hỗ trợ phương ngữ Nam Bộ và khẩu ngữ phổ biến (`dzìa`, `dzô`, `dzí`, `gởi`, `chánh`...).
- [x] **Tự Động Khôi Phục Tiếng Anh (Restore English)**: Kế thừa thuật toán từ điển tĩnh từ GoNhanh, tự khôi phục từ tiếng Anh / từ vựng lập trình (`text`, `post`, `struct`...) về nguyên bản khi bấm phím phân cách.
- [x] **Kiểu Gõ Đa Năng**: Hỗ trợ đồng thời Tự Động (Auto Telex & VNI), Telex truyền thống, VNI, gõ `w` thành `ư` (`w2u`) và biến đổi ngoặc vuông `[` $\to$ `ơ`, `]` $\to$ `ư`.
- [x] **Bảo Toàn Chữ Hoa Thông Minh (Smart Casing)**: Tự động giữ nguyên định dạng ALL CAPS, Title Case, và viết hoa cụm từ.
- [x] **Bảng Gõ Tắt Tức Thì & Hoàn Tác (Macro & Macro Undo)**: Mở rộng từ viết tắt nhanh và cho phép nhấn Backspace để khôi phục lại từ viết tắt ban đầu.
- [x] **Giao Diện Cài Đặt Native 100% Rust (`fcitx5-vnchkey-settings`)**: Viết bằng `egui` / `eframe`, khởi động tức thì trong mili-giây, giao diện Dark Theme trực quan với 9 tab quản trị.

---

### Phiên bản 0.2.0: Đột Phá Khắc Phục Lỗi Nhảy Chữ & Tối Ưu Môi Trường Web
- [x] **Kiến Trúc Xuất Chữ Kép (Hybrid Dual Mode)**:
  - **Direct Mode (Trực tiếp)**: Giữ nguyên cơ chế Micro-diff với độ trễ bằng $0$, mang lại cảm giác gõ trực tiếp như UniKey Windows cho Terminal, Vim, Neovim, VS Code.
  - **Preedit Mode (Gạch chân)**: Không phát bất kỳ phím Backspace giả lập nào ra ngoài ứng dụng. Từ được gom lại và cam kết nguyên tử (atomic commit) khi hoàn tất. **Đặc trị dứt điểm 100% lỗi nhảy chữ, lặp chữ kinh niên (`Thuưử`, `Tiengêngếng`, `vâyậy`)** trên Google Docs, StackEdit, Sider.ai, Slack, Discord.
- [x] **Khung Chữ Nổi (Floating Preedit Tooltip)**:
  - Hiển thị từ đang soạn trong một popup nhỏ xinh sát chân con trỏ chuột cho các ứng dụng tự vẽ Canvas/WebGL như Google Docs (nơi trình duyệt không vẽ được nét gạch chân nội dòng).
  - Tích hợp tùy chọn bật/tắt nhanh trên Menu khay hệ thống và GUI Cài Đặt để người dùng không bị phân tâm.
- [x] **Tự Động Nhớ Chế Độ Theo Từng Ứng Dụng (Per-App Mode Memory)**:
  - Tự động nhận diện tiến trình (`ic->program()`): Trình duyệt Web (`Chrome`, `Firefox`, `Brave`, `Edge`) và ứng dụng chat Electron (`Slack`, `Discord`, `Telegram`) tự bật Preedit; các Terminal và Code Editor tự bật Direct.
  - Ghi nhớ tùy chỉnh riêng biệt vào `~/.config/fcitx5/conf/vnchkey-apps.conf`.
- [x] **Bổ Sung Địa Danh Tây Nguyên & Chuẩn Hóa Ký Tự Đặc Biệt**:
  - Nhận diện đúng và bỏ qua kiểm tra âm học cho các địa danh Tây Nguyên đặc thù có cụm phụ âm lạ (`Đắk Lắk`, `Cư M'gar`, `Ea H'leo`, `M'Đrắk`, `Đắk Nông`, `Krông Nô`, `Pleiku`...).
  - Tự động chuẩn hóa dấu nháy đơn (curly quotes `’`, `‘` $\to$ `'`).
- [x] **Persistent Context Guard (Bảo Vệ Từ Trong IntelliJ / PyCharm / Android Studio / Wine)**:
  - Tích hợp **Grace Period Tracker**: Khắc phục triệt để hiện tượng ngắt kết nối giả mạo của giao thức XIM và Java Swing khi mở popup gợi ý code (autocomplete).
  - Lưu giữ trạng thái từ trong $350\text{ms}$, chấm dứt hoàn toàn hiện tượng đang gõ dở thì bị "gãy nhịp" mất dấu.
- [x] **Adaptive Micro-Pause Engine (Độ Trễ Thích Ứng Cho Direct Mode Trên Web)**:
  - Cho phép người dùng gõ **Direct Mode (Non-preedit) ngay trên trình duyệt web**.
  - Tự động áp dụng khoảng dừng thích ứng $4\text{ms}$ giữa lúc nhả phím `Backspace` và lúc bắn `commitString`, tạo "thời gian vàng" cho JavaScript Event Loop của trình duyệt cập nhật lại DOM cursor, chống chèn đè chữ.
  - Cung cấp thanh trượt tùy chỉnh $0 - 20\text{ms}$ trong Bảng Cài Đặt.
- [x] **Trình Quản Lý Ứng Dụng & Bộ Chọn Thông Minh Từ Hệ Thống (System App Picker & Mode Manager)**:
  - Thiết kế lại trang **Applications** theo phong cách hiện đại với bố cục 2 cột trực quan:
    - **Cột Sidebar**: Danh sách ứng dụng đã cấu hình, ô tìm kiếm thời gian thực, icon glyph danh mục, tên ứng dụng và trạng thái chế độ.
    - **Cột Chi Tiết**: Header ứng dụng nổi bật cùng lưới **Mode Cards** trực quan (Direct, Preedit, Bypass, Default) cho phép đổi chế độ chỉ bằng một cú nhấp chuột.
  - **Hộp thoại Thêm Ứng Dụng (Add Application Modal)**: Tự động quét và liệt kê toàn bộ ứng dụng cài đặt trong hệ thống (`/usr/share/applications/*.desktop`, `~/.local/share/applications`, Flatpak) và các tiến trình người dùng đang chạy (`/proc`), cho phép tìm kiếm và nhấp đúp để thêm ngay thay vì phải nhập tên thủ công. Hỗ trợ tab nhập thủ công cho các script/binary riêng biệt.

---

## 🚀 Kế Hoạch Phát Triển Tiếp Theo (Upcoming Milestones)

### Phiên bản 0.3.0: Tối Ưu Hóa Trải Nghiệm Thực Chiến Độc Quyền Cho Linux (Chắt lọc từ Funput & VMK)

#### 1. Khắc Phục Các Góc Khuất Khắc Nghiệt Trên Linux & Wayland
- [x] **Bảo Vệ GNOME Wayland (`gnome-shell` Proxy Guard)**:
  - Loại trừ chuỗi proxy `"gnome-shell"` khi đọc `ic->program()` trên GNOME Wayland.
  - Ngăn ngừa hiện tượng một ứng dụng bị lỗi (như Cursor IDE) làm ô nhiễm bộ đệm gán chế độ của toàn bộ hệ điều hành.
- [x] **Đặc Trị WPS Office (`hidesClientPreedit` Allowlist)**:
  - Tự động nhận diện danh sách bộ ứng dụng WPS Office (`wps`, `wpp`, `et`, `wpspdf`).
  - Ép Fcitx5 tự vẽ khung nổi (`panel.setPreedit(preedit)`), xóa bỏ hoàn toàn lỗi "gõ tàng hình" kinh niên do WPS khai báo hỗ trợ Preedit nhưng lại không tự vẽ gạch chân.
  - Ngăn chặn ép Non-preedit vào WPS vì engine của WPS không hỗ trợ `Qt::ImSurroundingText`.
- [x] **Dọn Dẹp Hậu Tố Gợi Ý Google Sheets (`deleteAfterChars`)**:
  - Tự động phát hiện vùng chọn autocomplete phía sau con trỏ (`surrounding.anchor() > surrounding.cursor()`) trong Google Sheets.
  - Xóa sạch phần gợi ý trước khi commit từ mới, chấm dứt hoàn toàn hiện tượng dính chữ kỳ quặc (`chư` + `ứ` $\to$ `chưứ`).
- [ ] **Thuật Toán `BlindWrites` & Tự Động Lùi Về Preedit Cho Editor Cứng Đầu (Cursor / VS Code)**:
  - Giám sát phản hồi tài liệu: nếu app trả lời rỗng $2$ lần liên tiếp khi đang nhận lệnh sửa $\to$ nhận diện app đang nuốt lệnh xóa và tự động chuyển sang chế độ Preedit an toàn, chống triệt để lỗi `Phuú`.

#### 2. Tính Năng Gõ Tiện Ích Đột Phá
- [x] **Phím Tắt Lật Từ Tức Thì (Flip Word — `Ctrl + Shift + Z`)**:
  - Cho phép người dùng hoặc lập trình viên hoán đổi tức thì giữa từ tiếng Việt và chuỗi ký tự thô ban đầu (`mas` $\leftrightarrow$ `má`), tiện lợi tuyệt đối khi gõ mã nguồn mà không cần Backspace gõ lại.
- [ ] **Khôi Phục Âm Tiết Sau Backspace (`adoptWordBeforeBackspace`)**:
  - Khi xóa dấu cách sau một từ đã hoàn tất, tự động đọc lại âm tiết trước con trỏ vào bộ nhớ đệm để người dùng có thể gõ tiếp phím dấu sửa đổi (ví dụ: `phủ` ␣ ⌫ `s` $\to$ `phú`).

---

### Giai đoạn hoàn thiện hệ sinh thái & Phân phối

#### 1. Context Guard Nâng Cao (Smart Auto-Bypass)
- [ ] **Nhận diện thanh địa chỉ URL & Đường dẫn thư mục**: Tự động phát hiện khi con trỏ đang nhập URL (`http://...`, `https://...`, `www....`), địa chỉ IP, hoặc đường dẫn tệp (`/usr/bin/...`, `~/.config/...`) để tự động chuyển sang chế độ gõ ký tự thô (Raw English Bypass), không can thiệp dấu tiếng Việt.
- [ ] **Nhận diện cú pháp mã nguồn (Code Identifier Guard)**: Tự động bỏ qua biến âm khi phát hiện quy tắc đặt tên biến lập trình (`camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`).
- [ ] **Bảo vệ mật khẩu & trường nhạy cảm**: Tuân thủ cờ `CapabilityFlag::Password` và `CapabilityFlag::Sensitive` của Wayland để tắt hoàn toàn bộ đệm từ.

#### 2. Tối Ưu Hóa Giao Thức Wayland Hiện Đại (Deep Wayland Compositor Integration)
- [ ] Nghiên cứu hỗ trợ trực tiếp các giao thức nhập liệu thế hệ mới `zwp_input_method_v2` và `zwp_virtual_keyboard_v1`.
- [ ] Tối ưu hóa trải nghiệm trên các Wayland Compositor hiện đại: **Hyprland, Sway, Niri, COSMIC Desktop**.

#### 3. Hoàn Thiện Hệ Thống Đóng Gói & Phân Phối (Packaging & Distribution)
- [ ] Đưa gói cài đặt lên **Arch Linux AUR** (`fcitx5-vnchkey-git`).
- [ ] Xây dựng PPA chính thức cho **Ubuntu / Debian / Linux Mint**.
- [ ] Xây dựng Copr Repository cho **Fedora / Red Hat**.
- [ ] Tạo gói Flatpak Addon cho môi trường sandboxed desktop.

#### 4. Sao Lưu & Đồng Bộ Đám Mây (Cloud / Git Backup Sync)
- [ ] Bổ sung tính năng sao lưu / đồng bộ tự động bảng gõ tắt (Macros) và từ điển người dùng (Dictionary) qua Git riêng tư hoặc dịch vụ đám mây cá nhân.

---

## 🤝 Đóng Góp & Góp Ý (Contributing)

Mọi ý tưởng đóng góp, phản hồi trải nghiệm gõ hoặc đề xuất tính năng mới đều được hoan nghênh nồng nhiệt!
- **Báo lỗi & Thảo luận**: Mở Issue hoặc Pull Request tại [GitHub Repository](https://github.com/no-vici/fcitx5-vnchkey).
- **Mã nguồn**: Tuân thủ giấy phép tự do **GNU GPLv3**.

