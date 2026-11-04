# Nền Tảng Ngôn Ngữ Học & Cơ Chế Hoạt Động của vnchKey

Tài liệu này phân tích chi tiết nền tảng ngữ âm học tiếng Việt (dựa trên các nghiên cứu ngôn ngữ học tiếng Việt truyền thống, tài liệu `vietnamese-language-system.md` từ dự án Gõ Nhanh, và kho tàng ngữ liệu *e-cadao.com/Ngonngu*) cùng cơ chế chuyển đổi âm tiết tốc độ cao, thông minh và an toàn của **vnchKey (Tự Do Edition)**.

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

Khác với tiếng Anh, cấu tạo âm tiết tiếng Việt tuân thủ các định luật ngữ âm cực kỳ nghiêm ngặt:

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

Theo khảo sát ngôn ngữ học dân gian miền Nam (nguồn *e-cadao.com*):
- Phương ngữ Nam Bộ bảo tồn nhiều từ cổ mang tính văn hóa và phát âm đặc trưng:
  - `chánh` (trong chánh trực, chánh quyền, chánh tả)
  - `nhứt` (nhứt quán, số nhứt)
  - `thiệt` (thật thà $\rightarrow$ thiệt thà)
  - Đại từ xưng hô địa phương: `ảnh` (anh ấy), `bả` (bà ấy), `ổng` (ông ấy), `cổ` (cô ấy)
  - Thán từ/trợ từ: `nghen`, `hổng` (không), `bển` (bên ấy), `trển` (trên ấy)
- **Từ lóng hiện đại Nam Bộ**:
  - Âm xát `d/gi` trong phương ngữ Nam Bộ phát âm tự nhiên tương tự âm `dz`. Người dùng thường gõ tiếng lóng: `dzìa` (về / dìa), `dzô` (vào / vô), `dzí` (với / dí), `dzậy` (vậy), `dza` (da).
  - **Quy tắc của vnchKey**: Hệ thống kiểm tra chánh tả nhận diện và cho phép toàn bộ các biến thể này đi qua mượt mà, không bị coi là lỗi ngữ pháp.

---

## 5. Các Trụ Cột Đột Phá Giúp vnchKey Đạt Hiệu Suất Cao

| Tính năng | Giải pháp kỹ thuật trong vnchKey | Lợi ích cho người dùng |
| :--- | :--- | :--- |
| **Micro-Diff LCP Replacement** | Thuật toán so khớp tiền tố chung dài nhất (Longest Common Prefix). Chỉ gửi số lượng Backspace tối thiểu cho đúng hậu tố bị thay đổi. | Triệt tiêu hoàn toàn hiện tượng nhấp nháy chữ (flicker) trên Wayland/X11. |
| **Restore English Tức Thời** | Bộ từ điển ~360 từ khóa lập trình & thuật ngữ tiếng Anh thông dụng kết hợp kiểm tra luật ngữ âm. | Gõ `text`, `user`, `filters`, `windows` không bị biến thành `têt`, `úẻ`, `ưindoưs`. Tự khôi phục ngay trên màn hình. |
| **Chánh Tả Tích Hợp** | ~6,750 âm tiết tiếng Việt nạp vào `OnceLock<HashSet<String>>` với độ phức tạp tra cứu $O(1)$. | Bắt lỗi chính tả tức thì mà không tiêu tốn bộ nhớ hay làm chậm máy. |
| **100% Native Rust & C-ABI** | Lõi logic viết bằng Safe Rust (`forbid(unsafe_code)`), giao tiếp C-ABI siêu mỏng với Fcitx5 C++. | Khởi động tức thì (< 2ms), tiêu thụ RAM < 10MB, an toàn tuyệt đối trước memory leak. |
