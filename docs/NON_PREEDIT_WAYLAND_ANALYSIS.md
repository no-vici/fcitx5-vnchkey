# Phân Tích Kỹ Thuật: Cơ Chế Non-Preedit, Mô Phỏng UniKey Trên Linux Wayland & Định Hướng Phát Triển Cho vnchKey

> **Tài liệu tham chiếu kiến trúc & nghiên cứu phát triển nội bộ**  
> **Dự án**: vnchKey (Tự Do Edition)  
> **Chủ đề**: Giải bài toán gõ tiếng Việt không gạch chân (Non-preedit) trên môi trường Linux / Wayland, so sánh cơ chế Windows UniKey, Fcitx5-Lotus, IBus-Bamboo và giải pháp kiến trúc của vnchKey.

---

## 1. Đặt Vấn Đề: Cơn Khát "Non-Preedit" Của Người Dùng Linux

Trên hệ điều hành Linux (đặc biệt khi chuyển dịch sang giao thức **Wayland** như Hyprland, Sway, KDE Plasma, GNOME Wayland), một trong những chủ đề được cộng đồng thảo luận sôi nổi nhất là: **Làm sao để gõ tiếng Việt tự nhiên, không gạch chân (Non-preedit), ăn ngay từng ký tự như UniKey trên Windows.**

Theo tiêu chuẩn quốc tế về bộ gõ (IME - Input Method Engine) trên Linux (IBus, Fcitx5):
- Cơ chế mặc định là **Preedit**: tạo một vùng đệm tạm thời (thường có nét gạch chân nội dòng) để gom các ký tự trước khi xác nhận (commit) vào văn bản.
- **Vấn đề của người dùng**: Đa số người dùng Việt Nam lớn lên với UniKey trên Windows, đã quen cảm giác "gõ mộc" — chữ ăn trực tiếp vào văn bản ngay lập tức. Nét gạch chân hoặc hộp chữ nổi đôi khi gây cảm giác lạ lẫm, vướng víu hoặc lệch con trỏ trên một số ứng dụng.
- **Thách thức**: Khi các lập trình viên cố gắng mô phỏng lại cơ chế UniKey trên Linux để "xóa sổ" Preedit, họ ngay lập tức va phải bài toán hóc búa nhất: **Lỗi lặp chữ, nhảy chữ kinh niên trên các ứng dụng nền tảng Web / Canvas (`Thu` + `ư` + `ử` $\to$ `Thuưử`)**, điển hình là **Google Docs, Slack, Discord, StackEdit, Sider.ai**.

---

## 2. Mổ Xẻ Bản Chất: Cơ Chế Huyền Thoại Của UniKey Trên Windows

Để hiểu vì sao UniKey trên Windows gõ "mượt như nhung" mà không cần Preedit, ta cần nhìn sâu vào kiến trúc xử lý sự kiện của Windows:

```
[Bàn Phím] 
   │
   ▼
[Windows OS Kernel]
   │
   ▼
[UniKey Global Hook: SetWindowsHookEx(WH_KEYBOARD_LL)] 
   │ 
   ├──> Kiểm tra State Machine (Telex / VNI)
   └──> Phát hiện cần ghép dấu (ví dụ: 'toan' + 's')
           │
           ▼
[SendInput(Array of Injected Events)] ──(NGUYÊN TỬ & ĐỒNG BỘ 100%)──> [Application Message Queue]
   ├── Event 1: VK_BACK (KeyDown)                                          │
   ├── Event 2: VK_BACK (KeyUp)                                            │
   ├── Event 3: Unicode 'á'                                                ▼
   └── Event 4: Unicode 'n'                                        [App xử lý tuần tự,
                                                                    triệt tiêu race condition]
```

