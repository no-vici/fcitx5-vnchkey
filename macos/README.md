# vnchKey for macOS

Bản port chính thức của bộ gõ **vnchKey (Tự Do Edition)** dành cho hệ điều hành **macOS** (Apple Silicon M1/M2/M3/M4 & Intel x86_64). Sử dụng trực tiếp lõi xử lý âm vị học Tiếng Việt [`vnchkey-core`](../crates/vnchkey-core) thông qua C-ABI FFI (`fcitx5-vnchkey-ffi`) kết hợp với `CGEventTap` và giao diện Menu Bar Cocoa原生 gọn nhẹ, phản hồi tức thì.

---

## ✨ Tính Năng Nổi Bật Trên macOS

1. **Đầy đủ 4 kiểu gõ**: Tự động (`Telex + VNI`), `Telex`, `VNI`, và `Simple Telex`.
2. **Tối ưu đặc biệt cho Neovim / Vim / Terminal trên macOS**:
   - **Khoảng lặng ngắt từ tự động (`idle_timeout_ms = 800ms`)**: Tự động dọn sạch bộ đệm khi bạn dừng tay $\ge 0.8\text{s}$ ở chế độ Normal mode (ví dụ bấm `u` undo rồi nghỉ một nhịp trước khi bấm `i` vào Insert mode và gõ số `1` sau `height="50"`), trị dứt điểm hiện tượng nhảy thành chữ `úi` thường gặp trên các bộ gõ macOS khác.
   - **Bảo vệ chuỗi số VNI (VNI Numeric Guard)** & tách âm tiết cuối (`extract_last_syllable_index`).
   - **Gắn nhãn sự kiện nội bộ (`VNCHKEY_SYNTHETIC_TAG`)**: Ngăn chặn vòng lặp sự kiện `CGEventTap` (re-entrancy), giúp gõ VNI (`dau61` $\to$ `dấu`, `gio72` $\to$ `giờ`) chính xác tuyệt đối.
3. **Quản lý Gõ Tắt (Macro) & Từ Điển Cá Nhân trực tiếp trên Menu Bar (như xKey)**:
   - Thêm, sửa, xoá nhanh mục Gõ Tắt (`➕ Thêm Gõ Tắt mới...`, hỗ trợ biến động `$DATE`, `$TIME`, `$YEAR`, `$NOW`, `$DATETIME`) và Từ Điển Cá Nhân (`➕ Thêm từ mới vô Từ Điển...`) ngay bằng hộp thoại trên thanh Menu mà không cần mở file cấu hình thủ công.
   - Khi chọn mở `macros.conf` hoặc `dictionary.conf`, ứng dụng tự động gọi `/usr/bin/open -t` để mở bằng **TextEdit** (tránh lỗi macOS báo không tìm thấy ứng dụng mở đuôi `.conf`).
4. **Cửa sổ "Bảng điều khiển..." (Dashboard) & Phím tắt chuyển chế độ Việt / Anh tuỳ chỉnh**:
   - Hỗ trợ chọn nhanh tổ hợp phím tắt toàn cục để Bật/Tắt Tiếng Việt ở bất kỳ đâu: `Control + Shift` (mặc định), `Command + Shift`, `Control + Option`, `Control + Space`, `Option + Z`, `Control + Shift + Z`, `Command + Shift + V`, hoặc tắt phím tắt.
   - Tích hợp cửa sổ **Bảng điều khiển...** (Cocoa原生 kiểu xKey / OpenKey) cho phép tuỳ chỉnh tập trung tất cả tính năng, quy tắc gõ phím `W → ư` và thanh trượt thời gian ngắt bộ đệm cho Neovim (`0ms` – `2000ms`).
5. **Kiểm tra Chánh Tả, Tự động khôi phục Tiếng Anh, Hoàn nguyên từ bằng phím `Esc`, Quy tắc dấu mới/cũ & Bảng mã Unicode NFC / NFD**.

---

## 🛠️ Yêu Cầu Trước Khi Build

