#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""
vnchKey Settings Window.
Modern dark theme UI matching modern Linux desktop guidelines and fcitx5-lilypad.
"""

from qtpy.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QComboBox,
    QCheckBox,
    QPushButton,
    QStackedWidget,
    QListWidget,
    QListWidgetItem,
    QFrame,
    QTableWidget,
    QTableWidgetItem,
    QHeaderView,
    QLineEdit,
    QMessageBox,
)
from qtpy.QtCore import Qt, QSize
from qtpy.QtGui import QIcon, QFont, QPixmap
from config_manager import ConfigManager
import os


STYLE_SHEET = """
QWidget {
    background-color: #1e1e2e;
    color: #cdd6f4;
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 13px;
}

QListWidget#sidebar {
    background-color: #181825;
    border: none;
    border-right: 1px solid #313244;
    padding: 12px 8px;
    outline: none;
}

QListWidget#sidebar::item {
    height: 38px;
    padding-left: 12px;
    margin-bottom: 4px;
    border-radius: 8px;
    color: #a6adc8;
    font-weight: 500;
}

QListWidget#sidebar::item:hover {
    background-color: #313244;
    color: #cdd6f4;
}

QListWidget#sidebar::item:selected {
    background-color: #5856d6;
    color: #ffffff;
    font-weight: 600;
}

QFrame.card {
    background-color: #252839;
    border: 1px solid #313244;
    border-radius: 10px;
    padding: 16px;
    margin-bottom: 14px;
}

QLabel.page-title {
    font-size: 20px;
    font-weight: 700;
    color: #cdd6f4;
    margin-bottom: 2px;
}

QLabel.page-desc {
    font-size: 12px;
    color: #7f849c;
    margin-bottom: 16px;
}

QLabel.group-title {
    font-size: 13px;
    font-weight: 600;
    color: #89b4fa;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 12px;
}

QLabel.field-label {
    font-size: 13px;
    color: #bac2de;
    font-weight: 400;
}

QComboBox, QLineEdit {
    background-color: #181825;
    border: 1px solid #45475a;
    border-radius: 6px;
    padding: 6px 12px;
    color: #cdd6f4;
    min-width: 180px;
    selection-background-color: #5856d6;
}

QComboBox:hover, QLineEdit:hover {
    border-color: #5856d6;
}

QComboBox::drop-down {
    border: none;
    width: 20px;
}

QCheckBox {
    spacing: 8px;
    color: #cdd6f4;
}

QCheckBox::indicator {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    border: 1px solid #45475a;
    background-color: #181825;
}

QCheckBox::indicator:checked {
    background-color: #5856d6;
    border-color: #5856d6;
}

QPushButton.primary-btn {
    background-color: #5856d6;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 8px 20px;
    font-weight: 600;
}

QPushButton.primary-btn:hover {
    background-color: #4b48c4;
}

QPushButton.secondary-btn {
    background-color: #313244;
    color: #cdd6f4;
    border: 1px solid #45475a;
    border-radius: 6px;
    padding: 6px 14px;
}

QPushButton.secondary-btn:hover {
    background-color: #45475a;
}

QTableWidget {
    background-color: #181825;
    border: 1px solid #313244;
    border-radius: 8px;
    gridline-color: #313244;
}

