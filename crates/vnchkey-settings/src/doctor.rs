//! System Diagnostics (`--doctor`) and `/proc/<PID>/cmdline` Electron Wayland Advisor.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    pub category: String,
    pub title: String,
    pub status: CheckStatus,
    pub detail: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ElectronProcReport {
    pub pid: u32,
    pub exe_name: String,
    pub has_wayland_ime_flag: bool,
    pub cmdline_preview: String,
    pub fix_hint: String,
}

#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub session_type: String,
    pub desktop_env: String,
    pub items: Vec<DiagnosticItem>,
    pub electron_procs: Vec<ElectronProcReport>,
}

impl DoctorReport {
    pub fn run() -> Self {
        let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| {
            if env::var("WAYLAND_DISPLAY").is_ok() {
                "wayland".to_string()
            } else if env::var("DISPLAY").is_ok() {
                "x11".to_string()
            } else {
                "unknown".to_string()
            }
        });
        let desktop_env = env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| "unknown".to_string());

        let is_wayland = session_type.eq_ignore_ascii_case("wayland")
            || env::var("WAYLAND_DISPLAY").is_ok();

        let mut items = Vec::new();

        // 1. Session & Compositor check
        items.push(DiagnosticItem {
            category: "Môi trường".to_string(),
            title: "Giao thức hiển thị (Session Type)".to_string(),
            status: CheckStatus::Pass,
            detail: format!("{} (Desktop: {})", session_type, desktop_env),
            hint: None,
        });

        // 2. Environment Variables (XMODIFIERS, GTK_IM_MODULE, QT_IM_MODULE)
        let xmodifiers = env::var("XMODIFIERS").unwrap_or_default();
        if xmodifiers.contains("fcitx") {
            items.push(DiagnosticItem {
                category: "Biến môi trường".to_string(),
                title: "XMODIFIERS".to_string(),
                status: CheckStatus::Pass,
                detail: format!("XMODIFIERS={}", xmodifiers),
                hint: None,
            });
        } else {
            items.push(DiagnosticItem {
                category: "Biến môi trường".to_string(),
                title: "XMODIFIERS".to_string(),
                status: CheckStatus::Warn,
                detail: if xmodifiers.is_empty() {
                    "Chưa thiết lập (Trống)".to_string()
                } else {
                    format!("Đang là '{}' (chưa trỏ về @im=fcitx)", xmodifiers)
                },
                hint: Some("Thêm `export XMODIFIERS=@im=fcitx` vào ~/.profile hoặc ~/.pam_environment".to_string()),
            });
        }

        let gtk_im = env::var("GTK_IM_MODULE").unwrap_or_default();
        if gtk_im.contains("fcitx") || is_wayland {
            items.push(DiagnosticItem {
                category: "Biến môi trường".to_string(),
                title: "GTK_IM_MODULE".to_string(),
                status: CheckStatus::Pass,
                detail: if gtk_im.is_empty() && is_wayland {
                    "Trống (Chuẩn trên Wayland text-input-v3)".to_string()
                } else {
                    format!("GTK_IM_MODULE={}", gtk_im)
                },
                hint: None,
            });
        } else {
            items.push(DiagnosticItem {
                category: "Biến môi trường".to_string(),
                title: "GTK_IM_MODULE".to_string(),
                status: CheckStatus::Warn,
                detail: format!("GTK_IM_MODULE='{}'", gtk_im),
                hint: Some("Trên X11 nên đặt `export GTK_IM_MODULE=fcitx`.".to_string()),
            });
        }

        let qt_im = env::var("QT_IM_MODULE").unwrap_or_default();
        if qt_im.contains("fcitx") {
            items.push(DiagnosticItem {
                category: "Biến môi trường".to_string(),
                title: "QT_IM_MODULE".to_string(),
                status: CheckStatus::Pass,
                detail: format!("QT_IM_MODULE={}", qt_im),
                hint: None,
            });
        } else {
            items.push(DiagnosticItem {
                category: "Biến môi trường".to_string(),
                title: "QT_IM_MODULE".to_string(),
                status: CheckStatus::Warn,
                detail: if qt_im.is_empty() {
                    "Chưa thiết lập (Trống)".to_string()
                } else {
                    format!("QT_IM_MODULE='{}'", qt_im)
                },
                hint: Some("Khuyến nghị thêm `export QT_IM_MODULE=fcitx` cho các ứng dụng Qt.".to_string()),
            });
        }

        // 3. Fcitx5 Daemon running status
        let fcitx_running = is_process_running("fcitx5");
        if fcitx_running {
            items.push(DiagnosticItem {
                category: "Dịch vụ Fcitx5".to_string(),
                title: "Tiến trình fcitx5 daemon".to_string(),
                status: CheckStatus::Pass,
                detail: "Đang hoạt động (Running)".to_string(),
                hint: None,
            });
        } else {
            items.push(DiagnosticItem {
                category: "Dịch vụ Fcitx5".to_string(),
                title: "Tiến trình fcitx5 daemon".to_string(),
                status: CheckStatus::Fail,
                detail: "Không tìm thấy tiến trình fcitx5 đang chạy".to_string(),
                hint: Some("Chạy lệnh `fcitx5 -d` hoặc thêm `exec fcitx5 -d` vào cấu hình khởi động.".to_string()),
            });
        }

        // 4. Addon & Rust FFI shared library installation check
        let addon_paths = [
            "/usr/lib/x86_64-linux-gnu/fcitx5/fcitx5-vnchkey-addon.so",
            "/usr/lib/fcitx5/fcitx5-vnchkey-addon.so",
            "/usr/lib64/fcitx5/fcitx5-vnchkey-addon.so",
            "/usr/local/lib/fcitx5/fcitx5-vnchkey-addon.so",
            "/usr/lib/aarch64-linux-gnu/fcitx5/fcitx5-vnchkey-addon.so",
        ];
        let found_addon = addon_paths.iter().find(|p| Path::new(p).exists());
        if let Some(p) = found_addon {
            items.push(DiagnosticItem {
                category: "Thư viện hệ thống".to_string(),
                title: "Fcitx5 Addon (fcitx5-vnchkey-addon.so)".to_string(),
                status: CheckStatus::Pass,
                detail: format!("Đã cài đặt tại {}", p),
                hint: None,
            });
        } else {
            items.push(DiagnosticItem {
                category: "Thư viện hệ thống".to_string(),
                title: "Fcitx5 Addon (fcitx5-vnchkey-addon.so)".to_string(),
                status: CheckStatus::Fail,
                detail: "Chưa tìm thấy fcitx5-vnchkey-addon.so trong /usr/lib".to_string(),
                hint: Some("Chạy `sudo ./install.sh` hoặc cài đặt gói `.deb`.".to_string()),
            });
        }

        let ffi_paths = [
            "/usr/lib/libfcitx5_vnchkey_ffi.so",
            "/usr/lib/x86_64-linux-gnu/libfcitx5_vnchkey_ffi.so",
            "/usr/lib64/libfcitx5_vnchkey_ffi.so",
            "/usr/local/lib/libfcitx5_vnchkey_ffi.so",
            "/usr/lib/aarch64-linux-gnu/libfcitx5_vnchkey_ffi.so",
        ];
        let found_ffi = ffi_paths.iter().find(|p| Path::new(p).exists());
        if let Some(p) = found_ffi {
            items.push(DiagnosticItem {
                category: "Thư viện hệ thống".to_string(),
                title: "Lõi Rust FFI (libfcitx5_vnchkey_ffi.so)".to_string(),
                status: CheckStatus::Pass,
                detail: format!("Đã cài đặt tại {}", p),
                hint: None,
            });
        } else {
            items.push(DiagnosticItem {
                category: "Thư viện hệ thống".to_string(),
                title: "Lõi Rust FFI (libfcitx5_vnchkey_ffi.so)".to_string(),
                status: CheckStatus::Fail,
                detail: "Chưa tìm thấy libfcitx5_vnchkey_ffi.so trong /usr/lib".to_string(),
                hint: Some("Chạy `cargo build --release` rồi copy `libfcitx5_vnchkey_ffi.so` vào `/usr/lib/`.".to_string()),
            });
        }

        // 5. Scan /proc/<PID>/cmdline for Electron / Chromium apps on Wayland
        let electron_procs = scan_electron_processes(is_wayland);

        Self {
            session_type,
            desktop_env,
            items,
            electron_procs,
        }
    }

    pub fn print_cli(&self) {
        println!("==================================================================");
        println!(" 🩺 Fcitx5 vnchKey — Báo Cáo Chẩn Đoán Hệ Thống (--doctor)");
        println!("==================================================================");
        for item in &self.items {
            let badge = match item.status {
                CheckStatus::Pass => "[✔ OK]  ",
                CheckStatus::Warn => "[⚠ WARN]",
                CheckStatus::Fail => "[✖ FAIL]",
            };
            println!("{} {} — {}: {}", badge, item.category, item.title, item.detail);
            if let Some(hint) = &item.hint {
                println!("         💡 Gợi ý: {}", hint);
            }
        }

        println!("\n------------------------------------------------------------------");
        println!(" 🔍 Cố vấn Tiến trình Electron / Chromium trên Wayland (/proc/PID)");
        println!("------------------------------------------------------------------");
        if self.electron_procs.is_empty() {
            println!("Không phát hiện ứng dụng Electron/Chromium nào đang chạy.");
        } else {
            for ep in &self.electron_procs {
                if ep.has_wayland_ime_flag {
                    println!(
                        "[✔ OK]   PID {:<6} ({:<14}) : Đã có cờ hỗ trợ Wayland IME",
                        ep.pid, ep.exe_name
                    );
                } else {
                    println!(
                        "[⚠ WARN] PID {:<6} ({:<14}) : Thiếu cờ --enable-wayland-ime!",
                        ep.pid, ep.exe_name
                    );
                    println!("         💡 Cách khắc phục: {}", ep.fix_hint);
                }
            }
        }
        println!("==================================================================");
    }
}