### Hai yếu tố cốt tử giúp UniKey Windows thành công:
1. **Móc phím mức thấp toàn cục (`SetWindowsHookEx`)**: UniKey can thiệp trực tiếp vào chuỗi phân phối phím trước khi sự kiện tới được cửa sổ ứng dụng.
2. **Hàng đợi thông điệp đồng bộ nguyên tử (`SendInput`)**:
   - Khi UniKey cần thay thế âm tiết (ví dụ: đổi `toan` + `s` thành `toán`), nó gọi hàm `SendInput()` truyền vào một mảng sự kiện: gửi phím Backspace xóa chữ `n` và lập tức gửi hai ký tự Unicode `á`, `n`.
   - Windows bảo đảm toàn bộ mảng sự kiện này được tống vào **Message Queue duy nhất của luồng đích một cách đồng bộ**. Ứng dụng (kể cả Microsoft Word, Chrome hay Notepad) **bắt buộc phải rút thông điệp Backspace ra xử lý xong xuôi rồi mới được nhận tiếp ký tự Unicode**.
   - Do đó, hiện tượng chữ mới đè lên chữ cũ là hoàn toàn không thể xảy ra trên Windows.

---

## 3. Vì Sao Mô Phỏng UniKey Lên Linux Wayland Bị "Vỡ Trận" Trên Web/Canvas?

Khi mang tư duy "bắn Backspace giả lập" lên Linux Wayland, sự khác biệt căn bản về kiến trúc hệ thống đã bộc lộ điểm yếu chết người:

```
                  ┌───────────────────────────────┐
                  │    Fcitx5 / vnchKey Addon     │
                  └───────┬───────────────┬───────┘
                          │               │
        [Kênh 1: Hardware Key Event]      │ [Kênh 2: Text-Input Protocol]
          ic->forwardKey(BackSpace)       │   ic->commitString("án")
                          │               │
                          ▼               ▼
                 [Wayland Compositor / X11 Server]
                          │               │
                          └───────┬───────┘
                                  │ (Hai kênh truyền bất đồng bộ)
                                  ▼
                 ┌─────────────────────────────────┐
                 │  Browser Engine (Chromium/Gecko) │
                 └────────────────┬────────────────┘
                                  │
      ┌───────────────────────────┴───────────────────────────┐
      │ DOM Event Loop:                                       │
      │ 1. Nhận KeyboardEvent("keydown", "Backspace")         │
      │    ──> Chuyển tiếp vào JavaScript Engine (Kix/React)  │
      │    ──> Cần vài mili-giây để tính toán vị trí Cursor   │
      │                                                       │
      │ 2. Sự kiện commitString("án") ẬP ĐẾN NGAY LẬP TỨC!   │
      │    ──> Con trỏ JS CHƯA KỊP LÙI VỀ!                    │
      │    ──> Chữ mới bị chèn đè lên chữ cũ: "toanán"        │
      │        (hoặc "Thuưử", "Tiengêngếng", "vâyậy")         │
      └───────────────────────────────────────────────────────┘
```

### Các nguyên nhân kỹ thuật cốt lõi:
1. **Kiến trúc bảo mật Wayland & IPC bất đồng bộ**:
   - Wayland cô lập các cửa sổ, không cho phép can thiệp bàn phím toàn cục tự do như Windows Hook. Bộ gõ phải giao tiếp với ứng dụng thông qua dịch vụ trung gian (Fcitx5 / IBus) bằng các kênh IPC bất đồng bộ (Wayland protocol `zwp_text_input_v3` hoặc D-Bus).
2. **Kênh truyền bị lệch pha (Out-of-band Race Condition)**:
   - Phím Backspace giả lập (`forwardKey`) đi theo kênh giả lập phần cứng bàn phím (`wl_keyboard`).
   - Chuỗi văn bản thay thế (`commitString`) đi theo kênh nhập liệu văn bản (`zwp_text_input_v3`).
   - Hai kênh này đi vào trình duyệt và ứng dụng qua hai đường xử lý khác nhau, không có cơ chế `SendInput` đồng bộ nguyên tử của Windows để ép trình duyệt phải xử lý xong Backspace mới được nhận Text Commit.
3. **Đặc thù của các ứng dụng Web hiện đại & Canvas Kix**:
   - **Google Docs**: Không dùng ô nhập liệu HTML (`<input>` / `<textarea>`) thông thường mà tự vẽ toàn bộ giao diện bằng **HTML5 Canvas / WebGL (Kix Engine)**. Mọi thao tác xóa và chèn ký tự đều phải chạy qua mã nguồn JavaScript nặng nề để render lại vector chữ.
   - **React / Vue Controlled Inputs** (Slack, Discord, Sider.ai, Notion, StackEdit): State của ô nhập liệu được quản lý bởi Virtual DOM. Khi Backspace giả lập gửi tới, React cần thời gian re-render state. Nếu `commitString` ập tới trước khi state cập nhật, hiện tượng lặp ký tự xuất hiện tức thì.

