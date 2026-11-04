#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""
Fcitx5 vnchKey Settings Window.
Refined dark theme UI faithfully matching fcitx5-lilypad.
"""

import os
import sys
import getpass
import tempfile
import subprocess
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
    QApplication,
)
from qtpy.QtCore import Qt, QSize, QUrl
from qtpy.QtGui import QIcon, QFont, QPixmap, QDesktopServices
from config_manager import ConfigManager

STYLE_SHEET = """
QWidget {
    background-color: #1e1e2d;
    color: #cdd6f4;
    font-family: system-ui, -apple-system, Cantarell, "Noto Sans", Roboto, sans-serif;
    font-size: 13px;
    font-weight: 400;
}

/* Category & Section Titles - Light and elegant, not distracting or thick */
QLabel#CategoryTitle {
    font-size: 22px;
    font-weight: 400;
    color: #cdd6f4;
    margin-bottom: 2px;
}

QLabel#CategoryDescription {
    font-size: 13px;
    color: #7f849c;
    margin-bottom: 18px;
}

QLabel#AboutTitle {
    font-size: 26px;
    font-weight: 400;
    color: #cdd6f4;
}

QLabel#GroupHeader {
    font-size: 13px;
    font-weight: 500;
    color: #89b4fa;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-top: 14px;
    margin-bottom: 8px;
}

QLabel.field-label {
    font-size: 13px;
    color: #cdd6f4;
    font-weight: 400;
}

/* Sidebar styling matching lilypad */
QListWidget#sidebar {
    background-color: transparent;
    border: none;
    border-right: 1px solid #282a36;
    padding-top: 15px;
    padding-bottom: 15px;
    outline: none;
}

QListWidget#sidebar::item {
    height: 38px;
    padding-left: 14px;
    border-radius: 8px;
    margin: 2px 10px;
    color: #a6adc8;
    font-weight: 400;
}

QListWidget#sidebar::item:hover:!selected {
    background-color: #2a2b3d;
    color: #cdd6f4;
}

QListWidget#sidebar::item:selected {
    background-color: #4b4463;
    color: #ffffff;
    font-weight: 500;
}

/* Controls */
QComboBox, QLineEdit {
    background-color: #252839;
    border: 1px solid #3b3e52;
    border-radius: 6px;
    padding: 6px 12px;
    color: #cdd6f4;
    font-weight: 400;
    selection-background-color: #5856d6;
}

QComboBox:hover, QLineEdit:hover {
    border-color: #5856d6;
}

QComboBox::drop-down {
    border: none;
    width: 22px;
}

QCheckBox {
    spacing: 10px;
    color: #cdd6f4;
    font-weight: 400;
}

QCheckBox::indicator {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    border: 1px solid #3b3e52;
    background-color: #252839;
}

QCheckBox::indicator:checked {
    background-color: #5856d6;
    border-color: #5856d6;
}

/* Buttons */
QPushButton {
    background-color: #2b2d42;
    color: #cdd6f4;
    border: 1px solid #3d405b;
    border-radius: 6px;
    padding: 7px 16px;
    font-weight: 400;
}

QPushButton:hover {
    background-color: #383a54;
    border-color: #5856d6;
}

QPushButton#Primary {
    background-color: #5856d6;
    color: #ffffff;
    border: none;
    font-weight: 500;
}

QPushButton#Primary:hover {
    background-color: #4b48c4;
}

/* Tables */
QTableWidget {
    background-color: #181825;
    border: 1px solid #282a36;
    border-radius: 8px;
    gridline-color: #282a36;
}

QHeaderView::section {
    background-color: #222436;
    color: #a6adc8;
    padding: 6px;
    border: none;
    border-bottom: 1px solid #282a36;
    font-weight: 500;
}

