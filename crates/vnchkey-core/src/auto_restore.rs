//! GoNhanh Innovation: English Auto-Restore Algorithm.
//!
//! Prevents common English words and programming keywords from being mutilated
//! into false Vietnamese diacritics (e.g. `text` -> `têt`, `post` -> `pót`).

use hashbrown::HashSet;

/// High-speed keyword lookup trie for English auto-restore.
#[derive(Debug, Clone)]
pub struct EnglishAutoRestore {
    keywords: HashSet<&'static str>,
}

impl EnglishAutoRestore {
    /// Creates a new auto-restore dictionary loaded with programming and technical English words.
    pub fn new() -> Self {
        let mut keywords = HashSet::with_capacity(256);
        
        // Common software development tokens that trigger false Telex accents
        let list = [
            "text", "test", "next", "cost", "post", "host", "most", "lost", "fast",
            "last", "past", "best", "rest", "west", "list", "rust", "must", "just",
            "case", "base", "import", "export", "class", "struct", "match", "catch",
            "while", "where", "which", "system", "print", "count", "point", "joint",
            "filter", "return", "format", "string", "vector", "state", "props", "hook",
            "code", "node", "mode", "feed", "seed", "need", "read", "lead", "road",
            "load", "file", "line", "type", "date", "rate", "gate", "path", "math",
            "true", "false", "null", "none", "void", "async", "await", "const", "var",
            "let", "fn", "pub", "mod", "use", "impl", "trait", "self", "super",
        ];

        for &word in &list {
            keywords.insert(word);
        }

        Self { keywords }
    }

    /// Checks if the raw keystroke sequence forms an English keyword.
    #[inline]
    pub fn is_english_keyword(&self, raw: &str) -> bool {
        self.keywords.contains(raw.to_lowercase().as_str())
    }
}

impl Default for EnglishAutoRestore {
    fn default() -> Self {
        Self::new()
    }
}