---

## 4. Khảo Sát & So Sánh Các Giải Pháp Hiện Nay

| Tiêu Chí | IBus-Bamboo | Fcitx5-Lotus | VI-IME | vnchKey (Hiện Tại) |
| :--- | :--- | :--- | :--- | :--- |
| **Cơ chế cốt lõi** | Đa chế độ (1 đến 6) chuyển bằng `Shift + ~` | Non-preedit ưu tiên, port từ `bamboo-core` sang C++ | Viết bằng Rust, tối giản, hướng Wayland Compositor | **Kiến Trúc Kép Thông Minh (Direct + Preedit)** |
| **Trải nghiệm Terminal/Code** | Tốt khi chọn đúng mode | Mượt mà, không gạch chân | Tốt trên một số compositor | **Cực tốt (Micro-diff LCP độ trễ = 0, tối ưu Neovim/Vim)** |
| **Đối với Google Docs / Web** | Phải chuyển thủ công sang mode Preedit hoặc ForwardKey | Vẫn có nguy cơ lặp chữ nếu app web render canvas chậm | Dễ lỗi lặp chữ trên các controlled inputs | **Triệt tiêu 100% lỗi lặp chữ nhờ Preedit Mode chuyên dụng** |
| **Khung chữ nổi (Tooltip)** | Không có | Có trên một số build | Không | **Có Floating Tooltip (tùy chọn bật/tắt chống phân tâm)** |
| **Chuyển đổi theo ứng dụng** | Thủ công qua phím tắt | Cấu hình trong options | Không tự động | **Tự động 100% (Per-App Memory: Browser $\to$ Preedit, IDE $\to$ Direct)** |

---

## 5. Kiến Trúc Kép Của vnchKey: Giải Pháp Thực Tế & Toàn Diện Nhất

Thay vì cố chấp theo đuổi duy nhất một cơ chế cực đoan (hoặc 100% Preedit khiến dân lập trình ức chế, hoặc 100% Non-preedit khiến người làm văn phòng trên Google Docs phát điên), **vnchKey** giải quyết vấn đề bằng **Kiến Trúc Kép Thực Dụng (Pragmatic Hybrid Architecture)**:

```
                        ┌──────────────────────────────┐
                        │   vnchKey Input Method       │
                        └──────────────┬───────────────┘
                                       │
                      Nhận diện tiến trình ứng dụng
                      (ic->program() / Per-App Memory)
                                       │
               ┌───────────────────────┴───────────────────────┐
               ▼                                               ▼
    [Nhóm Ứng Dụng Lập Trình]                       [Nhóm Ứng Dụng Web / Canvas]
  (Alacritty, Kitty, WezTerm,                    (Chrome, Firefox, Brave, Edge,
   VS Code, Neovim, Sublime)                      Google Docs, Slack, Sider.ai)
               │                                               │
               ▼                                               ▼
    ┌──────────────────────┐                       ┌──────────────────────┐
    │     DIRECT MODE      │                       │     PREEDIT MODE     │
    │  (Micro-diff LCP)    │                       │  (Zero-Backspace)    │
    └──────────┬───────────┘                       └──────────┬───────────┘
               │                                              │
  • Trải nghiệm như UniKey Windows               • KHÔNG BAO GIỜ bắn Backspace giả lập
  • Độ trễ bằng 0, không gạch chân               • Gom từ trong bộ đệm nội bộ
  • Xóa lùi tối thiểu (Minimal Replacement)      • Commit nguyên tử khi bấm Space/Enter
                                                 • Có Floating Tooltip sát con trỏ
```