QHeaderView::section {
    background-color: #252839;
    color: #a6adc8;
    padding: 6px;
    border: none;
    border-bottom: 1px solid #313244;
    font-weight: 600;
}
"""


class VnchkeySettingsWindow(QWidget):
    def __init__(self):
        super().__init__()
        self.mgr = ConfigManager()
        self.config = self.mgr.load_config()
        self.macros = self.mgr.load_macros()
        self.dictionary = self.mgr.load_dictionary()

        self.setWindowTitle("vnchKey Settings - Bộ Gõ Tiếng Việt")
        self.resize(780, 520)
        self.setMinimumSize(700, 460)
        self.setStyleSheet(STYLE_SHEET)

        # Set window icon
        icon_path = "/usr/share/icons/hicolor/scalable/apps/vnchkey.svg"
        if not os.path.exists(icon_path):
            icon_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../data/icons/vnchkey.svg"))
        if os.path.exists(icon_path):
            self.setWindowIcon(QIcon(icon_path))

        self.init_ui()

    def init_ui(self):
        main_layout = QHBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        # Left Sidebar
        self.sidebar = QListWidget()
        self.sidebar.setObjectName("sidebar")
        self.sidebar.setFixedWidth(190)

        items = [
            ("⚙️  General", "General"),
            ("⌨️  Typing", "Typing"),
            ("⚡  Macros", "Macros"),
            ("📖  Dictionary", "Dictionary"),
            ("ℹ️  About", "About"),
        ]

        for text, _ in items:
            it = QListWidgetItem(text)
            self.sidebar.addItem(it)

        self.sidebar.currentRowChanged.connect(self.switch_page)
        main_layout.addWidget(self.sidebar)

        # Right Content Area (Stacked pages)
        content_wrapper = QWidget()
        content_layout = QVBoxLayout(content_wrapper)
        content_layout.setContentsMargins(28, 24, 28, 20)

        self.stack = QStackedWidget()
        self.page_general = self.create_general_page()
        self.page_typing = self.create_typing_page()
        self.page_macros = self.create_macros_page()
        self.page_dictionary = self.create_dictionary_page()
        self.page_about = self.create_about_page()

        self.stack.addWidget(self.page_general)
        self.stack.addWidget(self.page_typing)
        self.stack.addWidget(self.page_macros)
        self.stack.addWidget(self.page_dictionary)
        self.stack.addWidget(self.page_about)

        content_layout.addWidget(self.stack)

        # Bottom Bar: Status + Save Button
        bottom_bar = QHBoxLayout()
        self.status_lbl = QLabel("")
        self.status_lbl.setStyleSheet("color: #a6e3a1; font-weight: 500;")
        bottom_bar.addWidget(self.status_lbl)

        bottom_bar.addStretch()

        self.btn_apply = QPushButton("Lưu Cài Đặt")
        self.btn_apply.setObjectName("primary")
        self.btn_apply.setProperty("class", "primary-btn")
        self.btn_apply.clicked.connect(self.save_all)
        bottom_bar.addWidget(self.btn_apply)

        content_layout.addLayout(bottom_bar)
        main_layout.addWidget(content_wrapper)

        self.sidebar.setCurrentRow(0)

    def switch_page(self, index):
        self.stack.setCurrentIndex(index)

    def create_general_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        # Page Header
        title = QLabel("General")
        title.setProperty("class", "page-title")
        desc = QLabel("Configure basic input method settings and behaviors.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        # Input Method Group Card
        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setSpacing(14)

        grp_title = QLabel("Input Method")
        grp_title.setProperty("class", "group-title")
        card_layout.addWidget(grp_title)

        # 1. Method: Auto / Telex / VNI
        row1 = QHBoxLayout()
        lbl1 = QLabel("Input Method")
        lbl1.setProperty("class", "field-label")
        self.combo_method = QComboBox()
        self.combo_method.addItems(["Auto (Telex & VNI)", "Telex", "VNI"])
        cur_m = self.config.get("InputMethod", "Auto")
        if cur_m == "VNI":
            self.combo_method.setCurrentIndex(2)
        elif cur_m == "Telex":
            self.combo_method.setCurrentIndex(1)
        else:
            self.combo_method.setCurrentIndex(0)
        row1.addWidget(lbl1)
        row1.addStretch()
        row1.addWidget(self.combo_method)
        card_layout.addLayout(row1)

        # 2. Mode
        row2 = QHBoxLayout()
        lbl2 = QLabel("Mode")
        lbl2.setProperty("class", "field-label")
        self.combo_mode = QComboBox()
        self.combo_mode.addItems(["Native Fcitx5 (Direct Commit)", "Uinput (Smooth)"])
        cur_mode = self.config.get("Mode", "Native Fcitx5 (Direct Commit)")
        if "Uinput" in cur_mode:
            self.combo_mode.setCurrentIndex(1)
        else:
            self.combo_mode.setCurrentIndex(0)
        row2.addWidget(lbl2)
        row2.addStretch()
        row2.addWidget(self.combo_mode)
        card_layout.addLayout(row2)

        # 3. Output Charset
        row3 = QHBoxLayout()
        lbl3 = QLabel("Output Charset")
        lbl3.setProperty("class", "field-label")
        self.combo_charset = QComboBox()
        self.combo_charset.addItems(["Unicode", "Unicode Tổ Hợp", "TCVN3 (ABC)"])
        row3.addWidget(lbl3)
        row3.addStretch()
        row3.addWidget(self.combo_charset)
        card_layout.addLayout(row3)

        layout.addWidget(card)

        # Typing Quick Rules Card
        card2 = QFrame()
        card2.setProperty("class", "card")
        card2_layout = QVBoxLayout(card2)
        card2_layout.setSpacing(14)

        grp_title2 = QLabel("Typing")
        grp_title2.setProperty("class", "group-title")
        card2_layout.addWidget(grp_title2)

        row4 = QHBoxLayout()
        lbl4 = QLabel("Type w to Produce ư")
        lbl4.setProperty("class", "field-label")
        self.combo_w2u = QComboBox()
        self.combo_w2u.addItems(["Non-Start (Giữa/Cuối từ)", "Always (Mọi vị trí)", "Disabled (Tắt)"])
        cur_w2u = self.config.get("TypeW2U", "Non-Start")
        if cur_w2u == "Always":
            self.combo_w2u.setCurrentIndex(1)
        elif cur_w2u == "Disabled":
            self.combo_w2u.setCurrentIndex(2)
        else:
            self.combo_w2u.setCurrentIndex(0)
        row4.addWidget(lbl4)
        row4.addStretch()
        row4.addWidget(self.combo_w2u)
        card2_layout.addLayout(row4)

        row5 = QHBoxLayout()
        lbl5 = QLabel("Type [ -> ơ, ] -> ư, { -> Ơ, } -> Ư")
        lbl5.setProperty("class", "field-label")
        self.combo_bracket = QComboBox()
        self.combo_bracket.addItems(["Disabled", "Enabled"])
        row5.addWidget(lbl5)
        row5.addStretch()
        row5.addWidget(self.combo_bracket)
        card2_layout.addLayout(row5)

        layout.addWidget(card2)
        layout.addStretch()
        return w

    def create_typing_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Typing Settings")
        title.setProperty("class", "page-title")
        desc = QLabel("Fine-tune tone placement, smart casing, and English auto-restore.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setSpacing(16)

        grp_title = QLabel("Tone & Spelling Options")
        grp_title.setProperty("class", "group-title")
        card_layout.addWidget(grp_title)

        # Tone Style (Modern vs Classic)
        row_tone = QHBoxLayout()
        lbl_tone = QLabel("Quy tắc đặt dấu thanh")
        lbl_tone.setProperty("class", "field-label")
        self.combo_tone = QComboBox()
        self.combo_tone.addItems(["Chuẩn mới: oà, uý, oè", "Chuẩn cũ: hòa, ủy, hòe"])
        if self.config.get("ModernStyle", "True") == "False":
            self.combo_tone.setCurrentIndex(1)
        else:
            self.combo_tone.setCurrentIndex(0)
        row_tone.addWidget(lbl_tone)
        row_tone.addStretch()
        row_tone.addWidget(self.combo_tone)
        card_layout.addLayout(row_tone)

        # Smart VNI Horn (duoc -> được)
        self.chk_smarthorn = QCheckBox("Tự động thêm dấu râu cho 'uo' (duoc95 -> được, nuoc1 -> nước)")
        self.chk_smarthorn.setChecked(self.config.get("SmartHorn", "True") == "True")
        card_layout.addWidget(self.chk_smarthorn)

        # GoNhanh Auto-Restore English
        self.chk_autorestore = QCheckBox("GoNhanh: Tự động khôi phục từ tiếng Anh khi gõ phím cách (post, text, class...)")
        self.chk_autorestore.setChecked(self.config.get("AutoRestoreEnglish", "True") == "True")
        card_layout.addWidget(self.chk_autorestore)

        # XKey Smart Casing
        self.chk_smartcase = QCheckBox("XKey: Giữ nguyên định dạng chữ hoa/chữ thường thông minh")
        self.chk_smartcase.setChecked(self.config.get("SmartCasing", "True") == "True")
        card_layout.addWidget(self.chk_smartcase)

        layout.addWidget(card)
        layout.addStretch()
        return w

    def create_macros_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Macros (Gõ Tắt)")
        title.setProperty("class", "page-title")
        desc = QLabel("Define shortcuts that automatically expand into full phrases.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        self.chk_enable_macro = QCheckBox("Bật tính năng gõ tắt (Macro expansion)")
        self.chk_enable_macro.setChecked(self.config.get("EnableMacro", "True") == "True")
        layout.addWidget(self.chk_enable_macro)

        # Table
        self.macro_table = QTableWidget(len(self.macros), 2)
        self.macro_table.setHorizontalHeaderLabels(["Từ Viết Tắt (Key)", "Nội Dung Thay Thế (Expansion)"])
        self.macro_table.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)

        for row, (k, v) in enumerate(self.macros):
            self.macro_table.setItem(row, 0, QTableWidgetItem(k))
            self.macro_table.setItem(row, 1, QTableWidgetItem(v))

        layout.addWidget(self.macro_table)

        # Row manipulation buttons
        btn_layout = QHBoxLayout()
        btn_add = QPushButton("+ Thêm Viết Tắt")
        btn_add.setProperty("class", "secondary-btn")
        btn_add.clicked.connect(self.add_macro_row)

        btn_del = QPushButton("- Xoá Mục Đang Chọn")
        btn_del.setProperty("class", "secondary-btn")
        btn_del.clicked.connect(self.del_macro_row)

        btn_layout.addWidget(btn_add)
        btn_layout.addWidget(btn_del)
        btn_layout.addStretch()
        layout.addLayout(btn_layout)

        return w

    def add_macro_row(self):
        row = self.macro_table.rowCount()
        self.macro_table.insertRow(row)
        self.macro_table.setItem(row, 0, QTableWidgetItem(""))
        self.macro_table.setItem(row, 1, QTableWidgetItem(""))
        self.macro_table.editItem(self.macro_table.item(row, 0))

    def del_macro_row(self):
        row = self.macro_table.currentRow()
        if row >= 0:
            self.macro_table.removeRow(row)

    def create_dictionary_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Dictionary (Từ Điển)")
        title.setProperty("class", "page-title")
        desc = QLabel("Manage your custom vocabulary and technical jargon.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)

        card_layout.addWidget(QLabel("Thêm từ mới vào từ điển:"))
        add_row = QHBoxLayout()
        self.txt_new_word = QLineEdit()
        self.txt_new_word.setPlaceholderText("Nhập từ tiếng Việt mới...")
        btn_add_word = QPushButton("+ Thêm")
        btn_add_word.setProperty("class", "secondary-btn")
        btn_add_word.clicked.connect(self.add_dict_word)
        add_row.addWidget(self.txt_new_word)
        add_row.addWidget(btn_add_word)
        card_layout.addLayout(add_row)

        self.list_dict = QListWidget()
        self.list_dict.setStyleSheet("background-color: #181825; border: 1px solid #313244; border-radius: 6px;")
        for word in self.dictionary:
            self.list_dict.addItem(word)
        card_layout.addWidget(self.list_dict)

        btn_del_word = QPushButton("- Xoá Từ Đang Chọn")
        btn_del_word.setProperty("class", "secondary-btn")
        btn_del_word.clicked.connect(self.del_dict_word)
        card_layout.addWidget(btn_del_word)

        layout.addWidget(card)
        return w

    def add_dict_word(self):
        text = self.txt_new_word.text().strip()
        if text:
            self.list_dict.addItem(text)
            self.txt_new_word.clear()

    def del_dict_word(self):
        row = self.list_dict.currentRow()
        if row >= 0:
            self.list_dict.takeItem(row)

    def create_about_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setAlignment(Qt.AlignCenter)
        card_layout.setSpacing(10)

        # Logo display
        logo_lbl = QLabel()
        logo_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../assets/logo.png"))
        if os.path.exists(logo_path):
            pix = QPixmap(logo_path).scaled(96, 96, Qt.KeepAspectRatio, Qt.SmoothTransformation)
            logo_lbl.setPixmap(pix)
        card_layout.addWidget(logo_lbl, alignment=Qt.AlignCenter)

        title = QLabel("vnchKey (Tự Do Edition)")
        title.setStyleSheet("font-size: 20px; font-weight: 700; color: #89b4fa; margin-top: 10px;")
        card_layout.addWidget(title, alignment=Qt.AlignCenter)

        version = QLabel("Phiên bản 0.2.0 • 100% Pure Safe Rust")
        version.setStyleSheet("color: #a6adc8; font-size: 13px;")
        card_layout.addWidget(version, alignment=Qt.AlignCenter)

        info = QLabel(
            "Bộ gõ Tiếng Việt an toàn bộ nhớ hiệu năng cao cho Fcitx5 trên Linux.\n"
            "Tích hợp thuật toán Auto-Restore của GoNhanh và Smart Casing của XKey.\n"
            "Mã nguồn mở hoàn toàn theo giấy phép GPL-3.0."
        )
        info.setAlignment(Qt.AlignCenter)
        info.setStyleSheet("color: #7f849c; margin: 12px 20px; line-height: 1.4;")
        card_layout.addWidget(info)

        link = QLabel("<a href='https://github.com/no-vici/fcitx5-vnchkey' style='color: #89b4fa; text-decoration: none;'>https://github.com/no-vici/fcitx5-vnchkey</a>")
        link.setOpenExternalLinks(True)
        card_layout.addWidget(link, alignment=Qt.AlignCenter)

        layout.addWidget(card)
        layout.addStretch()
        return w

    def save_all(self):
        # 1. Gather general config
        m_idx = self.combo_method.currentIndex()
        if m_idx == 1:
            self.config["InputMethod"] = "Telex"
        elif m_idx == 2:
            self.config["InputMethod"] = "VNI"
        else:
            self.config["InputMethod"] = "Auto"

        self.config["Mode"] = self.combo_mode.currentText()
        self.config["OutputCharset"] = self.combo_charset.currentText()
        self.config["ModernStyle"] = "True" if self.combo_tone.currentIndex() == 0 else "False"
        self.config["SmartHorn"] = "True" if self.chk_smarthorn.isChecked() else "False"
        self.config["AutoRestoreEnglish"] = "True" if self.chk_autorestore.isChecked() else "False"
        self.config["SmartCasing"] = "True" if self.chk_smartcase.isChecked() else "False"
        self.config["EnableMacro"] = "True" if self.chk_enable_macro.isChecked() else "False"

        w2u_idx = self.combo_w2u.currentIndex()
        if w2u_idx == 1:
            self.config["TypeW2U"] = "Always"
        elif w2u_idx == 2:
            self.config["TypeW2U"] = "Disabled"
        else:
            self.config["TypeW2U"] = "Non-Start"

        self.mgr.save_config(self.config)

        # 2. Gather macros
        macros = []
        for r in range(self.macro_table.rowCount()):
            k_item = self.macro_table.item(r, 0)
            v_item = self.macro_table.item(r, 1)
            k = k_item.text().strip() if k_item else ""
            v = v_item.text().strip() if v_item else ""
            if k:
                macros.append((k, v))
        self.mgr.save_macros(macros)

        # 3. Gather dictionary
        dict_words = []
        for i in range(self.list_dict.count()):
            it = self.list_dict.item(i)
            if it and it.text().strip():
                dict_words.append(it.text().strip())
        self.mgr.save_dictionary(dict_words)

        self.status_lbl.setText("✓ Đã lưu cài đặt và cập nhật Fcitx5!")
