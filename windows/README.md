# VnCharKey for Windows

**VnCharKey** là phiên bản bộ gõ tiếng Việt tốc độ cao, siêu nhẹ và trung tính dành cho hệ điều hành **Windows (Windows 10 / Windows 11)**, sử dụng chung lõi thuật toán Rust `vnchkey-core` với biểu tượng **Nón Lá Việt Nam (Conical Hat)**.

---

## ✨ Điểm Nổi Bật

- **Biểu tượng Nón Lá (Conical Hat) & Tên gọi Trung Tính (`VnCharKey`)**: Thân thiện, thanh lịch, phù hợp sử dụng ở mọi môi trường văn phòng, doanh nghiệp, trường học.
- **Thuật toán Micro-Diff chống chớp chữ**: Sử dụng `SendInput` gửi gói sự kiện nguyên tử (Atomic Input Batch), chỉ xóa đúng các ký tự cần đổi dấu thay vì xóa cả từ.
- **100% Native Rust + Win32 API (`WH_KEYBOARD_LL`)**: Không cần cài .NET Runtime, không cần WebView/Electron. File thực thi `.exe` độc lập (Portable) chỉ nặng ~200 KB, chạy ngay không cần quyền Administrator.
- **Tính năng thông minh**:
  - Tự động nhận diện kiểu gõ **Auto (Telex + VNI)**, hoặc chọn riêng **Telex** / **VNI**.
  - Phím tắt **`Ctrl + Shift`** (hoặc click trái vào icon dưới khay hệ thống) để Bật / Tắt nhanh Tiếng Việt.
  - Nhấn **`Esc`** để hoàn nguyên từ đang gõ về chuỗi ký tự thô tiếng Anh.
  - Kiểm tra Chánh Tả & Tự động khôi phục từ tiếng Anh (`Restore English`).

---

## 🛠️ Hướng Dẫn Build Trên Windows

### Yêu cầu:
- Đã cài đặt [Rust](https://rustup.rs/) trên Windows (`x86_64-pc-windows-msvc`).

### Lệnh biên dịch:
Mở PowerShell hoặc Command Prompt:

```powershell
cd windows
cargo build --release
```

File chạy độc lập sẽ được tạo tại:
👉 `windows\target\release\VnCharKey.exe`

Bạn có thể copy file `VnCharKey.exe` này đi bất kỳ máy Windows nào và click đúp chạy ngay!

