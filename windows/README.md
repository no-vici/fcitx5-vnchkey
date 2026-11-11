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
- **Mẹo hủy gõ tắt thông minh (Smart Undo)**: Nếu bạn vô tình gõ từ viết tắt và nó nở thành câu dài, chỉ cần **bấm `Backspace` một lần**, từ mở rộng sẽ tự thu gọn lại nguyên bản ban đầu!

---

### 3. Từ Điển Cá Nhân (User Dictionary) — Gõ Tiếng Anh / Mã Code Không Bị Dính Dấu
Khi gõ code hoặc văn bản có lẫn từ tiếng Anh/thuật ngữ kỹ thuật (như `server`, `false`, `const`, `input`, `reset`, `docker`), bộ gõ có thể nhận nhầm các phím `s, f, r, x, j` thành dấu tiếng Việt.

- **Kích hoạt**: Chuột phải vào icon $\to$ tích chọn **"Tự động khôi phục Tiếng Anh"** và **"Bật Từ Điển Cá Nhân"**.
- **Thêm từ vào từ điển ngoại lệ**:
  1. Chọn **"Chỉnh sửa Từ Điển Cá Nhân (dictionary.conf)..."** $\to$ Notepad sẽ mở ra.
  2. Gõ mỗi từ tiếng Anh hoặc tên riêng trên một dòng:
     ```text
     # Mỗi từ một dòng (chữ thường)
     github
     vscode
     react
     vue
     kubernetes
     docker
     figma
     powershell
     python
     rust
     database
     ```
  3. Lưu file (`Ctrl + S`) và bấm **"Nạp lại Gõ Tắt & Từ Điển"**.
- Từ nay, khi bạn gõ những từ này rồi bấm phím cách (Space) hoặc dấu chấm phẩy (`;`), bộ gõ sẽ tự động trả về nguyên bản tiếng Anh, tuyệt đối không bị dính dấu sai chính tả.

---

### 4. Tùy Chọn Tốc Ký (Quick Consonants)
Nếu bạn thường xuyên gõ văn bản nhanh, hãy bật các tùy chọn trong menu **"Tùy chọn Tốc ký..."**:
- **Quick Telex**: Gõ lặp phụ âm để ra phụ âm ghép (`cc` $\to$ `ch`, `gg` $\to$ `gi`, `kk` $\to$ `kh`, `nn` $\to$ `ng`, `pp` $\to$ `ph`, `qq` $\to$ `qu`, `tt` $\to$ `th`).
- **Phụ âm đầu nhanh**: Gõ `f` $\to$ `ph`, `j` $\to$ `gi`, `w` $\to$ `qu` khi ở đầu từ.
- **Phụ âm cuối nhanh**: Gõ `g` $\to$ `ng`, `h` $\to$ `nh`, `k` $\to$ `ch` sau nguyên âm.

---

### 5. Khôi Phục Từ Bằng Phím `Esc` (Dokey-Style Escape)
Khi đang gõ một từ tiếng Việt nhưng muốn hủy toàn bộ dấu vừa bỏ để quay lại ký tự phím thô ban đầu, bạn chỉ cần **bấm phím `Esc`**. Từ đó sẽ hoàn nguyên ngay lập tức về dạng chưa bỏ dấu mà không cần phải xóa đi gõ lại từ đầu.

---

### 6. Khởi Động Cùng Windows & Chế Độ Portable Tiện Dụng
- **Khởi động cùng máy**: Chuột phải icon $\to$ tích chọn **"Khởi động cùng Windows"**. Ứng dụng sẽ tự khởi động ngầm êm ái mỗi khi mở máy.
- **Chế độ Portable (Chép USB mang đi)**:
  - `VnCharKey.exe` ưu tiên lưu toàn bộ cấu hình (`config.conf`, `macros.conf`, `dictionary.conf`) **ngay trong cùng thư mục với file `.exe`**.
  - Bạn chỉ cần copy cả thư mục chứa `VnCharKey.exe` vào USB hoặc lưu ở Google Drive / OneDrive, mang sang máy khác là sử dụng được ngay với đầy đủ phím tắt và danh sách gõ tắt quen thuộc của bạn, không cần cài đặt hay phân quyền Administrator.

---

## 🛠️ Biên Dịch Từ Mã Nguồn (Dành cho Lập Trình Viên)

Yêu cầu máy cài [Rust](https://rustup.rs/):
```powershell
cd windows
cargo build --release
```
File thực thi độc lập sẽ nằm tại: `windows\target\release\VnCharKey.exe`.
