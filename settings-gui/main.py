#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""
vnchKey Settings Application Entry Point.
"""

import sys
import signal
from qtpy.QtWidgets import QApplication
from qtpy.QtCore import QTimer
from qtpy.QtGui import QIcon
from ui import VnchkeySettingsWindow


def main():
    signal.signal(signal.SIGINT, signal.SIG_DFL)
    app = QApplication(sys.argv)
    app.setApplicationName("Fcitx5 vnchKey Settings")
    app.setDesktopFileName("org.fcitx.Fcitx5.Addon.Vnchkey.Settings")
    app.setWindowIcon(QIcon.fromTheme("vnchkey"))

    # Let python handle Ctrl+C
    timer = QTimer()
    timer.start(500)
    timer.timeout.connect(lambda: None)

    window = VnchkeySettingsWindow()
    window.show()

    try:
        sys.exit(app.exec())
    except KeyboardInterrupt:
        app.quit()


if __name__ == "__main__":
    main()

