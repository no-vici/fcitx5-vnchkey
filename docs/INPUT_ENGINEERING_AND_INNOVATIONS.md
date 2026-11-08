# Kiến Trúc Kỹ Thuật Nhập Liệu & Các Đột Phá Công Nghệ Của vnchKey
> **Tài liệu tham chiếu chuyên sâu về kiến trúc phần mềm, cơ chế phát phím (Event Dispatching), bộ đệm gõ và các kỹ thuật tối ưu hóa trải nghiệm người dùng.**

---

## 1. Giới Thiệu & Triết Lý Thiết Kế

Khác với tài liệu [VIETNAMESE_LANGUAGE_SYSTEM.md](file:///home/codegpt/antigravity/vnchKey/docs/VIETNAMESE_LANGUAGE_SYSTEM.md) tập trung vào nền tảng ngữ âm học và cấu trúc âm tiết Tiếng Việt, tài liệu này tập trung giải quyết bài toán: **Làm thế nào để đưa các ký tự tiếng Việt vào hệ điều hành Linux (X11 & Wayland) một cách tức thì, không xung đột với các ứng dụng hiện đại (Web DOM, Electron, Terminal, IDE) và mang lại trải nghiệm gõ tự nhiên, thông minh nhất.**

vnchKey áp dụng triết lý:
1. **Zero-Friction**: Độ trễ xử lý $\le 0.1\text{ms}$, không can thiệp thô bạo vào tiến trình ứng dụng.
2. **Context-Aware**: Tự động nhận diện ngữ cảnh (lập trình, web, dòng lệnh terminal) để áp dụng chiến lược phát phím tối ưu nhất.
3. **Robustness & Memory Safety**: 100% Rust an toàn (`#![forbid(unsafe_code)]`), không rò rỉ bộ nhớ, không segfault.

---

## 2. Kiến Trúc Hybrid Dual-Mode & Bộ Nhớ Ứng Dụng (Per-App Memory)

vnchKey không cố định người dùng vào một cơ chế duy nhất mà kết hợp song song hai chế độ xuất chữ linh hoạt:

```
                    ┌───────────────────────────────┐
                    │  Bàn Phím Phần Cứng / Wayland │
                    └───────────────┬───────────────┘
                                    │ (Sự kiện phím)
                                    ▼
                    ┌───────────────────────────────┐
                    │    Lõi Xử Lý vnchkey-core     │
                    │   (Bóc tách âm tiết & vần)    │
                    └───────────────┬───────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
       [Chế Độ Direct Mode]            [Chế Độ Preedit Mode]
    - Phát trực tiếp (Micro-Diff)   - Gom từ nội dòng (Inline buffer)
    - Zero Latency                  - Hoàn toàn miễn nhiễm nuốt chữ
    - Cho Terminal, Code, Vim       - Dành riêng cho Canvas (Google Docs)
                    ▲                               ▲
                    └───────────────┬───────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │      Per-App Memory Engine    │
                    │   (Ghi nhớ chế độ từng App)   │
                    └───────────────────────────────┘
```

- **Direct Mode (Micro-Diff)**: Bộ gõ tính toán khoảng cách xê dịch tối thiểu (Longest Common Prefix - LCP) giữa từ cũ và từ mới, chỉ gửi số lượng phím Backspace tối thiểu và chèn phần đuôi thay đổi.
- **Preedit Mode (Atomic Commit)**: Giữ toàn bộ từ đang soạn thảo trong bộ đệm hiển thị gạch chân nội dòng, chỉ chốt (commit) một lần duy nhất khi kết thúc từ. Phù hợp hoàn hảo cho các ứng dụng nền Canvas/WebGL nặng (Google Docs, Google Sheets).
- **Per-App Memory**: Addon tự động lưu trạng thái chế độ nhập theo tên tiến trình ứng dụng (`app_id` / `program`), giúp lập trình viên vừa có Direct Mode siêu tốc trong Neovim/Kitty, vừa có Preedit an toàn tuyệt đối trong Google Docs.

---

## 3. Kỹ Thuật Select-and-Replace Cho Web & Electron Apps

### 3.1. Bản Chất Vấn Đề
Các trình soạn thảo Rich Text hiện đại (ProseMirror, Slate, Lexical, DraftJS trên Google Chrome, Discord, Slack, Notion, VS Code...) duy trì một cây DOM ảo nội bộ. Khi bộ gõ ở chế độ Direct Mode phát ra chuỗi $N$ lần phím `Backspace` giả lập ở tốc độ cao để xóa các ký tự cũ, các engine JavaScript này thường chặn (suppress) hoặc chỉ xử lý 1 sự kiện Backspace, dẫn đến hiện tượng:
- "Nuốt chữ", dính chữ hoặc nhân đôi chữ (ví dụ gõ `Haxy` biến thành `Hayãy`).

### 3.2. Giải Pháp Đột Phá Kế Thừa Từ Nghiên Cứu Dokey
vnchKey triển khai kỹ thuật **Select-and-Replace** thông minh:
1. Khi phát hiện cửa sổ mục tiêu là Web Browser hoặc Electron app (`chrome`, `chromium`, `firefox`, `brave`, `edge`, `opera`, `vivaldi`, `slack`, `discord`, `notion`, `element`, `teams`...):
   - Thay vì gửi $N$ lần `Backspace`, addon phát $N$ lần tổ hợp phím **`Shift + Left Arrow`** để tạo vùng bôi đen (selection) ngược lại đúng số lượng ký tự cần xóa.
2. Chuỗi ký tự mới được commit trực tiếp đè lên vùng chọn. Vì mọi trình soạn thảo web đều có hành vi mặc định chuẩn mực là thay thế vùng bôi đen bằng ký tự mới, vấn đề nuốt phím Backspace được triệt tiêu hoàn toàn.
3. **Bảo Toàn Terminal**: Kỹ thuật này tự động bỏ qua các ứng dụng Terminal (`kitty`, `alacritty`, `konsole`, `wezterm`, `foot`, `xterm`...) để tránh gửi chuỗi ANSI escape sequence `\x1b[1;2D`, giúp trải nghiệm dòng lệnh luôn mượt mà với Backspace truyền thống.

---

## 4. Hồi Sinh Từ Trước Sau Phím Space (Reopen Previous Word on Backspace)

### 4.1. Vấn Đề Thực Tế
Người dùng gõ xong chữ cái, theo thói quen gõ phím cách (`tiêng `) rồi mới nhận ra quên bỏ dấu sắc. Ở các bộ gõ truyền thống, từ đã bị chốt (commit) và người dùng buộc phải bấm Backspace nhiều lần để xóa hết cả từ rồi gõ lại từ đầu.

### 4.2. Cơ Chế Hoạt Động Của vnchKey
- Khi một từ kết thúc bằng một phím cách duy nhất, engine lưu trạng thái từ vào bộ nhớ đệm `reopen_state: Option<ReopenWordState>`.
- Nếu phím kế tiếp ngay sau là `Backspace`:
  1. Engine phục hồi lại cấu trúc âm tiết và danh sách phím thô đã gõ của từ trước đó vào bộ đệm soạn thảo chủ động.
  2. Người dùng chỉ cần gõ tiếp phím dấu (`s` $\to$ `tiếng`, `f` $\to$ `xìn`) là dấu thanh được đưa vào chính xác vị trí ngữ âm học.

---

## 5. Hoàn Nguyên Chữ Thô Bằng Phím Escape (Escape to Undo)

- **Ngữ cảnh**: Khi đang gõ mã lệnh hoặc một từ tiếng Anh lạ mà bộ gõ vô tình biến thành tiếng Việt (ví dụ `tieengs` $\to$ `tiếng`).
- **Giải pháp**: Người dùng chỉ cần nhấn phím **`Escape`**.
- **Luồng xử lý**:
  1. Engine tính toán Micro-Diff giữa từ tiếng Việt hiện tại và chuỗi phím thô đã gõ (`tieengs`).
  2. Phát số lượng Backspace tối thiểu và chèn lại đúng chuỗi phím gốc.
  3. Đánh dấu cờ khóa tiếng Anh (`english_locked = true`) cho từ hiện tại để người dùng có thể tiếp tục gõ mà không bị biến đổi lại.

---

## 6. Nhận Diện Mã Lập Trình Thông Minh (Developer-Aware Mode)

### 6.1. Nhận Diện Chuyển Tiếp camelCase / PascalCase
Trong mã nguồn, các định danh thường được viết dạng `camelCase` (ví dụ `parseString`, `fetchData`). Khi người dùng gõ:
`parse` + `S` $\to$ nếu không có cơ chế bảo vệ, phím `S` sẽ biến `parse` thành chữ tiếng Việt có dấu.
- **Thuật toán của vnchKey**:
  - Khi một chữ hoa xuất hiện ngay sau một hoặc nhiều chữ thường trong cùng một từ đang soạn thảo, engine tự động nhận diện đây là cú pháp định danh lập trình.
  - Tự động hoàn nguyên các dấu phụ nếu có ở phần tiền tố và khóa chế độ tiếng Anh cho toàn bộ định danh cho tới khi gặp phím phân cách (khoảng trắng, dấu chấm, ngoặc...).

### 6.2. Từ Điển Công Nghệ Mở Rộng
Tích hợp sẵn danh sách nhận diện hơn 40 công cụ phát triển, framework và lệnh DevOps phổ biến:
`npm`, `pnpm`, `yarn`, `cargo`, `docker`, `pacman`, `makepkg`, `k8s`, `kubectl`, `postgres`, `redis`, `nextjs`, `tailwind`, `prisma`, `vitest`, `wasm`, `tauri`...

---

## 7. Cơ Chế Quản Lý & Bảo Vệ Chống Trùng Lặp Macro (Macro Collision Guard)

Để đảm bảo hiệu năng và tính nhất quán khi người dùng định nghĩa hoặc nạp hàng loạt phím tắt gõ tắt:

1. **Kiểm Tra Trùng Lặp Không Phân Biệt Hoa/Thường (Case-Insensitive Deduplication)**:
   - Các phím tắt như `vn` và `VN` được xem là cùng một khóa gõ tắt để tránh xung đột hành vi khi gõ.
   - Khi nạp macro từ tệp tin (`saigon-xua-macro.conf` hoặc tệp tùy chỉnh), engine tự động lọc bỏ các phím tắt đã tồn tại, chỉ thêm mới các phím tắt chưa có.
2. **Cảnh Báo & Dọn Dẹp Trực Quan Trên Giao Diện (Settings UI Collision Warning)**:
   - Nếu xuất hiện 2 phím tắt trùng nhau trong bảng chỉnh sửa, giao diện gắn nhãn cảnh báo đỏ `⚠️ Trùng phím!` ngay tại dòng đó và hiển thị thanh thông báo phía trên.
   - Nút `🧹 Dọn Dẹp Trùng Lặp` cho phép người dùng chuẩn hóa lại toàn bộ bảng macro chỉ với 1 cú click.
3. **Tự Động Chuẩn Hóa Khi Lưu (Auto Clean-up on Save)**:
   - Trước khi ghi xuống tệp `vnchkey-macro-table.conf`, hàm `save_macros` luôn tự động loại bỏ trùng lặp và chuốt sạch khoảng trắng thừa, đảm bảo tệp cấu hình của người dùng luôn tinh gọn và an toàn 100%.

---

## 8. Macro Biến Động Theo Múi Giờ Hệ Thống Local & Tìm Kiếm Nhanh

1. **Dynamic Tokens Thời Gian Thực**:
   - Cho phép sử dụng các biến động thời gian thực trong bảng gõ tắt:
     - `$DATE`: Ngày hiện tại định dạng `DD/MM/YYYY`.
     - `$TIME`: Giờ hiện tại định dạng `HH:MM`.
     - `$NOW` / `$DATETIME`: Ngày và giờ `DD/MM/YYYY HH:MM`.
     - `$YEAR`: Năm hiện tại `YYYY`.
   - **Tương Thích Múi Giờ Toàn Cầu**: Tự động truy vấn thông qua múi giờ địa phương (local timezone) của hệ điều hành, đảm bảo hiển thị chuẩn xác cho người dùng ở bất kỳ quốc gia nào.
2. **Tìm Kiếm Macro Tức Thì**:
   - Giao diện `vnchkey-settings` trang bị thanh tìm kiếm trực quan (`🔍`) giúp lọc nhanh hàng trăm phím tắt theo từ khóa chỉ trong vài mili-giây.