1. **Máy Mac**: Chạy macOS 11.0 (Big Sur) trở lên (Apple Silicon hoặc Intel).
2. **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```
3. **Rust Toolchain**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

---

## 🚀 Quy Trình Build & Cài Đặt Chuẩn (Không Lỗi Quyền Accessibility)

> [!IMPORTANT]
> Trên macOS, quyền Trợ năng (**Accessibility / TCC**) được gắn chặt với **đường dẫn của ứng dụng (`/Applications/vnchKey.app`)** và **chữ ký số (`codesign` hash)** của bản build.
> Vì vậy, mỗi khi bạn vừa `git pull` và biên dịch lại bản mới, hãy luôn thực hiện đúng thứ tự các bước dưới đây để tránh tình trạng tiến trình cũ vẫn chạy ngầm hoặc macOS thu hồi quyền bắt phím của bản build mới!

### Bước 1: Build ứng dụng từ mã nguồn

```bash
git clone https://github.com/no-vici/fcitx5-vnchkey.git
cd fcitx5-vnchkey/macos
make clean && make app
```

Lệnh `make app` sẽ tự động:
1. Biên dịch lõi Rust `fcitx5-vnchkey-ffi` ở chế độ `--release` ra thư viện tĩnh `libfcitx5_vnchkey_ffi.a`.
2. Biên dịch mã nguồn Swift (`Sources/KeyTapManager.swift`, `Sources/main.swift`) và nhúng bộ biểu tượng `AppIcon.icns` + `flag_vnch.png`.
3. Đóng gói `build/vnchKey.app` và ký ad-hoc (`codesign --force --deep --sign -`).

### Bước 2: Tắt tiến trình cũ (nếu đang chạy) và chép vào `/Applications`

Luôn chạy 3 lệnh này để đảm bảo chỉ có **một bản duy nhất** trong `/Applications/vnchKey.app` được chạy:

```bash
killall vnchKey 2>/dev/null || true
cp -R build/vnchKey.app /Applications/
open /Applications/vnchKey.app
```

*(Không nên chạy lúc thì từ `build/vnchKey.app`, lúc thì từ `/Applications/vnchKey.app` vì macOS sẽ coi đó là 2 ứng dụng khác nhau khi kiểm tra quyền Accessibility).*

### Bước 3: Cấp quyền (hoặc làm mới quyền) Accessibility

- **Nếu cài lần đầu tiên**:
  1. Khi ứng dụng hiện thông báo cần quyền Trợ năng, bấm **Mở System Settings** (hoặc vào **System Settings > Privacy & Security > Accessibility**).
  2. Bật công tắc cho **vnchKey**.
  3. Bấm vào biểu tượng Cờ Vàng **`V`** trên thanh Menu Bar $\to$ chọn **Thoát vnchKey**, sau đó mở lại `/Applications/vnchKey.app`.

- **Nếu bạn vừa Re-build / Update bản mới mà thấy không gõ được Tiếng Việt**:
  Do bản build mới có chữ ký ad-hoc mới, macOS TCC sẽ âm thầm chặn quyền của bản mới dù nút gạt trong System Settings nhìn vẫn đang bật! Cách xử lý chuẩn:
  1. Mở **System Settings > Privacy & Security > Accessibility**.
  2. Chọn `vnchKey` trong danh sách rồi bấm dấu trừ **`(-)`** ở bên dưới để xoá hẳn mục cũ đi (hoặc gạt công tắc sang **OFF** rồi gạt lại sang **ON**).
  3. Chạy lại:
     ```bash
     killall vnchKey 2>/dev/null || true
     open /Applications/vnchKey.app
     ```
  4. Bật lại công tắc cho `/Applications/vnchKey.app` trong Accessibility là gõ được ngay cả Telex lẫn VNI!

---

## ⚠️ Xử Lý Các Tình Huống Thường Gặp (Troubleshooting)

| Hiện tượng | Nguyên nhân | Cách khắc phục nhanh |
| :--- | :--- | :--- |
| macOS báo *"vnchKey is damaged and can't be opened"* khi tải file `.dmg` từ GitHub Releases | File tải qua trình duyệt bị macOS gắn cờ cách ly `com.apple.quarantine` vì chưa có chứng chỉ Apple Developer trả phí (\$99/năm). | Chạy lệnh gỡ cờ cách ly:<br>`xattr -cr /Applications/vnchKey.app`<br>Hoặc clone repo về máy và chạy `make app` local. |
| Đã chọn Telex / VNI trên Menu Bar nhưng gõ chỉ ra chữ tiếng Anh không dấu | 1. Tiến trình `vnchKey` cũ trước khi build vẫn đang chạy ngầm.<br>2. Chưa làm mới quyền Accessibility sau khi build lại binary mới.<br>3. Mở `vnchKey` lúc chưa bật quyền nên `CGEventTap` chưa được gắn. | 1. Xoá `vnchKey` cũ trong **Settings > Accessibility** bằng nút `(-)`.<br>2. Chạy `killall vnchKey; cp -R build/vnchKey.app /Applications/; open /Applications/vnchKey.app` và cấp quyền lại. |
| Icon của `vnchKey.app` trong `/Applications` bị trống / trong suốt sau khi chép đè | Finder của macOS lưu cache icon cũ. | Chạy lệnh làm mới cache icon của Finder:<br>`touch /Applications/vnchKey.app && killall Finder` |
| Muốn `vnchKey` tự động chạy mỗi khi bật máy Mac | Chưa thêm vào Login Items. | Vào **System Settings > General > Login Items** $\to$ bấm dấu `(+)` dưới mục *Open at Login* và chọn `/Applications/vnchKey.app`. |

---

## 📂 Vị Trí Lưu Cấu Hình Trên macOS

- **Các tuỳ chọn kiểu gõ, bảng mã, kiểm tra chánh tả**: Lưu tự động trong `UserDefaults` của macOS (`com.vnchkey.macos`).
- **Bảng Gõ Tắt (Macros)**: `~/.config/vnchkey/macros.conf`
- **Từ Điển Cá Nhân (Custom Dictionary)**: `~/.config/vnchkey/dictionary.conf`
