# VnCharKey for Windows

**VnCharKey** là phiên bản bộ gõ Tiếng Việt tốc độ cao, siêu nhẹ và trung tính dành cho hệ điều hành **Windows (Windows 10 / Windows 11)**, sử dụng chung lõi thuật toán Rust [`vnchkey-core`](../crates/vnchkey-core) với biểu tượng **Nón Lá Việt Nam (Conical Hat)** thanh lịch trên nền trong suốt.

---

## ✨ Điểm Nổi Bật & Tinh Thần Tự Do

- **Biểu tượng Nón Lá Nền Trong Suốt & Tên gọi Trung Tính (`VnCharKey`)**: 
  - Biểu tượng Nón Lá vàng ấm cùng ký tự `V` xanh ngọc và viền bo mềm mại trên nền trong suốt (transparent), hiển thị sắc nét trên cả Light theme và Dark theme của Windows Taskbar.
- **Menu Khay Hệ Thống (System Tray) Đầy Đủ Tùy Chọn**:
  - **4 Kiểu gõ linh hoạt**: Tự động (`Auto: Telex + VNI`), `Telex`, `VNI`, và `Simple Telex`.
  - **Bảng mã & Dấu thanh**: Chuyển đổi giữa Unicode Dựng Sẵn (NFC) / Unicode Tổ Hợp (NFD) và Dấu mới (`oà`, `uý`) / Dấu cũ (`hòa`, `ủy`).
  - **Phím tắt chuyển Việt / Anh hoàn toàn tự do (Live Keystroke Recorder)**:
    - Bấm **"⌨️ Bấm để đặt phím tắt mới (Tự do)..."** rồi bấm bất kỳ tổ hợp phím nào bạn muốn (`Ctrl + Alt + V`, `Win + Shift + Space`, `Alt + Shift`, `Ctrl + Shift`, `Alt + Z`, `Ctrl + Space`...) hoặc chọn danh sách phím tắt thông dụng dựng sẵn.
    - Phím tắt được lưu vĩnh viễn trong file cấu hình `%APPDATA%\VnCharKey\config.conf`.
  - **Tính năng thông minh**: Bật/tắt *Kiểm tra Chánh Tả*, *Tự động khôi phục Tiếng Anh*, *Hoàn nguyên từ bằng Esc*, *Gõ `[` `]` thành `ơ` `ư`*.
  - **Quản lý Gõ Tắt (Macros) & Từ Điển Cá Nhân**: Mở và chỉnh sửa trực tiếp `macros.conf` và `dictionary.conf` bằng Notepad, nút "Nạp lại Gõ Tắt & Từ Điển" tức thì.
- **Thuật toán Micro-Diff chống chớp giật chữ**: Gửi gói phím nguyên tử qua `SendInput`, tối ưu cho Visual Studio, VS Code, Notepad, Office và trình duyệt web.
- **100% Native Rust + Win32 API (`WH_KEYBOARD_LL`)**: File thực thi `.exe` độc lập (Portable), chỉ ~200 KB, chạy ngay không cần cài đặt hay quyền Administrator.

---

## 🛠️ Hướng Dẫn Build & Sử Dụng

### 1. Yêu cầu:
- Máy tính chạy Windows 10/11 có cài đặt [Rust](https://rustup.rs/) (`x86_64-pc-windows-msvc`).

### 2. Biên dịch:
```powershell
cd windows
cargo build --release
```

File thực thi độc lập sẽ được tạo tại:
👉 `windows\target\release\VnCharKey.exe`

### 3. Vị trí lưu cấu hình trên Windows:
- Cấu hình phím tắt tự do: `%APPDATA%\VnCharKey\config.conf`
- Bảng Gõ Tắt: `%APPDATA%\VnCharKey\macros.conf`
- Từ Điển Cá Nhân: `%APPDATA%\VnCharKey\dictionary.conf`
