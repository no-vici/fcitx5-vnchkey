# Nền Tảng Ngôn Ngữ Học & Cơ Chế Hoạt Động của vnchKey

Tài liệu này phân tích chi tiết nền tảng ngữ âm học Tiếng Việt (dựa trên các nghiên cứu ngôn ngữ học Tiếng Việt truyền thống, tài liệu `vietnamese-language-system.md` từ dự án Gõ Nhanh, và kho tàng ngữ liệu *e-cadao.com/Ngonngu*) cùng cơ chế chuyển đổi âm tiết tốc độ cao, thông minh và an toàn của **vnchKey (Tự Do Edition)**.

---

## 1. Cấu Trúc Âm Tiết Tiếng Việt (Vietnamese Syllable Structure)

Tiếng Việt là ngôn ngữ đơn lập (isolating language), đơn vị cơ sở là **âm tiết** (tiếng). Mỗi âm tiết toàn vẹn tuân theo công thức cấu âm 5 thành phần:

$$\text{Âm tiết} = C_1 + G + V + C_2 + T$$

Trong đó:
1. **$C_1$ (Âm đầu - Initial Consonant)**:
   - Các phụ âm đơn: `b`, `c`, `d`, `đ`, `g`, `h`, `k`, `l`, `m`, `n`, `p`, `r`, `s`, `t`, `v`, `x`.
   - Các phụ âm ghép: `ch`, `gh`, `gi`, `kh`, `nh`, `ng`, `ngh`, `ph`, `qu`, `th`, `tr`.
   - Khuyết âm đầu: ví dụ `áo`, `em`, `út`.
2. **$G$ (Âm đệm - Glide / Onset Glide)**:
   - Thể hiện bằng ký tự `u` hoặc `o` (ví dụ: `oa`, `oe`, `uê`, `uy`). Trong từ có âm đầu `qu-`, chữ `u` đóng vai trò âm đệm; trong `gi-`, chữ `i` là bán nguyên âm hoặc phụ âm mặt lưỡi kết hợp.
3. **$V$ (Âm chính / Hạt nhân - Vowel Nucleus)**:
   - Nguyên âm đơn ngắn & dài: `a`, `ă`, `â`, `e`, `ê`, `i`, `y`, `o`, `ô`, `ơ`, `u`, `ư`.
   - Nguyên âm đôi: `ia`/`iê`/`yê`, `ua`/`uô`, `ưa`/`ươ`.
   - Nguyên âm ba: `iêu`, `oay`, `ươu`, `uôi`, `uyê`.
4. **$C_2$ (Âm cuối - Final Consonant / Offglide)**:
   - Bán âm cuối: `-i`, `-y`, `-u`, `-o` (ví dụ: `mai`, `mây`, `cau`, `mèo`).
   - Phụ âm tắc/khép (Stop finals): `-c`, `-ch`, `-p`, `-t`.
   - Phụ âm vang (Nasal finals): `-m`, `-n`, `-ng`, `-nh`.
   - Khuyết âm cuối (âm tiết mở): ví dụ `ba`, `mẹ`, `nhà`.
5. **$T$ (Thanh điệu - Tone)**:
   - 6 thanh: **Ngang** (Không dấu), **Huyền** (Grave), **Sắc** (Acute), **Hỏi** (Hook above), **Ngã** (Tilde), **Nặng** (Dot below).

---

## 2. Quy Tắc Ngữ Âm Học (Phonology Constraints)

Khác với Tiếng Anh, cấu tạo âm tiết Tiếng Việt tuân thủ các định luật ngữ âm cực kỳ nghiêm ngặt:

### Quy tắc 1: Cặp chính tả `c/k`
- Phụ âm `c` **không bao giờ** đi trước các nguyên âm dòng trước `e`, `ê`, `i`, `y`.
- Nếu âm chính là `e`, `ê`, `i`, `y`, chữ viết bắt buộc dùng phụ âm `k` (`kẻ`, `kiếm`, `kỳ`). Do đó các tổ hợp như `center`, `city`, `cell`, `circle` lập tức bị nhận diện là tiếng nước ngoài và không bị bỏ dấu nhầm.

### Quy tắc 2: Cặp chính tả `g/gh` & `ng/ngh`
- `g` và `ng` **không bao giờ** đứng trực tiếp trước `e`, `ê`, `i`. Bắt buộc phải thêm `h` thành `gh` (`ghế`, `ghi`) và `ngh` (`nghe`, `nghệ`, `nghĩ`).
- Do đó các từ như `get`, `general`, `gear`, `danger`, `engine` không bao giờ tạo thành âm tiết tiếng Việt hợp lệ.

