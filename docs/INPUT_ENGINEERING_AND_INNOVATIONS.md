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

## 5. Hoàn Nguyên Chữ Thô Bằng Phím `Shift + Escape` & Bảo Toàn `Escape` Cho Neovim

- **Bài toán thực tế trên Linux / Neovim**: Khi lập trình viên gõ tiếng Việt trong Neovim (ví dụ `xin chào`) rồi bấm `Escape` để thoát `INSERT` mode mà không bấm `Space` cuối câu, nếu bộ gõ bắt phím `Escape` đơn thuần để hủy dấu thì chữ `chào` sẽ bị biến thành `chaof` và phím `Escape` bị nuốt mất.
- **Giải pháp hài hòa của vnchKey**:
  1. **Khi nhấn `Escape` đơn thuần**:
     - Bộ gõ giữ nguyên chữ tiếng Việt vừa gõ (`chào`), chốt bộ đệm từ và **truyền thẳng sự kiện `Escape` xuống cho ứng dụng** (Neovim thoát `INSERT` mode ngay lập tức ở lần bấm đầu tiên; VS Code/Zed đóng bảng gợi ý Autocomplete mượt mà).
  2. **Khi nhấn `Shift + Escape` (hoặc `Ctrl + Shift + Z`)**:
     - Engine tính toán Micro-Diff giữa từ tiếng Việt hiện tại (`tiếng`) và chuỗi phím thô đã gõ (`tieengs`).
     - Phát số lượng Backspace tối thiểu và chèn lại đúng chuỗi phím gốc.
     - Đánh dấu cờ khóa tiếng Anh (`english_locked = true`) cho từ hiện tại để người dùng có thể tiếp tục gõ mà không bị biến đổi lại.

---

## 6. Nhận Diện Mã Lập Trình Thông Minh (Developer-Aware Mode)

### 6.1. Nhận Diện Chuyển Tiếp camelCase / PascalCase
Trong mã nguồn, các định danh thường được viết dạng `camelCase` hoặc `PascalCase` (ví dụ `parseString`, `ClassName`). Khi người dùng gõ:
`parse` + `S` $\to$ nếu không có cơ chế bảo vệ, phím `S` sẽ biến `parse` thành chữ tiếng Việt có dấu.
- **Thuật toán của vnchKey**:
  - Khi một chữ hoa xuất hiện ngay sau một hoặc nhiều chữ thường trong cùng một từ đang soạn thảo, engine tự động nhận diện đây là cú pháp định danh lập trình.
  - Tự động hoàn nguyên các dấu phụ nếu có ở phần tiền tố và khóa chế độ tiếng Anh cho toàn bộ định danh cho tới khi gặp phím phân cách (khoảng trắng, dấu chấm, ngoặc...).

### 6.2. Tiền Tố Gạch Dưới & Snake Case (`_variable`, `__init__`, `user_id`)
- **Từ bắt đầu bằng gạch dưới**: Khi gõ ký tự `_` ở đầu từ (ví dụ `_var`, `__init__`), engine lập tức khóa chế độ tiếng Anh (`english_locked = true`), đảm bảo các biến private hay hàm đặc biệt không bị dính dấu tiếng Việt.
- **Dạng snake_case**: Khi ký tự `_` xuất hiện giữa các chữ cái (ví dụ `user_id`, `parse_string`), engine tự động nhận diện đây là biến mã nguồn, hoàn nguyên dấu phụ nếu có ở tiền tố và bảo toàn định dạng thô.

### 6.3. Truy Xuất Phương Thức / Thuộc Tính Chuỗi Chấm (`db.collection.findOne`)
- Khi một dấu chấm `.` được gõ ngay sau một định danh mà không có khoảng trắng theo sau (khác với dấu chấm câu trong văn bản tiếng Việt), engine phát hiện đây là chuỗi truy xuất code (chained property/method call).
- Từ tiếp theo ngay sau dấu chấm tự động được bảo vệ ở chế độ tiếng Anh, triệt tiêu 100% hiện tượng nhảy dấu khi lập trình.

### 6.4. Từ Điển Công Nghệ Mở Rộng
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

---

## 9. Đột Phá Trải Nghiệm Gõ Macro: Hoàn Tác 2 Bước & Tự Động Hút Dấu Câu

