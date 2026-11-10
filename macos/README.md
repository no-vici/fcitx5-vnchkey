# vnchKey for macOS

Bản port của bộ gõ **vnchKey** dành cho hệ điều hành macOS, sử dụng trực tiếp lõi Rust `vnchkey-core` thông qua cơ chế `CGEventTap` và giao diện Swift Menu Bar nhẹ nhàng, thanh lịch.

---

## 🚀 Hướng dẫn Build & Chạy trên macOS

### Yêu cầu
- Máy Mac (Apple Silicon M1/M2/M3/M4 hoặc Intel x86_64).
- Đã cài đặt **Xcode Command Line Tools** (chạy `xcode-select --install` nếu chưa có).
- Đã cài đặt **Rust toolchain** (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).

---

### Cách 1: Build và đóng gói tự động bằng Makefile (Khuyên dùng)

Mở Terminal trên máy Mac của bạn, di chuyển vào thư mục `macos` và gõ:

```bash
cd macos
make app
```

Lệnh này sẽ tự động:
1. Biên dịch lõi Rust `fcitx5-vnchkey-ffi` ra thư viện tĩnh `libfcitx5_vnchkey_ffi.a`.
2. Biên dịch mã nguồn Swift (`KeyTapManager.swift`, `main.swift`).
3. Đóng gói ra ứng dụng macOS hoàn chỉnh: `build/vnchKey.app`.

---

### Khởi chạy và cấp quyền Accessibility

1. Khởi chạy ứng dụng:
   ```bash
   open build/vnchKey.app
   ```
2. **Cấp quyền Trợ Năng (Accessibility):**
   - Khi chạy lần đầu, macOS sẽ hiện thông báo yêu cầu cấp quyền.
   - Nhấn **Open System Settings** (hoặc vào *System Settings > Privacy & Security > Accessibility*).
   - Bật công tắc cho phép **vnchKey**.
3. Icon Cờ Vàng ba sọc đỏ **V** sẽ xuất hiện trên thanh trạng thái (Menu Bar) ở góc trên bên phải màn hình.

---

### Cài đặt vào hệ thống để dùng lâu dài

Bạn chỉ cần kéo thả `vnchKey.app` vào thư mục `/Applications`:

```bash
cp -r build/vnchKey.app /Applications/
```

Để ứng dụng tự khởi động cùng máy:
- Vào *System Settings > General > Login Items*.
- Thêm `vnchKey` vào danh sách *Open at Login*.