### Ưu điểm vượt trội của giải pháp vnchKey:
1. **Direct Mode (Micro-diff)**: Khi gõ trong Neovim hay VS Code, bộ gõ sử dụng thuật toán tìm Tiền Tố Chung Dài Nhất (Longest Common Prefix). Ví dụ `toan` + `s` $\to$ chỉ xóa đúng 1 ký tự `n` và gửi `án`. Triệt tiêu hoàn toàn nhấp nháy màn hình.
2. **Preedit Mode (Đặc trị Web)**: Khi vào Google Docs hay web chat, engine giữ toàn bộ từ trong bộ nhớ nội bộ. Dù bạn gõ sai, sửa dấu liên tục (`thêể` $\to$ `thể`), engine tự sắp xếp dấu bên trong mà **không gửi một phím Backspace nào ra trình duyệt**. Khi bấm phím cách (Space), từ hoàn chỉnh được gửi bằng một lệnh `commitString` duy nhất. Không có Backspace $\to$ không bao giờ có lỗi lặp chữ.
3. **Floating Preedit Tooltip**: Giải quyết triệt để vấn đề "gõ vào khoảng không" trên Canvas của Google Docs. Một tooltip nhỏ xuất hiện ngay dưới con trỏ để người gõ nhìn thấy rõ từng nét chữ. Nếu người dùng thấy mất tập trung, có thể tắt nhanh chỉ bằng một cú click chuột trên khay hệ thống.
4. **Per-App Memory Tự Động**: Bộ nhớ thông minh tự nhận biết tiến trình ứng dụng qua `ic->program()`. Người dùng không cần phải nhớ phím tắt chuyển mode mỗi khi nhảy qua lại giữa trình duyệt và trình soạn thảo code.

---

## 6. Lộ Trình Nâng Cấp Kỹ Thuật Tiếp Theo Cho vnchKey

Từ nghiên cứu cơ chế của UniKey và các dự án như `fcitx5-lotus` / `ibus-bamboo`, vnchKey có thể tiếp tục bổ sung các tinh chỉnh chuyên sâu sau:

### 1. Tận dụng tối đa Surrounding Text Atomic Protocol
- Khi ứng dụng hỗ trợ Surrounding Text (như các ứng dụng GTK4, Qt6 bản mới), thay vì dùng `forwardKey(Backspace)`, luôn gọi:
  ```cpp
  ic->deleteSurroundingText(-out_backspaces, out_backspaces);
  ic->commitString(out_commit);
  ```
- Trên Wayland, hai lệnh này được đóng gói trong **cùng một transaction commit**, giúp thao tác chỉnh sửa mang tính nguyên tử ngay cả ở chế độ Direct Mode.

### 2. Context Guard (Bảo vệ ngữ cảnh tự động)
- Khi phát hiện con trỏ đang nằm trong thanh địa chỉ duyệt web (`URL`), email, hoặc các chuỗi ký tự định danh lập trình (`camelCase`, `snake_case`, `UPPER_CASE`):
- Tự động chuyển về trạng thái gõ thô (Raw Bypass) để người dùng gõ URL, đường dẫn thư mục (`/home/user/...`) hay tên biến code mà không bị can thiệp dấu tiếng Việt hay xóa nhầm ký tự.

### 3. Tinh chỉnh độ trễ thích ứng (Adaptive Timing Calibration)
- Đối với các trường hợp người dùng vẫn muốn ép buộc dùng Direct Mode trên trình duyệt:
- Nghiên cứu cơ chế micro-pause thích ứng (khoảng 2–4ms giữa sự kiện KeyUp của Backspace và `commitString`) dựa trên tốc độ gõ (Inter-Keystroke Interval - IKI) để trình duyệt web có cửa sổ thời gian cập nhật lại DOM cursor.

---

## 7. Kết Luận

Nỗ lực tạo ra một bộ gõ "100% Non-preedit hoàn hảo trong mọi hoàn cảnh" trên Linux Wayland là một bài toán bất khả thi về mặt lý thuyết do sự lệch pha kiến trúc giữa Wayland IPC và JavaScript Event Loop của Web Engine.

Cách tiếp cận của **vnchKey** — kết hợp hài hòa giữa **Direct Micro-diff thuần khiết cho môi trường lập trình** và **Preedit nguyên tử cho môi trường Web/Canvas**, được điều phối mượt mà bởi **Bộ nhớ thông minh theo ứng dụng** — chính là lời giải thực tế, thanh lịch và ổn định nhất cho bài toán gõ tiếng Việt trên Linux hiện đại.

