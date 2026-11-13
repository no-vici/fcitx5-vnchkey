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
    use std::collections::HashMap;
    use std::fs;
    use std::mem::size_of;
    use std::path::PathBuf;
    use std::process::Command;
    use std::ptr::null_mut;
    use std::time::{SystemTime, UNIX_EPOCH};

    use vnchkey_core::{Charset, Engine, EngineConfig, InputMethod, ToneStyle};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    use windows_sys::Win32::UI::Shell::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const WM_TRAYICON: u32 = WM_USER + 1;

    // Menu Action IDs
    const ID_TRAY_TOGGLE: usize = 1001;
    const ID_TOGGLE_PER_APP: usize = 1006;

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
    const ID_TOGGLE_CASE_ANCHOR: usize = 1065;
    const ID_TOGGLE_MACRO: usize = 1034;
    const ID_TOGGLE_DICT: usize = 1035;
    const ID_TOGGLE_HABIT_LEARNING: usize = 1053;
    const ID_HABIT_THRESHOLD_2: usize = 1054;
    const ID_HABIT_THRESHOLD_3: usize = 1055;
    const ID_HABIT_THRESHOLD_4: usize = 1056;
    const ID_HABIT_THRESHOLD_5: usize = 1057;
    const ID_HABIT_THRESHOLD_6: usize = 1058;
    const ID_HABIT_THRESHOLD_8: usize = 1059;

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

    const ID_FLIP_CTRL_SHIFT_Z: usize = 1060;
    const ID_FLIP_ALT_X: usize = 1061;
    const ID_FLIP_CTRL_ALT_Z: usize = 1062;
    const ID_FLIP_DISABLED: usize = 1063;
    const ID_FLIP_RECORD: usize = 1064;

    const ID_OPEN_MACROS: usize = 1050;
    const ID_OPEN_DICT: usize = 1051;
    const ID_RELOAD_FILES: usize = 1052;

    const ID_ABOUT: usize = 1070;
    const ID_CHECK_UPDATE: usize = 1071;

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum RecordingMode {
        None,
        ToggleShortcut,
        FlipShortcut,
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
        per_app: bool,
        app_states: HashMap<String, bool>,
        current_app: String,
        shortcut: CustomShortcut,
        flip_shortcut: CustomShortcut,
        recording_mode: RecordingMode,
        hwnd: HWND,
        chord_armed: bool,
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
            per_app: true,
            app_states: HashMap::new(),
            current_app: String::new(),
            shortcut: CustomShortcut::default(),
            flip_shortcut: CustomShortcut::from_modifiers_and_key(true, true, false, false, 0x5A),
            recording_mode: RecordingMode::None,
            hwnd: null_mut(),
            chord_armed: false,
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
        // Chế độ 100% Portable: Luôn lưu toàn bộ cấu hình (config.conf, macros.conf,
        // dictionary.conf, habit.log) ngay trong thư mục chứa file VnCharKey.exe
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.to_path_buf();
            }
        }
        PathBuf::from(".")
    }

    fn macros_path() -> PathBuf {
        config_dir().join("macros.conf")
    }

    fn dict_path() -> PathBuf {
        config_dir().join("dictionary.conf")
    }

    fn habit_log_path() -> PathBuf {
        config_dir().join("habit.log")
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
                               method=auto\n\
                               charset=unicode\n\
                               tone_style=classic\n\
                               spell_check=true\n\
                               auto_restore=true\n\
                               esc_restore=true\n\
                               bracket_transform=false\n\
                               quick_telex=false\n\
                               quick_start_consonant=false\n\
                               quick_end_consonant=false\n\
                               macro_enabled=true\n\
                               dict_enabled=true\n\
                               ctrl=true\n\
                               shift=true\n\
                               alt=false\n\
                               win=false\n\
                               vk=0\n\
                               flip_ctrl=true\n\
                               flip_shift=true\n\
                               flip_alt=false\n\
                               flip_win=false\n\
                               flip_vk=90\n";
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

    fn default_flip_shortcut() -> CustomShortcut {
        CustomShortcut::from_modifiers_and_key(true, true, false, false, 0x5A) // Ctrl + Shift + Z
    }

    fn load_shortcut_config() -> (CustomShortcut, CustomShortcut) {
        let path = config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            let mut ctrl = false;
            let mut shift = false;
            let mut alt = false;
            let mut win = false;
            let mut vk: u16 = 0;

            let mut flip_ctrl = true;
            let mut flip_shift = true;
            let mut flip_alt = false;
            let mut flip_win = false;
            let mut flip_vk: u16 = 0x5A; // default 'Z'
            let mut has_flip_keys = false;

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
                        "flip_ctrl" => { flip_ctrl = v == "true" || v == "1"; has_flip_keys = true; }
                        "flip_shift" => { flip_shift = v == "true" || v == "1"; has_flip_keys = true; }
                        "flip_alt" => { flip_alt = v == "true" || v == "1"; has_flip_keys = true; }
                        "flip_win" => { flip_win = v == "true" || v == "1"; has_flip_keys = true; }
                        "flip_vk" => { flip_vk = v.parse::<u16>().unwrap_or(0); has_flip_keys = true; }
                        _ => {}
                    }
                }
            }
            let sc = if !ctrl && !shift && !alt && !win && vk == 0 {
                CustomShortcut::disabled()
            } else {
                CustomShortcut::from_modifiers_and_key(ctrl, shift, alt, win, vk)
            };

            let flip_sc = if !has_flip_keys {
                default_flip_shortcut()
            } else if !flip_ctrl && !flip_shift && !flip_alt && !flip_win && flip_vk == 0 {
                CustomShortcut::disabled()
            } else {
                CustomShortcut::from_modifiers_and_key(flip_ctrl, flip_shift, flip_alt, flip_win, flip_vk)
            };

            (sc, flip_sc)
        } else {
            (CustomShortcut::default(), default_flip_shortcut())
        }
    }

    fn load_full_config(st: &mut AppState) {
        let path = config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim().to_lowercase();
                    let v = v.trim().to_lowercase();
                    match k.as_str() {
                        "method" => {
                            match v.as_str() {
                                "telex" => {
                                    st.method = InputMethod::Telex;
                                    st.engine.set_method(InputMethod::Telex);
                                }
                                "vni" => {
                                    st.method = InputMethod::Vni;
                                    st.engine.set_method(InputMethod::Vni);
                                }
                                "simpletelex" | "simple_telex" => {
                                    st.method = InputMethod::SimpleTelex;
                                    st.engine.set_method(InputMethod::SimpleTelex);
                                }
                                _ => {
                                    st.method = InputMethod::Auto;
                                    st.engine.set_method(InputMethod::Auto);
                                }
                            }
                        }
                        "charset" => {
                            if v == "composite" || v == "nfd" {
                                st.charset = Charset::UnicodeComposite;
                                st.engine.set_charset(Charset::UnicodeComposite);
                            } else {
                                st.charset = Charset::Unicode;
                                st.engine.set_charset(Charset::Unicode);
                            }
                        }
                        "tone_style" => {
                            if v == "modern" {
                                st.tone_style = ToneStyle::Modern;
                                st.engine.set_tone_style(ToneStyle::Modern);
                            } else {
                                st.tone_style = ToneStyle::Classic;
                                st.engine.set_tone_style(ToneStyle::Classic);
                            }
                        }
                        "spell_check" => {
                            let val = v == "true" || v == "1";
                            st.spell_check = val;
                            st.engine.set_spell_check(val);
                        }
                        "auto_restore" => {
                            let val = v == "true" || v == "1";
                            st.auto_restore = val;
                            st.engine.set_auto_restore_english(val);
                        }
                        "esc_restore" => {
                            st.esc_restore = v == "true" || v == "1";
                        }
                        "bracket_transform" => {
                            let val = v == "true" || v == "1";
                            st.bracket_transform = val;
                            st.engine.set_bracket_transform(val);
                        }
                        "quick_telex" => {
                            st.engine.set_quick_telex(v == "true" || v == "1");
                        }
                        "quick_start_consonant" => {
                            st.engine.set_quick_start_consonant(v == "true" || v == "1");
                        }
                        "quick_end_consonant" => {
                            st.engine.set_quick_end_consonant(v == "true" || v == "1");
                        }
                        "macro_enabled" => {
                            let val = v == "true" || v == "1";
                            st.macro_enabled = val;
                            st.engine.set_enable_macro(val);
                        }
                        "dict_enabled" => {
                            let val = v == "true" || v == "1";
                            st.dict_enabled = val;
                            st.engine.set_enable_dictionary(val);
                        }
                        "learn_habits" | "habit_learning" => {
                            let val = v == "true" || v == "1";
                            st.engine.set_habit_learning_enabled(val);
                        }
                        "habit_threshold" => {
                            let val = v.parse::<u32>().unwrap_or(4).max(1);
                            st.engine.set_habit_promote_threshold(val);
                        }
                        "initial_case_anchor" => {
                            let val = v == "true" || v == "1";
                            st.engine.set_initial_case_anchor(val);
                        }
                        "per_app" => {
                            st.per_app = v == "true" || v == "1";
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn save_app_config(st: &AppState) {
        let path = config_path();
        let method_str = match st.method {
            InputMethod::Auto => "auto",
            InputMethod::Telex => "telex",
            InputMethod::Vni => "vni",
            InputMethod::SimpleTelex => "simpletelex",
        };
        let charset_str = match st.charset {
            Charset::Unicode => "unicode",
            Charset::UnicodeComposite => "composite",
        };
        let tone_str = match st.tone_style {
            ToneStyle::Modern => "modern",
            ToneStyle::Classic => "classic",
        };

        let content = format!(
            "# VnCharKey Windows Configuration\n\
             method={}\n\
             charset={}\n\
             tone_style={}\n\
             spell_check={}\n\
             auto_restore={}\n\
             esc_restore={}\n\
             bracket_transform={}\n\
             quick_telex={}\n\
             quick_start_consonant={}\n\
             quick_end_consonant={}\n\
             macro_enabled={}\n\
             dict_enabled={}\n\
             learn_habits={}\n\
             habit_threshold={}\n\
             initial_case_anchor={}\n\
             per_app={}\n\
             ctrl={}\n\
             shift={}\n\
             alt={}\n\
             win={}\n\
             vk={}\n\
             flip_ctrl={}\n\
             flip_shift={}\n\
             flip_alt={}\n\
             flip_win={}\n\
             flip_vk={}\n",
            method_str,
            charset_str,
            tone_str,
            st.spell_check,
            st.auto_restore,
            st.esc_restore,
            st.bracket_transform,
            st.engine.quick_telex(),
            st.engine.quick_start_consonant(),
            st.engine.quick_end_consonant(),
            st.macro_enabled,
            st.dict_enabled,
            st.engine.habit_learning_enabled(),
            st.engine.habit_promote_threshold(),
            st.engine.initial_case_anchor(),
            st.per_app,
            st.shortcut.ctrl, st.shortcut.shift, st.shortcut.alt, st.shortcut.win, st.shortcut.key_code,
            st.flip_shortcut.ctrl, st.flip_shortcut.shift, st.flip_shortcut.alt, st.flip_shortcut.win, st.flip_shortcut.key_code
        );
        let _ = fs::write(&path, content);
    }

    fn save_shortcut_config(sc: &CustomShortcut, flip_sc: &CustomShortcut) {
        STATE.with(|s| {
            if let Ok(mut st) = s.try_borrow_mut() {
                st.shortcut = sc.clone();
                st.flip_shortcut = flip_sc.clone();
                save_app_config(&st);
            }
        });
    }

    fn reload_files(engine: &mut Engine) {
        let m_path = macros_path();
        let d_path = dict_path();
        let h_path = habit_log_path();
        if let Some(p) = m_path.to_str() {
            engine.load_macros_from_file(p);
        }
        if let Some(p) = d_path.to_str() {
            engine.load_dictionary_from_file(p);
        }
        if let Some(p) = h_path.to_str() {
            engine.set_habit_log_path(p);
        }
    }

    type HKEY = *mut std::ffi::c_void;
    const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as usize as HKEY;
    const KEY_READ: u32 = 0x20019;
    const KEY_SET_VALUE: u32 = 0x0002;
    const REG_SZ: u32 = 1;

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: HKEY,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut HKEY,
        ) -> i32;
        fn RegQueryValueExW(
            hKey: HKEY,
            lpValueName: *const u16,
            lpReserved: *const u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;
        fn RegSetValueExW(
            hKey: HKEY,
            lpValueName: *const u16,
            Reserved: u32,
            dwType: u32,
            lpData: *const u8,
            cbData: u32,
        ) -> i32;
        fn RegDeleteValueW(hKey: HKEY, lpValueName: *const u16) -> i32;
        fn RegCloseKey(hKey: HKEY) -> i32;
    }

    fn is_autorun_enabled() -> bool {
        // Pure Win32 Registry API (zero reg.exe subprocess invocation, 0ms, 100% portable)
        let subkey = to_wstring(r"Software\Microsoft\Windows\CurrentVersion\Run");
        let val_name = to_wstring("VnCharKey");
        let mut hkey: HKEY = null_mut();
        unsafe {
            if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
                let mut val_type = 0u32;
                let mut data_len = 0u32;
                let status = RegQueryValueExW(
                    hkey,
                    val_name.as_ptr(),
                    null_mut(),
                    &mut val_type,
                    null_mut(),
                    &mut data_len,
                );
                RegCloseKey(hkey);
                return status == 0;
            }
        }
        false
    }

    fn set_autorun_enabled(enable: bool) {
        // Pure Win32 Registry API (zero reg.exe subprocess invocation, 0ms, 100% portable)
        let subkey = to_wstring(r"Software\Microsoft\Windows\CurrentVersion\Run");
        let val_name = to_wstring("VnCharKey");
        let mut hkey: HKEY = null_mut();
        unsafe {
            if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_SET_VALUE, &mut hkey) == 0 {
                if enable {
                    if let Ok(exe_path) = std::env::current_exe() {
                        let quoted = format!("\"{}\"", exe_path.to_string_lossy());
                        let val_data = to_wstring(&quoted);
                        let cb_data = (val_data.len() * 2) as u32;
                        let _ = RegSetValueExW(
                            hkey,
                            val_name.as_ptr(),
                            0,
                            REG_SZ,
                            val_data.as_ptr() as *const u8,
                            cb_data,
                        );
                    }
                } else {
                    let _ = RegDeleteValueW(hkey, val_name.as_ptr());
                }
                RegCloseKey(hkey);
            }
        }
    }

    fn open_file_in_notepad(path: PathBuf) {
        let _ = Command::new("notepad.exe").arg(path).spawn();
    }

    /// Hiển thị hộp thoại About với thông tin chi tiết về phiên bản và dự án
    unsafe fn show_about_dialog(hwnd: HWND) {
        let cur_ver = env!("CARGO_PKG_VERSION");
        let info = format!(
            "VnCharKey — Bộ Gõ Tiếng Việt\n\
             Phiên bản: v{}\n\n\
             • Bộ gõ Tiếng Việt hiện đại và gọn nhẹ cho Windows\n\
             • 100% Offline, bảo mật cao và không gởi bất kỳ dữ liệu nào ra khỏi máy người dùng\n\
             • Tự động khôi phục Tiếng Anh, Smart Casing, hồi sinh từ\n\
             • Hoạt động độc lập, không cần cài đặt (Portable Mode)\n\n\
             Mã nguồn & Giấy phép:\n\
             https://github.com/no-vici/fcitx5-vnchkey\n\
             Giấy phép: GNU General Public License v3.0 (GPLv3)\n\n\
             Bấm 'Yes' để mở trang GitHub phát hành chính thức.\n\
             Bấm 'No' để đóng hộp thoại này.",
            cur_ver
        );
        let title = format!("Thông tin VnCharKey v{}", cur_ver);
        let res = MessageBoxW(
            hwnd,
            to_wstring(&info).as_ptr(),
            to_wstring(&title).as_ptr(),
            MB_YESNO | MB_ICONINFORMATION,
        );
        if res == IDYES {
            let _ = Command::new("cmd")
                .args(&["/c", "start", "https://github.com/no-vici/fcitx5-vnchkey/releases"])
                .spawn();
        }
    }

    /// Kiểm tra phiên bản mới từ GitHub Releases và tự động tải/cập nhật
    unsafe fn check_for_updates(hwnd: HWND, silent_if_latest: bool) {
        let hwnd_raw = hwnd as usize;
        std::thread::spawn(move || {
            let hwnd = hwnd_raw as HWND;
            let cur_ver = env!("CARGO_PKG_VERSION");
            // Sử dụng PowerShell để gọi GitHub Releases API và lấy tag mới nhất
            let ps_script = r#"[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $res = Invoke-RestMethod -Uri "https://api.github.com/repos/no-vici/fcitx5-vnchkey/releases/latest" -Headers @{"User-Agent"="vnchKey-AutoUpdater"}; Write-Output $res.tag_name"#;
            let output = Command::new("powershell")
                .args(&["-NoProfile", "-NonInteractive", "-Command", ps_script])
                .output();

            if let Ok(out) = output {
                let latest_tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if latest_tag.is_empty() || latest_tag.to_lowercase().contains("error") {
                    if !silent_if_latest {
                        unsafe {
                            MessageBoxW(
                                hwnd,
                                to_wstring("Không thể kiểm tra bản cập nhật lúc này. Vui lòng kiểm tra lại kết nối mạng hoặc thử lại sau.").as_ptr(),
                                to_wstring("Kiểm tra cập nhật").as_ptr(),
                                MB_OK | MB_ICONWARNING,
                            );
                        }
                    }
                    return;
                }

                let clean_latest = latest_tag.trim_start_matches('v').trim();
                let clean_cur = cur_ver.trim_start_matches('v').trim();

                if clean_latest > clean_cur {
                    let msg = format!(
                        "Đã có phiên bản mới: {}\n(Phiên bản hiện tại của bạn: v{})\n\nBạn có muốn tự động tải và cập nhật VnCharKey ngay bây giờ không?",
                        latest_tag, cur_ver
                    );
                    let res = unsafe {
                        MessageBoxW(
                            hwnd,
                            to_wstring(&msg).as_ptr(),
                            to_wstring("Đã có bản cập nhật mới!").as_ptr(),
                            MB_YESNO | MB_ICONQUESTION,
                        )
                    };

                    if res == IDYES {
                        // Chạy script PowerShell tự động tải VnCharKey.exe mới, thay thế và khởi động lại
                        let update_ps = format!(
                            r#"$ErrorActionPreference = 'Stop';
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12;
$curExe = [System.Diagnostics.Process]::GetCurrentProcess().MainModule.FileName;
$tmpExe = "$env:TEMP\VnCharKey_new.exe";
$url = "https://github.com/no-vici/fcitx5-vnchkey/releases/download/{}/VnCharKey.exe";
Invoke-WebRequest -Uri $url -OutFile $tmpExe -UserAgent "vnchKey-AutoUpdater";
Start-Sleep -Seconds 1;
$batch = "$env:TEMP\update_vncharkey.bat";
$cmd = ":repeat`r`ntaskkill /f /im VnCharKey.exe >nul 2>&1`r`ntimeout /t 1 /nobreak >nul`r`nmove /y `"$tmpExe`" `"$curExe`" >nul 2>&1`r`nif errorlevel 1 goto repeat`r`nstart `"`" `"$curExe`"`r`ndel `"%~f0`"";
Set-Content -Path $batch -Value $cmd -Encoding ASCII;
Start-Process -FilePath $batch -WindowStyle Hidden;
"#,
                            latest_tag
                        );

                        let _ = Command::new("powershell")
                            .args(&["-NoProfile", "-NonInteractive", "-Command", &update_ps])
                            .spawn();
                        std::process::exit(0);
                    }
                } else if !silent_if_latest {
                    let msg = format!(
                        "Bạn đang sử dụng phiên bản mới nhất: v{}!\nKhông có bản cập nhật nào mới hơn.",
                        cur_ver
                    );
                    unsafe {
                        MessageBoxW(
                            hwnd,
                            to_wstring(&msg).as_ptr(),
                            to_wstring("Kiểm tra cập nhật").as_ptr(),
                            MB_OK | MB_ICONINFORMATION,
                        );
                    }
                }
            } else if !silent_if_latest {
                unsafe {
                    MessageBoxW(
                        hwnd,
                        to_wstring("Không thể khởi chạy tiến trình kiểm tra cập nhật.").as_ptr(),
                        to_wstring("Lỗi").as_ptr(),
                        MB_OK | MB_ICONERROR,
                    );
                }
            }
        });
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

        // Live hardware state synchronization for Shift and CapsLock:
        // When low-level hooks run, GetKeyboardState can contain stale modifier flags from previous strokes.
        // We explicitly sync VK_SHIFT, VK_LSHIFT, VK_RSHIFT and VK_CAPITAL with GetAsyncKeyState/GetKeyState.
        let shift_pressed = ((GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0)
            || ((GetAsyncKeyState(VK_LSHIFT as i32) as u16 & 0x8000) != 0)
            || ((GetAsyncKeyState(VK_RSHIFT as i32) as u16 & 0x8000) != 0);
        let shift_val = if shift_pressed { 0x80 } else { 0 };
        key_state[VK_SHIFT as usize] = shift_val;
        key_state[VK_LSHIFT as usize] = shift_val;
        key_state[VK_RSHIFT as usize] = shift_val;

        let caps_on = (GetKeyState(VK_CAPITAL as i32) & 0x0001) != 0;
        key_state[VK_CAPITAL as usize] = if caps_on { 0x01 } else { 0 };

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
        let is_mod = is_ctrl || is_shift || is_alt || is_win;

        // Read live modifier state directly from Win32 (plus current key event) to prevent state desync
        let ctrl_down = ((GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0 || (is_keydown && is_ctrl)) && !(is_keyup && is_ctrl);
        let shift_down = ((GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0 || (is_keydown && is_shift)) && !(is_keyup && is_shift);
        let alt_down = ((GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) != 0 || (is_keydown && is_alt)) && !(is_keyup && is_alt);
        let win_down = ((GetAsyncKeyState(VK_LWIN as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RWIN as i32) as u16 & 0x8000) != 0
            || (is_keydown && is_win))
            && !(is_keyup && is_win);

        // --- Custom Shortcut Recorder Mode ---
        let mut recorded_new_shortcut = false;
        STATE.with(|s| {
            let Ok(mut st) = s.try_borrow_mut() else { return; };
            if st.recording_mode != RecordingMode::None {
                if is_keydown {
                    if !is_mod {
                        // User pressed a regular key while holding modifiers!
                        if vk == VK_ESCAPE {
                            st.recording_mode = RecordingMode::None;
                            recorded_new_shortcut = true;
                        } else {
                            let new_sc = CustomShortcut::from_modifiers_and_key(
                                ctrl_down,
                                shift_down,
                                alt_down,
                                win_down,
                                vk,
                            );
                            if st.recording_mode == RecordingMode::ToggleShortcut {
                                st.shortcut = new_sc;
                            } else if st.recording_mode == RecordingMode::FlipShortcut {
                                st.flip_shortcut = new_sc;
                            }
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                            st.recording_mode = RecordingMode::None;
                            recorded_new_shortcut = true;
                        }
                    }
                } else if is_keyup && st.recording_mode == RecordingMode::ToggleShortcut {
                    // Modifier-only chord recording for Toggle Shortcut (when user releases one of the modifiers)
                    if is_mod && !st.other_key_during_chord {
                        let had_ctrl = ctrl_down || is_ctrl;
                        let had_shift = shift_down || is_shift;
                        let had_alt = alt_down || is_alt;
                        let had_win = win_down || is_win;
                        let mod_count = (had_ctrl as u8) + (had_shift as u8) + (had_alt as u8) + (had_win as u8);
                        if mod_count >= 1 {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(
                                had_ctrl,
                                had_shift,
                                had_alt,
                                had_win,
                                0,
                            );
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                            st.recording_mode = RecordingMode::None;
                            recorded_new_shortcut = true;
                        }
                    }
                }
            }
        });

        if recorded_new_shortcut {
            STATE.with(|s| {
                if let Ok(st) = s.try_borrow() {
                    update_tray_tooltip(st.hwnd, st.enabled, st.method);
                }
            });
            return 1;
        }

        // --- Active Toggle Shortcut Trigger Evaluation ---
        let mut shortcut_triggered = false;
        STATE.with(|s| {
            let Ok(mut st) = s.try_borrow_mut() else { return; };
            if is_keydown {
                if !is_mod && (ctrl_down || shift_down || alt_down || win_down) {
                    st.other_key_during_chord = true;
                    st.chord_armed = false;
                }

                if !st.shortcut.is_disabled() {
                    if st.shortcut.key_code != 0 {
                        if st.shortcut.key_code == vk
                            && st.shortcut.ctrl == ctrl_down
                            && st.shortcut.shift == shift_down
                            && st.shortcut.alt == alt_down
                            && st.shortcut.win == win_down
                        {
                            shortcut_triggered = true;
                        }
                    } else if is_mod && !st.other_key_during_chord {
                        // Arm modifier-only chord when exact required modifiers are down
                        if st.shortcut.ctrl == ctrl_down
                            && st.shortcut.shift == shift_down
                            && st.shortcut.alt == alt_down
                            && st.shortcut.win == win_down
                        {
                            st.chord_armed = true;
                        } else {
                            // Extra undesired modifier pressed
                            let extra_mod = (ctrl_down && !st.shortcut.ctrl)
                                || (shift_down && !st.shortcut.shift)
                                || (alt_down && !st.shortcut.alt)
                                || (win_down && !st.shortcut.win);
                            if extra_mod {
                                st.chord_armed = false;
                            }
                        }
                    }
                }
            } else if is_keyup {
                if !st.shortcut.is_disabled() && st.shortcut.key_code == 0 {
                    if is_mod && st.chord_armed && !st.other_key_during_chord {
                        shortcut_triggered = true;
                        st.chord_armed = false;
                    }
                }
                if !ctrl_down && !shift_down && !alt_down && !win_down {
                    st.other_key_during_chord = false;
                    st.chord_armed = false;
                }
            }
        });

        if shortcut_triggered {
            STATE.with(|s| {
                if let Ok(mut st) = s.try_borrow_mut() {
                    st.enabled = !st.enabled;
                    if st.per_app && !st.current_app.is_empty() {
                        let cur_app = st.current_app.clone();
                        let enabled = st.enabled;
                        st.app_states.insert(cur_app, enabled);
                    }
                    st.engine.reset();
                    update_tray_tooltip(st.hwnd, st.enabled, st.method);
                }
            });
            return 1;
        }

        if is_mod || !is_keydown {
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        // --- Flip Word Shortcut (Default: Ctrl + Shift + Z, customizable) ---
        // Only intercept if engine currently has an active word in buffer (`has_active_word()`),
        // so normal Redo (`Ctrl+Shift+Z`) in editors works 100% undisturbed when not mid-word!
        let flipped = STATE.with(|s| {
            let Ok(mut st) = s.try_borrow_mut() else { return false; };
            if st.enabled
                && !st.flip_shortcut.is_disabled()
                && st.flip_shortcut.key_code != 0
                && st.flip_shortcut.key_code == vk
                && st.flip_shortcut.ctrl == ctrl_down
                && st.flip_shortcut.shift == shift_down
                && st.flip_shortcut.alt == alt_down
                && st.flip_shortcut.win == win_down
                && st.engine.has_active_word()
            {
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

        if ctrl_down || alt_down || win_down {
            STATE.with(|s| {
                if let Ok(mut st) = s.try_borrow_mut() {
                    st.engine.reset();
                }
            });
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        let consumed = STATE.with(|s| {
            let Ok(mut st) = s.try_borrow_mut() else { return false; };
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
                STATE.with(|s| {
                    if let Ok(mut st) = s.try_borrow_mut() {
                        st.engine.reset();
                    }
                });
            }
        }
        CallNextHookEx(null_mut(), code, wparam, lparam)
    }

    unsafe fn get_tray_icon(enabled: bool) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
        let hinstance = GetModuleHandleW(null_mut());
        // Resource ID 2: 'V' badge icon, Resource ID 3: 'E' badge icon, Resource ID 1: default icon
        let icon_id = if enabled { 2 } else { 3 };
        let icon = LoadIconW(hinstance, icon_id as *const u16);
        if !icon.is_null() {
            icon
        } else {
            let fallback_icon = LoadIconW(hinstance, 1 as *const u16);
            if !fallback_icon.is_null() {
                fallback_icon
            } else {
                LoadIconW(null_mut(), IDI_APPLICATION)
            }
        }
    }

    unsafe fn add_tray_icon(hwnd: HWND) {
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = get_tray_icon(true);

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
        nid.uFlags = NIF_TIP | NIF_ICON;
        nid.hIcon = get_tray_icon(enabled);

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
        let Some((
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
            per_app,
            habit_en,
            habit_threshold,
            case_anchor,
            shortcut,
            flip_shortcut,
        )) = STATE.with(|s| {
            s.try_borrow().ok().map(|st| (
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
                st.per_app,
                st.engine.habit_learning_enabled(),
                st.engine.habit_promote_threshold(),
                st.engine.initial_case_anchor(),
                st.shortcut.clone(),
                st.flip_shortcut.clone(),
            ))
        }) else {
            DestroyMenu(menu);
            return;
        };

        // 1. Toggle Vietnamese
        let toggle_label = to_wstring("Bật gõ Tiếng Việt");
        AppendMenuW(
            menu,
            MF_STRING | if enabled { MF_CHECKED } else { MF_UNCHECKED },
            ID_TRAY_TOGGLE,
            toggle_label.as_ptr(),
        );

        let per_app_label = to_wstring("Nhớ theo từng ứng dụng (Per-App)");
        AppendMenuW(
            menu,
            MF_STRING | if per_app { MF_CHECKED } else { MF_UNCHECKED },
            ID_TOGGLE_PER_APP,
            per_app_label.as_ptr(),
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

        // 4b. Phím tắt Đảo Từ (Flip Word) Submenu
        let flip_sub = CreatePopupMenu();
        let cur_flip_label = format!("Hiện tại: [{}]", flip_shortcut.display_name);
        AppendMenuW(
            flip_sub,
            MF_STRING | MF_GRAYED,
            0,
            to_wstring(&cur_flip_label).as_ptr(),
        );
        AppendMenuW(flip_sub, MF_SEPARATOR, 0, null_mut());

        AppendMenuW(
            flip_sub,
            MF_STRING,
            ID_FLIP_RECORD,
            to_wstring("⌨️ Bấm để đặt phím tắt Đảo Từ mới...").as_ptr(),
        );
        AppendMenuW(flip_sub, MF_SEPARATOR, 0, null_mut());

        let is_flip_csz = flip_shortcut.ctrl && flip_shortcut.shift && !flip_shortcut.alt && !flip_shortcut.win && flip_shortcut.key_code == 'Z' as u16;
        let is_flip_alt_x = !flip_shortcut.ctrl && !flip_shortcut.shift && flip_shortcut.alt && !flip_shortcut.win && flip_shortcut.key_code == 'X' as u16;
        let is_flip_caz = flip_shortcut.ctrl && !flip_shortcut.shift && flip_shortcut.alt && !flip_shortcut.win && flip_shortcut.key_code == 'Z' as u16;

        AppendMenuW(
            flip_sub,
            MF_STRING | if is_flip_csz { MF_CHECKED } else { 0 },
            ID_FLIP_CTRL_SHIFT_Z,
            to_wstring("Ctrl + Shift + Z (Chỉ khi đang gõ từ)").as_ptr(),
        );
        AppendMenuW(
            flip_sub,
            MF_STRING | if is_flip_alt_x { MF_CHECKED } else { 0 },
            ID_FLIP_ALT_X,
            to_wstring("Alt + X").as_ptr(),
        );
        AppendMenuW(
            flip_sub,
            MF_STRING | if is_flip_caz { MF_CHECKED } else { 0 },
            ID_FLIP_CTRL_ALT_Z,
            to_wstring("Ctrl + Alt + Z").as_ptr(),
        );
        AppendMenuW(
            flip_sub,
            MF_STRING | if flip_shortcut.is_disabled() { MF_CHECKED } else { 0 },
            ID_FLIP_DISABLED,
            to_wstring("Tắt phím tắt Đảo Từ").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_POPUP,
            flip_sub as usize,
            to_wstring("Phím tắt Đảo Từ (Flip Word)").as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // 5. Tính năng thông minh Checkboxes
        let (quick_telex, quick_start, quick_end) = STATE.with(|s| {
            s.try_borrow()
                .ok()
                .map(|st| {
                    (
                        st.engine.quick_telex(),
                        st.engine.quick_start_consonant(),
                        st.engine.quick_end_consonant(),
                    )
                })
                .unwrap_or((false, false, false))
        });

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
        AppendMenuW(
            menu,
            MF_STRING | if case_anchor { MF_CHECKED } else { 0 },
            ID_TOGGLE_CASE_ANCHOR,
            to_wstring("Đồng nhất chữ hoa theo chữ cái đầu (Được, được)").as_ptr(),
        );

        // Tùy chọn Tốc ký
        let quick_sub = CreatePopupMenu();
        AppendMenuW(
            quick_sub,
            MF_STRING | if quick_telex { MF_CHECKED } else { 0 },
            ID_TOGGLE_QUICK_TELEX,
            to_wstring("Quick Telex (cc->ch, kk->kh, tt->th...)").as_ptr(),
        );
        AppendMenuW(
            quick_sub,
            MF_STRING | if quick_start { MF_CHECKED } else { 0 },
            ID_TOGGLE_QUICK_START,
            to_wstring("Phụ âm đầu nhanh (f->ph, j->gi, w->qu)").as_ptr(),
        );
        AppendMenuW(
            quick_sub,
            MF_STRING | if quick_end { MF_CHECKED } else { 0 },
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

        // 6b. Tự Động Học Thói Quen (Adaptive Habit Learning)
        let habit_sub = CreatePopupMenu();
        AppendMenuW(
            habit_sub,
            MF_STRING | if habit_en { MF_CHECKED } else { 0 },
            ID_TOGGLE_HABIT_LEARNING,
            to_wstring("Bật Tự Động Học Sửa Từ").as_ptr(),
        );
        AppendMenuW(habit_sub, MF_SEPARATOR, 0, null_mut());
        let cur_th_label = format!("Ngưỡng học hiện tại: [{} lần sửa/xoá]", habit_threshold);
        AppendMenuW(habit_sub, MF_STRING | MF_GRAYED, 0, to_wstring(&cur_th_label).as_ptr());
        AppendMenuW(
            habit_sub,
            MF_STRING | if habit_threshold == 2 { MF_CHECKED } else { 0 },
            ID_HABIT_THRESHOLD_2,
            to_wstring("2 lần (Nhanh nhất)").as_ptr(),
        );
        AppendMenuW(
            habit_sub,
            MF_STRING | if habit_threshold == 3 { MF_CHECKED } else { 0 },
            ID_HABIT_THRESHOLD_3,
            to_wstring("3 lần").as_ptr(),
        );
        AppendMenuW(
            habit_sub,
            MF_STRING | if habit_threshold == 4 { MF_CHECKED } else { 0 },
            ID_HABIT_THRESHOLD_4,
            to_wstring("4 lần (Khuyên dùng)").as_ptr(),
        );
        AppendMenuW(
            habit_sub,
            MF_STRING | if habit_threshold == 5 { MF_CHECKED } else { 0 },
            ID_HABIT_THRESHOLD_5,
            to_wstring("5 lần (An toàn)").as_ptr(),
        );
        AppendMenuW(
            habit_sub,
            MF_STRING | if habit_threshold == 6 { MF_CHECKED } else { 0 },
            ID_HABIT_THRESHOLD_6,
            to_wstring("6 lần").as_ptr(),
        );
        AppendMenuW(
            habit_sub,
            MF_STRING | if habit_threshold == 8 { MF_CHECKED } else { 0 },
            ID_HABIT_THRESHOLD_8,
            to_wstring("8 lần").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_POPUP,
            habit_sub as usize,
            to_wstring("Học thói quen sửa từ (Adaptive Learning)...").as_ptr(),
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

        // 8. Thông tin & Cập nhật
        AppendMenuW(
            menu,
            MF_STRING,
            ID_CHECK_UPDATE,
            to_wstring("Kiểm tra bản cập nhật mới...").as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING,
            ID_ABOUT,
            to_wstring("Thông tin VnCharKey (About)...").as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // 9. Exit
        AppendMenuW(
            menu,
            MF_STRING,
            ID_TRAY_EXIT,
            to_wstring("Thoát VnCharKey").as_ptr(),
        );

        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);

        // Microsoft Knowledge Base (Q139526): To ensure TrackPopupMenu dismisses correctly
        // when the user clicks on another window or app outside the menu, the calling window
        // must be set as foreground window BEFORE calling TrackPopupMenu, and a WM_NULL message
        // must be posted to force an asynchronous task switch.
        SetForegroundWindow(hwnd);

        let cmd = TrackPopupMenu(
            menu,
            TPM_BOTTOMALIGN | TPM_LEFTALIGN | TPM_RETURNCMD | TPM_NONOTIFY,
            pt.x,
            pt.y,
            0,
            hwnd,
            null_mut(),
        );
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);

        if cmd != 0 {
            SendMessageW(hwnd, WM_COMMAND, cmd as usize, 0);
        }
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
                        if let Ok(mut st) = s.try_borrow_mut() {
                            st.enabled = !st.enabled;
                            if st.per_app && !st.current_app.is_empty() {
                                let cur_app = st.current_app.clone();
                                let enabled = st.enabled;
                                st.app_states.insert(cur_app, enabled);
                            }
                            st.engine.reset();
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        }
                    });
                }
                0
            }
            WM_COMMAND => {
                let id = (wparam & 0xFFFF) as usize;
                let mut record_dialog: Option<(&str, &str)> = None;
                STATE.with(|s| {
                    let Ok(mut st) = s.try_borrow_mut() else { return; };
                    match id {
                        ID_TRAY_TOGGLE => {
                            st.enabled = !st.enabled;
                            if st.per_app && !st.current_app.is_empty() {
                                let cur_app = st.current_app.clone();
                                let enabled = st.enabled;
                                st.app_states.insert(cur_app, enabled);
                            }
                            st.engine.reset();
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                        }
                        ID_TOGGLE_PER_APP => {
                            st.per_app = !st.per_app;
                            save_app_config(&st);
                        }
                        ID_METHOD_AUTO => {
                            st.method = InputMethod::Auto;
                            st.engine.set_method(InputMethod::Auto);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                            save_app_config(&st);
                        }
                        ID_METHOD_TELEX => {
                            st.method = InputMethod::Telex;
                            st.engine.set_method(InputMethod::Telex);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                            save_app_config(&st);
                        }
                        ID_METHOD_VNI => {
                            st.method = InputMethod::Vni;
                            st.engine.set_method(InputMethod::Vni);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                            save_app_config(&st);
                        }
                        ID_METHOD_SIMPLETELEX => {
                            st.method = InputMethod::SimpleTelex;
                            st.engine.set_method(InputMethod::SimpleTelex);
                            update_tray_tooltip(hwnd, st.enabled, st.method);
                            save_app_config(&st);
                        }
                        ID_TONE_MODERN => {
                            st.tone_style = ToneStyle::Modern;
                            st.engine.set_tone_style(ToneStyle::Modern);
                            save_app_config(&st);
                        }
                        ID_TONE_CLASSIC => {
                            st.tone_style = ToneStyle::Classic;
                            st.engine.set_tone_style(ToneStyle::Classic);
                            save_app_config(&st);
                        }
                        ID_CHARSET_NFC => {
                            st.charset = Charset::Unicode;
                            st.engine.set_charset(Charset::Unicode);
                            save_app_config(&st);
                        }
                        ID_CHARSET_NFD => {
                            st.charset = Charset::UnicodeComposite;
                            st.engine.set_charset(Charset::UnicodeComposite);
                            save_app_config(&st);
                        }
                        ID_SHORTCUT_CTRL_SHIFT => {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(true, true, false, false, 0);
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_SHORTCUT_ALT_Z => {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(false, false, true, false, 'Z' as u16);
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_SHORTCUT_CTRL_SPACE => {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(true, false, false, false, VK_SPACE);
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_SHORTCUT_WIN_SPACE => {
                            st.shortcut = CustomShortcut::from_modifiers_and_key(false, false, false, true, VK_SPACE);
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_SHORTCUT_DISABLED => {
                            st.shortcut = CustomShortcut::disabled();
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_SHORTCUT_RECORD => {
                            st.recording_mode = RecordingMode::ToggleShortcut;
                            st.chord_armed = false;
                            st.other_key_during_chord = false;
                            record_dialog = Some((
                                "Đang ở chế độ Đặt Phím Tắt Chuyển Việt/Anh!\n\n\
                                 1. Bấm tổ hợp phím bạn muốn đặt (ví dụ: Ctrl+Alt+V, Win+Shift+Space, Alt+Shift,...).\n\
                                 2. Hoặc bấm phím Esc để Hủy bỏ.\n\n\
                                 Bấm OK rồi bấm tổ hợp phím ngay trên bàn phím của bạn.",
                                "Cài đặt Phím tắt Tự do - VnCharKey",
                            ));
                        }
                        ID_FLIP_CTRL_SHIFT_Z => {
                            st.flip_shortcut = CustomShortcut::from_modifiers_and_key(true, true, false, false, 'Z' as u16);
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_FLIP_ALT_X => {
                            st.flip_shortcut = CustomShortcut::from_modifiers_and_key(false, false, true, false, 'X' as u16);
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_FLIP_CTRL_ALT_Z => {
                            st.flip_shortcut = CustomShortcut::from_modifiers_and_key(true, false, true, false, 'Z' as u16);
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_FLIP_DISABLED => {
                            st.flip_shortcut = CustomShortcut::disabled();
                            save_shortcut_config(&st.shortcut, &st.flip_shortcut);
                        }
                        ID_FLIP_RECORD => {
                            st.recording_mode = RecordingMode::FlipShortcut;
                            st.chord_armed = false;
                            st.other_key_during_chord = false;
                            record_dialog = Some((
                                "Đang ở chế độ Đặt Phím Tắt Đảo Từ (Flip Word)!\n\n\
                                 1. Bấm tổ hợp phím kèm chữ/số bạn muốn đặt (ví dụ: Alt+X, Ctrl+Alt+Z, Ctrl+Shift+F,...).\n\
                                 2. Hoặc bấm phím Esc để Hủy bỏ.\n\n\
                                 Bấm OK rồi bấm tổ hợp phím ngay trên bàn phím của bạn.",
                                "Cài đặt Phím tắt Đảo Từ - VnCharKey",
                            ));
                        }
                        ID_TOGGLE_SPELLCHECK => {
                            let val = !st.spell_check;
                            st.spell_check = val;
                            st.engine.set_spell_check(val);
                            save_app_config(&st);
                        }
                        ID_TOGGLE_AUTORESTORE => {
                            let val = !st.auto_restore;
                            st.auto_restore = val;
                            st.engine.set_auto_restore_english(val);
                            save_app_config(&st);
                        }
                        ID_TOGGLE_ESCRESTORE => {
                            st.esc_restore = !st.esc_restore;
                            save_app_config(&st);
                        }
                        ID_TOGGLE_BRACKET => {
                            let val = !st.bracket_transform;
                            st.bracket_transform = val;
                            st.engine.set_bracket_transform(val);
                            save_app_config(&st);
                        }
                        ID_TOGGLE_CASE_ANCHOR => {
                            let val = !st.engine.initial_case_anchor();
                            st.engine.set_initial_case_anchor(val);
                            save_app_config(&st);
                        }
                        ID_TOGGLE_QUICK_TELEX => {
                            let val = !st.engine.quick_telex();
                            st.engine.set_quick_telex(val);
                            save_app_config(&st);
                        }
                        ID_TOGGLE_QUICK_START => {
                            let val = !st.engine.quick_start_consonant();
                            st.engine.set_quick_start_consonant(val);
                            save_app_config(&st);
                        }
                        ID_TOGGLE_QUICK_END => {
                            let val = !st.engine.quick_end_consonant();
                            st.engine.set_quick_end_consonant(val);
                            save_app_config(&st);
                        }
                        ID_TOGGLE_MACRO => {
                            let val = !st.macro_enabled;
                            st.macro_enabled = val;
                            st.engine.set_enable_macro(val);
                            save_app_config(&st);
                        }
                        ID_OPEN_MACROS => {
                            open_file_in_notepad(macros_path());
                        }
                        ID_TOGGLE_DICT => {
                            let val = !st.dict_enabled;
                            st.dict_enabled = val;
                            st.engine.set_enable_dictionary(val);
                            save_app_config(&st);
                        }
                        ID_OPEN_DICT => {
                            open_file_in_notepad(dict_path());
                        }
                        ID_RELOAD_FILES => {
                            reload_files(&mut st.engine);
                        }
                        ID_TOGGLE_HABIT_LEARNING => {
                            let val = !st.engine.habit_learning_enabled();
                            st.engine.set_habit_learning_enabled(val);
                            save_app_config(&st);
                        }
                        ID_HABIT_THRESHOLD_2 => {
                            st.engine.set_habit_promote_threshold(2);
                            save_app_config(&st);
                        }
                        ID_HABIT_THRESHOLD_3 => {
                            st.engine.set_habit_promote_threshold(3);
                            save_app_config(&st);
                        }
                        ID_HABIT_THRESHOLD_4 => {
                            st.engine.set_habit_promote_threshold(4);
                            save_app_config(&st);
                        }
                        ID_HABIT_THRESHOLD_5 => {
                            st.engine.set_habit_promote_threshold(5);
                            save_app_config(&st);
                        }
                        ID_HABIT_THRESHOLD_6 => {
                            st.engine.set_habit_promote_threshold(6);
                            save_app_config(&st);
                        }
                        ID_HABIT_THRESHOLD_8 => {
                            st.engine.set_habit_promote_threshold(8);
                            save_app_config(&st);
                        }
                        ID_TOGGLE_AUTORUN => {
                            let current = is_autorun_enabled();
                            set_autorun_enabled(!current);
                        }
                        ID_ABOUT => {
                            show_about_dialog(hwnd);
                        }
                        ID_CHECK_UPDATE => {
                            check_for_updates(hwnd, false);
                        }
                        ID_TRAY_EXIT => {
                            remove_tray_icon(hwnd);
                            PostQuitMessage(0);
                        }
                        _ => {}
                    }
                });
                if let Some((body, title)) = record_dialog {
                    MessageBoxW(
                        hwnd,
                        to_wstring(body).as_ptr(),
                        to_wstring(title).as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
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

    unsafe fn get_process_name_for_hwnd(hwnd: HWND) -> Option<String> {
        let mut pid: u32 = 0;
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }

        let h_proc = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if h_proc.is_null() {
            return None;
        }

        let mut buf = [0u16; 260];
        let len = GetModuleBaseNameW(h_proc, null_mut(), buf.as_mut_ptr(), buf.len() as u32);
        windows_sys::Win32::Foundation::CloseHandle(h_proc);

        if len > 0 {
            let name = String::from_utf16_lossy(&buf[..len as usize]).to_lowercase();
            Some(name)
        } else {
            None
        }
    }

    unsafe extern "system" fn win_event_proc(
        _h_win_event_hook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        id_object: i32,
        id_child: i32,
        _id_event_thread: u32,
        _dwms_event_time: u32,
    ) {
        if event == EVENT_SYSTEM_FOREGROUND && id_object == OBJID_WINDOW && id_child == 0 {
            if hwnd.is_null() || IsWindow(hwnd) == 0 {
                return;
            }

            if let Some(proc_name) = get_process_name_for_hwnd(hwnd) {
                // Ignore our own process
                if proc_name.contains("vncharkey") {
                    return;
                }

                STATE.with(|s| {
                    if let Ok(mut st) = s.try_borrow_mut() {
                        if !st.per_app {
                            st.current_app = proc_name;
                            return;
                        }

                        // Save current app's state if we are leaving it
                        if !st.current_app.is_empty() {
                            let cur_app = st.current_app.clone();
                            let enabled = st.enabled;
                            st.app_states.insert(cur_app, enabled);
                        }

                        st.current_app = proc_name.clone();

                        // Restore state for newly focused app (default to true)
                        let next_state = *st.app_states.get(&proc_name).unwrap_or(&true);
                        if st.enabled != next_state {
                            st.enabled = next_state;
                            st.engine.reset();
                            let h = st.hwnd;
                            let m = st.method;
                            update_tray_tooltip(h, next_state, m);
                        }
                    }
                });
            }
        }
    }

    pub unsafe fn run() {
        ensure_config_files();
        let (initial_shortcut, initial_flip_shortcut) = load_shortcut_config();
        STATE.with(|s| {
            if let Ok(mut st) = s.try_borrow_mut() {
                st.shortcut = initial_shortcut;
                st.flip_shortcut = initial_flip_shortcut;
                load_full_config(&mut st);
                reload_files(&mut st.engine);
            }
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
            HWND_MESSAGE,
            null_mut(),
            hinstance,
            null_mut(),
        );

        STATE.with(|s| {
            if let Ok(mut st) = s.try_borrow_mut() {
                st.hwnd = hwnd;
            }
        });
        add_tray_icon(hwnd);

        let kbd_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), hinstance, 0);
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), hinstance, 0);

        // WinEventHook for tracking window focus changes (Per-App Input State)
        let event_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            null_mut(),
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if !event_hook.is_null() {
            UnhookWinEvent(event_hook);
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
