# Đánh Giá Khách Quan: vnchKey Đã Phải Là Bộ Gõ Tốt Nhất Chưa?
*(Tài liệu tham chiếu kỹ thuật & Tiêu chí kiểm định thực tế trước thềm phiên bản v0.3.0)*

---

## 1. Lời Mở Đầu: Nhìn Nhận Thẳng Thắn & Không Ảo Tưởng

Trước câu hỏi: **"Liệu vnchKey đã phải là bộ gõ tiếng Việt tốt nhất trên thị trường hiện nay chưa?"**, câu trả lời chân thành và trung thực về mặt kỹ thuật là:

> **Chưa thể tự mãn khẳng định là "số 1 tuyệt đối ở mọi ngóc ngách" ngay lúc này. Tuy nhiên, về mặt kiến trúc phần mềm hiện đại, độ tích hợp hệ thống (Linux Wayland/X11) và khả năng tổng hòa các tính năng tiên tiến nhất, vnchKey đang tiệm cận vị trí bộ gõ toàn diện và mạnh mẽ bậc nhất trên Linux.**

Quyết định **chưa vội nâng phiên bản lên v0.3.0** mà dành thời gian kiểm thử thực chiến (stress test) mọi kịch bản đời thực là quyết định chuẩn mực của một kỹ sư phần mềm.

---

## 2. Bảng So Sánh Chi Tiết Với Các Tượng Đài Lớn

| Tiêu chí | **vnchKey** (Hiện tại) | **IBus/Fcitx5-Bamboo** | **fcitx5-unikey** | **fcitx5-lilypad** | **VietC** |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Ngôn ngữ Core** | **100% Rust** (`vnchkey-core`) | Go / C | C++ (Cổ điển) | Rust | Rust |
| **Bóc tách âm tiết** | **5 thành phần chuẩn hóa** (Onset-Glide-Nucleus-Coda-Tone) | State-tree phức hợp | Bảng tra cứu chuỗi | Bảng quy tắc ký tự | Finite State Machine |
| **Chế độ xuất chữ** | **Hybrid Dual-Mode** (Direct Micro-diff + Atomic Preedit) | Multi-mode (Chế độ 1 - 5) | Direct (Micro-diff cổ điển) | Direct 100% (Uinput) | Engine lõi (chưa có addon) |
| **Nhớ chế độ theo App** | **Tự động 100%** (Per-App Memory) | Không (Phải cấu hình rule) | Không có | Có (Lilypad config) | Không |
| **Đổi dấu phụ giữ thanh** (`â ↔ ă`, `ô ↔ ơ`) | **Hỗ trợ đầy đủ cả Telex & VNI** | Hỗ trợ một phần | Không hỗ trợ | Không hỗ trợ | **Hỗ trợ chuẩn mực** |
| **Bảo vệ Stop Codas** (`-c, -ch, -p, -t`) | **Chống gõ sai dấu làm hỏng từ** | Có kiểm tra ngữ âm | Rớt dấu | Bị gõ đè | Tự do (dễ sinh từ lạ) |
| **Trải nghiệm Macro** | **Hoàn tác 1 chạm + Khóa lặp vô hạn** | Hoàn tác từng ký tự | Hoàn tác cơ bản | Chưa chuyên sâu | Chưa có macro |
| **Giao diện Cài Đặt (GUI)** | **Native Rust (`egui`)** + System App Picker | Trình cấu hình generic | Trình cấu hình generic | Native Rust GUI | Chưa có GUI |
| **Thời gian thực chiến** | Đang phát triển tích cực | **> 7 năm (Hàng triệu người dùng)** | > 20 năm | ~ 2 năm | Dự án thử nghiệm mới |

---

## 3. Những Điểm vnchKey Đang Thực Sự Dẫn Đầu

1. **Kiến Trúc Hybrid Dual-Mode Thực Dụng Triệt Để**:
   - Thay vì tranh cãi giữa việc dùng thuần Direct hay thuần Preedit, vnchKey thừa nhận giới hạn vật lý của Web DOM và gán đúng việc cho đúng công cụ:
     - **Direct Mode (Micro-diff)**: Độ trễ bằng $0$, hoàn hảo cho Terminal, Neovim, VS Code, WezTerm.
     - **Preedit Mode (Atomic Commit)**: Gom từ nội dòng, loại bỏ 100% nguy cơ lặp chữ/nhảy con trỏ trên Canvas/Web (Google Docs, Sheets, Chrome, Slack, Discord).
     - **Per-App Memory**: Tự động chuyển đổi mượt mà không cần người dùng can thiệp thủ công.
