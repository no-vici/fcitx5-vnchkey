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
4. **Cửa sổ "Bảng điều khiển..." (Dashboard) & Tự do đặt phím tắt bất kỳ theo ý muốn**:
   - **Tự bấm phím tắt trực tiếp (Live Key Recorder)**: Bấm nút `⌨️ Tự bấm phím...` trong Bảng điều khiển (hoặc từ Menu Bar), sau đó nhấn bất kỳ tổ hợp phím nào bạn thích trên bàn phím (ví dụ `Ctrl + Shift`, `Cmd + Option`, `Option + Z`, `Ctrl + Space`, `Cmd + Shift + X`, `F12`...) — `vnchKey` sẽ tự thu nhận và lưu lại ngay lập tức!
   - Bạn cũng có thể tự tích chọn rời từng phím bổ trợ (`⌃ Ctrl`, `⌥ Opt`, `⇧ Shift`, `⌘ Cmd`) ngay trên Bảng điều khiển như xKey / OpenKey.
   - Tích hợp cửa sổ **Bảng điều khiển...** (Cocoa原生) cho phép tuỳ chỉnh tập trung tất cả tính năng, quy tắc gõ phím `W → ư` và thanh trượt thời gian ngắt bộ đệm cho Neovim (`0ms` – `2000ms`).
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
| Muốn `vnchKey` tự động chạy mỗi khi bật máy Mac | Chưa bật khởi động cùng máy. | Chuột phải icon trên Menu Bar (hoặc mở Bảng điều khiển) $\to$ tích chọn **"Khởi động cùng máy (Open at login)"**. |

---

## ⚡ Điều Khiển Bằng Dòng Lệnh (CLI) & Tích Hợp Neovim Trên macOS

Đối với lập trình viên dùng **Neovim / Vim** hoặc muốn điều khiển bộ gõ qua script / terminal, `vnchKey` tích hợp sẵn một **IPC Server siêu nhẹ qua Unix Domain Socket** tại `/tmp/vnchkey.sock`.

### 1. Các lệnh CLI có sẵn

Bạn có thể chạy trực tiếp từ Terminal (hoặc tạo alias `alias vnchkey="/Applications/vnchKey.app/Contents/MacOS/vnchKey"`):

```bash
# Tắt gõ Tiếng Việt (chuyển sang Tiếng Anh)
/Applications/vnchKey.app/Contents/MacOS/vnchKey --off

# Bật gõ Tiếng Việt
/Applications/vnchKey.app/Contents/MacOS/vnchKey --on

# Đảo qua lại giữa Tiếng Việt và Tiếng Anh
/Applications/vnchKey.app/Contents/MacOS/vnchKey --toggle

# Kiểm tra trạng thái kết nối
/Applications/vnchKey.app/Contents/MacOS/vnchKey --status
```
*Lưu ý: Lệnh CLI giao tiếp trực tiếp qua socket `/tmp/vnchkey.sock` nên phản hồi tức thì trong vòng `< 2ms`, không gây độ trễ.*

### 2. Tích hợp tự động vào Neovim bằng Lua Module

Dự án cung cấp sẵn module tích hợp chính thức tại [`data/neovim/lua/vnchkey.lua`](../data/neovim/lua/vnchkey.lua). Module này tự động chuyển sang Tiếng Anh khi bạn rời `INSERT` mode (`InsertLeave`) để bảo vệ các lệnh Normal mode (`u`, `h,j,k,l`, `ciw`...), và bật lại Tiếng Việt khi bạn vào `INSERT` mode (`InsertEnter`).

#### Cách thiết lập trong Neovim (init.lua):

1. **Cách 1: Chép file `vnchkey.lua` vào config Neovim của bạn**:
   ```bash
   mkdir -p ~/.config/nvim/lua
   cp data/neovim/lua/vnchkey.lua ~/.config/nvim/lua/
   ```
   Sau đó trong `~/.config/nvim/init.lua`, chỉ cần thêm 1 dòng:
   ```lua
   require("vnchkey").setup()
   ```

2. **Cách 2: Gọi trực tiếp autocommands trong `init.lua` (không cần copy file)**:
   Nếu bạn thích viết trực tiếp trong file cấu hình Neovim của mình:
   ```lua
   local uv = vim.uv or vim.loop
   local sock = "/tmp/vnchkey.sock"

   local function send_vnchkey(cmd)
     local client = uv.new_pipe(false)
     if client then
       client:connect(sock, function(err)
         if not err then
           client:write(cmd .. "\n", function() client:close() end)
         else
           client:close()
         end
       end)
     end
   end

   local group = vim.api.nvim_create_augroup("vnchkey_toggle", { clear = true })
   -- Rời Insert mode: Tắt bộ gõ ngay lập tức
   vim.api.nvim_create_autocmd("InsertLeave", {
     group = group,
     callback = function() send_vnchkey("off") end,
   })
   -- Vào Insert mode: Bật lại bộ gõ
   vim.api.nvim_create_autocmd("InsertEnter", {
     group = group,
     callback = function() send_vnchkey("on") end,
   })
   ```

---

## 🔀 Đảo Từ Nhanh (Flip Word) & Sửa Dấu Nhanh

- **Đảo từ (Flip Word)**:
  - Bấm **`Cmd + Shift + Z`** (hoặc `Ctrl + Shift + Z`) bất kỳ lúc nào để lật qua lại giữa từ Tiếng Việt vừa gõ và chuỗi phím thô:
    - `tiếng` $\longleftrightarrow$ `tieengs`
    - `việt` $\longleftrightarrow$ `vieetj`
    - `được` $\longleftrightarrow$ `dduowjc`
- **Sửa dấu nhanh không cần xóa chữ**:
  - Khi đang gõ một từ (hoặc vừa bấm dấu cách nhưng bấm Backspace 1 lần để mở lại từ), bạn chỉ cần bấm đè phím dấu mới (ví dụ gõ nhầm dấu sắc `s`, chỉ cần bấm dấu huyền `f` $\to$ bộ gõ tự chuyển dấu sắc thành dấu huyền mà không cần xóa đi gõ lại từ đầu).
  - Bấm phím **`z`** để xóa dấu thanh hoàn toàn về chữ không dấu.

---

## 📂 Vị Trí Lưu Cấu Hình Trên macOS

- **Các tuỳ chọn kiểu gõ, bảng mã, kiểm tra chánh tả**: Lưu tự động trong `UserDefaults` của macOS (`com.vnchkey.macos`).
- **Bảng Gõ Tắt (Macros)**: `~/.config/vnchkey/macros.conf`
- **Từ Điển Cá Nhân (Custom Dictionary)**: `~/.config/vnchkey/dictionary.conf`