QFrame#BottomBar {
    background-color: #181825;
    border-top: 1px solid #282a36;
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

        self.setWindowTitle("Fcitx5 vnchKey Settings")
        self._setup_window_size()
        self.setStyleSheet(STYLE_SHEET)

        # Window icon
        icon_path = "/usr/share/icons/hicolor/scalable/apps/vnchkey.svg"
        if not os.path.exists(icon_path):
            icon_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../data/icons/vnchkey.svg"))
        if os.path.exists(icon_path):
            self.setWindowIcon(QIcon(icon_path))

        self.init_ui()

    def _setup_window_size(self):
        try:
            screen = QApplication.primaryScreen().availableGeometry()
            w = int(screen.width() * 0.46)
            h = int(screen.height() * 0.62)
            self.setMinimumSize(780, 520)
            self.resize(max(w, 780), max(h, 520))
        except Exception:
            self.resize(840, 560)

    def init_ui(self):
        central = QWidget()
        self.setCentralWidget(central)
        main_layout = QVBoxLayout(central)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        # Body Layout: Sidebar on Left, Content on Right
        body_layout = QHBoxLayout()
        body_layout.setContentsMargins(0, 0, 0, 0)
        body_layout.setSpacing(0)

        # Sidebar
        self.sidebar = QListWidget()
        self.sidebar.setObjectName("sidebar")
        self.sidebar.setFixedWidth(200)

        main_pages = [
            ("General", "preferences-system"),
            ("Typing", "input-keyboard"),
            ("Applications", "applications-other"),
            ("Macros", "accessories-text-editor"),
            ("Dictionary", "edit-copy"),
            ("Keymap", "preferences-desktop-keyboard"),
            ("Shortcuts", "preferences-desktop-keyboard-shortcuts"),
            ("Appearance", "preferences-desktop-theme"),
            ("Backup", "document-save-as"),
        ]

        self.stack = QStackedWidget()

        for title, icon_name in main_pages:
            it = QListWidgetItem(QIcon.fromTheme(icon_name), title)
            self.sidebar.addItem(it)

        # 20px Spacer between main categories and About matching lilypad
        spacer = QListWidgetItem()
        spacer.setFlags(Qt.NoItemFlags)
        spacer.setSizeHint(QSize(0, 20))
        self.sidebar.addItem(spacer)

        # About item
        it_about = QListWidgetItem(QIcon.fromTheme("help-about"), "About")
        self.sidebar.addItem(it_about)

        # Build pages
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

        self.sidebar.currentRowChanged.connect(self._on_sidebar_changed)
        body_layout.addWidget(self.sidebar)

        content_wrapper = QWidget()
        content_layout = QVBoxLayout(content_wrapper)
        content_layout.setContentsMargins(35, 24, 35, 20)
        content_layout.addWidget(self.stack)

        body_layout.addWidget(content_wrapper, 1)
        main_layout.addLayout(body_layout, 1)

        # Bottom Bar matching lilypad
        bottom_frame = QFrame()
        bottom_frame.setObjectName("BottomBar")
        bottom_layout = QHBoxLayout(bottom_frame)
        bottom_layout.setContentsMargins(20, 10, 20, 10)
        bottom_layout.setSpacing(10)

        self.btn_reset = QPushButton(QIcon.fromTheme("edit-undo"), "&Reset")
        self.btn_reset.clicked.connect(self.on_reset_defaults)
        bottom_layout.addWidget(self.btn_reset)

        self.status_lbl = QLabel("")
        self.status_lbl.setStyleSheet("color: #a6e3a1; font-size: 12px; margin-left: 10px;")
        bottom_layout.addWidget(self.status_lbl)

        bottom_layout.addStretch()

        self.btn_cancel = QPushButton(QIcon.fromTheme("dialog-cancel"), "&Cancel")
        self.btn_cancel.clicked.connect(self.close)
        bottom_layout.addWidget(self.btn_cancel)

        self.btn_apply = QPushButton(QIcon.fromTheme("document-save"), "&Apply")
        self.btn_apply.clicked.connect(lambda: self.save_all(close_after=False))
        bottom_layout.addWidget(self.btn_apply)

        self.btn_ok = QPushButton(QIcon.fromTheme("dialog-ok"), "&OK")
        self.btn_ok.setObjectName("Primary")
        self.btn_ok.clicked.connect(lambda: self.save_all(close_after=True))
        bottom_layout.addWidget(self.btn_ok)

        main_layout.addWidget(bottom_frame)

        self.sidebar.setCurrentRow(0)

    def _on_sidebar_changed(self, row):
        # Handle spacer item click
        if row == 9:  # Spacer index
            self.sidebar.setCurrentRow(10)
            return
        elif row == 10:  # About page index
            self.stack.setCurrentIndex(9)
        else:
            self.stack.setCurrentIndex(row)
        self.status_lbl.clear()

    # 1. General Page (matching lilypad General tab screenshot)
    def create_general_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(10)

        title = QLabel("General")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Configure basic input method settings and behaviors.")
        desc.setObjectName("CategoryDescription")
        layout.addWidget(title)
        layout.addWidget(desc)

        # Section: Input Method
        h_im = QLabel("Input Method")
        h_im.setObjectName("GroupHeader")
        layout.addWidget(h_im)

        r1 = QHBoxLayout()
        lbl1 = QLabel("Input Method")
        lbl1.setProperty("class", "field-label")
        self.combo_method = QComboBox()
        self.combo_method.setFixedWidth(200)
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
        layout.addLayout(r1)

        r2 = QHBoxLayout()
        lbl2 = QLabel("Mode")
        lbl2.setProperty("class", "field-label")
        self.combo_mode = QComboBox()
        self.combo_mode.setFixedWidth(200)
        self.combo_mode.addItems(["Uinput (Smooth)", "Native Fcitx5 (Direct Commit)"])
        cur_mode = self.config.get("Mode", "Uinput (Smooth)")
        if "Native" in cur_mode:
            self.combo_mode.setCurrentIndex(1)
        else:
            self.combo_mode.setCurrentIndex(0)
        r2.addWidget(lbl2)
        r2.addStretch()
        r2.addWidget(self.combo_mode)
        layout.addLayout(r2)

        r3 = QHBoxLayout()
        lbl3 = QLabel("Output Charset")
        lbl3.setProperty("class", "field-label")
        self.combo_charset = QComboBox()
        self.combo_charset.setFixedWidth(200)
        self.combo_charset.addItems(["Unicode", "Unicode Tổ Hợp", "TCVN3 (ABC)"])
        cur_cs = self.config.get("OutputCharset", "Unicode")
        if cur_cs in ["Unicode", "Unicode Tổ Hợp", "TCVN3 (ABC)"]:
            self.combo_charset.setCurrentText(cur_cs)
        r3.addWidget(lbl3)
        r3.addStretch()
        r3.addWidget(self.combo_charset)
        layout.addLayout(r3)

        layout.addSpacing(12)

        # Section: Typing
        h_type = QLabel("Typing")
        h_type.setObjectName("GroupHeader")
        layout.addWidget(h_type)

        r4 = QHBoxLayout()
        lbl4 = QLabel("Type w to Produce ư")
        lbl4.setProperty("class", "field-label")
        self.combo_w2u = QComboBox()
        self.combo_w2u.setFixedWidth(200)
        self.combo_w2u.addItems(["Non-Start", "Always", "Disabled"])
        cur_w = self.config.get("TypeW2U", "Non-Start")
        if cur_w in ["Non-Start", "Always", "Disabled"]:
            self.combo_w2u.setCurrentText(cur_w)
        r4.addWidget(lbl4)
        r4.addStretch()
        r4.addWidget(self.combo_w2u)
        layout.addLayout(r4)

        r5 = QHBoxLayout()
        lbl5 = QLabel("Type [ -> ơ, ] -> ư, { -> Ơ, } -> Ư")
        lbl5.setProperty("class", "field-label")
        self.combo_bracket = QComboBox()
        self.combo_bracket.setFixedWidth(200)
        self.combo_bracket.addItems(["Disabled", "Enabled"])
        cur_b = self.config.get("BracketTransform", "Disabled")
        if cur_b in ["Disabled", "Enabled"]:
            self.combo_bracket.setCurrentText(cur_b)
        r5.addWidget(lbl5)
        r5.addStretch()
        r5.addWidget(self.combo_bracket)
        layout.addLayout(r5)

        layout.addStretch()
        return w

    # 2. Typing Page
    def create_typing_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        title = QLabel("Typing")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Fine-tune spelling corrections and advanced typing options.")
        desc.setObjectName("CategoryDescription")
        layout.addWidget(title)
        layout.addWidget(desc)

        h_tone = QLabel("Spelling & Tone Options")
        h_tone.setObjectName("GroupHeader")
        layout.addWidget(h_tone)

        row_tone = QHBoxLayout()
        lbl_tone = QLabel("Quy tắc đặt dấu thanh (Tone Style)")
        lbl_tone.setProperty("class", "field-label")
        self.combo_tone = QComboBox()
        self.combo_tone.setFixedWidth(200)
        self.combo_tone.addItems(["Modern: oà, uý, oè", "Classic: hòa, ủy, hòe"])
        if self.config.get("ModernStyle", "True") == "False":
            self.combo_tone.setCurrentIndex(1)
        else:
            self.combo_tone.setCurrentIndex(0)
        row_tone.addWidget(lbl_tone)
        row_tone.addStretch()
        row_tone.addWidget(self.combo_tone)
        layout.addLayout(row_tone)

        self.chk_smarthorn = QCheckBox("Tự động thêm dấu râu cho 'uo' (duoc95 -> được, nuoc1 -> nước)")
        self.chk_smarthorn.setChecked(self.config.get("SmartHorn", "True") == "True")
        layout.addWidget(self.chk_smarthorn)

        self.chk_autorestore = QCheckBox("GoNhanh: Tự động khôi phục từ tiếng Anh khi gõ phím cách (post, text, class...)")
        self.chk_autorestore.setChecked(self.config.get("AutoRestoreEnglish", "True") == "True")
        layout.addWidget(self.chk_autorestore)

        self.chk_smartcase = QCheckBox("XKey: Giữ nguyên định dạng chữ hoa/chữ thường thông minh")
        self.chk_smartcase.setChecked(self.config.get("SmartCasing", "True") == "True")
        layout.addWidget(self.chk_smartcase)

        self.chk_freemarking = QCheckBox("Cho phép bỏ dấu tự do trên bất kỳ nguyên âm nào")
        self.chk_freemarking.setChecked(self.config.get("FreeMarking", "True") == "True")
        layout.addWidget(self.chk_freemarking)

        self.chk_fastconsonant = QCheckBox("Gõ nhanh phụ âm đầu (cc -> ch, gg -> gi, kk -> kh)")
        self.chk_fastconsonant.setChecked(self.config.get("FastConsonantDoubling", "False") == "True")
        layout.addWidget(self.chk_fastconsonant)

        layout.addStretch()
        return w

    # 3. Applications Page
    def create_applications_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(10)

        title = QLabel("Applications")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Configure per-application input behaviors and bypass exclusions.")
        desc.setObjectName("CategoryDescription")
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
        btn_add.clicked.connect(self.add_app_row)

        btn_del = QPushButton("- Xoá Mục Đang Chọn")
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
        layout.setSpacing(10)

        title = QLabel("Macros")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Define shortcuts that automatically expand into full phrases.")
        desc.setObjectName("CategoryDescription")
        layout.addWidget(title)
        layout.addWidget(desc)

        chk_row = QHBoxLayout()
        self.chk_enable_macro = QCheckBox("Bật tính năng gõ tắt (Enable Macro)")
        self.chk_enable_macro.setChecked(self.config.get("EnableMacro", "True") == "True")
        chk_row.addWidget(self.chk_enable_macro)

        self.chk_cap_macro = QCheckBox("Viết hoa thông minh (Capitalize Macro)")
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
        btn_add.clicked.connect(self.add_macro_row)

        btn_del = QPushButton("- Xoá Mục Đang Chọn")
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
        layout.setSpacing(10)

        title = QLabel("Dictionary")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Manage your custom vocabulary and technical jargon.")
        desc.setObjectName("CategoryDescription")
        layout.addWidget(title)
        layout.addWidget(desc)

        add_row = QHBoxLayout()
        self.txt_new_word = QLineEdit()
        self.txt_new_word.setPlaceholderText("Nhập từ tiếng Việt mới...")
        btn_add_word = QPushButton("+ Thêm")
        btn_add_word.clicked.connect(self.add_dict_word)
        add_row.addWidget(self.txt_new_word)
        add_row.addWidget(btn_add_word)
        layout.addLayout(add_row)

        self.list_dict = QListWidget()
        self.list_dict.setStyleSheet("background-color: #181825; border: 1px solid #282a36; border-radius: 6px;")
        for word in self.dictionary:
            self.list_dict.addItem(word)
        layout.addWidget(self.list_dict)

        btn_del_word = QPushButton("- Xoá Từ Đang Chọn")
        btn_del_word.clicked.connect(self.del_dict_word)
        layout.addWidget(btn_del_word)

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
        layout.setSpacing(10)

        title = QLabel("Keymap")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Custom key mapping and layout configurations.")
        desc.setObjectName("CategoryDescription")
        layout.addWidget(title)
        layout.addWidget(desc)

        h_map = QLabel("Bảng Quy Ước Phím Gõ")
        h_map.setObjectName("GroupHeader")
        layout.addWidget(h_map)

        info_lbl = QLabel(
            "• VNI:\n"
            "  1: Sắc  |  2: Huyền  |  3: Hỏi  |  4: Ngã  |  5: Nặng\n"
            "  6: Dấu nón (^ cho â, ê, ô)  |  7: Dấu râu (ơ, ư)  |  8: Dấu trăng (ă)  |  9: Chữ đ\n\n"
            "• Telex:\n"
            "  s: Sắc  |  f: Huyền  |  r: Hỏi  |  x: Ngã  |  j: Nặng  |  z: Xoá dấu\n"
            "  aa: â  |  ee: ê  |  oo: ô  |  dd: đ  |  aw/w: ă  |  ow/w: ơ  |  uw/w: ư"
        )
        info_lbl.setStyleSheet("color: #a6adc8; line-height: 1.6; font-family: monospace; background-color: #181825; padding: 16px; border-radius: 8px; border: 1px solid #282a36;")
        layout.addWidget(info_lbl)

        layout.addStretch()
        return w

    # 7. Shortcuts Page
    def create_shortcuts_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        title = QLabel("Shortcuts")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Manage keyboard shortcuts for switching languages and modes.")
        desc.setObjectName("CategoryDescription")
        layout.addWidget(title)
        layout.addWidget(desc)

        h_sc = QLabel("Chuyển Đổi Nhanh")
        h_sc.setObjectName("GroupHeader")
        layout.addWidget(h_sc)

        r1 = QHBoxLayout()
        lbl1 = QLabel("Phím chuyển Anh / Việt")
        lbl1.setProperty("class", "field-label")
        self.combo_switch_key = QComboBox()
        self.combo_switch_key.setFixedWidth(200)
        self.combo_switch_key.addItems(["Ctrl+Shift", "Alt+Shift", "Super+Space"])
        cur_sk = self.config.get("SwitchShortcut", "Ctrl+Shift")
        if cur_sk in ["Ctrl+Shift", "Alt+Shift", "Super+Space"]:
            self.combo_switch_key.setCurrentText(cur_sk)
        r1.addWidget(lbl1)
        r1.addStretch()
        r1.addWidget(self.combo_switch_key)
        layout.addLayout(r1)

        r2 = QHBoxLayout()
        lbl2 = QLabel("Tạm thời gõ tiếng Anh khi giữ phím")
        lbl2.setProperty("class", "field-label")
        self.combo_temp_eng = QComboBox()
        self.combo_temp_eng.setFixedWidth(200)
        self.combo_temp_eng.addItems(["Disabled", "Hold Shift", "Right Alt"])
        cur_te = self.config.get("TempEnglishMode", "Disabled")
        if cur_te in ["Disabled", "Hold Shift", "Right Alt"]:
            self.combo_temp_eng.setCurrentText(cur_te)
        r2.addWidget(lbl2)
        r2.addStretch()
        r2.addWidget(self.combo_temp_eng)
        layout.addLayout(r2)

        layout.addStretch()
        return w

    # 8. Appearance Page
    def create_appearance_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        title = QLabel("Appearance")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Customize the look and feel of status icons and themes.")
        desc.setObjectName("CategoryDescription")
        layout.addWidget(title)
        layout.addWidget(desc)

        h_app = QLabel("Giao Diện Biểu Tượng")
        h_app.setObjectName("GroupHeader")
        layout.addWidget(h_app)

        r1 = QHBoxLayout()
        lbl1 = QLabel("Chủ đề biểu tượng trên thanh tác vụ (Tray Icon)")
        lbl1.setProperty("class", "field-label")
        self.combo_theme = QComboBox()
        self.combo_theme.setFixedWidth(200)
        self.combo_theme.addItems(["Default (vnchKey)", "Breeze Dark", "Monochrome White", "Minimal"])
        cur_th = self.config.get("TrayIconTheme", "Default")
        if cur_th in ["Default", "Breeze Dark", "Monochrome White", "Minimal"]:
            self.combo_theme.setCurrentText(cur_th)
        r1.addWidget(lbl1)
        r1.addStretch()
        r1.addWidget(self.combo_theme)
        layout.addLayout(r1)

        self.chk_badge = QCheckBox("Hiển thị phù hiệu trạng thái (Status badge)")
        self.chk_badge.setChecked(self.config.get("ShowIndicatorBadge", "True") == "True")
        layout.addWidget(self.chk_badge)

        layout.addStretch()
        return w

    # 9. Backup Page
    def create_backup_page(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        title = QLabel("Backup")
        title.setObjectName("CategoryTitle")
        desc = QLabel("Export or restore your configuration, macros, and dictionaries.")
        desc.setObjectName("CategoryDescription")
        layout.addWidget(title)
        layout.addWidget(desc)

        h_bk = QLabel("Sao Lưu & Phục Hồi Dữ Liệu")
        h_bk.setObjectName("GroupHeader")
        layout.addWidget(h_bk)

        btn_export = QPushButton("📥 Xuất Sao Lưu Ra Tệp JSON (Export Backup)")
        btn_export.clicked.connect(self.export_backup_file)
        layout.addWidget(btn_export)

        btn_import = QPushButton("📤 Phục Hồi Cài Đặt Từ Tệp JSON (Import Backup)")
        btn_import.clicked.connect(self.import_backup_file)
        layout.addWidget(btn_import)

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

    # 10. About Page (Matching Lilypad About Page Screenshot Exactly)
    def create_about_page(self) -> QWidget:
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.NoFrame)
        scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)

        content = QWidget()
        layout = QVBoxLayout(content)
        layout.setContentsMargins(40, 20, 40, 30)
        layout.setSpacing(16)
        layout.setAlignment(Qt.AlignTop | Qt.AlignHCenter)

        # 1. Logo
        logo = QLabel()
        logo_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../assets/logo.png"))
        if not os.path.exists(logo_path):
            logo_path = "/usr/share/icons/hicolor/scalable/apps/vnchkey.svg"
        if os.path.exists(logo_path):
            pix = QIcon(logo_path).pixmap(80, 80)
            logo.setPixmap(pix)
        else:
            logo.setText("🇻🇳")
            logo.setStyleSheet("font-size: 64px;")
        layout.addWidget(logo, alignment=Qt.AlignCenter)

        # 2. Title
        title = QLabel("Fcitx5 vnchKey")
        title.setObjectName("AboutTitle")
        layout.addWidget(title, alignment=Qt.AlignCenter)

        # 3. Version Pill Tag
        version = QLabel("Version 0.2.0 (Stable)")
        version.setAlignment(Qt.AlignCenter)
        version.setStyleSheet("""
            background-color: #4b4463;
            color: #b4befe;
            border-radius: 10px;
            padding: 3px 12px;
            font-size: 11px;
            font-weight: 500;
        """)
        layout.addWidget(version, alignment=Qt.AlignCenter)

        # 4. Description
        desc = QLabel(
            '<div style="text-align: center; line-height: 1.4; padding: 4px 0; color: #a6adc8;">'
            'Next-Gen Native Wayland Vietnamese IME for Linux.<br>'
            'Powered by 100% Safe Rust &amp; Sentinel Barrier.'
            '</div>'
        )
        desc.setWordWrap(True)
        desc.setAlignment(Qt.AlignCenter)
        layout.addWidget(desc, alignment=Qt.AlignCenter)

        # 5. GitHub Link
        link = QLabel('<a href="https://github.com/no-vici/fcitx5-vnchkey" style="color: #89b4fa; text-decoration: none;">https://github.com/no-vici/fcitx5-vnchkey</a>')
        link.setOpenExternalLinks(True)
        layout.addWidget(link, alignment=Qt.AlignCenter)

        # 6. Buttons Row: Report Bug + Request Feature (width 200 each)
        support_row = QHBoxLayout()
        support_row.setSpacing(15)
        support_row.setAlignment(Qt.AlignCenter)

        btn_bug = QPushButton("Report Bug")
        btn_bug.setFixedWidth(200)
        btn_bug.clicked.connect(lambda: QDesktopServices.openUrl(QUrl("https://github.com/no-vici/fcitx5-vnchkey/issues/new")))

        btn_feature = QPushButton("Request Feature")
        btn_feature.setFixedWidth(200)
        btn_feature.clicked.connect(lambda: QDesktopServices.openUrl(QUrl("https://github.com/no-vici/fcitx5-vnchkey/issues/new")))

        support_row.addWidget(btn_bug)
        support_row.addWidget(btn_feature)
        layout.addLayout(support_row)

        # 7. Export Debug Logs (width 415)
        btn_logs = QPushButton("Export Debug Logs")
        btn_logs.setFixedWidth(415)
        btn_logs.clicked.connect(self._on_export_logs)
        layout.addWidget(btn_logs, alignment=Qt.AlignCenter)

        # 8. Divider line
        line1 = QFrame()
        line1.setFrameShape(QFrame.HLine)
        line1.setStyleSheet("color: #282a36;")
        layout.addWidget(line1)

        # 9. Credits section
        credits_title = QLabel("Developed & Maintained by")
        credits_title.setStyleSheet("color: #7f849c; font-size: 13px;")
        layout.addWidget(credits_title, alignment=Qt.AlignCenter)

        authors = [
            ("no-vici (Creator & Maintainer)", "https://github.com/no-vici"),
            ("Fcitx5 Input Method Framework", "https://github.com/fcitx/fcitx5"),
            ("vnchKey Engine (100% Safe Rust Core)", "https://github.com/no-vici/fcitx5-vnchkey"),
        ]
        for name, url_str in authors:
            lbl = QLabel(f'<a href="{url_str}" style="color: #b4befe; text-decoration: none;">{name}</a>')
            lbl.setOpenExternalLinks(True)
            lbl.setCursor(Qt.PointingHandCursor)
            lbl.setAlignment(Qt.AlignCenter)
            layout.addWidget(lbl)

        layout.addSpacing(10)

        # 10. License Footer
        license_lbl = QLabel("Licensed under the GNU General Public License v3.0")
        license_lbl.setStyleSheet("color: #6c7086; font-size: 12px;")
        layout.addWidget(license_lbl, alignment=Qt.AlignCenter)

        scroll.setWidget(content)
        return scroll

    def _on_export_logs(self):
        save_path, _ = QFileDialog.getSaveFileName(
            self,
            "Save Debug Log",
            os.path.expanduser("~/fcitx5-vnchkey-debug.log"),
            "Log Files (*.log);;All Files (*)",
        )
        if not save_path:
            return

        try:
            with open(save_path, "w", encoding="utf-8") as f:
                f.write("=== Fcitx5 vnchKey Debug Log Export ===\n")
                f.write("Version: 0.2.0 (Stable)\n")
                f.write(f"User: {getpass.getuser()}\n")
                f.write("--------------------------------------\n\n")

                f.write("--- Configuration (~/.config/fcitx5/conf/vnchkey.conf) ---\n")
                conf_p = os.path.expanduser("~/.config/fcitx5/conf/vnchkey.conf")
                if os.path.exists(conf_p):
                    with open(conf_p, "r", encoding="utf-8") as cf:
                        f.write(cf.read())
                else:
                    f.write("Configuration file not found.\n")
                f.write("\n\n")

                f.write("--- Recent Systemd Journal Log (fcitx5) ---\n")
                try:
                    p = subprocess.run(
                        ["journalctl", "--user", "-u", "fcitx5", "--no-pager", "-n", "100"],
                        capture_output=True,
                        text=True,
                        timeout=5,
                    )
                    f.write(p.stdout if p.stdout else "No journalctl output.\n")
                except Exception as ex:
                    f.write(f"Journal error: {ex}\n")
                f.write("\n--- End of Log ---\n")

            QMessageBox.information(self, "Success", f"Debug logs exported to:\n{save_path}")
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to export logs:\n{e}")

    def on_reset_defaults(self):
        reply = QMessageBox.question(
            self,
            "Confirm Reset",
            "Restore all settings to defaults?",
            QMessageBox.Yes | QMessageBox.No,
        )
        if reply == QMessageBox.Yes:
            self.mgr.reset_to_defaults()
            self.config = self.mgr.load_config()
            self.status_lbl.setText("✓ Restored defaults!")

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

        self.status_lbl.setText("✓ Settings saved and Fcitx5 updated!")
        if close_after:
            self.close()
