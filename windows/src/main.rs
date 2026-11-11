//! # VnCharKey for Windows
//!
//! Native Windows Vietnamese Input Method Engine powered by `vnchkey-core`.
//! Features:
//! - Low-level keyboard hook (`WH_KEYBOARD_LL`) + Mouse click reset (`WH_MOUSE_LL`)
//! - Zero-flicker Micro-Diff LCP replacement via atomic `SendInput` batches
//! - System Tray icon with Conical Hat (Nón Lá) + V/E indicator
//! - Comprehensive Tray Context Menu (Kiểu gõ, Bảng mã, Dấu mới/cũ, Chánh tả, Gõ tắt, Khôi phục Tiếng Anh, v.v.)
//! - Dialog / Text Editor integration for Macros & Custom Dictionary
//! - Escape to restore raw keystrokes & Backspace macro undo

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod win_app {
    use std::cell::RefCell;
    use std::fs;
    use std::mem::size_of;
    use std::path::PathBuf;
    use std::process::Command;
    use std::ptr::null_mut;
    use std::time::{SystemTime, UNIX_EPOCH};

    use vnchkey_core::{Charset, Engine, EngineConfig, InputMethod, ToneStyle, W2UType};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    use windows_sys::Win32::UI::Shell::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const WM_TRAYICON: u32 = WM_USER + 1;

    // Menu Action IDs
    const ID_TRAY_TOGGLE: usize = 1001;

    const ID_METHOD_AUTO: usize = 1002;
    const ID_METHOD_TELEX: usize = 1003;
    const ID_METHOD_VNI: usize = 1004;
    const ID_METHOD_SIMPLETELEX: usize = 1005;

    const ID_CHARSET_NFC: usize = 1010;
    const ID_CHARSET_NFD: usize = 1011;

    const ID_TONE_MODERN: usize = 1020;
    const ID_TONE_CLASSIC: usize = 1021;

    const ID_TOGGLE_SPELLCHECK: usize = 1030;
    const ID_TOGGLE_AUTORESTORE: usize = 1031;
    const ID_TOGGLE_ESCRESTORE: usize = 1032;
    const ID_TOGGLE_BRACKET: usize = 1033;
    const ID_TOGGLE_MACRO: usize = 1034;
    const ID_TOGGLE_DICT: usize = 1035;

    const ID_TOGGLE_QUICK_TELEX: usize = 1037;
    const ID_TOGGLE_QUICK_START: usize = 1038;
    const ID_TOGGLE_QUICK_END: usize = 1039;

    const ID_TOGGLE_AUTORUN: usize = 1036;

    const ID_SHORTCUT_CTRL_SHIFT: usize = 1040;
    const ID_SHORTCUT_ALT_Z: usize = 1041;
    const ID_SHORTCUT_CTRL_SPACE: usize = 1042;
    const ID_SHORTCUT_WIN_SPACE: usize = 1043;
    const ID_SHORTCUT_DISABLED: usize = 1044;
    const ID_SHORTCUT_RECORD: usize = 1045;

    const ID_OPEN_MACROS: usize = 1050;
    const ID_OPEN_DICT: usize = 1051;
    const ID_RELOAD_FILES: usize = 1052;

    const ID_TRAY_EXIT: usize = 1099;

    /// Magic tag in `dwExtraInfo` so our hook ignores synthetic keystrokes injected by `VnCharKey`.
    const VNCHARKEY_INJECTED_TAG: usize = 0x564E_4348;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CustomShortcut {
        ctrl: bool,
        shift: bool,
        alt: bool,
        win: bool,
        /// Virtual Key code (e.g. 'Z' as u16, VK_SPACE, etc.).
        /// If 0, it means a modifier-only chord (e.g. Ctrl + Shift, Alt + Shift).
        key_code: u16,
        display_name: String,
    }

    impl Default for CustomShortcut {
        fn default() -> Self {
            Self {
                ctrl: true,
                shift: true,
                alt: false,
                win: false,
                key_code: 0,
                display_name: "Ctrl + Shift".to_string(),
            }
        }
    }

    impl CustomShortcut {
        fn disabled() -> Self {
            Self {
                ctrl: false,
                shift: false,
                alt: false,
                win: false,
                key_code: 0,
                display_name: "Đã tắt phím tắt".to_string(),
            }
        }

        fn is_disabled(&self) -> bool {
            !self.ctrl && !self.shift && !self.alt && !self.win && self.key_code == 0
        }

        fn from_modifiers_and_key(ctrl: bool, shift: bool, alt: bool, win: bool, vk: u16) -> Self {
            let mut parts = Vec::new();
            if ctrl { parts.push("Ctrl"); }
            if alt { parts.push("Alt"); }
            if shift { parts.push("Shift"); }
            if win { parts.push("Win"); }

            let name = if vk != 0 {
                let key_name = match vk {
                    VK_SPACE => "Space".to_string(),
                    VK_ESCAPE => "Esc".to_string(),
                    VK_TAB => "Tab".to_string(),
                    VK_OEM_3 => "~ (Tilde)".to_string(),
                    0x30..=0x39 => ((vk as u8) as char).to_string(),
                    0x41..=0x5A => ((vk as u8) as char).to_string(),
                    VK_F1..=VK_F24 => format!("F{}", vk - VK_F1 + 1),
                    _ => format!("Key_{}", vk),
                };
                parts.push(&key_name);
                parts.join(" + ")
            } else if !parts.is_empty() {
                format!("{} (Nhấn rồi thả)", parts.join(" + "))
            } else {
                "Đã tắt phím tắt".to_string()
            };

            Self {
                ctrl,
                shift,
                alt,
                win,
                key_code: vk,
                display_name: name,
            }
        }
    }

    struct AppState {
        engine: Engine,
        enabled: bool,
        method: InputMethod,
        charset: Charset,
        tone_style: ToneStyle,
        spell_check: bool,
        auto_restore: bool,
        esc_restore: bool,
        bracket_transform: bool,
        macro_enabled: bool,
        dict_enabled: bool,
        shortcut: CustomShortcut,
        recording_shortcut: bool,
        hwnd: HWND,
        ctrl_pressed: bool,
        shift_pressed: bool,
        alt_pressed: bool,
        win_pressed: bool,
        other_key_during_chord: bool,
    }

    thread_local! {
        static STATE: RefCell<AppState> = RefCell::new(AppState {
            engine: Engine::new(EngineConfig::default()),
            enabled: true,
            method: InputMethod::Auto,
            charset: Charset::Unicode,
            tone_style: ToneStyle::Classic,
            spell_check: true,
            auto_restore: true,
            esc_restore: true,
            bracket_transform: false,
            macro_enabled: true,
            dict_enabled: true,
            shortcut: CustomShortcut::default(),
            recording_shortcut: false,
            hwnd: null_mut(),
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            win_pressed: false,
            other_key_during_chord: false,
        });
    }

    fn now_ms() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }

    fn to_wstring(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn config_dir() -> PathBuf {
        // Ưu tiên Chế độ Portable: Lưu trực tiếp trong thư mục chứa file VnCharKey.exe
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let portable_marker = exe_dir.join("portable.txt");
                let portable_cfg = exe_dir.join("config.conf");
                // Nếu có file marker hoặc đã có config.conf tại chỗ, hoặc thư mục có quyền ghi
                if portable_marker.exists() || portable_cfg.exists() {
                    return exe_dir.to_path_buf();
                }
                // Thử tạo một test file để kiểm tra quyền ghi vào thư mục exe
                let test_file = exe_dir.join(".write_test");
                if fs::write(&test_file, b"ok").is_ok() {
                    let _ = fs::remove_file(&test_file);
                    return exe_dir.to_path_buf();
                }
            }
        }

        // Fallback: Nếu không ghi được vào thư mục exe (ví dụ Program Files), dùng %APPDATA%\VnCharKey
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        base.join("VnCharKey")
    }

    fn macros_path() -> PathBuf {
        config_dir().join("macros.conf")
    }

    fn dict_path() -> PathBuf {
        config_dir().join("dictionary.conf")
    }

    fn config_path() -> PathBuf {
        config_dir().join("config.conf")
    }

    fn ensure_config_files() {
        let dir = config_dir();
        let _ = fs::create_dir_all(&dir);

        let c_path = config_path();
        if !c_path.exists() {
            let default_cfg = "# VnCharKey Windows Configuration\n\
                               ctrl=true\n\
                               shift=true\n\
                               alt=false\n\
                               win=false\n\
                               vk=0\n";
            let _ = fs::write(&c_path, default_cfg);
        }

        let m_path = macros_path();
        if !m_path.exists() {
            let default_macros = "# Bang Go Tat (Macros) cho VnCharKey\n\
                                  # Cu phap: viet_tat=cum_tu_day_du\n\
                                  vn=Việt Nam\n\
                                  vnch=Việt Nam Cộng Hòa\n\
                                  td=Tự Do\n\
                                  sg=Sài Gòn\n\
                                  dc=được\n\
                                  ko=không\n\
                                  ng=người\n\
                                  hn=hôm nay\n\
                                  cv=công việc\n";
            let _ = fs::write(&m_path, default_macros);
        }

        let d_path = dict_path();
        if !d_path.exists() {
            let default_dict = "# Tu Dien Ca Nhan (English / Bypass Words)\n\
                                # Moi tu mot dong\n\
                                windows\n\
                                visualstudio\n\
                                code\n\
                                rust\n\
                                github\n\
                                powershell\n";
            let _ = fs::write(&d_path, default_dict);
        }
    }

    fn load_shortcut_config() -> CustomShortcut {
        let path = config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            let mut ctrl = false;
            let mut shift = false;
            let mut alt = false;
            let mut win = false;
            let mut vk: u16 = 0;

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim().to_lowercase();
                    let v = v.trim().to_lowercase();
                    match k.as_str() {
                        "ctrl" => ctrl = v == "true" || v == "1",
                        "shift" => shift = v == "true" || v == "1",
                        "alt" => alt = v == "true" || v == "1",
                        "win" => win = v == "true" || v == "1",
                        "vk" => vk = v.parse::<u16>().unwrap_or(0),
                        _ => {}
                    }
                }
            }
            if !ctrl && !shift && !alt && !win && vk == 0 {
                CustomShortcut::disabled()
            } else {
                CustomShortcut::from_modifiers_and_key(ctrl, shift, alt, win, vk)
            }
        } else {
            CustomShortcut::default()
        }
    }

    fn save_shortcut_config(sc: &CustomShortcut) {
        let path = config_path();
        let content = format!(
            "# VnCharKey Windows Configuration\n\
             ctrl={}\n\
             shift={}\n\
             alt={}\n\
             win={}\n\
             vk={}\n",
            sc.ctrl, sc.shift, sc.alt, sc.win, sc.key_code
        );
        let _ = fs::write(&path, content);
    }

    fn reload_files(engine: &mut Engine) {
        let m_path = macros_path();
        let d_path = dict_path();
        if let Some(p) = m_path.to_str() {
            engine.load_macros_from_file(p);
        }
        if let Some(p) = d_path.to_str() {
            engine.load_dictionary_from_file(p);
        }
    }

    fn is_autorun_enabled() -> bool {
        // Kiểm tra qua registry HKCU\Software\Microsoft\Windows\CurrentVersion\Run
        let output = Command::new("reg")
            .args(&["query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "/v", "VnCharKey"])
            .output();
        if let Ok(out) = output {
            out.status.success()
        } else {
            false
        }
    }

    fn set_autorun_enabled(enable: bool) {
        if enable {
            if let Ok(exe_path) = std::env::current_exe() {
                let exe_str = exe_path.to_string_lossy().to_string();
                let _ = Command::new("reg")
                    .args(&[
                        "add",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                        "/v",
                        "VnCharKey",
                        "/t",
                        "REG_SZ",
                        "/d",
                        &format!("\"{}\"", exe_str),
                        "/f",
                    ])
                    .output();
            }
        } else {
            let _ = Command::new("reg")
                .args(&[
                    "delete",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "VnCharKey",
                    "/f",
                ])
                .output();
        }
    }

    fn open_file_in_notepad(path: PathBuf) {
        let _ = Command::new("notepad.exe").arg(path).spawn();
    }

    /// Sends an atomic batch of Backspaces and UTF-16 characters via `SendInput`.
    unsafe fn emit_output(backspaces: usize, commit_text: &str) {
        let utf16: Vec<u16> = commit_text.encode_utf16().collect();
        let total_events = (backspaces * 2) + (utf16.len() * 2);
        if total_events == 0 {
            return;
        }

        let mut inputs: Vec<INPUT> = Vec::with_capacity(total_events);

        // 1. Queue Backspace key down/up pairs
        for _ in 0..backspaces {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_BACK,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: VNCHARKEY_INJECTED_TAG,
                    },
                },
            });
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_BACK,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: VNCHARKEY_INJECTED_TAG,
                    },
                },
            });
        }

        // 2. Queue UTF-16 Unicode characters
        for code_unit in utf16 {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: 0,
                        wScan: code_unit,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: VNCHARKEY_INJECTED_TAG,
                    },
                },
            });
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: 0,
                        wScan: code_unit,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: VNCHARKEY_INJECTED_TAG,
                    },
                },
            });
        }

        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        );
    }

    /// Translates virtual key and current keyboard state to a Unicode `char`.
    unsafe fn vk_to_char(vk: u32, scan_code: u32) -> Option<char> {
        let mut key_state = [0u8; 256];
        if GetKeyboardState(key_state.as_mut_ptr()) == 0 {
            return None;
        }
        if (GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0 {
            key_state[VK_SHIFT as usize] = 0x80;
        }
        if (GetKeyState(VK_CAPITAL as i32) & 0x0001) != 0 {
            key_state[VK_CAPITAL as usize] = 0x01;
        }

        let mut buf = [0u16; 4];
        let len = ToUnicode(vk, scan_code, key_state.as_ptr(), buf.as_mut_ptr(), 4, 0);
        if len == 1 {
            char::from_u32(buf[0] as u32)
        } else {
            None
        }
    }

    pub unsafe extern "system" fn keyboard_hook_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code < 0 {
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        let kbd = &*(lparam as *const KBDLLHOOKSTRUCT);

        // Ignore synthetic events injected by VnCharKey itself
        if kbd.dwExtraInfo == VNCHARKEY_INJECTED_TAG {
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        let msg = wparam as u32;
        let is_keydown = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_keyup = msg == WM_KEYUP || msg == WM_SYSKEYUP;
        let vk = kbd.vkCode as u16;

        let is_ctrl = vk == VK_LCONTROL || vk == VK_RCONTROL || vk == VK_CONTROL;
        let is_shift = vk == VK_LSHIFT || vk == VK_RSHIFT || vk == VK_SHIFT;
        let is_alt = vk == VK_LMENU || vk == VK_RMENU || vk == VK_MENU;
        let is_win = vk == VK_LWIN || vk == VK_RWIN;

        // --- Custom Shortcut Recorder Mode ---
        let mut recorded_new_shortcut = false;
        STATE.with(|s| {
            let mut st = s.borrow_mut();
            if st.recording_shortcut {
                if is_keydown {
                    if is_ctrl { st.ctrl_pressed = true; }
                    if is_shift { st.shift_pressed = true; }
                    if is_alt { st.alt_pressed = true; }
                    if is_win { st.win_pressed = true; }

                    if !is_ctrl && !is_shift && !is_alt && !is_win {
                        // User pressed a regular key while holding modifiers!
                        if vk == VK_ESCAPE {
                            // Cancel recording
                            st.recording_shortcut = false;
                            recorded_new_shortcut = true;
                        } else {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(
                                st.ctrl_pressed,
                                st.shift_pressed,
                                st.alt_pressed,
                                st.win_pressed,
                                vk,
                            );
                            save_shortcut_config(&st.shortcut);
                            st.recording_shortcut = false;
                            recorded_new_shortcut = true;
                        }
                    }
                } else if is_keyup {
                    // Check if user released modifiers without pressing any other key (modifier chord recording)
                    if (is_ctrl || is_shift || is_alt || is_win) && !st.other_key_during_chord {
                        let mod_count = (st.ctrl_pressed as u8)
                            + (st.shift_pressed as u8)
                            + (st.alt_pressed as u8)
                            + (st.win_pressed as u8);
                        if mod_count >= 1 {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(
                                st.ctrl_pressed,
                                st.shift_pressed,
                                st.alt_pressed,
                                st.win_pressed,
                                0,
                            );
                            save_shortcut_config(&st.shortcut);
                            st.recording_shortcut = false;
                            recorded_new_shortcut = true;
                        }
                    }
                    if is_ctrl { st.ctrl_pressed = false; }
                    if is_shift { st.shift_pressed = false; }
                    if is_alt { st.alt_pressed = false; }
                    if is_win { st.win_pressed = false; }
                }
            }
        });

        if recorded_new_shortcut {
            STATE.with(|s| {
                let st = s.borrow();
                update_tray_tooltip(st.hwnd, st.enabled, st.method);
            });
            return 1;
        }

        // --- Active Shortcut Trigger Evaluation ---
        let mut shortcut_triggered = false;
        STATE.with(|s| {
            let mut st = s.borrow_mut();
            if is_keydown {
                if is_ctrl { st.ctrl_pressed = true; }
                if is_shift { st.shift_pressed = true; }
                if is_alt { st.alt_pressed = true; }
                if is_win { st.win_pressed = true; }

                if !st.shortcut.is_disabled() && st.shortcut.key_code != 0 {
                    let req_ctrl = st.shortcut.ctrl;
                    let req_shift = st.shortcut.shift;
                    let req_alt = st.shortcut.alt;
                    let req_win = st.shortcut.win;

                    if st.shortcut.key_code == vk
                        && (!req_ctrl || st.ctrl_pressed)
                        && (!req_shift || st.shift_pressed)
                        && (!req_alt || st.alt_pressed)
                        && (!req_win || st.win_pressed)
                    {
                        shortcut_triggered = true;
                    }
                }
            } else if is_keyup {
                if !st.shortcut.is_disabled() && st.shortcut.key_code == 0 {
                    // Modifier chord check on release
                    let req_ctrl = st.shortcut.ctrl;
                    let req_shift = st.shortcut.shift;
                    let req_alt = st.shortcut.alt;
                    let req_win = st.shortcut.win;

                    let matches_ctrl = !req_ctrl || st.ctrl_pressed;
                    let matches_shift = !req_shift || st.shift_pressed;
                    let matches_alt = !req_alt || st.alt_pressed;
                    let matches_win = !req_win || st.win_pressed;

                    if matches_ctrl && matches_shift && matches_alt && matches_win && !st.other_key_during_chord {
                        shortcut_triggered = true;
                    }
                }

                if is_ctrl { st.ctrl_pressed = false; }
                if is_shift { st.shift_pressed = false; }
                if is_alt { st.alt_pressed = false; }
                if is_win { st.win_pressed = false; }
                if !st.ctrl_pressed && !st.shift_pressed && !st.alt_pressed && !st.win_pressed {
                    st.other_key_during_chord = false;
                }
            }
        });

        if shortcut_triggered {
            STATE.with(|s| {
                let mut st = s.borrow_mut();
                st.enabled = !st.enabled;
                st.engine.reset();
                update_tray_tooltip(st.hwnd, st.enabled, st.method);
            });
            return 1;
        }

        if is_ctrl || is_shift || is_alt || is_win {
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        if !is_keydown {
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        STATE.with(|s| {
            let mut st = s.borrow_mut();
            if st.ctrl_pressed || st.shift_pressed || st.alt_pressed || st.win_pressed {
                st.other_key_during_chord = true;
            }
        });

        // Check shortcut combinations (Ctrl/Alt/Win)
        let ctrl_down = (GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0;
        let shift_down = (GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0;
        let alt_down = (GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) != 0;
        let win_down = (GetAsyncKeyState(VK_LWIN as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RWIN as i32) as u16 & 0x8000) != 0;

        // Đảo từ nhanh bằng Ctrl + Shift + Z (Flip Word giữa Tiếng Việt và chuỗi phím thô)
        if ctrl_down && shift_down && !alt_down && !win_down && vk == 0x5A { // 0x5A = 'Z'
            let flipped = STATE.with(|s| {
                let mut st = s.borrow_mut();
                if st.enabled {
                    if let Some(out) = st.engine.flip_current_word() {
                        emit_output(out.backspaces, &out.commit_text);
                        return true;
                    }
                }
                false
            });
            if flipped {
                return 1;
            }
        }

        if ctrl_down || alt_down || win_down {
            STATE.with(|s| s.borrow_mut().engine.reset());
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        let consumed = STATE.with(|s| {
            let mut st = s.borrow_mut();
            if !st.enabled {
                return false;
            }

            // Escape: Dokey-style revert
            if vk == VK_ESCAPE {
                if st.esc_restore {
                    if let Some(output) = st.engine.undo_to_raw() {
                        emit_output(output.backspaces, &output.commit_text);
                        return true;
                    }
                }
                st.engine.reset();
                return false;
            }

            // Backspace: Macro undo check
            if vk == VK_BACK {
                if st.macro_enabled {
                    if let Some((bs, commit)) = st.engine.undo_macro_expansion() {
                        emit_output(bs, &commit);
                        return true;
                    }
                }
                st.engine.handle_backspace();
                return false;
            }

            // Navigation keys
            if matches!(
                vk,
                VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_TAB | VK_RETURN
            ) {
                st.engine.reset();
                return false;
            }

            if let Some(ch) = vk_to_char(kbd.vkCode, kbd.scanCode) {
                let output = st.engine.process_key(ch, now_ms());
                if output.consumed {
                    emit_output(output.backspaces, &output.commit_text);
                    return true;
                }
            }
            false
        });

        if consumed {
            1
        } else {
            CallNextHookEx(null_mut(), code, wparam, lparam)
        }
    }

    pub unsafe extern "system" fn mouse_hook_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code >= 0 {
            let msg = wparam as u32;
            if msg == WM_LBUTTONDOWN || msg == WM_RBUTTONDOWN || msg == WM_MBUTTONDOWN {
                STATE.with(|s| s.borrow_mut().engine.reset());
            }
        }
        CallNextHookEx(null_mut(), code, wparam, lparam)
    }

    unsafe fn add_tray_icon(hwnd: HWND) {
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        let hinstance = GetModuleHandleW(null_mut());
        let custom_icon = LoadIconW(hinstance, 1 as *const u16);
        nid.hIcon = if !custom_icon.is_null() {
            custom_icon
        } else {
            LoadIconW(null_mut(), IDI_APPLICATION)
        };

        let tip = to_wstring("VnCharKey [V] - Gõ Tiếng Việt (Tự do)\nNhấp trái để Bật/Tắt • Chuột phải để Mở Menu");
        for (i, &ch) in tip.iter().take(127).enumerate() {
            nid.szTip[i] = ch;
        }

        Shell_NotifyIconW(NIM_ADD, &nid);
    }

    unsafe fn update_tray_tooltip(hwnd: HWND, enabled: bool, method: InputMethod) {
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_TIP;

        let m_str = match method {
            InputMethod::Auto => "Tự động (Auto)",
            InputMethod::Telex => "Telex",
            InputMethod::Vni => "VNI",
            InputMethod::SimpleTelex => "Simple Telex",
        };
        let status = if enabled {
            format!("VnCharKey [V] - Đang Bật ({})\nBấm để Tắt", m_str)
        } else {
            "VnCharKey [E] - Tạm Tắt (Tiếng Anh)\nBấm để Bật".to_string()
        };

        let tip = to_wstring(&status);
        for (i, &ch) in tip.iter().take(127).enumerate() {
            nid.szTip[i] = ch;
        }
        Shell_NotifyIconW(NIM_MODIFY, &nid);
    }

    unsafe fn remove_tray_icon(hwnd: HWND) {
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        Shell_NotifyIconW(NIM_DELETE, &nid);
    }

    unsafe fn show_context_menu(hwnd: HWND) {
        let menu = CreatePopupMenu();
        let (
            enabled,
            method,
            charset,
            tone,
            spell,
            restore,
            esc,
            bracket,
            macro_en,
            dict_en,
            shortcut,
        ) = STATE.with(|s| {
            let st = s.borrow();
            (
                st.enabled,
                st.method,
                st.charset,
                st.tone_style,
                st.spell_check,
                st.auto_restore,
                st.esc_restore,
                st.bracket_transform,
                st.macro_enabled,
                st.dict_enabled,
                st.shortcut.clone(),
            )
        });

        // 1. Toggle Vietnamese
        let toggle_label = to_wstring("Bật gõ Tiếng Việt");
        AppendMenuW(
            menu,
            MF_STRING | if enabled { MF_CHECKED } else { MF_UNCHECKED },
            ID_TRAY_TOGGLE,
            toggle_label.as_ptr(),
        );
        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // 2. Kiểu Gõ Submenu
        let method_sub = CreatePopupMenu();
        AppendMenuW(
            method_sub,
            MF_STRING | if method == InputMethod::Auto { MF_CHECKED } else { 0 },
            ID_METHOD_AUTO,
            to_wstring("Tự động (Telex + VNI)").as_ptr(),
        );
        AppendMenuW(
            method_sub,
            MF_STRING | if method == InputMethod::Telex { MF_CHECKED } else { 0 },
            ID_METHOD_TELEX,
            to_wstring("Telex").as_ptr(),
        );
        AppendMenuW(
            method_sub,
            MF_STRING | if method == InputMethod::Vni { MF_CHECKED } else { 0 },
            ID_METHOD_VNI,
            to_wstring("VNI").as_ptr(),
        );
        AppendMenuW(
            method_sub,
            MF_STRING | if method == InputMethod::SimpleTelex { MF_CHECKED } else { 0 },
            ID_METHOD_SIMPLETELEX,
            to_wstring("Simple Telex").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_POPUP,
            method_sub as usize,
            to_wstring("Kiểu gõ").as_ptr(),
        );

        // 3. Bảng mã & Dấu thanh Submenu
        let style_sub = CreatePopupMenu();
        AppendMenuW(
            style_sub,
            MF_STRING | if tone == ToneStyle::Modern { MF_CHECKED } else { 0 },
            ID_TONE_MODERN,
            to_wstring("Dấu kiểu mới (oà, uý)").as_ptr(),
        );
        AppendMenuW(
            style_sub,
            MF_STRING | if tone == ToneStyle::Classic { MF_CHECKED } else { 0 },
            ID_TONE_CLASSIC,
            to_wstring("Dấu kiểu cũ (hòa, ủy)").as_ptr(),
        );
        AppendMenuW(style_sub, MF_SEPARATOR, 0, null_mut());
        AppendMenuW(
            style_sub,
            MF_STRING | if charset == Charset::Unicode { MF_CHECKED } else { 0 },
            ID_CHARSET_NFC,
            to_wstring("Unicode Dựng Sẵn (NFC)").as_ptr(),
        );
        AppendMenuW(
            style_sub,
            MF_STRING | if charset == Charset::UnicodeComposite { MF_CHECKED } else { 0 },
            ID_CHARSET_NFD,
            to_wstring("Unicode Tổ Hợp (NFD)").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_POPUP,
            style_sub as usize,
            to_wstring("Bảng mã & Dấu thanh").as_ptr(),
        );

        // 4. Phím tắt Bật/Tắt Submenu
        let sc_sub = CreatePopupMenu();
        let cur_sc_label = format!("Hiện tại: [{}]", shortcut.display_name);
        AppendMenuW(
            sc_sub,
            MF_STRING | MF_GRAYED,
            0,
            to_wstring(&cur_sc_label).as_ptr(),
        );
        AppendMenuW(sc_sub, MF_SEPARATOR, 0, null_mut());

        AppendMenuW(
            sc_sub,
            MF_STRING,
            ID_SHORTCUT_RECORD,
            to_wstring("⌨️ Bấm để đặt phím tắt mới (Tự do)...").as_ptr(),
        );
        AppendMenuW(sc_sub, MF_SEPARATOR, 0, null_mut());

        let is_ctrl_shift = shortcut.ctrl && shortcut.shift && !shortcut.alt && !shortcut.win && shortcut.key_code == 0;
        let is_alt_z = !shortcut.ctrl && !shortcut.shift && shortcut.alt && !shortcut.win && shortcut.key_code == 'Z' as u16;
        let is_ctrl_space = shortcut.ctrl && !shortcut.shift && !shortcut.alt && !shortcut.win && shortcut.key_code == VK_SPACE;
        let is_win_space = !shortcut.ctrl && !shortcut.shift && !shortcut.alt && shortcut.win && shortcut.key_code == VK_SPACE;

        AppendMenuW(
            sc_sub,
            MF_STRING | if is_ctrl_shift { MF_CHECKED } else { 0 },
            ID_SHORTCUT_CTRL_SHIFT,
            to_wstring("Ctrl + Shift (Nhấn rồi thả)").as_ptr(),
        );
        AppendMenuW(
            sc_sub,
            MF_STRING | if is_alt_z { MF_CHECKED } else { 0 },
            ID_SHORTCUT_ALT_Z,
            to_wstring("Alt + Z").as_ptr(),
        );
        AppendMenuW(
            sc_sub,
            MF_STRING | if is_ctrl_space { MF_CHECKED } else { 0 },
            ID_SHORTCUT_CTRL_SPACE,
            to_wstring("Ctrl + Space").as_ptr(),
        );
        AppendMenuW(
            sc_sub,
            MF_STRING | if is_win_space { MF_CHECKED } else { 0 },
            ID_SHORTCUT_WIN_SPACE,
            to_wstring("Win + Space").as_ptr(),
        );
        AppendMenuW(
            sc_sub,
            MF_STRING | if shortcut.is_disabled() { MF_CHECKED } else { 0 },
            ID_SHORTCUT_DISABLED,
            to_wstring("Tắt phím tắt chuyển nhanh").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_POPUP,
            sc_sub as usize,
            to_wstring("Phím tắt chuyển Việt / Anh").as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // 5. Tính năng thông minh Checkboxes
        AppendMenuW(
            menu,
            MF_STRING | if spell { MF_CHECKED } else { 0 },
            ID_TOGGLE_SPELLCHECK,
            to_wstring("Kiểm tra Chánh Tả").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING | if restore { MF_CHECKED } else { 0 },
            ID_TOGGLE_AUTORESTORE,
            to_wstring("Tự động khôi phục Tiếng Anh").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING | if esc { MF_CHECKED } else { 0 },
            ID_TOGGLE_ESCRESTORE,
            to_wstring("Hoàn nguyên từ bằng Esc").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING | if bracket { MF_CHECKED } else { 0 },
            ID_TOGGLE_BRACKET,
            to_wstring("Gõ [ ] thành ơ ư").as_ptr(),
        );

        // Tùy chọn Tốc ký
        let quick_sub = CreatePopupMenu();
        AppendMenuW(
            quick_sub,
            MF_STRING,
            ID_TOGGLE_QUICK_TELEX,
            to_wstring("Quick Telex (cc->ch, kk->kh, tt->th...)").as_ptr(),
        );
        AppendMenuW(
            quick_sub,
            MF_STRING,
            ID_TOGGLE_QUICK_START,
            to_wstring("Phụ âm đầu nhanh (f->ph, j->gi, w->qu)").as_ptr(),
        );
        AppendMenuW(
            quick_sub,
            MF_STRING,
            ID_TOGGLE_QUICK_END,
            to_wstring("Phụ âm cuối nhanh (g->ng, h->nh, k->ch)").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_POPUP,
            quick_sub as usize,
            to_wstring("Tùy chọn Tốc ký...").as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // 6. Gõ Tắt & Từ Điển
        AppendMenuW(
            menu,
            MF_STRING | if macro_en { MF_CHECKED } else { 0 },
            ID_TOGGLE_MACRO,
            to_wstring("Bật Gõ Tắt (Macro)").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING,
            ID_OPEN_MACROS,
            to_wstring("Chỉnh sửa bảng Gõ Tắt (macros.conf)...").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING | if dict_en { MF_CHECKED } else { 0 },
            ID_TOGGLE_DICT,
            to_wstring("Bật Từ Điển Cá Nhân").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING,
            ID_OPEN_DICT,
            to_wstring("Chỉnh sửa Từ Điển Cá Nhân (dictionary.conf)...").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING,
            ID_RELOAD_FILES,
            to_wstring("Nạp lại Gõ Tắt & Từ Điển").as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // 7. Khởi động cùng Windows
        let autorun_on = is_autorun_enabled();
        AppendMenuW(
            menu,
            MF_STRING | if autorun_on { MF_CHECKED } else { 0 },
            ID_TOGGLE_AUTORUN,
            to_wstring("Khởi động cùng Windows").as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // 8. Exit
        AppendMenuW(
            menu,
            MF_STRING,
            ID_TRAY_EXIT,
            to_wstring("Thoát VnCharKey").as_ptr(),
        );

        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            menu,
            TPM_BOTTOMALIGN | TPM_LEFTALIGN,
            pt.x,
            pt.y,
            0,
            hwnd,
            null_mut(),
        );
        DestroyMenu(menu);
    }

    pub unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_TRAYICON => {
                let event = (lparam & 0xFFFF) as u32;
                if event == WM_RBUTTONUP {
                    show_context_menu(hwnd);
                } else if event == WM_LBUTTONUP {
                    // Left click on tray icon toggles Vietnamese / English immediately
                    STATE.with(|s| {
                        let mut st = s.borrow_mut();
                        st.enabled = !st.enabled;
                        st.engine.reset();
                        update_tray_tooltip(hwnd, st.enabled, st.method);
                    });
                }
                0
            }
            WM_COMMAND => {
                let id = (wparam & 0xFFFF) as usize;
                STATE.with(|s| {
                    let mut st = s.borrow_mut();
                    match id {
                        ID_TRAY_TOGGLE => {
                            st.enabled = !st.enabled;
                            st.engine.reset();
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        }
                        ID_METHOD_AUTO => {
                            st.method = InputMethod::Auto;
                            st.engine.set_method(InputMethod::Auto);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        }
                        ID_METHOD_TELEX => {
                            st.method = InputMethod::Telex;
                            st.engine.set_method(InputMethod::Telex);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        }
                        ID_METHOD_VNI => {
                            st.method = InputMethod::Vni;
                            st.engine.set_method(InputMethod::Vni);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        }
                        ID_METHOD_SIMPLETELEX => {
                            st.method = InputMethod::SimpleTelex;
                            st.engine.set_method(InputMethod::SimpleTelex);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        }
                        ID_TONE_MODERN => {
                            st.tone_style = ToneStyle::Modern;
                            st.engine.set_tone_style(ToneStyle::Modern);
                        }
                        ID_TONE_CLASSIC => {
                            st.tone_style = ToneStyle::Classic;
                            st.engine.set_tone_style(ToneStyle::Classic);
                        }
                        ID_CHARSET_NFC => {
                            st.charset = Charset::Unicode;
                            st.engine.set_charset(Charset::Unicode);
                        }
                        ID_CHARSET_NFD => {
                            st.charset = Charset::UnicodeComposite;
                            st.engine.set_charset(Charset::UnicodeComposite);
                        }
                        ID_SHORTCUT_CTRL_SHIFT => {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(true, true, false, false, 0);
                            save_shortcut_config(&st.shortcut);
                        }
                        ID_SHORTCUT_ALT_Z => {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(false, false, true, false, 'Z' as u16);
                            save_shortcut_config(&st.shortcut);
                        }
                        ID_SHORTCUT_CTRL_SPACE => {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(true, false, false, false, VK_SPACE);
                            save_shortcut_config(&st.shortcut);
                        }
                        ID_SHORTCUT_WIN_SPACE => {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(false, false, false, true, VK_SPACE);
                            save_shortcut_config(&st.shortcut);
                        }
                        ID_SHORTCUT_DISABLED => {
                            st.shortcut = CustomShortcut::disabled();
                            save_shortcut_config(&st.shortcut);
                        }
                        ID_SHORTCUT_RECORD => {
                            st.recording_shortcut = true;
                            st.ctrl_pressed = false;
                            st.shift_pressed = false;
                            st.alt_pressed = false;
                            st.win_pressed = false;
                            st.other_key_during_chord = false;
                            MessageBoxW(
                                hwnd,
                                to_wstring(
                                    "Đang ở chế độ Đặt Phím Tắt Tự Do!\n\n\
                                     1. Bấm tổ hợp phím bạn muốn đặt (ví dụ: Ctrl+Alt+V, Win+Shift+Space, Alt+Shift,...).\n\
                                     2. Hoặc bấm phím Esc để Hủy bỏ.\n\n\
                                     Bấm OK rồi bấm tổ hợp phím ngay trên bàn phím của bạn."
                                ).as_ptr(),
                                to_wstring("Cài đặt Phím tắt Tự do - VnCharKey").as_ptr(),
                                MB_OK | MB_ICONINFORMATION,
                            );
                        }
                        ID_TOGGLE_SPELLCHECK => {
                            let val = !st.spell_check;
                            st.spell_check = val;
                            st.engine.set_spell_check(val);
                        }
                        ID_TOGGLE_AUTORESTORE => {
                            let val = !st.auto_restore;
                            st.auto_restore = val;
                            st.engine.set_auto_restore_english(val);
                        }
                        ID_TOGGLE_ESCRESTORE => {
                            st.esc_restore = !st.esc_restore;
                        }
                        ID_TOGGLE_BRACKET => {
                            let val = !st.bracket_transform;
                            st.bracket_transform = val;
                            st.engine.set_bracket_transform(val);
                        }
                        ID_TOGGLE_QUICK_TELEX => {
                            let val = !st.engine.quick_telex();
                            st.engine.set_quick_telex(val);
                        }
                        ID_TOGGLE_QUICK_START => {
                            let val = !st.engine.quick_start_consonant();
                            st.engine.set_quick_start_consonant(val);
                        }
                        ID_TOGGLE_QUICK_END => {
                            let val = !st.engine.quick_end_consonant();
                            st.engine.set_quick_end_consonant(val);
                        }
                        ID_TOGGLE_MACRO => {
                            let val = !st.macro_enabled;
                            st.macro_enabled = val;
                            st.engine.set_enable_macro(val);
                        }
                        ID_OPEN_MACROS => {
                            open_file_in_notepad(macros_path());
                        }
                        ID_TOGGLE_DICT => {
                            let val = !st.dict_enabled;
                            st.dict_enabled = val;
                            st.engine.set_enable_dictionary(val);
                        }
                        ID_OPEN_DICT => {
                            open_file_in_notepad(dict_path());
                        }
                        ID_RELOAD_FILES => {
                            reload_files(&mut st.engine);
                        }
                        ID_TOGGLE_AUTORUN => {
                            let current = is_autorun_enabled();
                            set_autorun_enabled(!current);
                        }
                        ID_TRAY_EXIT => {
                            remove_tray_icon(hwnd);
                            PostQuitMessage(0);
                        }
                        _ => {}
                    }
                });
                0
            }
            WM_DESTROY => {
                remove_tray_icon(hwnd);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub unsafe fn run() {
        ensure_config_files();
        let initial_shortcut = load_shortcut_config();
        STATE.with(|s| {
            let mut st = s.borrow_mut();
            st.shortcut = initial_shortcut;
            reload_files(&mut st.engine);
        });

        let hinstance = GetModuleHandleW(null_mut());
        let class_name = to_wstring("VnCharKeyHiddenWindowClass");

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            to_wstring("VnCharKey").as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut(),
        );

        STATE.with(|s| s.borrow_mut().hwnd = hwnd);
        add_tray_icon(hwnd);

        let kbd_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), hinstance, 0);
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), hinstance, 0);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if !kbd_hook.is_null() {
            UnhookWindowsHookEx(kbd_hook);
        }
        if !mouse_hook.is_null() {
            UnhookWindowsHookEx(mouse_hook);
        }
    }
}

fn main() {
    #[cfg(windows)]
    unsafe {
        win_app::run();
    }

    #[cfg(not(windows))]
    {
        println!("VnCharKey for Windows must be compiled targeting Windows (e.g. x86_64-pc-windows-msvc).");
    }
}