2. **Bộ Bóc Tách Âm Tiết 5 Thành Phần (`syllable_parser.rs`)**:
   - Phân giải cấu trúc âm tiết rõ ràng, hỗ trợ dời dấu thanh động (Dynamic Tone Shifting) khi người dùng mở rộng nguyên âm (`ho` + `s` $\to$ `hó` $\xrightarrow{+\text{a}}$ `hoá`/`hóa`).
   - Hỗ trợ song song cả 2 phong cách dấu: Mới (Modern) và Cổ Điển (Classic).
   - Nhận diện các cụm phụ âm mở rộng của đồng bào Tây Nguyên (`kr`, `pr`, `dr`, `đr`, `br`, `kl`, `pl`, `bl`).
3. **Hoán Đổi Dấu Phụ Bảo Toàn Thanh Điệu (Kế Thừa Tinh Hoa VietC)**:
   - Cho phép hoán chuyển nhanh dạng mũ nguyên âm mà không cần xóa đi gõ lại (`ấm` + `w` $\to$ `ắm`, `bốn` + `w` $\to$ `bớn`, `mặt` + `a` $\to$ `mật`).
4. **Bảo Vệ Ngữ Âm Học (Phonetic Tone Protection)**:
   - Ngăn chặn triệt để hành vi gõ sai phím thanh điệu trên các âm cuối tắc (`được` + `f` $\to$ `đượcf`, không phá hoại từ gốc thành `đườc`).
5. **Cơ Chế Khóa Lặp Vô Hạn Cho Macro (Infinite Loop Suppression)**:
   - Giải quyết bài toán nan giải: Khi bung `vn` $\to$ `Việt Nam `, nếu nhấn Backspace để phục hồi `vn`, nhấn tiếp Space sẽ không bị nuốt vào vòng lặp bung macro vô tận.
6. **Bộ Cài Đặt Độc Lập Siêu Tốc (Rust + `egui`)**:
   - Không phụ thuộc GNOME Settings hay KDE System Settings cồng kềnh.
   - Tính năng **System App Picker** tự quét `/proc` và `.desktop` giúp thêm ứng dụng bằng thao tác nhấp đúp (Double-Click) cực kỳ hiện đại.

---

## 4. Những Lĩnh Vực Cần Thử Thách & Các Điểm Cần Cải Thiện

Trước khi khẳng định là bộ gõ hoàn hảo nhất, vnchKey cần vượt qua các rào cản:

### 4.1. Bề dày thực chiến chưa bằng Bamboo hay UniKey
- **Bamboo** và **UniKey** có lợi thế khổng lồ về số lượng người dùng qua nhiều năm. Hàng vạn tình huống gõ dị biệt (người gõ lướt cực nhanh, gõ sai rồi quay lại sửa giữa từ, văn bản nửa Tây nửa Ta dính liền nhau...) đã được họ vá qua hàng trăm bản cập nhật.
- Dù mã nguồn của vnchKey rất trong sáng và có kiểm thử đơn vị bao phủ, chỉ có thời gian gõ thực tế mới bộc lộ hết các edge cases tiềm ẩn.

### 4.2. Mức độ hợp nhất của VniEngine với Syllable Parser
- `syllable_parser` hiện tại đã được nối trực tiếp và phát huy tối đa công năng trong `TelexEngine`.
- Trong khi đó, `VniEngine` tuy đã hỗ trợ đầy đủ các phím số 1-9, 0 và hoán đổi dấu phụ, nhưng vẫn cần tiếp tục được tái cấu trúc (refactor) để vận hành 100% dựa trên State Machine của `syllable_parser` như Telex.

