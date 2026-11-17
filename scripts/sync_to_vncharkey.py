#!/usr/bin/env python3
"""
sync_to_vncharkey.py
Synchronizes changes from vnchKey repository to VnCharKey repository.
- Detects newer/modified files in vnchKey and copies them over.
- Replaces naming: vnchKey -> VnCharKey, vnchkey -> vncharkey, VNCHKEY -> VNCHARKEY, etc.
- Preserves VnCharKey's neutral non-flag logos and branding icons.
- Excludes build artifacts, caches, and VCS directories.
"""

import os
import sys
import shutil
import filecmp
from pathlib import Path

# Paths
SRC_DIR = Path(__file__).resolve().parent.parent
DEST_DIR = SRC_DIR.parent / "VnCharKey"

# Directories/Files to exclude completely
EXCLUDE_DIRS = {
    ".git",
    "target",
    "build",
    ".idea",
    ".vscode",
    ".cache",
    "debian/.debhelper",
}

EXCLUDE_EXTENSIONS = {
    ".deb",
    ".lock", # Cargo.lock can be regenerated or synced if needed, but often OS specific or updated
}

# Binary extensions: copy as-is, do not text-replace
BINARY_EXTENSIONS = {
    ".png",
    ".ico",
    ".icns",
    ".ttf",
    ".otf",
    ".woff",
    ".woff2",
    ".tar",
    ".gz",
    ".xz",
    ".zip",
    ".o",
    ".a",
    ".so",
    ".dll",
    ".dylib",
    ".exe",
}

# Specific asset & doc files that VnCharKey keeps custom (neutral logo, custom README, etc.)
CUSTOM_ASSETS = {
    "README.md",
    "assets/logo.png",
    "macos/Resources/AppIcon.png",
    "macos/Resources/AppIcon.icns",
    "macos/Resources/flag_vnch.png",
    "data/icons/vnchkey.png",
    "data/icons/fcitx-vnchkey.png",
    "data/icons/vnchkey.svg",
    "data/icons/fcitx-vnchkey.svg",
    "windows/assets/vncharkey.ico",
    "windows/assets/vncharkey.png",
}

def is_custom_asset(rel_src_str: str) -> bool:
    """Return True if the asset is custom in VnCharKey and shouldn't be overwritten."""
    if rel_src_str in CUSTOM_ASSETS:
        return True
    if "data/icons/" in rel_src_str:
        return True
    return False

def transform_name(name: str) -> str:
    """Transforms filenames/directory names from vnchkey to vncharkey."""
    res = name
    res = res.replace("fcitx5-vnchkey", "fcitx5-vncharkey")
    res = res.replace("fcitx-vnchkey", "fcitx-vncharkey")
    res = res.replace("vnchkey", "vncharkey")
    res = res.replace("Vnchkey", "VnCharKey")
    res = res.replace("vnchKey", "VnCharKey")
    res = res.replace("VNCHKEY", "VNCHARKEY")
    return res

def transform_content(text: str) -> str:
    """Replaces branding tokens in text files."""
    # Order matters: longer / specific tokens first
    replacements = [
        ("https://github.com/no-vici/fcitx5-vnchkey", "https://github.com/no-vici/vncharkey"),
        ("https://api.github.com/repos/no-vici/fcitx5-vnchkey", "https://api.github.com/repos/no-vici/vncharkey"),
        ("no-vici/fcitx5-vnchkey", "no-vici/vncharkey"),
        ("fcitx5-vnchkey", "fcitx5-vncharkey"),
        ("fcitx-vnchkey", "fcitx-vncharkey"),
        ("Fcitx5.vnchkey", "Fcitx5.vncharkey"),
        ("Fcitx5.Vnchkey", "Fcitx5.VnCharKey"),
        ("vnchkey-core", "vncharkey-core"),
        ("vnchkey-settings", "vncharkey-settings"),
        ("vnchkey-dev", "vncharkey-dev"),
        ("vnchkey_core", "vncharkey_core"),
        ("vnchkey_settings", "vncharkey_settings"),
        ("vnchkey_dev", "vncharkey_dev"),
        ("vnchkey_addon", "vncharkey_addon"),
        ("VNCHKEY", "VNCHARKEY"),
        ("vnchKey", "VnCharKey"),
        ("VnchKey", "VnCharKey"),
        ("Vnchkey", "VnCharKey"),
        ("vnchkey", "vncharkey"),
    ]
    for old, new in replacements:
        text = text.replace(old, new)
    return text

def should_exclude(rel_path: Path) -> bool:
    for part in rel_path.parts:
        if part in EXCLUDE_DIRS:
            return True
    if rel_path.suffix in EXCLUDE_EXTENSIONS:
        return True
    if rel_path.name.endswith(".deb"):
        return True
    return False

def sync():
    if not DEST_DIR.exists():
        DEST_DIR.mkdir(parents=True, exist_ok=True)

    print(f"[*] Syncing from {SRC_DIR} -> {DEST_DIR}...")
    copied_count = 0
    skipped_count = 0

    for root, dirs, files in os.walk(SRC_DIR):
        # Filter directories in-place
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

        for file in files:
            src_file = Path(root) / file
            rel_src = src_file.relative_to(SRC_DIR)

            if should_exclude(rel_src):
                continue

            # Determine destination path with transformed names for directories & file
            dest_parts = [transform_name(p) for p in rel_src.parts]
            dest_file = DEST_DIR.joinpath(*dest_parts)

            rel_str = str(rel_src).replace("\\", "/")

            # Check if this file is a custom logo in VnCharKey and already exists
            if is_custom_asset(rel_str) and dest_file.exists():
                skipped_count += 1
                continue

            is_binary = src_file.suffix.lower() in BINARY_EXTENSIONS

            # Check if destination exists and is up to date
            dest_file.parent.mkdir(parents=True, exist_ok=True)

            if is_binary:
                # Compare binary modification time or content
                if dest_file.exists() and dest_file.stat().st_mtime >= src_file.stat().st_mtime:
                    skipped_count += 1
                    continue
                shutil.copy2(src_file, dest_file)
                copied_count += 1
                print(f"[BINARY] Copied: {rel_src} -> {dest_file.relative_to(DEST_DIR)}")
            else:
                # Text file: read, transform content, write
                try:
                    with open(src_file, "r", encoding="utf-8") as f:
                        src_text = f.read()
                except UnicodeDecodeError:
                    # Treat as binary fallback
                    if dest_file.exists() and dest_file.stat().st_mtime >= src_file.stat().st_mtime:
                        skipped_count += 1
                        continue
                    shutil.copy2(src_file, dest_file)
                    copied_count += 1
                    continue

                transformed = transform_content(src_text)

                # Check if dest exists and text matches
                if dest_file.exists():
                    try:
                        with open(dest_file, "r", encoding="utf-8") as f:
                            existing_dest = f.read()
                        if existing_dest == transformed and dest_file.stat().st_mtime >= src_file.stat().st_mtime:
                            skipped_count += 1
                            continue
                    except Exception:
                        pass

                with open(dest_file, "w", encoding="utf-8") as f:
                    f.write(transformed)
                # Keep permissions
                shutil.copymode(src_file, dest_file)
                copied_count += 1
                print(f"[TEXT] Updated: {rel_src} -> {dest_file.relative_to(DEST_DIR)}")

    print(f"\n[+] Sync completed: {copied_count} files synced/updated, {skipped_count} files up-to-date/skipped.")

if __name__ == "__main__":
    sync()