fn is_process_running(target_comm: &str) -> bool {
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let s = fname.to_string_lossy();
            if s.chars().all(|c| c.is_ascii_digit()) {
                let comm_path = entry.path().join("comm");
                if let Ok(comm) = fs::read_to_string(comm_path) {
                    if comm.trim() == target_comm {
                        return true;
                    }
                }
            }
        }
    }
    // Fallback to pgrep
    Command::new("pgrep")
        .arg("-x")
        .arg(target_comm)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn scan_electron_processes(is_wayland: bool) -> Vec<ElectronProcReport> {
    let mut reports = Vec::new();
    let mut seen_apps = BTreeSet::new();

    let known_electron_bins = [
        "code",
        "code-oss",
        "codium",
        "cursor",
        "windsurf",
        "antigravity",
        "zed",
        "discord",
        "vesktop",
        "slack",
        "obsidian",
        "notion-app",
        "element-desktop",
        "signal-desktop",
        "teams-for-linux",
        "spotify",
        "postman",
        "insomnia",
        "electron",
        "chrome",
        "chromium",
        "brave",
        "msedge",
        "vivaldi-bin",
    ];

    let Ok(entries) = fs::read_dir("/proc") else {
        return reports;
    };

    for entry in entries.flatten() {
        let fname = entry.file_name();
        let pid_str = fname.to_string_lossy();
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };

        let cmdline_path = entry.path().join("cmdline");
        let Ok(raw_bytes) = fs::read(&cmdline_path) else {
            continue;
        };
        if raw_bytes.is_empty() {
            continue;
        }

        let args: Vec<String> = raw_bytes
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .collect();
        if args.is_empty() {
            continue;
        }

        // Skip renderer/zygote/gpu/utility child processes; only inspect main application processes
        let full_cmd = args.join(" ");
        if full_cmd.contains("--type=") {
            continue;
        }

        let first_arg = &args[0];
        let base_exe = Path::new(first_arg)
            .file_name()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let matched_app = known_electron_bins
            .iter()
            .find(|&&bin| base_exe == bin || base_exe.starts_with(&format!("{}-", bin)));

        let Some(&app_label) = matched_app else {
            continue;
        };

        if seen_apps.contains(app_label) {
            continue;
        }
        seen_apps.insert(app_label.to_string());

        let has_wayland_ime = !is_wayland
            || full_cmd.contains("--enable-wayland-ime")
            || full_cmd.contains("--ozone-platform=wayland")
            || full_cmd.contains("--ozone-platform-hint=auto")
            || full_cmd.contains("--ozone-platform-hint=wayland");

        let fix_hint = if app_label.contains("code") || app_label == "codium" {
            format!(
                "Chạy với cờ `{} --ozone-platform-hint=auto --enable-wayland-ime` hoặc thêm vào ~/.config/code-flags.conf",
                app_label
            )
        } else {
            format!(
                "Khởi động `{}` kèm cờ: `--enable-features=UseOzonePlatform --ozone-platform=wayland --enable-wayland-ime`",
                app_label
            )
        };

        reports.push(ElectronProcReport {
            pid,
            exe_name: app_label.to_string(),
            has_wayland_ime_flag: has_wayland_ime,
            cmdline_preview: if full_cmd.len() > 90 {
                format!("{}...", &full_cmd[..90])
            } else {
                full_cmd
            },
            fix_hint,
        });
    }

    reports
}