### Quy tắc 3: Luật Thanh Tắc (Checked Tone Rule)
- Các âm tiết kết thúc bằng phụ âm tắc vô thanh ($-c, -ch, -p, -t$) **chỉ có thể mang 2 thanh**: thanh **Sắc** (Tone 1) hoặc thanh **Nặng** (Tone 5).
- Nghiêm cấm mang thanh Huyền, Hỏi, Ngã. Ví dụ: `bác`, `bạc`, `mát`, `mạt` là hợp lệ; `bàc`, `thỉch`, `bẽp` là sai ngữ âm 100%.

---

## 3. Quy Ước Bỏ Dấu Thanh: Dấu Mới (Modern) vs Dấu Cũ (Classic)

vnchKey hỗ trợ cả 2 phong cách bỏ dấu thanh:

1. **Chuẩn Mới (Modern Standard)**:
   - Đặt dấu thanh vào **nguyên âm chính** (hạt nhân âm tiết mang trọng âm thực sự).
   - Ví dụ: `hoà` (o là âm đệm, a là âm chính), `toán`, `thuỷ` (u là âm đệm, y là âm chính), `thuế`, `khoẻ`.
2. **Chuẩn Cũ (Classic Standard)**:
   - Đặt dấu thanh vào nguyên âm đệm đứng trước hoặc theo thói quen in ấn từ điển thời trước 1980.
   - Ví dụ: `hòa`, `thủy`, `khỏe`.

**Bảng mã chuẩn**: vnchKey sử dụng thuần túy **Unicode Dựng Sẵn (NFC)** chuẩn quốc tế, loại bỏ hoàn toàn các bảng mã lỗi thời như TCVN3/VNI-Windows để đạt độ tương thích 100% trên mọi ứng dụng hiện đại.

---

## 4. Đặc Thù Ngữ Liệu Phương Ngữ Miền Nam & Tiếng Lóng

Theo khảo sát ngôn ngữ học dân gian miền Nam (nguồn *e-cadao.com* và tuyển tập *"Gom góp từ ngữ của miền Nam và Saigon xưa"* của tác giả Nguyễn Cao Trường):
- Phương ngữ Nam Bộ bảo tồn nhiều từ cổ mang tính văn hóa và phát âm đặc trưng:
  - `chánh` (trong chánh trực, chánh quyền, chánh tả)
  - `nhứt` (nhứt quán, số nhứt)
  - `thiệt` (thật thà $\rightarrow$ thiệt thà)
  - Đại từ xưng hô địa phương: `ảnh` (anh ấy), `bả` (bà ấy), `ổng` (ông ấy), `cổ` (cô ấy)
  - Thán từ/trợ từ: `nghen`, `hổng` (không), `bển` (bên ấy), `trển` (trên ấy), `mèn ơi`, `mèn đéc ơi`
  - Từ ngữ sinh hoạt & khẩu ngữ trứ danh: `quởn` / `quỡn` (rảnh rỗi, thong thả), `số dzách`, `trớt quớt`, `cà chớn`, `bá cháy bù chét`, `hết xí quách`, `bảnh tỏn`, `làm nư`, `chùm hum`, `quỉ tha ma bắt`, `thọc cù lét`, `xề xòa`...
- **Từ lóng hiện đại Nam Bộ**:
  - Âm xát `d/gi` trong phương ngữ Nam Bộ phát âm tự nhiên tương tự âm `dz`. Người dùng thường gõ tiếng lóng: `dzìa` (về / dìa), `dzô` (vào / vô), `dzí` (với / dí), `dzậy` (vậy), `dza` (da).
  - **Quy tắc của vnchKey**: Hệ thống kiểm tra chánh tả nhận diện và cho phép toàn bộ các biến thể này đi qua mượt mà, không bị coi là lỗi ngữ pháp và bảo vệ an toàn trước cơ chế Auto-Restore English.

---

## 5. Các Trụ Cột Đột Phá Giúp vnchKey Đạt Hiệu Suất Cao