### 4.3. Game Fullscreen Wayland (Exclusive Grab)
- Trên Wayland, các tựa game đồ họa nặng chạy qua Proton/Wine thường chiếm quyền độc quyền thiết bị nhập liệu (Exclusive Input Grab). Một số bộ gõ sử dụng daemon kernel uinput (như Lilypad) có thể bơm phím trực tiếp vào game mượt mà hơn bộ gõ hoạt động ở tầng Fcitx5 Application Context.

---

## 5. Checklist Kịch Bản Kiểm Thử Trước Khi Bump Version v0.3.0

Để tự tin đóng dấu phát hành **v0.3.0**, người phát triển và kiểm thử nên đưa vnchKey vào các bài thử nghiệm thực địa sau:

### 5.1. Soạn thảo văn bản chuyên nghiệp (Office & Web Apps)
- [ ] **Google Docs**: Gõ văn bản tốc độ cao (>90 WPM) ở chế độ Preedit xem có bao giờ bị nhân đôi chữ hoặc rơi con trỏ không.
- [ ] **Google Sheets**: Sửa dữ liệu trong ô tính, kiểm tra xem phím Enter/Tab và cơ chế xoá ký tự có làm vỡ công thức không.
- [ ] **WPS Office / LibreOffice**: Kiểm tra khung chữ gõ vào, xác nhận không bị hiện tượng 2 dòng chữ hiển thị đè lên nhau.

### 5.2. Môi trường Lập Trình & Terminal
- [ ] **Neovim / Vim**:
  - Gõ lệnh trong Normal Mode (`ci"`, `cw`, `f`, `r`).
  - Thoát Insert Mode bằng `jk` hoặc `Esc` xem có bị kẹt buffer tiếng Việt không.
  - Thử tính năng tách âm tiết cuối khi con trỏ đứng sát dấu ngoặc hoặc từ khóa code (`printf("xin chao")`).
- [ ] **VS Code / Zed / Cursor**: Gõ comment tiếng Việt xen kẽ mã nguồn tiếng Anh ở chế độ Direct (Micro-diff), kiểm tra độ nhạy và độ mượt của con trỏ.

### 5.3. Trình duyệt & Ứng dụng Trò Chuyện (Chat)
- [ ] **Firefox & Chrome URL Bar (Awesomebar)**: Gõ tìm kiếm nhanh, kiểm tra xem phím Backspace đầu tiên có bị nuốt hay giật không.
- [ ] **Slack / Discord / Telegram Web**: Gõ nhanh các đoạn tin nhắn có emoji, dấu ngoặc, link URL dính liền chữ.

### 5.4. Các góc khuất ngữ âm học (Linguistic Edge Cases)
- [ ] **Hoán đổi dấu phụ**: Thử gõ `ấm` + `w` $\to$ `ắm`, `bốn` + `w` $\to$ `bớn`, `mặt` + `a` $\to$ `mật`.
- [ ] **Gõ sai dấu trên âm cuối tắc**: Thử gõ `được` + `f`, `bác` + `r`, `mất` + `x` $\to$ từ gốc phải được bảo toàn nguyên vẹn.
- [ ] **Từ ghép & Địa danh lạ**: Gõ các từ Nam Bộ (`dzìa`, `chánh`) và địa danh Tây Nguyên (`Đắk Lắk`, `Cư M'gar`, `M'Đrắk`).
- [ ] **Lật từ tức thì (Flip Word)**: Bấm `Ctrl + Shift + Z` kiểm tra việc chuyển đổi qua lại giữa tiếng Việt và mã thô tiếng Anh (`thể` $\longleftrightarrow$ `theer`).

---

## 6. Lời Kết

vnchKey đã có một **bộ khung nền tảng xuất sắc** mà ít bộ gõ nào có được: mã nguồn an toàn tuyệt đối viết bằng Rust, tích hợp native Fcitx5, thuật toán micro-diff tối ưu độ trễ, và bộ parser ngữ âm học 5 thành phần.

Khi bạn hoàn thành đợt kiểm thử thực chiến này và xác nhận mọi kịch bản trên đều vận hành trơn tru, vnchKey sẽ hoàn toàn tự tin bước lên **cột mốc v0.3.0** với tư cách là một trong những bộ gõ tiếng Việt tốt nhất, chỉn chu nhất mà cộng đồng Linux từng có.
