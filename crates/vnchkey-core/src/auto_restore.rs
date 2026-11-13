//! User Dictionary of English Bypass Words & Auto-Restore Engine.
//!
//! Ported from OpenViet IME (`dictionary.js`), preventing common English words,
//! developer terms, and user-defined vocabulary from being accented into false
//! Vietnamese diacritics (e.g. `text` -> `têt`, `post` -> `pót`, `window` -> `ưindoư`).

use crate::vietnamese_dict::is_in_vietnamese_dictionary;
use std::collections::HashSet;
use std::fs;

/// Built-in default list of common English and technical words bypassed by default
/// (ported from OpenViet IME's `COMMON_ENGLISH_WORDS` / `DEFAULT_USER_WORDS`).
pub const DEFAULT_ENGLISH_WORDS: &[&str] = &[
    // Telex trap: words ending with 's' (sắc)
    "as", "is", "os", "us", "vs",
    "always", "access", "address", "alias", "basis", "bonus", "boss", "business",
    "campus", "canvas", "census", "chaos", "chorus", "circus", "class", "corpus",
    "crisis", "cross", "dress", "express", "famous", "focus", "genius", "glass",
    "guess", "hiatus", "less", "loss", "lotus", "mass", "minus", "miss", "news",
    "nexus", "oasis", "paris", "pass", "perhaps", "plus", "press", "process",
    "radius", "series", "sometimes", "species", "status", "success", "texas",
    "versus", "virus", "walrus", "yes",
    // Telex trap: words with 'x' (ngã)
    "ajax", "annex", "box", "complex", "context", "convex", "cortex", "duplex",
    "exact", "exam", "example", "exceed", "except", "excess", "exchange", "excite",
    "exclude", "excuse", "execute", "exercise", "exist", "exit", "expand", "expect",
    "expend", "expense", "expert", "expire", "explain", "explore", "export",
    "extent", "extra", "extreme", "fax", "fix", "galaxy", "index", "latex",
    "linux", "luxury", "matrix", "max", "maximum", "mix", "next", "oxide",
    "oxygen", "pixel", "prefix", "reflex", "sax", "sex", "six", "suffix",
    "syntax", "tax", "taxi", "text", "toxic", "unix", "vertex",
    // Telex trap: words with 'r' (hỏi)
    "after", "armor", "bear", "better", "Browser", "browser", "care", "center",
    "clear", "color", "core", "cursor", "dear", "doctor", "driver", "error",
    "ever", "factor", "favor", "fear", "filter", "flavor", "for", "gear",
    "harbor", "hard", "hear", "horror", "junior", "labor", "letter", "master",
    "matter", "mayor", "member", "mirror", "modern", "near", "never", "number", "order",
    "other", "over", "paper", "pear", "player", "power", "prior", "rear",
    "render", "router", "sector", "senior", "sensor", "server", "super", "tear",
    "their", "under", "user", "vapor", "vector", "vigor", "water", "wear",
    "worker", "year", "your",
    // Telex trap: words with 'w' (horn/breve: ơ, ư, ă)
    "wait", "wall", "want", "warm", "warn", "wash", "waste", "watch", "wave",
    "way", "wayland", "web", "webpack", "website", "week", "well", "west",
    "whale", "what", "wheat", "wheel", "when", "where", "which", "while",
    "whip", "whisper", "white", "who", "why", "wide", "wife", "wifi", "wild",
    "will", "wind", "window", "windows", "wine", "wing", "wire", "wise", "wish",
    "with", "within", "without", "wood", "wool", "word", "work", "world",
    "worth", "worthy", "would", "wrap", "write", "wrong",
    // Telex trap: words with 'f' (huyền)
    "face", "fact", "fail", "fair", "fall", "false", "farm", "fast", "feat",
    "feed", "feel", "feet", "fell", "felt", "few", "figma", "file", "fill",
    "film", "find", "fine", "fire", "firefox", "firm", "first", "fish", "fit",
    "five", "flat", "flight", "floor", "flow", "fly", "folk", "font", "food",
    "foot", "force", "fork", "form", "free", "friend", "from", "front",
    "frontend", "fruit", "full", "fund", "funny", "future", "life", "off", "safe",
    // Telex trap: words with 'j' (nặng)
    "jail", "jar", "javascript", "jaw", "jean", "jeep", "jelly", "jest", "jet",
    "jewel", "jig", "jinx", "job", "jockey", "jog", "join", "joint", "joke",
    "jolly", "jolt", "journal", "journey", "joy", "json", "judge", "judo",
    "jug", "juggle", "juggler", "juice", "jumbo", "jump", "jumper", "junction",
    "jungle", "junk", "junkie", "jupiter", "juror", "jury", "just", "justice",
    "justify",
    // Tech, Developer & Linux Words
    "in", "or", "of", "if", "on", "to", "at", "by", "do", "go", "no", "so", "up",
    "alacritty", "api", "app", "arch", "assets", "async", "await", "back", "base",
    "bash", "because", "best", "blog", "branch", "bug", "build", "bundle", "cargo",
    "case", "catch", "cell", "chat", "chrome", "circle", "city", "class", "click",
    "client", "clone", "cloud", "cmake", "code", "commit", "component", "config",
    "console", "const", "cost", "could", "count", "crates", "css", "data", "date",
    "debug", "deploy", "dev", "developer", "diff", "disk", "dist", "div", "docker",
    "dom", "download", "edge", "egui", "electron", "email", "env", "escape", "even",
    "extension", "facebook", "fcitx", "fcitx5", "fetch", "flatpak", "fn", "format",
    "game", "gate", "general", "get", "gift", "git", "github", "gitlab", "gnome",
    "going", "gone", "good", "google", "graphql", "great", "grpc", "group", "guard",
    "guest", "guide", "have", "helm", "home", "hook", "host", "html", "http",
    "https", "hyprland", "ibus", "icon", "iframe", "image", "impl", "import",
    "input", "install", "internet", "into", "issue", "jest", "k8s", "kafka",
    "kde", "kernel", "key", "kitty", "know", "kubectl", "last", "lead", "let",
    "license", "like", "line", "link", "lint", "list", "load", "local", "log",
    "look", "lost", "macos", "make", "makepkg", "match", "math", "merge", "meta",
    "minify", "modal", "mode", "mod", "mongodb", "module", "most", "must", "mysql",
    "need", "neovim", "net", "network", "nextjs", "nginx", "ninja", "node", "none",
    "npm", "null", "nuxt", "nvim", "offline", "online", "only", "open", "output",
    "package", "pacman", "page", "password", "past", "patch", "path", "people",
    "ping", "pip", "plasma", "playwright", "plugin", "pnpm", "point", "port",
    "post", "postgres", "postgresql", "print", "prisma", "props", "proto",
    "protobuf", "pub", "public", "pull", "push", "pytest", "python", "rate",
    "react", "read", "readme", "rebase", "redis", "refactor", "regex", "release", "repo",
    "request", "reset", "response", "rest", "restore", "return", "road", "rust", "rustc", "saas",
    "safari", "save", "screen", "script", "scss", "seed", "self", "semver",
    "service", "setup", "shell", "site", "some", "sort", "source", "span",
    "sql", "sqlite", "ssh", "stash", "state", "steam", "string", "struct",
    "style", "sudo", "svelte", "sway", "sync", "system", "tab", "tag", "tailwind",
    "take", "target", "task", "tauri", "terminal", "terraform", "tesla", "test",
    "that", "them", "then", "think", "time", "tmux", "token", "toml", "tool",
    "trait", "true", "type", "ubuntu", "undefined", "update", "upload", "url",
    "use", "utf8", "var", "version", "video", "view", "vim", "vite", "vitest",
    "void", "vscode", "vulkan", "wasm", "wezterm", "while", "yaml", "yarn",
    "yay", "zip", "zsh",
];