| Tính năng | Giải pháp kỹ thuật trong vnchKey | Lợi ích cho người dùng |
| :--- | :--- | :--- |
| **Micro-Diff LCP Replacement** | Thuật toán so khớp tiền tố chung dài nhất (Longest Common Prefix). Chỉ gởi số lượng Backspace tối thiểu cho đúng hậu tố bị thay đổi. | Triệt tiêu hoàn toàn hiện tượng nhấp nháy chữ (flicker) trên Wayland/X11. |
| **Restore English Tức Thời** | Bộ từ điển ~360 từ khóa lập trình & thuật ngữ tiếng Anh thông dụng kết hợp kiểm tra luật ngữ âm. | Gõ `text`, `user`, `filters`, `windows` không bị biến thành `têt`, `úẻ`, `ưindoưs`. Tự khôi phục ngay trên màn hình. |
| **Chánh Tả Tích Hợp** | ~6,750 âm tiết tiếng Việt nạp vào `OnceLock<HashSet<String>>` với độ phức tạp tra cứu $O(1)$. | Bắt lỗi chính tả tức thì mà không tiêu tốn bộ nhớ hay làm chậm máy. |
| **100% Native Rust & C-ABI** | Lõi logic viết bằng Safe Rust (`forbid(unsafe_code)`), giao tiếp C-ABI siêu mỏng với Fcitx5 C++. | Khởi động tức thì (< 2ms), tiêu thụ RAM < 10MB, an toàn tuyệt đối trước memory leak. |

---

## 6. Xử Lý Các Góc Khuất Ngữ Âm Kinh Điển (Linguistic Edge Cases)

### 6.1. Bộ Ba Ngoại Lệ Của Vần `uơ` (The Trio of `uơ`)
Trong tiếng Việt, vần `uo` đi với phím râu thông thường sẽ biến thành nguyên âm đôi `ươ` (`duoc` $\to$ `được`, `nuoc` $\to$ `nước`). Tuy nhiên, **chỉ có duy nhất 3 trường hợp** chữ `u` đóng vai trò âm đệm và **chỉ có chữ `o` được nhận dấu móc `ơ`**:
1. **`thuở`** *(muôn thuở, thuở ấy)*: `thuo` + `w`/`7` $\to$ `thuơ` $\to$ `thuở` *(tuyệt đối không thành `thưở`)*.
2. **`quởn` / `quỡn` / `quơ`** *(quởn đời, quơ tay)*: Trong tiếng Việt, phụ âm `q` **không bao giờ đi với `ư`** (không tồn tại `qư` hay `qươn`). Do đó `quon` + `w`/`7` $\to$ `quơn` $\to$ `quởn` *(chữ `u` giữ nguyên)*.
3. **`huơ`** *(huơ tay, huơ dao)*: Tiếng Việt không có từ `hươ` đơn lẻ (chỉ có con `hươu` có âm cuối `u`). Khi kết thúc bằng `o`, từ đúng phải là `huơ` (`huo` + `w`/`7` $\to$ `huơ`).

### 6.2. Cụm Nguyên Âm Ba Dị Biệt: `khuỷu` và `ngoèo`
- **`khuỷu`** *(khuỷu tay)*: Cấu tạo Onset `kh` + Glide `u` + Nucleus `y` + Coda `u`. Dấu hỏi bắt buộc phải nằm trên chữ `y` hạt nhân $\to$ `khuỷu` *(chống lỗi rớt dấu thành `khủyu` hay `khuyủ`)*.
- **`ngoèo` / `khoèo`** *(ngoằn ngoèo)*: Cấu tạo Onset `ng` + Glide `o` + Nucleus `e` + Coda `o`. Dấu huyền bắt buộc phải nằm trên `e` ở giữa $\to$ `ngoèo`.

### 6.3. Sự Biến Ảo Của Chữ Cái `i` Trong Âm Đầu `gi`
- Đi cùng nguyên âm khác: `gió`, `giúp`, `giữa`, `giết`, `gieo` (chữ `i` hòa vào phụ âm đầu `gi`, dấu đặt lên nguyên âm sau).
- Đi cùng âm cuối phụ âm: `gìn` *(giữ gìn)*, `gịt`, `gíp` (chỉ có 1 chữ `i` làm hạt nhân âm tiết mang dấu, không sinh lỗi `giìn`).
- Đứng một mình: `gì`, `gí` (chữ `i` chính là âm chính mang dấu).

