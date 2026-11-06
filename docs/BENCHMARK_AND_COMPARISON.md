# Đánh Giá Khách Quan & So Sánh Kiến Trúc Bộ Gõ Tiếng Việt Trên Linux

Tài liệu này ghi nhận báo cáo phân tích kỹ thuật khách quan và thẳng thắn về bức tranh toàn cảnh các bộ gõ tiếng Việt trên hệ điều hành Linux (X11 & Wayland), đồng thời xác định vị thế, thế mạnh thực sự và những hạn chế mà **vnchKey** cần tiếp tục khắc phục.

---

## 1. Bảng So Sánh Tổng Thể

| Tiêu chí | **vnchKey** (Dự án này) | **IBus/Fcitx5-Bamboo** *(Lương Thanh Lam)* | **fcitx5-unikey** *(Cổ điển)* | **fcitx5-lilypad** *(chiconcota)* | **OpenKey** *(Mai Vũ Tuyên)* |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Ngôn ngữ Core** | **100% Rust** (`vnchkey-core`) | Go (IBus) / C (Fcitx5) | C++ (Core UniKey 2000s) | Rust (`lilypad-core`) | C++ |
| **Kiến trúc Tích hợp** | Native Fcitx5 C-ABI Addon | IBus Engine / Fcitx5 Module | Fcitx5 Module | Fcitx5 + Uinput Kernel Daemon | Standalone X11 / XTest |
| **Chế độ xuất chữ** | **Hybrid Dual-Mode** (Direct + Preedit) | Multi-mode (Chế độ 1 - 5) | Direct (Micro-diff cổ điển) | Direct Clean Commit 100% | Direct (Hook hệ thống) |
| **Hỗ trợ Wayland** | **Tuyệt đối** (GNOME, KDE, Hyprland, Sway) | Tốt (nhưng dễ kẹt phím Backspace) | Trung bình (lỗi trên Canvas/Web) | Rất tốt (nhờ `/dev/uinput`) | Kém trên Wayland thuần |
| **Bộ cài đặt GUI** | **Native Rust (`egui`)** + App Picker | Generic Fcitx / IBus GUI | Generic Fcitx GUI | Native Rust GUI | Qt GUI |
| **Nhớ chế độ theo App** | **Tự động 100%** (Per-App Mode Memory) | Thủ công (Cần cấu hình rules) | Không có | Có (Lilypad config) | Không có |
| **Độ dày dạn thực chiến** | Đang phát triển (v0.2.x - v0.3.x) | **Hơn 7 năm (Số 1 trên Linux)** | Hơn 20 năm | 1 - 2 năm | Hơn 5 năm (mạnh trên Win/Mac) |

---

## 2. Phân Tích Chuyên Sâu Từng Bộ Gõ

### 2.1. IBus / Fcitx5-Bamboo
- **Ưu điểm lớn nhất**: Độ dày dạn thực chiến về ngữ âm. Trải qua hơn 7 năm phát triển và hàng triệu người dùng Linux tại Việt Nam, Bamboo có bộ luật ngữ âm và kiểm soát lỗi gõ cực kỳ uyên bác, xử lý được vô vàn thói quen gõ quái dị, gõ nhanh, gõ ẩu của người dùng.
- **Hạn chế cố hữu**:
  - Dựa nhiều vào cơ chế *Surrounding Text* và *Forward-Delete*. Khi gặp ứng dụng không hỗ trợ surrounding text (như terminal cũ, app Qt hoặc web rich text), Bamboo phát chuỗi Backspace liên tục dễ dẫn tới giật lag, trôi con trỏ hoặc nuốt chữ.
  - Phân chia 5 chế độ gõ (Mode 1 đến Mode 5) khiến người dùng phổ thông bối rối khi không biết phần mềm nào nên gán Mode nào.

### 2.2. fcitx5-unikey
- **Ưu điểm**: Nhẹ, ổn định tuyệt đối với các phần mềm soạn thảo truyền thống nhờ lõi UniKey C++ huyền thoại.
- **Hạn chế**: Kiến trúc không được nâng cấp cho môi trường Web hiện đại. Hoàn toàn bất lực trước hiện tượng lặp chữ trên nền Canvas/DOM của Google Docs, StackEdit, Slack, Discord. Không có tính năng tự nhớ chế độ theo ứng dụng.

### 2.3. fcitx5-lilypad
- **Ưu điểm**: Tiên phong tư duy Direct Clean Commit (loại bỏ hoàn toàn gạch chân Preedit) kết hợp cảm biến nhịp gõ mili-giây (IKI Engine) và Uinput Server Daemon (bơm phím trực tiếp từ `/dev/uinput` kernel).
- **Hạn chế**:
  - Đòi hỏi quyền truy cập `/dev/uinput` (yêu cầu cấu hình udev rule hoặc chạy daemon quyền đặc biệt).
  - Ép buộc gõ Direct 100% trên Web dẫn tới giới hạn toán học: JavaScript Event Loop của trình duyệt khi gõ tốc độ cao (>80 WPM) không thể đồng bộ 0ms với phím Backspace nhân tạo, vẫn có rủi ro nhảy chữ trên Google Docs.

