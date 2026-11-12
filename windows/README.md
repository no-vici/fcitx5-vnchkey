# VnCharKey for Windows — Hướng Dẫn Sử Dụng & Tối Ưu Năng Suất (Productivity Guide)

**VnCharKey** là bộ gõ Tiếng Việt độc lập (Portable), siêu nhẹ (~200 KB) viết bằng Native Rust cho Windows 10 & 11. Bộ gõ tập trung tối đa vào **tốc độ, độ ổn định và trải nghiệm gõ hàng ngày** cho lập trình viên, nhân viên văn phòng và người dùng chuyên nghiệp.

---

## ⚡ Các Tiện Ích Tăng Tốc Độ Gõ (Productivity Boosters)

### 1. Phím Tắt Chuyển Việt / Anh Hoàn Toàn Tự Do
Bạn không bị bó buộc vào vài phím tắt cố định. Bạn có thể gán bất kỳ tổ hợp phím nào quen tay nhất:
- **Cách đặt phím tắt**: Nhấp chuột phải vào icon ở khay hệ thống (System Tray) $\to$ chọn **"Phím tắt chuyển Việt / Anh"** $\to$ bấm **"⌨️ Bấm để đặt phím tắt mới (Tự do)..."**.
- Một hộp thoại hiện ra, bạn chỉ cần bấm tổ hợp phím mong muốn trên bàn phím:
  - Các tổ hợp modifier: `Ctrl + Shift`, `Alt + Shift`, `Ctrl + Alt`...
  - Các tổ hợp có phím ký tự: `Alt + Z`, `Ctrl + Space`, `Win + Space`, `Ctrl + Alt + V`...
  - Bấm `Esc` nếu muốn hủy.
- **Chuyển nhanh [V] / [E] bằng chuột**: Nhấp chuột trái trực tiếp lên biểu tượng ở Taskbar để bật/tắt tức thì.

---

### 2. Gõ Tắt Siêu Tốc (Macros) — Viết Tắt 2–3 Chữ Thành Cả Đoạn Văn
Tính năng Gõ Tắt giúp bạn tiết kiệm hàng ngàn lượt bấm phím mỗi ngày cho các cụm từ, email, câu chào lặp đi lặp lại.

- **Kích hoạt**: Chuột phải vào icon $\to$ tích chọn **"Bật Gõ Tắt (Macro)"**.
- **Chỉnh sửa danh sách gõ tắt**:
  1. Chọn **"Chỉnh sửa bảng Gõ Tắt (macros.conf)..."** $\to$ file sẽ tự mở bằng Notepad.
  2. Thêm các từ viết tắt theo cú pháp:
     ```ini
     # Cú pháp: tu_viet_tat=cụm từ đầy đủ
     vn=Việt Nam
     td=Tự Do
     sg=Sài Gòn
     dc=được
     ko=không
     ng=người
     hn=hôm nay
     cv=công việc
     ema=bodatvang@gmail.com
     sdt=số điện thoại
     stk=số tài khoản
     diachi=Số 123 Đường Tự Do, Quận 1, Sài Gòn
     ```
  3. Bấm `Ctrl + S` lưu lại trong Notepad.
  4. Chuột phải icon $\to$ bấm **"Nạp lại Gõ Tắt & Từ Điển"** để áp dụng ngay.
- **Cơ chế Hủy Gõ Tắt thông minh 2 bước**:
  - Khi bung cụm từ gõ tắt, bộ gõ tự động thêm dấu cách (`Space`) phía sau.
  - **Bấm `Backspace` lần 1**: Xóa dấu cách vừa bung để bạn có thể gõ thêm dấu thanh hoặc dấu câu; bộ gõ sẽ tự động nuốt khoảng trắng để dấu dính liền sát vào cụm từ vừa bung.
  - **Bấm `Backspace` lần 2**: Hoàn nguyên (undo) toàn bộ cụm từ mở rộng trở về đúng chuỗi phím tắt ban đầu mà bạn đã gõ!

---

### 3. Đảo Từ Nhanh (Flip Word) — Tùy Chỉnh Phím Tắt Tự Do & Bảo Vệ Lệnh Redo
Khi đang gõ một từ Tiếng Việt (hoặc chuỗi phím thô), bạn muốn lật qua lại giữa chữ Tiếng Việt có dấu và chuỗi ký tự gõ phím gốc mà không cần xóa đi gõ lại:
- **Tổ hợp phím mặc định**: **`Ctrl + Shift + Z`** (hoặc tự chọn `Alt + X`, `Ctrl + Alt + Z`, hay thu bất kỳ tổ hợp phím nào qua menu **"Phím tắt Đảo Từ (Flip Word)"**).
- **Thông minh, không nuốt lệnh Redo của Editor**: Phím tắt Đảo Từ **chỉ kích hoạt khi bạn đang thực sự gõ dở một từ trong bộ đệm** (`has_active_word()`). Khi bộ đệm đang rỗng (sau khi bạn đã bấm Space hoặc đang chỉnh sửa tài liệu), `Ctrl + Shift + Z` sẽ được trả thẳng về cho ứng dụng (VS Code, Word, trình duyệt...) thực hiện lệnh Redo bình thường!
- **Ví dụ**:
  - Gõ `tiếng` $\xrightarrow{\text{Ctrl+Shift+Z}}$ đảo thành **`tieengs`**
  - Bấm tiếp `Ctrl+Shift+Z` $\to$ lật ngược lại thành **`tiếng`**
  - Gõ `việt` $\longleftrightarrow$ `vieetj`, `được` $\longleftrightarrow$ `dduowjc`

