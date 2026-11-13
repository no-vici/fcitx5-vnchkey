//! System application discovery for desktop entries and running user processes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
extern "C" {
    fn getuid() -> u32;
}

#[derive(Debug, Clone)]
pub struct SystemAppEntry {
    pub display_name: String,
    pub exec_name: String,
    pub icon_name: String,
    pub comment: String,
    pub glyph: &'static str,
}

impl SystemAppEntry {
    pub fn guess_glyph(name: &str, exec: &str) -> &'static str {
        let text = format!("{} {}", name, exec).to_lowercase();
        if text.contains("browser")
            || text.contains("chrome")
            || text.contains("firefox")
            || text.contains("edge")
            || text.contains("brave")
            || text.contains("opera")
            || text.contains("vivaldi")
            || text.contains("web")
            || text.contains("tor")
        {
            "🌐"
        } else if text.contains("terminal")
            || text.contains("console")
            || text.contains("alacritty")
            || text.contains("kitty")
            || text.contains("ghostty")
            || text.contains("rio")
            || text.contains("wezterm")
            || text.contains("foot")
            || text.contains("tmux")
            || text.contains("shell")
            || text.contains("prompt")
        {
            "💻"
        } else if text.contains("code")
            || text.contains("nvim")
            || text.contains("vim")
            || text.contains("edit")
            || text.contains("ide")
            || text.contains("studio")
            || text.contains("sublime")
            || text.contains("emacs")
            || text.contains("intellij")
            || text.contains("pycharm")
            || text.contains("clion")
            || text.contains("rustrover")
            || text.contains("antigravity")
        {
            "📝"
        } else if text.contains("discord")
            || text.contains("slack")
            || text.contains("telegram")
            || text.contains("chat")
            || text.contains("teams")
            || text.contains("signal")
            || text.contains("zalo")
            || text.contains("mail")
            || text.contains("thunderbird")
        {
            "💬"
        } else if text.contains("steam")
            || text.contains("game")
            || text.contains("play")
            || text.contains("lutris")
            || text.contains("heroic")
            || text.contains("minecraft")
            || text.contains("wine")
            || text.contains("proton")
        {
            "🎮"
        } else if text.contains("music")
            || text.contains("spotify")
            || text.contains("video")
            || text.contains("player")
            || text.contains("vlc")
            || text.contains("media")
            || text.contains("celluloid")
            || text.contains("mpv")
        {
            "🎵"
        } else if text.contains("setting")
            || text.contains("config")
            || text.contains("system")
            || text.contains("manage")
            || text.contains("tweak")
            || text.contains("fcitx")
        {
            "⚙️"
        } else {
            "📦"
        }
    }
}