### 2.4. OpenKey
- **Ưu điểm**: Xử lý macro và ngữ âm rất chỉn chu, có tính năng hoàn tác Macro thông minh khi xóa dấu cách.
- **Hạn chế**: Linux chỉ là nền tảng phụ so với Windows và macOS. Trên Linux, OpenKey hoạt động chủ yếu thông qua X11/XTest, khả năng tích hợp vào Wayland hiện đại còn nhiều rào cản.

---

## 3. vnchKey Đang Đứng Ở Đâu?

### 3.1. Những Thế Mạnh Thực Sự Của vnchKey
1. **Kiến Trúc Hybrid Dual-Mode Thực Dụng**:
   - Thừa nhận thẳng thắn giới hạn của Direct Mode trên Web và Preedit trên Editor để xây dựng cơ chế **Per-App Mode Memory** tự động. Người dùng vào Terminal/Neovim tự gõ Direct 0ms, chuyển sang Firefox/Chrome tự gõ Preedit an toàn tuyệt đối.
2. **Giao Diện Cài Đặt (GUI) Vượt Trội**:
   - `fcitx5-vnchkey-settings` (100% Rust + egui) khởi động trong chớp mắt.
   - **System App Picker**: Quét toàn bộ desktop app và tiến trình `/proc` hệ thống, nhấp đúp để gán chế độ gõ thay vì phải nhớ và nhập thủ công tên nhị phân.
3. **Các Tiện Ích Độc Quyền Cho Dân Kỹ Thuật**:
   - Phím tắt lật từ tức thì `Ctrl + Shift + Z` (Flip Word).
   - Bảo vệ `gnome-shell` proxy cache trên GNOME Wayland.
   - Ép vẽ khung nổi cho WPS Office, chống lỗi gõ tàng hình.
   - Dọn sạch hậu tố autocomplete trên Google Sheets.
   - **Hoàn tác Macro một chạm (Macro Revert on Backspace)** kết hợp **Chống lặp vô tận (Loop Suppression)**.

### 3.2. Những Điểm vnchKey Vẫn Thua Và Cần Khắc Phục (Nhìn Nhận Thẳng Thắn)
1. **Độ Dày Dạn Ngữ Âm (Battle-tested Rules)**:
   - Dù vnchKey đã có ~6.750 từ gốc và kiểm tra âm học, bộ engine âm tiết vẫn chưa trải qua hàng triệu giờ gõ thực tế như Bamboo. Chắc chắn sẽ còn những trường hợp biên (edge cases) chưa được lường hết khi người dùng gõ nhanh hoặc gõ sai ngữ pháp.
2. **Chơi Game Toàn Màn Hình Trên Wayland (Exclusive Grab)**:
   - Một số tựa game fullscreen trên Wine/Proton/Steam chiếm toàn quyền bàn phím và bỏ qua Fcitx5. Lilypad với Uinput Kernel Daemon có thể đưa phím trực tiếp vào game tốt hơn vnchKey.
3. **Hệ Thống Phân Phối & Đóng Gói (Distribution)**:
   - Chưa có gói chính thức trên AUR (`fcitx5-vnchkey-git`), chưa có PPA (Ubuntu/Debian) hay Copr (Fedora). Người dùng hiện tại vẫn phải build từ source hoặc cài thủ công qua script.

---

## 4. Kế Hoạch Học Hỏi & Nâng Cấp Ngữ Âm Từ Bamboo

Để đưa độ ổn định ngữ âm của vnchKey lên ngang tầm với Bamboo, kế hoạch triển khai sẽ gồm 3 giai đoạn:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Trích Xuất & Phân Loại Quy Tắc Ngữ Âm (Phonetic Rules)   │
│    - Bóc tách bộ luật chuyển đổi âm tiết (Triphone / Syllable)│
│    - Luật đặt dấu tự do (Free-marking) & di chuyển dấu      │
│    - Luật triệt tiêu biến âm khi vi phạm ngữ âm học         │
├─────────────────────────────────────────────────────────────┤
│ 2. Triển Khai Kiến Trúc Thuần Rust Trong vnchkey-core      │
│    - syllable_parser.rs: Bóc tách Onset - Nucleus - Coda    │
│    - phonetic_rules.rs: Ma trận tổ hợp âm hợp lệ            │
│    - speller.rs: Bộ kiểm định và ra quyết định hoàn tác     │
├─────────────────────────────────────────────────────────────┤
│ 3. Thừa Hưởng Toàn Bộ Test Suite Thực Chiến Của Bamboo      │
│    - Chuyển ngữ hàng trăm test cases gõ thực tế sang #[test]│
│    - Kiểm thử tự động hồi quy liên tục (CI/CD Regression)   │
└─────────────────────────────────────────────────────────────┘
```