/// Vietnamese Telex input sequences that happen to spell common English words (`thí`, `thế`, `thể`, `rết`, `cháo`)
/// and must never override valid Vietnamese words unless the user explicitly forces them.
const VIETNAMESE_COLLISION_EXCEPTIONS: &[&str] = &[
    "this", "these", "there", "those", "has", "his", "bus", "her", "see", "can", "man", "son", "con",
    "chaos", "its", "boss", "less", "reset",
];

/// High-speed dictionary for English auto-restore and user custom bypass words.
#[derive(Debug, Clone)]
pub struct EnglishAutoRestore {
    keywords: HashSet<String>,
}

impl EnglishAutoRestore {
    /// Creates a new auto-restore dictionary loaded with the built-in English bypass words.
    pub fn new() -> Self {
        let mut keywords = HashSet::with_capacity(DEFAULT_ENGLISH_WORDS.len() + 64);
        for &word in DEFAULT_ENGLISH_WORDS {
            let clean = word.trim().to_lowercase();
            if !clean.is_empty() {
                keywords.insert(clean);
            }
        }
        Self { keywords }
    }

    /// Loads user dictionary words from a configuration file (one word per line).
    /// Merges with the built-in English dictionary.
    pub fn load_from_file(&mut self, path: &str) {
        if let Ok(content) = fs::read_to_string(path) {
            self.keywords.clear();
            for &word in DEFAULT_ENGLISH_WORDS {
                let clean = word.trim().to_lowercase();
                if !clean.is_empty() {
                    self.keywords.insert(clean);
                }
            }
            for line in content.lines() {
                let trimmed = line.trim().to_lowercase();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    self.keywords.insert(trimmed);
                }
            }
        }
    }

    /// Dynamically inserts a learned word into the in-memory bypass dictionary.
    pub fn add_keyword(&mut self, word: &str) {
        let clean = word.trim().to_lowercase();
        if clean.len() >= 2 && !clean.chars().any(|c| c.is_ascii_digit()) {
            self.keywords.insert(clean);
        }
    }

    /// Removes a word from the in-memory bypass dictionary (e.g. when unlearned/deleted by user).
    pub fn remove_keyword(&mut self, word: &str) {
        let clean = word.trim().to_lowercase();
        self.keywords.remove(&clean);
    }

    /// Checks if the raw keystroke sequence forms an English or user-bypassed keyword.
    #[inline]
    pub fn is_english_keyword(&self, raw: &str) -> bool {
        let clean = raw.trim().to_lowercase();
        if clean.len() < 2 || clean.chars().any(|c| c.is_ascii_digit()) {
            return false;
        }
        if VIETNAMESE_COLLISION_EXCEPTIONS.contains(&clean.as_str()) {
            return false;
        }
        self.keywords.contains(&clean)
    }

    /// Determines if `current_word` (transformed from `raw_strokes`) should be restored
    /// to `raw_strokes` based on the User Dictionary and Built-in Vietnamese Dictionary.
    pub fn should_restore(&self, raw_strokes: &str, current_word: &str, use_dictionary: bool) -> bool {
        let raw_clean = raw_strokes.trim().to_lowercase();
        let cur_clean = current_word.trim().to_lowercase();

        // Rule 1: Never restore VNI digit sequences or very short words (< 2 raw strokes or <= 1 char result)
        if raw_clean.len() < 2
            || cur_clean.chars().count() <= 1
            || raw_clean.chars().any(|c| c.is_ascii_digit())
        {
            return false;
        }

        // Rule 2: Protect common Vietnamese collision words ("this" -> "thí", "these" -> "thế", "there" -> "thể")
        if VIETNAMESE_COLLISION_EXCEPTIONS.contains(&raw_clean.as_str())
            && is_in_vietnamese_dictionary(&cur_clean)
        {
            return false;
        }

        // Rule 3: If User Dictionary is enabled and raw_strokes is in the English/User dictionary:
        // - Restore if raw_strokes is >= 4 chars (e.g. "text", "post", "cost", "user", "window", "linux")
        // - Or restore if the resulting 2-3 letter word is NOT in the Vietnamese dictionary (e.g. "fix" -> "fĩ", "web" -> "ưeb", "yes" -> "ýe")
        if use_dictionary && self.keywords.contains(&raw_clean) {
            if raw_clean.len() >= 4 || !is_in_vietnamese_dictionary(&cur_clean) {
                return true;
            }
        }

        false
    }
}

impl Default for EnglishAutoRestore {
    fn default() -> Self {
        Self::new()
    }
}