pub fn scan_system_applications() -> Vec<SystemAppEntry> {
    let mut map: HashMap<String, SystemAppEntry> = HashMap::new();

    // 1. Scan standard XDG application directories
    let mut search_dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    ];

    if let Ok(home) = std::env::var("HOME") {
        search_dirs.push(PathBuf::from(&home).join(".local/share/applications"));
        search_dirs.push(PathBuf::from(&home).join(".local/share/flatpak/exports/share/applications"));
    }

    if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
        for d in xdg_dirs.split(':') {
            if !d.is_empty() {
                let p = PathBuf::from(d).join("applications");
                if !search_dirs.contains(&p) {
                    search_dirs.push(p);
                }
            }
        }
    }

    for dir in search_dirs {
        if !dir.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("NoDisplay=true") {
                        continue;
                    }

                    let mut name = String::new();
                    let mut exec = String::new();
                    let mut icon = String::new();
                    let mut comment = String::new();
                    let mut wm_class = String::new();
                    let mut is_app = false;

                    for line in content.lines() {
                        let line = line.trim();
                        if line == "[Desktop Entry]" {
                            is_app = true;
                            continue;
                        }
                        if line.starts_with('[') && line != "[Desktop Entry]" {
                            break; // Stop at next section
                        }

                        if let Some((k, v)) = line.split_once('=') {
                            let k = k.trim();
                            let v = v.trim();
                            match k {
                                "Type" => {
                                    if v != "Application" {
                                        is_app = false;
                                        break;
                                    }
                                }
                                "Name" if name.is_empty() => name = v.to_string(),
                                "Exec" if exec.is_empty() => exec = v.to_string(),
                                "Icon" if icon.is_empty() => icon = v.to_string(),
                                "Comment" if comment.is_empty() => comment = v.to_string(),
                                "StartupWMClass" if wm_class.is_empty() => wm_class = v.to_string(),
                                _ => {}
                            }
                        }
                    }

                    if !is_app || name.is_empty() || exec.is_empty() {
                        continue;
                    }

                    // Extract executable binary name
                    let exec_cmd = exec
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches('"')
                        .trim_matches('\'');
                    let raw_bin = Path::new(exec_cmd)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    let exec_name = if !wm_class.is_empty() {
                        wm_class.to_lowercase()
                    } else if !raw_bin.is_empty() {
                        raw_bin.to_lowercase()
                    } else {
                        continue;
                    };

                    let glyph = SystemAppEntry::guess_glyph(&name, &exec_name);

                    let entry = SystemAppEntry {
                        display_name: name,
                        exec_name: exec_name.clone(),
                        icon_name: icon,
                        comment,
                        glyph,
                    };

                    map.entry(exec_name).or_insert(entry);
                }
            }
        }
    }

    // 2. Discover running user processes from /proc
    let my_uid = unsafe { getuid() };
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let name_str = filename.to_string_lossy();
            if !name_str.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            let pid_dir = entry.path();
            let status_file = pid_dir.join("status");
            if let Ok(status_content) = fs::read_to_string(&status_file) {
                let mut uid_match = false;
                for line in status_content.lines() {
                    if line.starts_with("Uid:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 1 && parts[1].parse::<u32>().unwrap_or(u32::MAX) == my_uid {
                            uid_match = true;
                        }
                        break;
                    }
                }

                if !uid_match {
                    continue;
                }

                let comm_file = pid_dir.join("comm");
                if let Ok(comm) = fs::read_to_string(&comm_file) {
                    let comm = comm.trim().to_string();
                    if comm.is_empty() || comm.starts_with('.') || comm.starts_with("kworker") {
                        continue;
                    }

                    let lower = comm.to_lowercase();
                    // Blacklist common system daemons and background agents
                    let blacklist = [
                        "systemd", "dbus-broker", "pipewire", "wireplumber", "fcitx5",
                        "gnome-shell", "kwin", "plasmashell", "agent", "helper", "daemon",
                        "xwayland", "at-spi", "dconf", "gvfs", "sh", "bash", "zsh", "fish",
                    ];
                    if blacklist.iter().any(|b| lower.contains(b)) {
                        continue;
                    }

                    let glyph = SystemAppEntry::guess_glyph(&comm, &lower);

                    map.entry(lower.clone()).or_insert_with(|| SystemAppEntry {
                        display_name: format!("{} (Running Process)", comm),
                        exec_name: lower,
                        icon_name: "application-x-executable".to_string(),
                        comment: "Currently active process".to_string(),
                        glyph,
                    });
                }
            }
        }
    }

    // 3. Ensure popular developer tools & terminal apps are always available
    let common_tools = [
        ("Neovim", "nvim", "📝"),
        ("Vim", "vim", "📝"),
        ("WezTerm", "wezterm", "💻"),
        ("Alacritty", "alacritty", "💻"),
        ("Kitty", "kitty", "💻"),
        ("Ghostty", "ghostty", "💻"),
        ("Rio Terminal", "rio", "💻"),
        ("Foot", "foot", "💻"),
        ("Tmux", "tmux", "💻"),
        ("Helix Editor", "hx", "📝"),
        ("Zellij", "zellij", "💻"),
        ("Visual Studio Code", "code", "📝"),
        ("Google Chrome", "google-chrome", "🌐"),
        ("Chromium", "chromium", "🌐"),
        ("Mozilla Firefox", "firefox", "🌐"),
        ("Brave Browser", "brave", "🌐"),
        ("Microsoft Edge", "microsoft-edge", "🌐"),
        ("Slack", "slack", "💬"),
        ("Discord", "discord", "💬"),
        ("Telegram Desktop", "telegram-desktop", "💬"),
    ];

    for (disp, exec, glyph) in common_tools {
        map.entry(exec.to_string()).or_insert_with(|| SystemAppEntry {
            display_name: disp.to_string(),
            exec_name: exec.to_string(),
            icon_name: exec.to_string(),
            comment: format!("Pre-configured definition for {}", disp),
            glyph,
        });
    }

    let mut list: Vec<SystemAppEntry> = map.into_values().collect();
    list.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
    list
}
