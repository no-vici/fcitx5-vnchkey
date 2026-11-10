//! # VnCharKey for Windows
//!
//! Native Windows Vietnamese Input Method Engine powered by `vnchkey-core`.
//! Features:
//! - Low-level keyboard hook (`WH_KEYBOARD_LL`) + Mouse click reset (`WH_MOUSE_LL`)
//! - Zero-flicker Micro-Diff LCP replacement via atomic `SendInput` batches
//! - System Tray icon with Conical Hat (Nón Lá) + V/E indicator
//! - Quick toggle shortcut: `Ctrl + Shift` (or menu selection)
//! - Escape to restore raw keystrokes & Backspace macro undo

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod win_app {
    use std::cell::RefCell;
    use std::mem::size_of;
    use std::ptr::null_mut;
    use std::time::{SystemTime, UNIX_EPOCH};

    use vnchkey_core::{Engine, EngineConfig, InputMethod};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    use windows_sys::Win32::UI::Shell::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const WM_TRAYICON: u32 = WM_USER + 1;
    const ID_TRAY_TOGGLE: usize = 1001;
    const ID_METHOD_AUTO: usize = 1002;
    const ID_METHOD_TELEX: usize = 1003;
    const ID_METHOD_VNI: usize = 1004;
    const ID_TRAY_EXIT: usize = 1099;

    /// Magic tag in `dwExtraInfo` so our hook ignores synthetic keystrokes injected by `VnCharKey`.
    const VNCHARKEY_INJECTED_TAG: usize = 0x564E_4348;

    struct AppState {
        engine: Engine,
        enabled: bool,
        method: InputMethod,
        hwnd: HWND,
        ctrl_pressed: bool,
        shift_pressed: bool,
        other_key_during_chord: bool,
    }

    thread_local! {
        static STATE: RefCell<AppState> = RefCell::new(AppState {
            engine: Engine::new(EngineConfig::default()),
            enabled: true,
            method: InputMethod::Auto,
            hwnd: null_mut(),
            ctrl_pressed: false,
            shift_pressed: false,
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
        // Ensure Shift & CapsLock state are accurately reflected from async hardware state
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

        // Handle Ctrl+Shift quick toggle
        let is_ctrl = vk == VK_LCONTROL || vk == VK_RCONTROL || vk == VK_CONTROL;
        let is_shift = vk == VK_LSHIFT || vk == VK_RSHIFT || vk == VK_SHIFT;

        if is_ctrl || is_shift {
            STATE.with(|s| {
                let mut st = s.borrow_mut();
                if is_keydown {
                    if is_ctrl {
                        st.ctrl_pressed = true;
                    }
                    if is_shift {
                        st.shift_pressed = true;
                    }
                } else if is_keyup {
                    if st.ctrl_pressed && st.shift_pressed && !st.other_key_during_chord {
                        st.enabled = !st.enabled;
                        st.engine.reset();
                        update_tray_tooltip(st.hwnd, st.enabled, st.method);
                    }
                    if is_ctrl {
                        st.ctrl_pressed = false;
                    }
                    if is_shift {
                        st.shift_pressed = false;
                    }
                    if !st.ctrl_pressed && !st.shift_pressed {
                        st.other_key_during_chord = false;
                    }
                }
            });
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        if !is_keydown {
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        // Mark that another key was pressed while holding Ctrl/Shift
        STATE.with(|s| {
            let mut st = s.borrow_mut();
            if st.ctrl_pressed || st.shift_pressed {
                st.other_key_during_chord = true;
            }
        });

        // Pass through shortcuts with Ctrl, Alt, or Win keys
        let ctrl_down = (GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0;
        let alt_down = (GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) != 0;
        let win_down = (GetAsyncKeyState(VK_LWIN as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RWIN as i32) as u16 & 0x8000) != 0;

        if ctrl_down || alt_down || win_down {
            STATE.with(|s| s.borrow_mut().engine.reset());
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        let consumed = STATE.with(|s| {
            let mut st = s.borrow_mut();
            if !st.enabled {
                return false;
            }

            // Escape: Dokey-style revert to raw keystrokes
            if vk == VK_ESCAPE {
                if let Some(output) = st.engine.undo_to_raw() {
                    emit_output(output.backspaces, &output.commit_text);
                    return true;
                }
                st.engine.reset();
                return false;
            }

            // Backspace: Check macro undo first, otherwise update syllable buffer
            if vk == VK_BACK {
                if let Some((bs, commit)) = st.engine.undo_macro_expansion() {
                    emit_output(bs, &commit);
                    return true;
                }
                st.engine.handle_backspace();
                return false;
            }

            // Navigation keys reset buffer
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
            1 // Swallow original key event
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

        let tip = to_wstring("VnCharKey [V] - Gõ Tiếng Việt (Auto)\nNhấn Ctrl+Shift để Bật/Tắt");
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
            InputMethod::Auto => "Tự động",
            InputMethod::Telex => "Telex",
            InputMethod::Vni => "VNI",
            InputMethod::SimpleTelex => "Simple Telex",
        };
        let status = if enabled {
            format!("VnCharKey [V] - Đang Bật ({})", m_str)
        } else {
            "VnCharKey [E] - Tạm Tắt (Tiếng Anh)".to_string()
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
        let (enabled, method) = STATE.with(|s| {
            let st = s.borrow();
            (st.enabled, st.method)
        });

        let toggle_label = to_wstring("Bật gõ Tiếng Việt (Ctrl+Shift)");
        AppendMenuW(
            menu,
            MF_STRING | if enabled { MF_CHECKED } else { MF_UNCHECKED },
            ID_TRAY_TOGGLE,
            toggle_label.as_ptr(),
        );
        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        let auto_label = to_wstring("Kiểu gõ: Tự động (Auto)");
        let telex_label = to_wstring("Kiểu gõ: Telex");
        let vni_label = to_wstring("Kiểu gõ: VNI");

        AppendMenuW(
            menu,
            MF_STRING | if method == InputMethod::Auto { MF_CHECKED } else { 0 },
            ID_METHOD_AUTO,
            auto_label.as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING | if method == InputMethod::Telex { MF_CHECKED } else { 0 },
            ID_METHOD_TELEX,
            telex_label.as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING | if method == InputMethod::Vni { MF_CHECKED } else { 0 },
            ID_METHOD_VNI,
            vni_label.as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());
        let exit_label = to_wstring("Thoát VnCharKey");
        AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, exit_label.as_ptr());

        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(menu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, pt.x, pt.y, 0, hwnd, null_mut());
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
                let id = wparam & 0xFFFF;
                match id {
                    ID_TRAY_TOGGLE => {
                        STATE.with(|s| {
                            let mut st = s.borrow_mut();
                            st.enabled = !st.enabled;
                            st.engine.reset();
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        });
                    }
                    ID_METHOD_AUTO => {
                        STATE.with(|s| {
                            let mut st = s.borrow_mut();
                            st.method = InputMethod::Auto;
                            st.engine.set_method(InputMethod::Auto);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        });
                    }
                    ID_METHOD_TELEX => {
                        STATE.with(|s| {
                            let mut st = s.borrow_mut();
                            st.method = InputMethod::Telex;
                            st.engine.set_method(InputMethod::Telex);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        });
                    }
                    ID_METHOD_VNI => {
                        STATE.with(|s| {
                            let mut st = s.borrow_mut();
                            st.method = InputMethod::Vni;
                            st.engine.set_method(InputMethod::Vni);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        });
                    }
                    ID_TRAY_EXIT => {
                        remove_tray_icon(hwnd);
                        PostQuitMessage(0);
                    }
                    _ => {}
                }
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