### 6.4. Vần Đôi `oo` Gốc Pháp: `xoong`, `boong`, `goòng`
- Các từ mượn gốc Pháp (`casserole` $\to$ cái xoong, `pont` $\to$ boong tàu, `wagon` $\to$ xe goòng) được công nhận trong chính tả chuẩn.
- Khi người dùng gõ `xoong` hay `boong`, engine không tự ý gộp thành `xông` hay `bông`. Khi bỏ dấu (`goong` + huyền) $\to$ đặt dấu trên chữ `o` thứ hai thành `goòng`.

### 6.5. Cụm `qu` Và Âm Chính `y`: `quý`, `suýt`, `chuyện`
- `quý`, `quỳ`, `quỷ`: Chữ `u` trong `qu` là âm đệm, dấu thanh luôn luôn nằm trên `y` *(không bao giờ trên `u`)*.
- `suýt`: Dấu sắc bắt buộc nằm trên `y` $\to$ `suýt`.
- Cụm `u-y-ê` (`chuyện`, `khuyên`, `quyết`): Dù theo chuẩn Mới hay Cổ Điển, nguyên âm `ê` có độ ưu tiên tuyệt đối $\to$ `chuyện`, `quyết`.

### 6.6. Độ Ưu Tiên Tuyệt Đối Của Nguyên Âm Có Dấu Phụ (`ê, ơ, â, ă, ô`) Trong Phong Cách Cổ Điển (Classic)
- **Quy tắc phân biệt**: Trong phong cách Cổ Điển (Classic), các nhị trùng âm mở không có phụ âm cuối như `oa`, `oe`, `uy` đặt dấu thanh lên âm đệm đi trước (`hòa`, `khỏe`, `thủy`).
- **Ngoại lệ hạt nhân có dấu phụ**: Khi âm chính là nguyên âm mang dấu phụ/mũ (`ê`, `ơ`, `â`, `ă`, `ô`) như trong `thuê`, `huê`, `quê`, `thuở`, `huơ`:
  - Trọng âm ngữ âm tập trung toàn bộ vào hạt nhân mang mũ/móc.
  - Do đó, dấu thanh **luôn luôn rơi vào hạt nhân** mang mũ/móc: `thue61` $\to$ `thuế`, `hue61` $\to$ `huế` *(tuyệt đối không bị lỗi rớt dấu lên âm đệm thành `thúê`, `húê`)*.

### 6.7. Cơ Chế Gõ Dấu Trung Gian Tự Do & Khôi Phục Raw Text Khi Nhấn Space
- **Quy tắc ngữ âm chuẩn cho nguyên âm đôi khép kín (`uo`, `ie`, `ye`)**:
  - Khi đứng trước phụ âm cuối, các nguyên âm đôi `uo`, `ie`, `ye` chưa có dấu mũ/móc (`ươ`, `uô`, `iê`, `yê`) **không thể mang dấu thanh** trong ngữ âm tiếng Việt chuẩn.
  - Tuy nhiên, để tối ưu trải nghiệm gõ phím linh hoạt, vnchKey cho phép người dùng **gõ dấu nào trước cũng được** (ví dụ: gõ phím thanh trước phím mũ/móc: `duong` $\to$ `duong2` $\to$ `duong27` $\to$ `dường`, hoặc `Huyen` $\to$ `Huyèn` $\to$ `Huyền`).
  - **Cơ chế khôi phục Raw Text**: Nếu người dùng nhấn phím `Space` khi từ vẫn đang ở trạng thái trung gian chưa hoàn tất (chưa có dấu thứ 2 để tạo thành từ có nghĩa tiếng Việt như `duong2`, `tuongf`, `nuoc1`, `tiens`, `Huyèn`), engine sẽ **tự động hoàn nguyên về raw strokes** (`duong2 `, `tuongf `, `Huyen2 `), ngăn chặn tuyệt đối lỗi nuốt số trong tài khoản/tên biến hoặc xuất từ vô nghĩa.

---

> [!TIP]
> **Tài Liệu Kỹ Thuật Liên Quan:**
> Để tìm hiểu chi tiết về kiến trúc phần mềm, cơ chế phát phím Micro-Diff, Select-and-Replace trên Web/Electron, khôi phục từ bằng Backspace, Escape to Undo, và hệ thống bảo vệ Macro, xin vui lòng tham khảo:
> 👉 **[INPUT_ENGINEERING_AND_INNOVATIONS.md](file:///home/codegpt/antigravity/vnchKey/docs/INPUT_ENGINEERING_AND_INNOVATIONS.md)**