---

### 4. Tự Động Học Thói Quen Sửa Từ (Adaptive Habit Learner — Log 100 MB) & Từ Điển Cá Nhân
Khi gõ code hoặc văn bản có lẫn từ tiếng Anh/thuật ngữ kỹ thuật hay tên riêng lạ, nếu bộ gõ lỡ biến thành dấu tiếng Việt và bạn sửa lại (bằng cách bấm `Backspace` xóa đi gõ lại chữ không dấu, gõ đúp phím dấu `ss`/`ff`/`ww`, bấm `Esc`, hoặc bấm `Flip Word`) từ **2 lần trở lên**:
- **Tự động đưa vào Whitelist**: `VnCharKey` sẽ tự động học từ đó, ghi nối vào file `dictionary.conf` và lưu nhật ký tại `habit.log` (dung lượng tối đa **100 MB** tự động xoay vòng FIFO).
- **Gặp lại là cho qua (Pass-through)**: Từ lần sau trở đi, khi bạn gõ lại từ đó, bộ gõ sẽ tự động giữ nguyên bản tiếng Anh mà bạn không bao giờ phải xóa đi sửa lại nữa!
- **Thêm từ thủ công vào từ điển ngoại lệ**:
  1. Chuột phải vào icon $\to$ tích chọn **"Tự động khôi phục Tiếng Anh"** và **"Bật Từ Điển Cá Nhân"**.
  2. Chọn **"Chỉnh sửa Từ Điển Cá Nhân (dictionary.conf)..."** $\to$ Notepad sẽ mở ra.
  3. Gõ mỗi từ tiếng Anh hoặc tên riêng trên một dòng, lưu file (`Ctrl + S`) và bấm **"Nạp lại Gõ Tắt & Từ Điển"**.

---

### 5. Tùy Chọn Tốc Ký (Quick Consonants)
Nếu bạn thường xuyên gõ văn bản nhanh, hãy bật các tùy chọn trong menu **"Tùy chọn Tốc ký..."**:
- **Quick Telex**: Gõ lặp phụ âm để ra phụ âm ghép (`cc` $\to$ `ch`, `gg` $\to$ `gi`, `kk` $\to$ `kh`, `nn` $\to$ `ng`, `pp` $\to$ `ph`, `qq` $\to$ `qu`, `tt` $\to$ `th`).
- **Phụ âm đầu nhanh**: Gõ `f` $\to$ `ph`, `j` $\to$ `gi`, `w` $\to$ `qu` khi ở đầu từ.
- **Phụ âm cuối nhanh**: Gõ `g` $\to$ `ng`, `h` $\to$ `nh`, `k` $\to$ `ch` sau nguyên âm.

---

### 6. Khôi Phục Từ Bằng Phím `Esc` (Dokey-Style Escape)
Khi đang gõ một từ tiếng Việt nhưng muốn hủy toàn bộ dấu vừa bỏ để quay lại ký tự phím thô ban đầu, bạn chỉ cần **bấm phím `Esc`**. Từ đó sẽ hoàn nguyên ngay lập tức về dạng chưa bỏ dấu mà không cần phải xóa đi gõ lại từ đầu.

---

### 7. Khởi Động Cùng Windows & Chế Độ 100% Portable
- **Khởi động cùng máy**: Chuột phải icon $\to$ tích chọn **"Khởi động cùng Windows"**. Ứng dụng sẽ tự khởi động ngầm êm ái mỗi khi mở máy.
- **Chế độ 100% Portable (Lưu tất cả tại một nơi duy nhất)**:
  - `VnCharKey.exe` luôn lưu toàn bộ dữ liệu (`config.conf`, `macros.conf`, `dictionary.conf`, và `habit.log`) **trực tiếp ngay trong cùng thư mục chứa file `VnCharKey.exe`** (tuyệt đối không rải rác vào `%APPDATA%`).
  - Bạn chỉ cần copy cả thư mục chứa `VnCharKey.exe` vào USB hoặc ổ đĩa đám mây, mang sang bất kỳ máy nào khác là sử dụng được ngay với đầy đủ phím tắt, bảng gõ tắt và toàn bộ từ vựng mà bộ gõ đã tự học được.

---

## 🛠️ Biên Dịch Từ Mã Nguồn (Dành cho Lập Trình Viên)

Yêu cầu máy cài [Rust](https://rustup.rs/):
```powershell
cd windows
cargo build --release
```
File thực thi độc lập sẽ nằm tại: `windows\target\release\VnCharKey.exe`.
