//! Inline Math Symbols and Emoji Catalog.
//!
//! Provides shortcode lookup (e.g. `:smile:` -> 😄, `:alpha:` -> α, `:check:` -> ✓)
//! and prefix search for the inline candidate picker popup.

/// Represents an entry in the math symbols & emoji catalog.
pub struct SymbolEntry {
    /// Shortcode identifier without leading/trailing colons (e.g. "smile", "alpha", "check").
    pub shortcode: &'static str,
    /// UTF-8 symbol or emoji character(s).
    pub symbol: &'static str,
}

/// Static catalog of common mathematical, technical, and emoji symbols.
pub const SYMBOL_TABLE: &[SymbolEntry] = &[
    SymbolEntry { shortcode: "alpha", symbol: "α" },
    SymbolEntry { shortcode: "beta", symbol: "β" },
    SymbolEntry { shortcode: "gamma", symbol: "γ" },
    SymbolEntry { shortcode: "delta", symbol: "δ" },
    SymbolEntry { shortcode: "epsilon", symbol: "ε" },
    SymbolEntry { shortcode: "zeta", symbol: "ζ" },
    SymbolEntry { shortcode: "eta", symbol: "η" },
    SymbolEntry { shortcode: "theta", symbol: "θ" },
    SymbolEntry { shortcode: "iota", symbol: "ι" },
    SymbolEntry { shortcode: "kappa", symbol: "κ" },
    SymbolEntry { shortcode: "lambda", symbol: "λ" },
    SymbolEntry { shortcode: "mu", symbol: "μ" },
    SymbolEntry { shortcode: "nu", symbol: "ν" },
    SymbolEntry { shortcode: "xi", symbol: "ξ" },
    SymbolEntry { shortcode: "pi", symbol: "π" },
    SymbolEntry { shortcode: "rho", symbol: "ρ" },
    SymbolEntry { shortcode: "sigma", symbol: "σ" },
    SymbolEntry { shortcode: "tau", symbol: "τ" },
    SymbolEntry { shortcode: "upsilon", symbol: "υ" },
    SymbolEntry { shortcode: "phi", symbol: "φ" },
    SymbolEntry { shortcode: "chi", symbol: "χ" },
    SymbolEntry { shortcode: "psi", symbol: "ψ" },
    SymbolEntry { shortcode: "omega", symbol: "ω" },
    SymbolEntry { shortcode: "Gamma", symbol: "Γ" },
    SymbolEntry { shortcode: "Delta", symbol: "Δ" },
    SymbolEntry { shortcode: "Theta", symbol: "Θ" },
    SymbolEntry { shortcode: "Lambda", symbol: "Λ" },
    SymbolEntry { shortcode: "Xi", symbol: "Ξ" },
    SymbolEntry { shortcode: "Pi", symbol: "Π" },
    SymbolEntry { shortcode: "Sigma", symbol: "Σ" },
    SymbolEntry { shortcode: "Phi", symbol: "Φ" },
    SymbolEntry { shortcode: "Psi", symbol: "Ψ" },
    SymbolEntry { shortcode: "Omega", symbol: "Ω" },
    SymbolEntry { shortcode: "sum", symbol: "∑" },
    SymbolEntry { shortcode: "prod", symbol: "∏" },
    SymbolEntry { shortcode: "int", symbol: "∫" },
    SymbolEntry { shortcode: "iint", symbol: "∬" },
    SymbolEntry { shortcode: "iiint", symbol: "∭" },
    SymbolEntry { shortcode: "oint", symbol: "∮" },
    SymbolEntry { shortcode: "sqrt", symbol: "√" },
    SymbolEntry { shortcode: "cbrt", symbol: "∛" },
    SymbolEntry { shortcode: "inf", symbol: "∞" },
    SymbolEntry { shortcode: "infinity", symbol: "∞" },
    SymbolEntry { shortcode: "partial", symbol: "∂" },
    SymbolEntry { shortcode: "nabla", symbol: "∇" },
    SymbolEntry { shortcode: "times", symbol: "×" },
    SymbolEntry { shortcode: "div", symbol: "÷" },
    SymbolEntry { shortcode: "pm", symbol: "±" },
    SymbolEntry { shortcode: "plusminus", symbol: "±" },
    SymbolEntry { shortcode: "mp", symbol: "∓" },
    SymbolEntry { shortcode: "cdot", symbol: "·" },
    SymbolEntry { shortcode: "dot", symbol: "·" },
    SymbolEntry { shortcode: "approx", symbol: "≈" },
    SymbolEntry { shortcode: "neq", symbol: "≠" },
    SymbolEntry { shortcode: "ne", symbol: "≠" },
    SymbolEntry { shortcode: "leq", symbol: "≤" },
    SymbolEntry { shortcode: "le", symbol: "≤" },
    SymbolEntry { shortcode: "geq", symbol: "≥" },
    SymbolEntry { shortcode: "ge", symbol: "≥" },
    SymbolEntry { shortcode: "ll", symbol: "≪" },
    SymbolEntry { shortcode: "gg", symbol: "≫" },
    SymbolEntry { shortcode: "equiv", symbol: "≡" },
    SymbolEntry { shortcode: "sim", symbol: "∼" },
    SymbolEntry { shortcode: "propto", symbol: "∝" },
    SymbolEntry { shortcode: "prop", symbol: "∝" },
    SymbolEntry { shortcode: "in", symbol: "∈" },
    SymbolEntry { shortcode: "notin", symbol: "∉" },
    SymbolEntry { shortcode: "ni", symbol: "∋" },
    SymbolEntry { shortcode: "subset", symbol: "⊂" },
    SymbolEntry { shortcode: "supset", symbol: "⊃" },
    SymbolEntry { shortcode: "subseteq", symbol: "⊆" },
    SymbolEntry { shortcode: "supseteq", symbol: "⊇" },
    SymbolEntry { shortcode: "cup", symbol: "∪" },
    SymbolEntry { shortcode: "cap", symbol: "∩" },
    SymbolEntry { shortcode: "empty", symbol: "∅" },
    SymbolEntry { shortcode: "emptyset", symbol: "∅" },
    SymbolEntry { shortcode: "forall", symbol: "∀" },
    SymbolEntry { shortcode: "exists", symbol: "∃" },
    SymbolEntry { shortcode: "nexists", symbol: "∄" },
    SymbolEntry { shortcode: "therefore", symbol: "∴" },
    SymbolEntry { shortcode: "because", symbol: "∵" },
    SymbolEntry { shortcode: "and", symbol: "∧" },
    SymbolEntry { shortcode: "or", symbol: "∨" },
    SymbolEntry { shortcode: "not", symbol: "¬" },
    SymbolEntry { shortcode: "neg", symbol: "¬" },
    SymbolEntry { shortcode: "implies", symbol: "⇒" },
    SymbolEntry { shortcode: "iff", symbol: "⇔" },
    SymbolEntry { shortcode: "perp", symbol: "⊥" },
    SymbolEntry { shortcode: "parallel", symbol: "∥" },
    SymbolEntry { shortcode: "angle", symbol: "∠" },
    SymbolEntry { shortcode: "deg", symbol: "°" },
    SymbolEntry { shortcode: "degree", symbol: "°" },
    SymbolEntry { shortcode: "prime", symbol: "′" },
    SymbolEntry { shortcode: "dprime", symbol: "″" },
    SymbolEntry { shortcode: "real", symbol: "ℝ" },
    SymbolEntry { shortcode: "complex", symbol: "ℂ" },
    SymbolEntry { shortcode: "nat", symbol: "ℕ" },
    SymbolEntry { shortcode: "intset", symbol: "ℤ" },
    SymbolEntry { shortcode: "rat", symbol: "ℚ" },
    SymbolEntry { shortcode: "arrow", symbol: "➔" },
    SymbolEntry { shortcode: "rarrow", symbol: "→" },
    SymbolEntry { shortcode: "larrow", symbol: "←" },
    SymbolEntry { shortcode: "uarrow", symbol: "↑" },
    SymbolEntry { shortcode: "darrow", symbol: "↓" },
    SymbolEntry { shortcode: "lrarrow", symbol: "↔" },
    SymbolEntry { shortcode: "udarrow", symbol: "↕" },
    SymbolEntry { shortcode: "rarrow2", symbol: "⇒" },
    SymbolEntry { shortcode: "larrow2", symbol: "⇐" },
    SymbolEntry { shortcode: "lrarrow2", symbol: "⇔" },
    SymbolEntry { shortcode: "check", symbol: "✓" },
    SymbolEntry { shortcode: "check2", symbol: "✅" },
    SymbolEntry { shortcode: "cross", symbol: "✗" },
    SymbolEntry { shortcode: "cross2", symbol: "❌" },
    SymbolEntry { shortcode: "bullet", symbol: "•" },
    SymbolEntry { shortcode: "circ", symbol: "○" },
    SymbolEntry { shortcode: "star", symbol: "⭐" },
    SymbolEntry { shortcode: "star2", symbol: "★" },
    SymbolEntry { shortcode: "heart", symbol: "❤️" },
    SymbolEntry { shortcode: "heart2", symbol: "♥" },
    SymbolEntry { shortcode: "sparkles", symbol: "✨" },
    SymbolEntry { shortcode: "fire", symbol: "🔥" },
    SymbolEntry { shortcode: "rocket", symbol: "🚀" },
    SymbolEntry { shortcode: "warning", symbol: "⚠️" },
    SymbolEntry { shortcode: "100", symbol: "💯" },
    SymbolEntry { shortcode: "smile", symbol: "😄" },
    SymbolEntry { shortcode: "grin", symbol: "😁" },
    SymbolEntry { shortcode: "joy", symbol: "😂" },
    SymbolEntry { shortcode: "laugh", symbol: "😂" },
    SymbolEntry { shortcode: "rofl", symbol: "🤣" },
    SymbolEntry { shortcode: "smiley", symbol: "😃" },
    SymbolEntry { shortcode: "sweat_smile", symbol: "😅" },
    SymbolEntry { shortcode: "wink", symbol: "😉" },
    SymbolEntry { shortcode: "blush", symbol: "😊" },
    SymbolEntry { shortcode: "innocent", symbol: "😇" },
    SymbolEntry { shortcode: "heart_eyes", symbol: "😍" },
    SymbolEntry { shortcode: "love", symbol: "😍" },
    SymbolEntry { shortcode: "kissing", symbol: "😘" },
    SymbolEntry { shortcode: "yum", symbol: "😋" },
    SymbolEntry { shortcode: "stuck_out_tongue", symbol: "😛" },
    SymbolEntry { shortcode: "cool", symbol: "😎" },
    SymbolEntry { shortcode: "sunglasses", symbol: "😎" },
    SymbolEntry { shortcode: "smirk", symbol: "😏" },
    SymbolEntry { shortcode: "unamused", symbol: "😒" },
    SymbolEntry { shortcode: "thinking", symbol: "🤔" },
    SymbolEntry { shortcode: "neutral", symbol: "😐" },
    SymbolEntry { shortcode: "expressionless", symbol: "😑" },
    SymbolEntry { shortcode: "rolling_eyes", symbol: "🙄" },
    SymbolEntry { shortcode: "grimacing", symbol: "😬" },
    SymbolEntry { shortcode: "relieved", symbol: "😌" },
    SymbolEntry { shortcode: "pensive", symbol: "😔" },
    SymbolEntry { shortcode: "sleepy", symbol: "😪" },
    SymbolEntry { shortcode: "sleeping", symbol: "😴" },
    SymbolEntry { shortcode: "mask", symbol: "😷" },
    SymbolEntry { shortcode: "dizzy", symbol: "💫" },
    SymbolEntry { shortcode: "scream", symbol: "😱" },
    SymbolEntry { shortcode: "sob", symbol: "😭" },
    SymbolEntry { shortcode: "cry", symbol: "😢" },
    SymbolEntry { shortcode: "angry", symbol: "😠" },
    SymbolEntry { shortcode: "rage", symbol: "😡" },
    SymbolEntry { shortcode: "poop", symbol: "💩" },
    SymbolEntry { shortcode: "clap", symbol: "👏" },
    SymbolEntry { shortcode: "pray", symbol: "🙏" },
    SymbolEntry { shortcode: "thumbsup", symbol: "👍" },
    SymbolEntry { shortcode: "like", symbol: "👍" },
    SymbolEntry { shortcode: "thumbsdown", symbol: "👎" },
    SymbolEntry { shortcode: "dislike", symbol: "👎" },
    SymbolEntry { shortcode: "punch", symbol: "👊" },
    SymbolEntry { shortcode: "fist", symbol: "✊" },
    SymbolEntry { shortcode: "ok", symbol: "👌" },
    SymbolEntry { shortcode: "ok_hand", symbol: "👌" },
    SymbolEntry { shortcode: "wave", symbol: "👋" },
    SymbolEntry { shortcode: "hand", symbol: "✋" },
    SymbolEntry { shortcode: "open_hands", symbol: "👐" },
    SymbolEntry { shortcode: "muscle", symbol: "💪" },
    SymbolEntry { shortcode: "eyes", symbol: "👀" },
    SymbolEntry { shortcode: "tongue", symbol: "👅" },
    SymbolEntry { shortcode: "coffee", symbol: "☕" },
    SymbolEntry { shortcode: "beer", symbol: "🍺" },
    SymbolEntry { shortcode: "cheers", symbol: "🍻" },
    SymbolEntry { shortcode: "pizza", symbol: "🍕" },
    SymbolEntry { shortcode: "burger", symbol: "🍔" },
    SymbolEntry { shortcode: "apple", symbol: "🍎" },
    SymbolEntry { shortcode: "cake", symbol: "🍰" },
    SymbolEntry { shortcode: "party", symbol: "🎉" },
    SymbolEntry { shortcode: "tada", symbol: "🎉" },
    SymbolEntry { shortcode: "gift", symbol: "🎁" },
    SymbolEntry { shortcode: "bulb", symbol: "💡" },
    SymbolEntry { shortcode: "idea", symbol: "💡" },
    SymbolEntry { shortcode: "pin", symbol: "📌" },
    SymbolEntry { shortcode: "computer", symbol: "💻" },
    SymbolEntry { shortcode: "pc", symbol: "💻" },
    SymbolEntry { shortcode: "phone", symbol: "📱" },
    SymbolEntry { shortcode: "email", symbol: "📧" },
    SymbolEntry { shortcode: "mail", symbol: "📧" },
    SymbolEntry { shortcode: "bug", symbol: "🐛" },
    SymbolEntry { shortcode: "hammer", symbol: "🔨" },
    SymbolEntry { shortcode: "wrench", symbol: "🔧" },
    SymbolEntry { shortcode: "lock", symbol: "🔒" },
    SymbolEntry { shortcode: "unlock", symbol: "🔓" },
    SymbolEntry { shortcode: "key", symbol: "🔑" },

];