### 9.1. Tự Động Hút Dấu Câu (Smart Punctuation Snapping)
- **Vấn đề thực tế**: Khi người dùng gõ phím tắt `vn` + `Space` để bung cụm từ `"Việt Nam "`, đuôi từ luôn kèm một khoảng trắng. Nếu người dùng muốn viết tiếp dấu phẩy hay chấm (`"Việt Nam, "` hoặc `"Việt Nam!"`), việc xuất hiện khoảng trắng trước dấu câu là sai quy chuẩn chính tả nghiêm trọng.
- **Giải pháp của vnchKey**: Khi phát hiện ký tự tiếp theo ngay sau khi bung macro là một dấu câu thuộc nhóm `[ , . ; : ! ? ) ] } > ]`:
  - Engine tự động phát $1$ Backspace để triệt tiêu ký tự Space thừa và dán dấu câu dính sát vào đuôi từ (`"Việt Nam,"`). Người dùng gõ hoàn toàn tự nhiên mà không cần bận tâm về khoảng trắng.

### 9.2. Hoàn Tác Macro 2 Bước (2-Step Macro Backspace Undo)
- **Bước 1 (1st Backspace)**: Thay vì xóa sạch toàn bộ cụm từ vừa bung như các bộ gõ truyền thống, lần Backspace đầu tiên chỉ xóa đúng $1$ ký tự Space ở đuôi (`"Việt Nam "` $\to$ `"Việt Nam"`). Người dùng có thể tiếp tục gõ dấu câu hoặc ký tự tùy ý.
- **Bước 2 (2nd Backspace liên tiếp)**: Nếu người dùng thực sự muốn hủy bỏ cụm từ đã bung, lần Backspace thứ hai sẽ xóa `"Việt Nam"` và phục hồi lại chuỗi phím tắt gốc (`"vn"`).
- **Chống Lặp Vô Hạn (Suppression Guard)**: Từ phục hồi được đưa vào trạng thái `suppressed_macro`. Nếu người dùng quyết định giữ lại phím tắt đó và nhấn `Space`, từ sẽ không bị bung lặp vô hạn mà xuất ra khoảng trắng bình thường.

---

## 10. Tiện Ích Cứu Chữ & Cứu Dấu (Ngó Sen Style Rescue Innovations)

Kế thừa phát kiến xuất sắc của bộ gõ Ngó Sen (`fcitx5-ngosen`), vnchKey tích hợp đồng bộ cả 2 giải pháp "chữa cháy" thông minh:

### 10.1. Tự Động Cứu Chữ Khi Bật Bộ Gõ (On-Activation Word Rescue)
- **Bản chất vấn đề**: Người dùng đang để bàn phím tiếng Anh, mải mê gõ một đoạn văn bản dài mà quên bật bộ gõ tiếng Việt (ví dụ: `Chaof banj, tieengs vieetj nef`). Khi nhận ra và bấm phím tắt chuyển sang vnchKey (`Ctrl + Space` hoặc `Super + Space`), ở các bộ gõ truyền thống, người dùng phải xóa đi gõ lại toàn bộ rất mất thời gian.
- **Giải pháp của vnchKey**:
  - Tại thời điểm nhận sự kiện kích hoạt (`VnchkeyEngine::activate`), addon tự động đọc chuỗi văn bản nằm ngay trước con trỏ thông qua `ic->surroundingText()`.
  - Phân tích và đưa chuỗi thô qua bộ giải mã `vnchkey-core::rescue_sentence`.
  - Nhận diện các từ vựng tiếng Việt có dấu (`Chaof banj, tieengs vieetj nef` $\to$ `Chào bạn, tiếng việt nè`), bảo toàn nguyên vẹn các từ tiếng Anh chuẩn (`server`, `linux`, `window`).
  - Xóa chuỗi thô cũ và dán trực tiếp chuỗi tiếng Việt đã được cứu vào tài liệu.
  - **Cơ chế Hoàn tác an toàn (Escape/Backspace Undo)**: Nếu việc tự động cứu chữ chưa đúng ý muốn người dùng, chỉ cần bấm phím **`Escape`** hoặc **`Backspace`**, bộ gõ sẽ hoàn trả nguyên trạng chuỗi thô ban đầu ngay lập tức.

