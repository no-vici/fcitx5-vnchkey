//! Utility script to bump the version of vnchKey across all project files.
//!
//! Usage:
//!   cargo run --bin set-version <new_version>
//!   or:
//!   rustc scripts/set-version.rs -o set-version && ./set-version <new_version>

use std::env;
use std::fs;
use std::path::Path;
use std::process;

struct ReplacementRule {
    file_path: &'static str,
    patterns: Vec<(fn(&str, &str) -> String, fn(&str, &str) -> String)>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin set-version <new-version>");
        eprintln!("Example: cargo run --bin set-version 0.3.4");
        process::exit(1);
    }

    let new_version = args[1].trim().trim_start_matches('v').trim();
    if new_version.is_empty() {
        eprintln!("Error: Version cannot be empty");
        process::exit(1);
    }

    println!("Updating vnchKey to version: v{}", new_version);

    let rules = vec![
        // 1. Root workspace Cargo.toml
        ReplacementRule {
            file_path: "Cargo.toml",
            patterns: vec![(
                |content, new_ver| {
                    let re = regex_lite_replace(content, r#"version\s*=\s*"[^"]+""#, &format!("version = \"{}\"", new_ver), 1);
                    re
                },
                |_old, _new| String::new(),
            )],
        },
        // 2. Windows Cargo.toml
        ReplacementRule {
            file_path: "windows/Cargo.toml",
            patterns: vec![(
                |content, new_ver| {
                    regex_lite_replace(content, r#"version\s*=\s*"[^"]+""#, &format!("version = \"{}\"", new_ver), 1)
                },
                |_old, _new| String::new(),
            )],
        },
        // 3. Fcitx5 addon CMakeLists.txt
        ReplacementRule {
            file_path: "fcitx5-addon/CMakeLists.txt",
            patterns: vec![(
                |content, new_ver| {
                    regex_lite_replace(
                        content,
                        r#"project\(fcitx5-vnchkey-addon VERSION [^ ]+ LANGUAGES CXX\)"#,
                        &format!("project(fcitx5-vnchkey-addon VERSION {} LANGUAGES CXX)", new_ver),
                        1,
                    )
                },
                |_old, _new| String::new(),
            )],
        },
        // 4. Fcitx5 addon config
        ReplacementRule {
            file_path: "data/addon/vnchkey.conf",
            patterns: vec![(
                |content, new_ver| {
                    regex_lite_replace(content, r#"Version=[^\r\n]+"#, &format!("Version={}", new_ver), 1)
                },
                |_old, _new| String::new(),
            )],
        },
        // 5. Arch Linux PKGBUILD
        ReplacementRule {
            file_path: "PKGBUILD",
            patterns: vec![(
                |content, new_ver| {
                    let mut result = String::new();
                    for line in content.lines() {
                        if line.starts_with("pkgver=") {
                            result.push_str(&format!("pkgver={}.r1\n", new_ver));
                        } else if line.contains("printf \"") && line.contains(".r%s.%s\"") {
                            let indent = &line[..line.len() - line.trim_start().len()];
                            result.push_str(&format!("{}printf \"{}.r%s.%s\" \"$(git rev-list --count HEAD 2>/dev/null || echo 1)\" \"$(git rev-parse --short HEAD 2>/dev/null || echo g0)\"\n", indent, new_ver));
                        } else if line.contains("printf \"") && line.contains(".r1\"") {
                            let indent = &line[..line.len() - line.trim_start().len()];
                            result.push_str(&format!("{}printf \"{}.r1\"\n", indent, new_ver));
                        } else {
                            result.push_str(line);
                            result.push('\n');
                        }
                    }
                    if !content.ends_with('\n') && result.ends_with('\n') {
                        result.pop();
                    }
                    result
                },
                |_old, _new| String::new(),
            )],
        },
        // 6. Debian packaging script
        ReplacementRule {
            file_path: "scripts/build-deb.sh",
            patterns: vec![(
                |content, new_ver| {
                    regex_lite_replace(content, r#"VERSION="[^"]+""#, &format!("VERSION=\"{}\"", new_ver), 1)
                },
                |_old, _new| String::new(),
            )],
        },
        // 7. macOS Makefile
        ReplacementRule {
            file_path: "macos/Makefile",
            patterns: vec![(
                |content, new_ver| {
                    let mut result = String::new();
                    let mut in_short_version = false;
                    for line in content.lines() {
                        if line.starts_with("DMG_NAME = ") {
                            result.push_str(&format!("DMG_NAME = $(BUILD_DIR)/$(APP_NAME)-{}.dmg\n", new_ver));
                        } else if in_short_version {
                            let indent = &line[..line.len() - line.trim_start().len()];
                            result.push_str(&format!("{}<string>{}</string>\\\n", indent, new_ver));
                            in_short_version = false;
                        } else if line.contains("CFBundleShortVersionString") {
                            result.push_str(line);
                            result.push('\n');
                            in_short_version = true;
                        } else {
                            result.push_str(line);
                            result.push('\n');
                        }
                    }
                    if !content.ends_with('\n') && result.ends_with('\n') {
                        result.pop();
                    }
                    result
                },
                |_old, _new| String::new(),
            )],
        },
        // 8. macOS Sources/main.swift
        ReplacementRule {
            file_path: "macos/Sources/main.swift",
            patterns: vec![(
                |content, new_ver| {
                    // Replace version in backup dictionary and curVer definitions
                    let c1 = regex_lite_replace(content, r#""version":\s*"[^"]+""#, &format!("\"version\": \"{}\"", new_ver), 1);
                    let c2 = regex_lite_replace(&c1, r#"let curVer = "[^"]+""#, &format!("let curVer = \"{}\"", new_ver), 10);
                    c2
                },
                |_old, _new| String::new(),
            )],
        },
    ];

    let mut updated_files = 0;
    for rule in rules {
        let path = Path::new(rule.file_path);
        if !path.exists() {
            eprintln!("Warning: file not found: {}", rule.file_path);
            continue;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                let mut new_content = content.clone();
                for (apply_fn, _) in &rule.patterns {
                    new_content = apply_fn(&new_content, new_version);
                }

                if new_content != content {
                    if let Err(e) = fs::write(path, new_content) {
                        eprintln!("Error writing {}: {}", rule.file_path, e);
                    } else {
                        println!("  ✓ Updated: {}", rule.file_path);
                        updated_files += 1;
                    }
                } else {
                    println!("  - No change needed: {}", rule.file_path);
                }
            }
            Err(e) => eprintln!("Error reading {}: {}", rule.file_path, e),
        }
    }

    println!("\nSuccessfully synchronized version {} across {} files!", new_version, updated_files);
}

/// Simple regex-like string replacer without external dependencies
fn regex_lite_replace(haystack: &str, pattern: &str, replacement: &str, max_replacements: usize) -> String {
    // If pattern contains standard tokens like version = "..."
    if pattern.starts_with(r#"version\s*=\s*"[^"]+""#) {
        let mut result = String::new();
        let mut replaced = 0;
        let mut lines = haystack.lines().peekable();
        while let Some(line) = lines.next() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("version = ") && replaced < max_replacements {
                let indent = &line[..line.len() - trimmed.len()];
                result.push_str(&format!("{}{}\n", indent, replacement));
                replaced += 1;
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.starts_with("project(fcitx5-vnchkey-addon VERSION") {
        let mut result = String::new();
        for line in haystack.lines() {
            if line.contains("project(fcitx5-vnchkey-addon VERSION") {
                result.push_str(replacement);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.starts_with("Version=") {
        let mut result = String::new();
        for line in haystack.lines() {
            if line.starts_with("Version=") {
                result.push_str(replacement);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.starts_with("pkgver=") {
        let mut result = String::new();
        for line in haystack.lines() {
            if line.starts_with("pkgver=") {
                result.push_str(replacement);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.starts_with("printf") {
        let mut result = String::new();
        for line in haystack.lines() {
            if pattern.contains("%s") && line.contains("printf \"") && line.contains("%s") {
                let indent = &line[..line.len() - line.trim_start().len()];
                result.push_str(&format!("{}{}\n", indent, replacement));
            } else if !pattern.contains("%s") && line.contains("printf \"") && line.contains(".r1\"") {
                let indent = &line[..line.len() - line.trim_start().len()];
                result.push_str(&format!("{}{}\n", indent, replacement));
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.starts_with("VERSION=") {
        let mut result = String::new();
        for line in haystack.lines() {
            if line.starts_with("VERSION=\"") {
                result.push_str(replacement);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.starts_with("DMG_NAME") {
        let mut result = String::new();
        for line in haystack.lines() {
            if line.starts_with("DMG_NAME = ") {
                result.push_str(replacement);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.contains("CFBundleShortVersionString") {
        let mut result = String::new();
        let mut matched = false;
        for line in haystack.lines() {
            if matched {
                let indent = &line[..line.len() - line.trim_start().len()];
                let new_val = replacement.lines().nth(1).unwrap_or(replacement).trim();
                result.push_str(&format!("{}{}\n", indent, new_val));
                matched = false;
            } else if line.contains("CFBundleShortVersionString") {
                result.push_str(line);
                result.push('\n');
                matched = true;
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.contains("\"version\":") {
        let mut result = String::new();
        for line in haystack.lines() {
            if line.trim().starts_with("\"version\":") {
                let indent = &line[..line.len() - line.trim_start().len()];
                result.push_str(&format!("{}{},\n", indent, replacement));
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    if pattern.contains("curVer") {
        let mut result = String::new();
        for line in haystack.lines() {
            if line.trim().starts_with("let curVer = \"") {
                let indent = &line[..line.len() - line.trim_start().len()];
                result.push_str(&format!("{}{}\n", indent, replacement));
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        if !haystack.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        return result;
    }

    haystack.to_string()
}