/// Looks up a symbol by exact shortcode (with or without surrounding colons).
pub fn lookup_symbol(code: &str) -> Option<&'static str> {
    let mut clean = code.trim();
    if let Some(stripped) = clean.strip_prefix(':') {
        clean = stripped;
    }
    if let Some(stripped) = clean.strip_suffix(':') {
        clean = stripped;
    }
    SYMBOL_TABLE
        .iter()
        .find(|entry| entry.shortcode.eq_ignore_ascii_case(clean))
        .map(|entry| entry.symbol)
}

/// Searches for candidate symbols matching the prefix query.
pub fn search_symbols(query: &str, limit: usize) -> Vec<(&'static str, &'static str)> {
    let mut clean_str = query.trim();
    if let Some(stripped) = clean_str.strip_prefix(':') {
        clean_str = stripped;
    }
    if let Some(stripped) = clean_str.strip_suffix(':') {
        clean_str = stripped;
    }
    let clean = clean_str.to_lowercase();
    if clean.is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    for entry in SYMBOL_TABLE {
        if entry.shortcode.to_lowercase().starts_with(&clean) {
            results.push((entry.symbol, entry.shortcode));
            if results.len() >= limit {
                break;
            }
        }
    }
    // Secondary pass: contains match if not full
    if results.len() < limit {
        for entry in SYMBOL_TABLE {
            let entry_lower = entry.shortcode.to_lowercase();
            if !entry_lower.starts_with(&clean) && entry_lower.contains(&clean) {
                results.push((entry.symbol, entry.shortcode));
                if results.len() >= limit {
                    break;
                }
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_symbols() {
        assert_eq!(lookup_symbol(":smile:"), Some("😄"));
        assert_eq!(lookup_symbol("smile"), Some("😄"));
        assert_eq!(lookup_symbol(":alpha:"), Some("α"));
        assert_eq!(lookup_symbol(":sum:"), Some("∑"));
        assert_eq!(lookup_symbol(":check:"), Some("✓"));
        assert_eq!(lookup_symbol(":unknown:"), None);
    }

    #[test]
    fn test_search_symbols() {
        let res = search_symbols(":sm", 5);
        assert!(!res.is_empty());
        assert!(res.iter().any(|(sym, code)| *code == "smile" && *sym == "😄"));
    }
}