### 10.2. Cứu Dấu Trên Vùng Bôi Đen (Selection Accent Rescue)
- **Bản chất vấn đề**: Người dùng có sẵn một từ trên màn hình nhưng muốn thêm dấu, đổi dấu thanh hoặc xóa dấu mà không muốn xóa cả từ.
- **Giải pháp của vnchKey**:
  - Người dùng dùng chuột hoặc bàn phím bôi đen (highlight) từ cần sửa (ví dụ `nhanh`).
  - Gõ phím dấu thanh hoặc phím mũ nguyên âm:
    - `nhanh` + `s` $\to$ `nhánh`
    - `nhánh` + `f` $\to$ `nhành`
    - `nhánh` + `z` $\to$ `nhanh` (xóa dấu)
    - `hoa` + `r` $\to$ `hỏa`
    - `tieng` + `s` $\to$ `tiếng`
    - Chế độ VNI: `nhanh` + `1` $\to$ `nhánh`, `nước` + `0` $\to$ `nươc`.
  - Nếu phím bấm không phải là phím dấu/mũ hợp lệ (ví dụ gõ chữ `k`), bộ gõ tự động giữ nguyên hành vi chuẩn của hệ thống (gõ đè ký tự mới lên vùng chọn).

---

## 11. Hỗ Trợ Bảng Mã Unicode Tổ Hợp (NFD) & Nhập Liệu Macro Đa Định Dạng

### 11.1. Bảng Mã Unicode Tổ Hợp Sâu (Decomposed Unicode / NFD)
- **Vấn đề thực tế**: Một số phần mềm đồ họa, thiết kế hoặc ứng dụng cổ điển (Inkscape, Blender bản cũ, một số phần mềm CAD) chạy qua lớp tương thích XWayland thường gặp lỗi hiển thị ô vuông hoặc không tìm kiếm được văn bản tiếng Việt khi dùng Unicode Dựng Sẵn (NFC).
- **Giải pháp của vnchKey**:
  - Tích hợp chuẩn mã hóa `Charset::UnicodeComposite` trực tiếp từ lõi `vnchkey-core`.
  - Hàm `to_composite_unicode` phân tách canonical chính xác toàn bộ nguyên âm có dấu thành nguyên âm cơ sở + ký tự dấu kết hợp (combining diacritics `U+0300`, `U+0301`, `U+0302`, `U+0303`, `U+0306`, `U+0309`, `U+031B`, `U+0323`).
  - Người dùng có thể dễ dàng chuyển đổi giữa *Unicode Dựng Sẵn (NFC)* và *Unicode Tổ Hợp (NFD)* trực tiếp từ menu cấu hình của `vnchkey-settings`.

### 11.2. Đồng Bộ Macro Đa Nền Tảng (UniKey, EVKey, Ngó Sen, vnchKey)
- Engine `MacroTable` và giao diện cấu hình tự động nhận diện và phân tích cú pháp của mọi định dạng file gõ tắt phổ biến:
  - Định dạng vnchKey (`k=v`)
  - Định dạng UniKey / EVKey (`k:v` hoặc `k;v`)
  - Định dạng Ngó Sen (`k=v` hoặc `k:v`)
- Người dùng có thể trực tiếp sao chép các tệp `ukmacro.txt` từ Windows hoặc các bộ gõ khác vào vnchKey mà không cần bất kỳ bước chuyển đổi định dạng thủ công nào.

---

## 12. Cơ Chế Khôi Phục Tiếng Anh Thông Minh Khi Trùng Phím Undo (Double-Tap Trap)

- **Vấn đề thực tế**: Trong kiểu gõ Telex, các phím lặp như `rr`, `ss`, `ff`, `xx`, `jj` đóng vai trò là phím hủy dấu (Undo). Khi gõ các từ tiếng Anh kỹ thuật chứa ký tự lặp (`error`, `mirror`, `sorry`), ký tự lặp thứ hai kích hoạt hoàn tác dấu khiến một chữ cái bị nuốt mất trên màn hình (`error` $\to$ `eror`). Vì `eror` không còn mang dấu tiếng Việt, các bộ gõ truyền thống không nhận diện được để hoàn tác.
- **Giải pháp của vnchKey**:
  - Engine theo dõi chuỗi phím thô (`raw_strokes`).
  - Khi nhấn phím `Space`, engine đối chiếu `raw_strokes` với kho từ khóa tiếng Anh kỹ thuật và lập trình (`is_english_keyword`).
  - Nếu `raw_strokes` là một từ tiếng Anh hợp lệ, engine lập tức tự động hoàn nguyên về nguyên bản (`error `, `mirror `), đảm bảo lập trình viên gõ code và lệnh console mượt mà không bị ngắt quãng.





