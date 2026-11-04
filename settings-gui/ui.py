#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""
vnchKey Settings Window.
Modern dark theme UI matching modern Linux desktop guidelines and fcitx5-lilypad.
"""

import os
import sys
from qtpy.QtWidgets import (
    QWidget,
    QMainWindow,
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
    QFileDialog,
    QScrollArea,
)
from qtpy.QtCore import Qt, QSize
from qtpy.QtGui import QIcon, QFont, QPixmap
from config_manager import ConfigManager


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
    padding: 14px 8px;
    outline: none;
}

QListWidget#sidebar::item {
    height: 38px;
    padding-left: 10px;
    margin-bottom: 3px;
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
    font-size: 22px;
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
    min-width: 190px;
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
    padding: 8px 22px;
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
    padding: 7px 16px;
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


class VnchkeySettingsWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.mgr = ConfigManager()
        self.config = self.mgr.load_config()
        self.macros = self.mgr.load_macros()
        self.dictionary = self.mgr.load_dictionary()
        self.app_modes = self.mgr.load_app_modes()

        self.setWindowTitle("vnchKey Settings")
        self.resize(840, 560)
        self.setMinimumSize(740, 480)
        self.setStyleSheet(STYLE_SHEET)

        # Set window icon
        icon_path = "/usr/share/icons/hicolor/scalable/apps/vnchkey.svg"
        if not os.path.exists(icon_path):
            icon_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../data/icons/vnchkey.svg"))
        if os.path.exists(icon_path):
            self.setWindowIcon(QIcon(icon_path))

        self.init_ui()

    def init_ui(self):
        central = QWidget()
        self.setCentralWidget(central)
        main_layout = QVBoxLayout(central)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        # Content Horizontal Layout (Sidebar + Pages)
        body_layout = QHBoxLayout()
        body_layout.setContentsMargins(0, 0, 0, 0)
        body_layout.setSpacing(0)

        # Left Sidebar (10 Categories matching fcitx5-lilypad)
        self.sidebar = QListWidget()
        self.sidebar.setObjectName("sidebar")
        self.sidebar.setFixedWidth(200)

        categories = [
            ("General", "preferences-system"),
            ("Typing", "input-keyboard"),
            ("Applications", "applications-other"),
            ("Macros", "accessories-text-editor"),
            ("Dictionary", "edit-copy"),
            ("Keymap", "preferences-desktop-keyboard"),
            ("Shortcuts", "preferences-desktop-keyboard-shortcuts"),
            ("Appearance", "preferences-desktop-theme"),
            ("Backup", "document-save-as"),
            ("About", "help-about"),
        ]

        for title, icon_name in categories:
            icon = QIcon.fromTheme(icon_name)
            it = QListWidgetItem(icon, title)
            self.sidebar.addItem(it)

        self.sidebar.currentRowChanged.connect(self.switch_page)
        body_layout.addWidget(self.sidebar)

        # Right Content Area (Stacked pages in ScrollArea)
        self.stack = QStackedWidget()

        self.page_general = self.create_general_page()
        self.page_typing = self.create_typing_page()
        self.page_applications = self.create_applications_page()
        self.page_macros = self.create_macros_page()
        self.page_dictionary = self.create_dictionary_page()
        self.page_keymap = self.create_keymap_page()
        self.page_shortcuts = self.create_shortcuts_page()
        self.page_appearance = self.create_appearance_page()
        self.page_backup = self.create_backup_page()
        self.page_about = self.create_about_page()

        self.stack.addWidget(self.page_general)
        self.stack.addWidget(self.page_typing)
        self.stack.addWidget(self.page_applications)
        self.stack.addWidget(self.page_macros)
        self.stack.addWidget(self.page_dictionary)
        self.stack.addWidget(self.page_keymap)
        self.stack.addWidget(self.page_shortcuts)
        self.stack.addWidget(self.page_appearance)
        self.stack.addWidget(self.page_backup)
        self.stack.addWidget(self.page_about)

        content_wrapper = QWidget()
        content_layout = QVBoxLayout(content_wrapper)
        content_layout.setContentsMargins(28, 20, 28, 16)
        content_layout.addWidget(self.stack)

        body_layout.addWidget(content_wrapper, 1)
        main_layout.addLayout(body_layout, 1)

        # Bottom Action Bar
        bottom_frame = QFrame()
        bottom_frame.setStyleSheet("background-color: #181825; border-top: 1px solid #313244; padding: 6px 16px;")
        bottom_layout = QHBoxLayout(bottom_frame)
        bottom_layout.setContentsMargins(12, 8, 12, 8)

        self.status_lbl = QLabel("")
        self.status_lbl.setStyleSheet("color: #a6e3a1; font-weight: 500;")
        bottom_layout.addWidget(self.status_lbl)

        bottom_layout.addStretch()

        self.btn_reset = QPushButton(QIcon.fromTheme("edit-undo"), "Khôi Phục Mặc Định")
        self.btn_reset.setProperty("class", "secondary-btn")
        self.btn_reset.clicked.connect(self.on_reset_defaults)
        bottom_layout.addWidget(self.btn_reset)

        self.btn_cancel = QPushButton(QIcon.fromTheme("dialog-cancel"), "Hủy Bỏ")
        self.btn_cancel.setProperty("class", "secondary-btn")
        self.btn_cancel.clicked.connect(self.close)
        bottom_layout.addWidget(self.btn_cancel)

        self.btn_apply = QPushButton(QIcon.fromTheme("document-save"), "Áp Dụng")
        self.btn_apply.setProperty("class", "secondary-btn")
        self.btn_apply.clicked.connect(lambda: self.save_all(close_after=False))
        bottom_layout.addWidget(self.btn_apply)

        self.btn_ok = QPushButton(QIcon.fromTheme("dialog-ok"), "Đồng Ý")
        self.btn_ok.setProperty("class", "primary-btn")
        self.btn_ok.clicked.connect(lambda: self.save_all(close_after=True))
        bottom_layout.addWidget(self.btn_ok)

        main_layout.addWidget(bottom_frame)

        self.sidebar.setCurrentRow(0)

    def switch_page(self, index):
        self.stack.setCurrentIndex(index)
        self.status_lbl.clear()

    # 1. General Page
    def create_general_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("General")
        title.setProperty("class", "page-title")
        desc = QLabel("Configure basic input method settings and behaviors.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setSpacing(14)

        grp1 = QLabel("Input Method")
        grp1.setProperty("class", "group-title")
        card_layout.addWidget(grp1)

        # Input Method
        r1 = QHBoxLayout()
        lbl1 = QLabel("Input Method")
        lbl1.setProperty("class", "field-label")
        self.combo_method = QComboBox()
        self.combo_method.addItems(["VNI", "Telex", "Auto (Telex & VNI)"])
        cur_m = self.config.get("InputMethod", "VNI")
        if cur_m == "Telex":
            self.combo_method.setCurrentIndex(1)
        elif "Auto" in cur_m:
            self.combo_method.setCurrentIndex(2)
        else:
            self.combo_method.setCurrentIndex(0)
        r1.addWidget(lbl1)
        r1.addStretch()
        r1.addWidget(self.combo_method)
        card_layout.addLayout(r1)

        # Mode
        r2 = QHBoxLayout()
        lbl2 = QLabel("Mode")
        lbl2.setProperty("class", "field-label")
        self.combo_mode = QComboBox()
        self.combo_mode.addItems(["Uinput (Smooth)", "Native Fcitx5 (Direct Commit)"])
        cur_mode = self.config.get("Mode", "Uinput (Smooth)")
        if "Native" in cur_mode:
            self.combo_mode.setCurrentIndex(1)
        else:
            self.combo_mode.setCurrentIndex(0)
        r2.addWidget(lbl2)
        r2.addStretch()
        r2.addWidget(self.combo_mode)
        card_layout.addLayout(r2)

        # Output Charset
        r3 = QHBoxLayout()
        lbl3 = QLabel("Output Charset")
        lbl3.setProperty("class", "field-label")
        self.combo_charset = QComboBox()
        self.combo_charset.addItems(["Unicode", "Unicode Tổ Hợp", "TCVN3 (ABC)"])
        cur_cs = self.config.get("OutputCharset", "Unicode")
        if cur_cs in ["Unicode", "Unicode Tổ Hợp", "TCVN3 (ABC)"]:
            self.combo_charset.setCurrentText(cur_cs)
        r3.addWidget(lbl3)
        r3.addStretch()
        r3.addWidget(self.combo_charset)
        card_layout.addLayout(r3)

        layout.addWidget(card)

        # Typing Rules Card
        card2 = QFrame()
        card2.setProperty("class", "card")
        card2_layout = QVBoxLayout(card2)
        card2_layout.setSpacing(14)

        grp2 = QLabel("Typing")
        grp2.setProperty("class", "group-title")
        card2_layout.addWidget(grp2)

        r4 = QHBoxLayout()
        lbl4 = QLabel("Type w to Produce ư")
        lbl4.setProperty("class", "field-label")
        self.combo_w2u = QComboBox()
        self.combo_w2u.addItems(["Non-Start", "Always", "Disabled"])
        cur_w = self.config.get("TypeW2U", "Non-Start")
        if cur_w in ["Non-Start", "Always", "Disabled"]:
            self.combo_w2u.setCurrentText(cur_w)
        r4.addWidget(lbl4)
        r4.addStretch()
        r4.addWidget(self.combo_w2u)
        card2_layout.addLayout(r4)

        r5 = QHBoxLayout()
        lbl5 = QLabel("Type [ -> ơ, ] -> ư, { -> Ơ, } -> Ư")
        lbl5.setProperty("class", "field-label")
        self.combo_bracket = QComboBox()
        self.combo_bracket.addItems(["Disabled", "Enabled"])
        cur_b = self.config.get("BracketTransform", "Disabled")
        if cur_b in ["Disabled", "Enabled"]:
            self.combo_bracket.setCurrentText(cur_b)
        r5.addWidget(lbl5)
        r5.addStretch()
        r5.addWidget(self.combo_bracket)
        card2_layout.addLayout(r5)

        layout.addWidget(card2)
        layout.addStretch()
        return w

    # 2. Typing Page
    def create_typing_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Typing")
        title.setProperty("class", "page-title")
        desc = QLabel("Fine-tune spelling corrections and advanced typing options.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setSpacing(14)

        grp = QLabel("Spelling & Tone Options")
        grp.setProperty("class", "group-title")
        card_layout.addWidget(grp)

        row_tone = QHBoxLayout()
        lbl_tone = QLabel("Quy tắc đặt dấu thanh (Tone Style)")
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

        self.chk_smarthorn = QCheckBox("Tự động thêm dấu râu cho 'uo' (duoc95 -> được, nuoc1 -> nước)")
        self.chk_smarthorn.setChecked(self.config.get("SmartHorn", "True") == "True")
        card_layout.addWidget(self.chk_smarthorn)

        self.chk_autorestore = QCheckBox("GoNhanh: Tự động khôi phục từ tiếng Anh khi gõ phím cách (post, text, class...)")
        self.chk_autorestore.setChecked(self.config.get("AutoRestoreEnglish", "True") == "True")
        card_layout.addWidget(self.chk_autorestore)

        self.chk_smartcase = QCheckBox("XKey: Giữ nguyên định dạng chữ hoa/chữ thường thông minh")
        self.chk_smartcase.setChecked(self.config.get("SmartCasing", "True") == "True")
        card_layout.addWidget(self.chk_smartcase)

        self.chk_freemarking = QCheckBox("Cho phép bỏ dấu tự do trên bất kỳ nguyên âm nào")
        self.chk_freemarking.setChecked(self.config.get("FreeMarking", "True") == "True")
        card_layout.addWidget(self.chk_freemarking)

        self.chk_fastconsonant = QCheckBox("Gõ nhanh phụ âm đầu (cc -> ch, gg -> gi, kk -> kh)")
        self.chk_fastconsonant.setChecked(self.config.get("FastConsonantDoubling", "False") == "True")
        card_layout.addWidget(self.chk_fastconsonant)

        layout.addWidget(card)
        layout.addStretch()
        return w

    # 3. Applications Page
    def create_applications_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Applications")
        title.setProperty("class", "page-title")
        desc = QLabel("Configure per-application input behaviors and bypass exclusions.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        self.app_table = QTableWidget(len(self.app_modes), 2)
        self.app_table.setHorizontalHeaderLabels(["Ứng Dụng (App Name)", "Chế Độ (Mode)"])
        self.app_table.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)

        for row, (app, mode) in enumerate(self.app_modes):
            self.app_table.setItem(row, 0, QTableWidgetItem(app))
            self.app_table.setItem(row, 1, QTableWidgetItem(mode))

        layout.addWidget(self.app_table)

        btn_row = QHBoxLayout()
        btn_add = QPushButton("+ Thêm Ứng Dụng")
        btn_add.setProperty("class", "secondary-btn")
        btn_add.clicked.connect(self.add_app_row)

        btn_del = QPushButton("- Xoá Mục Đang Chọn")
        btn_del.setProperty("class", "secondary-btn")
        btn_del.clicked.connect(self.del_app_row)

        btn_row.addWidget(btn_add)
        btn_row.addWidget(btn_del)
        btn_row.addStretch()
        layout.addLayout(btn_row)

        return w

    def add_app_row(self):
        row = self.app_table.rowCount()
        self.app_table.insertRow(row)
        self.app_table.setItem(row, 0, QTableWidgetItem(""))
        self.app_table.setItem(row, 1, QTableWidgetItem("Bypass (Raw English)"))
        self.app_table.editItem(self.app_table.item(row, 0))

    def del_app_row(self):
        row = self.app_table.currentRow()
        if row >= 0:
            self.app_table.removeRow(row)

    # 4. Macros Page
    def create_macros_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Macros")
        title.setProperty("class", "page-title")
        desc = QLabel("Define shortcuts that automatically expand into full phrases.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        chk_row = QHBoxLayout()
        self.chk_enable_macro = QCheckBox("Bật tính năng gõ tắt (Macro expansion)")
        self.chk_enable_macro.setChecked(self.config.get("EnableMacro", "True") == "True")
        chk_row.addWidget(self.chk_enable_macro)

        self.chk_cap_macro = QCheckBox("Viết hoa thông minh theo tiền tố")
        self.chk_cap_macro.setChecked(self.config.get("CapitalizeMacro", "True") == "True")
        chk_row.addWidget(self.chk_cap_macro)
        chk_row.addStretch()
        layout.addLayout(chk_row)

        self.macro_table = QTableWidget(len(self.macros), 2)
        self.macro_table.setHorizontalHeaderLabels(["Từ Viết Tắt (Shortcut)", "Nội Dung Thay Thế (Expansion)"])
        self.macro_table.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)

        for row, (k, v) in enumerate(self.macros):
            self.macro_table.setItem(row, 0, QTableWidgetItem(k))
            self.macro_table.setItem(row, 1, QTableWidgetItem(v))

        layout.addWidget(self.macro_table)

        btn_row = QHBoxLayout()
        btn_add = QPushButton("+ Thêm Viết Tắt")
        btn_add.setProperty("class", "secondary-btn")
        btn_add.clicked.connect(self.add_macro_row)

        btn_del = QPushButton("- Xoá Mục Đang Chọn")
        btn_del.setProperty("class", "secondary-btn")
        btn_del.clicked.connect(self.del_macro_row)

        btn_row.addWidget(btn_add)
        btn_row.addWidget(btn_del)
        btn_row.addStretch()
        layout.addLayout(btn_row)

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

    # 5. Dictionary Page
    def create_dictionary_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Dictionary")
        title.setProperty("class", "page-title")
        desc = QLabel("Manage your custom vocabulary and technical jargon.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)

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

    # 6. Keymap Page
    def create_keymap_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Keymap")
        title.setProperty("class", "page-title")
        desc = QLabel("Custom key mapping and layout configurations.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setSpacing(12)

        grp = QLabel("Bảng Quy Ước Phím Gõ")
        grp.setProperty("class", "group-title")
        card_layout.addWidget(grp)

        info_lbl = QLabel(
            "• VNI:\n"
            "  1: Sắc  |  2: Huyền  |  3: Hỏi  |  4: Ngã  |  5: Nặng\n"
            "  6: Dấu nón (^ cho â, ê, ô)  |  7: Dấu râu (ơ, ư)  |  8: Dấu trăng (ă)  |  9: Chữ đ\n\n"
            "• Telex:\n"
            "  s: Sắc  |  f: Huyền  |  r: Hỏi  |  x: Ngã  |  j: Nặng  |  z: Xoá dấu\n"
            "  aa: â  |  ee: ê  |  oo: ô  |  dd: đ  |  aw/w: ă  |  ow/w: ơ  |  uw/w: ư"
        )
        info_lbl.setStyleSheet("color: #a6adc8; line-height: 1.5; font-family: monospace;")
        card_layout.addWidget(info_lbl)

        layout.addWidget(card)
        layout.addStretch()
        return w

    # 7. Shortcuts Page
    def create_shortcuts_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Shortcuts")
        title.setProperty("class", "page-title")
        desc = QLabel("Manage keyboard shortcuts for switching languages and modes.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setSpacing(14)

        grp = QLabel("Chuyển Đổi Nhanh")
        grp.setProperty("class", "group-title")
        card_layout.addWidget(grp)

        r1 = QHBoxLayout()
        lbl1 = QLabel("Phím chuyển Anh / Việt")
        lbl1.setProperty("class", "field-label")
        self.combo_switch_key = QComboBox()
        self.combo_switch_key.addItems(["Ctrl+Shift", "Alt+Shift", "Super+Space"])
        cur_sk = self.config.get("SwitchShortcut", "Ctrl+Shift")
        if cur_sk in ["Ctrl+Shift", "Alt+Shift", "Super+Space"]:
            self.combo_switch_key.setCurrentText(cur_sk)
        r1.addWidget(lbl1)
        r1.addStretch()
        r1.addWidget(self.combo_switch_key)
        card_layout.addLayout(r1)

        r2 = QHBoxLayout()
        lbl2 = QLabel("Tạm thời gõ tiếng Anh khi giữ phím")
        lbl2.setProperty("class", "field-label")
        self.combo_temp_eng = QComboBox()
        self.combo_temp_eng.addItems(["Disabled", "Hold Shift", "Right Alt"])
        cur_te = self.config.get("TempEnglishMode", "Disabled")
        if cur_te in ["Disabled", "Hold Shift", "Right Alt"]:
            self.combo_temp_eng.setCurrentText(cur_te)
        r2.addWidget(lbl2)
        r2.addStretch()
        r2.addWidget(self.combo_temp_eng)
        card_layout.addLayout(r2)

        layout.addWidget(card)
        layout.addStretch()
        return w

    # 8. Appearance Page
    def create_appearance_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Appearance")
        title.setProperty("class", "page-title")
        desc = QLabel("Customize the look and feel of status icons and themes.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setSpacing(14)

        grp = QLabel("Giao Diện Biểu Tượng")
        grp.setProperty("class", "group-title")
        card_layout.addWidget(grp)

        r1 = QHBoxLayout()
        lbl1 = QLabel("Chủ đề biểu tượng trên thanh tác vụ (Tray Icon)")
        lbl1.setProperty("class", "field-label")
        self.combo_theme = QComboBox()
        self.combo_theme.addItems(["Default (vnchKey)", "Breeze Dark", "Monochrome White", "Minimal"])
        cur_th = self.config.get("TrayIconTheme", "Default")
        if cur_th in ["Default", "Breeze Dark", "Monochrome White", "Minimal"]:
            self.combo_theme.setCurrentText(cur_th)
        r1.addWidget(lbl1)
        r1.addStretch()
        r1.addWidget(self.combo_theme)
        card_layout.addLayout(r1)

        self.chk_badge = QCheckBox("Hiển thị phù hiệu trạng thái (Status badge)")
        self.chk_badge.setChecked(self.config.get("ShowIndicatorBadge", "True") == "True")
        card_layout.addWidget(self.chk_badge)

        layout.addWidget(card)
        layout.addStretch()
        return w

    # 9. Backup Page
    def create_backup_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Backup")
        title.setProperty("class", "page-title")
        desc = QLabel("Export or restore your configuration, macros, and dictionaries.")
        desc.setProperty("class", "page-desc")
        layout.addWidget(title)
        layout.addWidget(desc)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setSpacing(14)

        grp = QLabel("Sao Lưu & Phục Hồi Dữ Liệu")
        grp.setProperty("class", "group-title")
        card_layout.addWidget(grp)

        btn_export = QPushButton("📥 Xuất Sao Lưu Ra Tệp JSON (Export Backup)")
        btn_export.setProperty("class", "secondary-btn")
        btn_export.clicked.connect(self.export_backup_file)
        card_layout.addWidget(btn_export)

        btn_import = QPushButton("📤 Phục Hồi Cài Đặt Từ Tệp JSON (Import Backup)")
        btn_import.setProperty("class", "secondary-btn")
        btn_import.clicked.connect(self.import_backup_file)
        card_layout.addWidget(btn_import)

        layout.addWidget(card)
        layout.addStretch()
        return w

    def export_backup_file(self):
        path, _ = QFileDialog.getSaveFileName(self, "Lưu bản sao lưu vnchKey", "vnchkey-backup.json", "JSON (*.json)")
        if path:
            if self.mgr.export_backup(path):
                QMessageBox.information(self, "Sao Lưu Thành Công", f"Đã xuất cấu hình ra:\n{path}")
            else:
                QMessageBox.warning(self, "Lỗi", "Không thể xuất bản sao lưu.")

    def import_backup_file(self):
        path, _ = QFileDialog.getOpenFileName(self, "Chọn bản sao lưu vnchKey", "", "JSON (*.json)")
        if path:
            if self.mgr.import_backup(path):
                QMessageBox.information(self, "Phục Hồi Thành Công", "Đã nạp lại toàn bộ cấu hình!")
                self.config = self.mgr.load_config()
                self.macros = self.mgr.load_macros()
                self.dictionary = self.mgr.load_dictionary()
                self.app_modes = self.mgr.load_app_modes()
            else:
                QMessageBox.warning(self, "Lỗi", "Không thể phục hồi từ tệp đã chọn.")

    # 10. About Page
    def create_about_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)

        card = QFrame()
        card.setProperty("class", "card")
        card_layout = QVBoxLayout(card)
        card_layout.setAlignment(Qt.AlignCenter)
        card_layout.setSpacing(10)

        logo_lbl = QLabel()
        logo_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../assets/logo.png"))
        if os.path.exists(logo_path):
            pix = QPixmap(logo_path).scaled(96, 96, Qt.KeepAspectRatio, Qt.SmoothTransformation)
            logo_lbl.setPixmap(pix)
        card_layout.addWidget(logo_lbl, alignment=Qt.AlignCenter)

        title = QLabel("vnchKey (Tự Do Edition)")
        title.setStyleSheet("font-size: 22px; font-weight: 700; color: #89b4fa; margin-top: 10px;")
        card_layout.addWidget(title, alignment=Qt.AlignCenter)

        version = QLabel("Phiên bản 0.2.0 • 100% Pure Safe Rust")
        version.setStyleSheet("color: #a6adc8; font-size: 13px;")
        card_layout.addWidget(version, alignment=Qt.AlignCenter)

        info = QLabel(
            "Bộ gõ Tiếng Việt an toàn bộ nhớ hiệu năng cao cho Fcitx5 trên Linux.\n"
            "Direct Clean Commit 100% không gạch chân, không kẹt phím.\n"
            "Tích hợp thuật toán Auto-Restore của GoNhanh và Smart Casing của XKey.\n"
            "Mã nguồn mở hoàn toàn theo giấy phép GPL-3.0."
        )
        info.setAlignment(Qt.AlignCenter)
        info.setStyleSheet("color: #7f849c; margin: 12px 20px; line-height: 1.5;")
        card_layout.addWidget(info)

        link = QLabel("<a href='https://github.com/no-vici/fcitx5-vnchkey' style='color: #89b4fa; text-decoration: none;'>https://github.com/no-vici/fcitx5-vnchkey</a>")
        link.setOpenExternalLinks(True)
        card_layout.addWidget(link, alignment=Qt.AlignCenter)

        layout.addWidget(card)
        layout.addStretch()
        return w

    def on_reset_defaults(self):
        reply = QMessageBox.question(
            self,
            "Khôi phục mặc định",
            "Bạn có chắc chắn muốn thiết lập lại toàn bộ cài đặt gốc?",
            QMessageBox.Yes | QMessageBox.No,
        )
        if reply == QMessageBox.Yes:
            self.mgr.reset_to_defaults()
            self.config = self.mgr.load_config()
            self.status_lbl.setText("✓ Đã khôi phục cài đặt mặc định gốc!")

    def save_all(self, close_after=False):
        # 1. General Config
        self.config["InputMethod"] = self.combo_method.currentText()
        self.config["Mode"] = self.combo_mode.currentText()
        self.config["OutputCharset"] = self.combo_charset.currentText()
        self.config["TypeW2U"] = self.combo_w2u.currentText()
        self.config["BracketTransform"] = self.combo_bracket.currentText()

        # 2. Typing Config
        self.config["ModernStyle"] = "True" if self.combo_tone.currentIndex() == 0 else "False"
        self.config["SmartHorn"] = "True" if self.chk_smarthorn.isChecked() else "False"
        self.config["AutoRestoreEnglish"] = "True" if self.chk_autorestore.isChecked() else "False"
        self.config["SmartCasing"] = "True" if self.chk_smartcase.isChecked() else "False"
        self.config["FreeMarking"] = "True" if self.chk_freemarking.isChecked() else "False"
        self.config["FastConsonantDoubling"] = "True" if self.chk_fastconsonant.isChecked() else "False"

        # 3. Macro & Shortcut Config
        self.config["EnableMacro"] = "True" if self.chk_enable_macro.isChecked() else "False"
        self.config["CapitalizeMacro"] = "True" if self.chk_cap_macro.isChecked() else "False"
        self.config["SwitchShortcut"] = self.combo_switch_key.currentText()
        self.config["TempEnglishMode"] = self.combo_temp_eng.currentText()
        self.config["TrayIconTheme"] = self.combo_theme.currentText()
        self.config["ShowIndicatorBadge"] = "True" if self.chk_badge.isChecked() else "False"

        self.mgr.save_config(self.config)

        # 4. Save Applications table
        apps = []
        for r in range(self.app_table.rowCount()):
            it_app = self.app_table.item(r, 0)
            it_mode = self.app_table.item(r, 1)
            app = it_app.text().strip() if it_app else ""
            mode = it_mode.text().strip() if it_mode else "Bypass (Raw English)"
            if app:
                apps.append((app, mode))
        self.mgr.save_app_modes(apps)

        # 5. Save Macros table
        macros = []
        for r in range(self.macro_table.rowCount()):
            k_item = self.macro_table.item(r, 0)
            v_item = self.macro_table.item(r, 1)
            k = k_item.text().strip() if k_item else ""
            v = v_item.text().strip() if v_item else ""
            if k:
                macros.append((k, v))
        self.mgr.save_macros(macros)

        # 6. Save Dictionary
        dict_words = []
        for i in range(self.list_dict.count()):
            it = self.list_dict.item(i)
            if it and it.text().strip():
                dict_words.append(it.text().strip())
        self.mgr.save_dictionary(dict_words)

        self.status_lbl.setText("✓ Đã lưu cài đặt và cập nhật Fcitx5 thành công!")
        if close_after:
            self.close()
