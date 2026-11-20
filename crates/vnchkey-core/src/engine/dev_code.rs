// Developer code-block & token-scoped guard state

/// State machine for Developer Mode (Markdown inline code & fenced blocks).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DevCodeState {
    consecutive_backticks: u8,
    in_inline_code: bool,
    in_fenced_code: bool,
}

impl DevCodeState {
    /// Resets the code block state machine.
    pub fn reset(&mut self) {
        self.consecutive_backticks = 0;
        self.in_inline_code = false;
        self.in_fenced_code = false;
    }

    /// Updates the FSM when a backtick (`` ` ``) is typed.
    pub fn on_backtick(&mut self) {
        self.consecutive_backticks = self.consecutive_backticks.saturating_add(1);
        if self.consecutive_backticks >= 3 {
            self.in_fenced_code = !self.in_fenced_code;
            self.in_inline_code = false;
            self.consecutive_backticks = 0;
        } else if !self.in_fenced_code {
            self.in_inline_code = !self.in_inline_code;
        }
    }

    /// Updates the FSM when any non-backtick character is typed.
    pub fn on_non_backtick(&mut self) {
        self.consecutive_backticks = 0;
    }

    /// Updates the FSM on newline (`\n` or `\r`), closing any unclosed inline code span while keeping fenced blocks open.
    pub fn on_newline(&mut self) {
        self.consecutive_backticks = 0;
        self.in_inline_code = false;
    }

    /// Returns `true` if the cursor is currently inside an inline or fenced Markdown code block.
    pub fn is_bypassing(&self) -> bool {
        self.in_inline_code || self.in_fenced_code
    }
}

/// Returns `true` if `word_lower` is a high-frequency English function/anchor word
/// that establishes English sentence momentum for subsequent ambiguous collision words.
pub fn is_english_context_anchor(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "a"
            | "about"
            | "after"
            | "already"
            | "also"
            | "although"
            | "always"
            | "am"
            | "an"
            | "and"
            | "are"
            | "at"
            | "be"
            | "because"
            | "been"
            | "before"
            | "being"
            | "between"
            | "but"
            | "by"
            | "can"
            | "could"
            | "did"
            | "do"
            | "does"
            | "during"
            | "even"
            | "for"
            | "from"
            | "get"
            | "go"
            | "had"
            | "has"
            | "have"
            | "having"
            | "he"
            | "her"
            | "here"
            | "his"
            | "i"
            | "in"
            | "into"
            | "is"
            | "it"
            | "its"
            | "make"
            | "might"
            | "must"
            | "my"
            | "never"
            | "not"
            | "on"
            | "our"
            | "safe"
            | "shall"
            | "she"
            | "should"
            | "state"
            | "still"
            | "take"
            | "that"
            | "the"
            | "their"
            | "there"
            | "these"
            | "they"
            | "this"
            | "those"
            | "though"
            | "through"
            | "to"
            | "under"
            | "unless"
            | "until"
            | "was"
            | "we"
            | "were"
            | "what"
            | "when"
            | "where"
            | "whether"
            | "which"
            | "while"
            | "who"
            | "whom"
            | "whose"
            | "will"
            | "with"
            | "without"
            | "would"
            | "you"
            | "your"
    )
}

/// Returns `true` if `word_lower` is a common Vietnamese word (including ASCII-only Vietnamese syllables)
/// that resets English sentence momentum back to Vietnamese.
pub fn is_vietnamese_context_reset(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "anh"
            | "bạn"
            | "bộ"
            | "chưa"
            | "chắc"
            | "chị"
            | "con"
            | "cá"
            | "các"
            | "cũng"
            | "em"
            | "hai"
            | "hẳn"
            | "họ"
            | "không"
            | "liệu"
            | "lùn"
            | "mai"
            | "mình"
            | "một"
            | "nhỏ"
            | "những"
            | "nó"
            | "ông"
            | "rất"
            | "sao"
            | "ta"
            | "thì"
            | "tôi"
            | "tui"
            | "và"
            | "với"
    )
}

/// Returns `true` if `word_lower` is a trigonometric/math function anchor (`sin`, `cos`, `tan`, `cot`, etc.).
pub fn is_math_function_anchor(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "sin" | "cos" | "tan" | "cot" | "sine" | "cosine" | "tangent" | "trig" | "trigonometry"
    )
}

/// Returns `true` if `raw_lower` is a bilingual collision word that is valid in Vietnamese without context
/// (e.g. `tree` -> `trê`, `is` -> `í`, `seen` -> `sên`, `cost` -> `cót`, `room` -> `rôm`, `must` -> `mút`, `rust` -> `rút`)
/// and should resolve to English when preceded by English sentence momentum.
pub fn is_context_english_collision_word(raw_lower: &str) -> bool {
    matches!(
        raw_lower,
        "air"
            | "am"
            | "an"
            | "arm"
            | "arms"
            | "arts"
            | "as"
            | "at"
            | "bar"
            | "barn"
            | "bars"
            | "be"
            | "bee"
            | "been"
            | "beer"
            | "bees"
            | "best"
            | "bist"
            | "bits"
            | "boom"
            | "boost"
            | "boots"
            | "born"
            | "bow"
            | "box"
            | "burn"
            | "burns"
            | "burst"
            | "bus"
            | "by"
            | "cans"
            | "caps"
            | "car"
            | "cars"
            | "cast"
            | "cats"
            | "chair"
            | "chairs"
            | "chaos"
            | "char"
            | "charm"
            | "charts"
            | "cheers"
            | "chef"
            | "chest"
            | "chips"
            | "choir"
            | "chose"
            | "chown"
            | "conf"
            | "cons"
            | "corn"
            | "corps"
            | "cors"
            | "cos"
            | "cost"
            | "cow"
            | "cups"
            | "cuts"
            | "dawn"
            | "days"
            | "deer"
            | "def"
            | "dir"
            | "dist"
            | "do"
            | "docs"
            | "dos"
            | "dust"
            | "est"
            | "gaps"
            | "gas"
            | "gif"
            | "go"
            | "hair"
            | "hangs"
            | "harm"
            | "has"
            | "hats"
            | "he"
            | "her"
            | "here"
            | "hero"
            | "his"
            | "hist"
            | "hits"
            | "horn"
            | "host"
            | "how"
            | "if"
            | "in"
            | "inf"
            | "is"
            | "ist"
            | "it"
            | "its"
            | "keen"
            | "keeps"
            | "kist"
            | "kits"
            | "laos"
            | "last"
            | "lawn"
            | "lays"
            | "lee"
            | "lens"
            | "les"
            | "lets"
            | "lips"
            | "list"
            | "loans"
            | "longs"
            | "loops"
            | "lose"
            | "lots"
            | "low"
            | "mans"
            | "mar"
            | "mars"
            | "mats"
            | "max"
            | "me"
            | "meets"
            | "mere"
            | "mist"
            | "mix"
            | "moon"
            | "must"
            | "my"
            | "nest"
            | "nextjs"
            | "nist"
            | "no"
            | "noon"
            | "nor"
            | "norm"
            | "now"
            | "nuts"
            | "of"
            | "on"
            | "oops"
            | "or"
            | "os"
            | "own"
            | "owns"
            | "par"
            | "parts"
            | "past"
            | "pee"
            | "per"
            | "pest"
            | "pets"
            | "pics"
            | "queen"
            | "queens"
            | "quest"
            | "rams"
            | "rats"
            | "rays"
            | "reef"
            | "ref"
            | "refer"
            | "refers"
            | "res"
            | "reset"
            | "rest"
            | "rist"
            | "roms"
            | "roof"
            | "room"
            | "roots"
            | "row"
            | "rows"
            | "runs"
            | "rust"
            | "sans"
            | "see"
            | "seen"
            | "seer"
            | "sets"
            | "sex"
            | "sir"
            | "six"
            | "so"
            | "sofa"
            | "songs"
            | "sons"
            | "sorts"
            | "suns"
            | "surf"
            | "tar"
            | "tax"
            | "taxi"
            | "tee"
            | "teen"
            | "term"
            | "terms"
            | "test"
            | "tex"
            | "texts"
            | "thats"
            | "thee"
            | "there"
            | "these"
            | "this"
            | "thus"
            | "tips"
            | "tist"
            | "to"
            | "toes"
            | "too"
            | "tops"
            | "tos"
            | "town"
            | "towns"
            | "trans"
            | "tree"
            | "trust"
            | "turn"
            | "tust"
            | "up"
            | "ups"
            | "uri"
            | "us"
            | "usa"
            | "usr"
            | "vans"
            | "var"
            | "vary"
            | "vast"
            | "ver"
            | "verse"
            | "virus"
            | "visa"
            | "vist"
            | "wa"
            | "wars"
            | "was"
            | "we"
            | "wr"
            | "ws"
            | "wu"
            | "xerox"
            | "yrs"
    )
}

/// Converts precomposed Vietnamese characters to decomposed Unicode (NFD / Unicode Tổ Hợp).
pub fn to_composite_unicode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            'á' => out.push_str("a\u{0301}"),
            'à' => out.push_str("a\u{0300}"),
            'ả' => out.push_str("a\u{0309}"),
            'ã' => out.push_str("a\u{0303}"),
            'ạ' => out.push_str("a\u{0323}"),
            'â' => out.push_str("a\u{0302}"),
            'ấ' => out.push_str("a\u{0302}\u{0301}"),
            'ầ' => out.push_str("a\u{0302}\u{0300}"),
            'ẩ' => out.push_str("a\u{0302}\u{0309}"),
            'ẫ' => out.push_str("a\u{0302}\u{0303}"),
            'ậ' => out.push_str("a\u{0302}\u{0323}"),
            'ă' => out.push_str("a\u{0306}"),
            'ắ' => out.push_str("a\u{0306}\u{0301}"),
            'ằ' => out.push_str("a\u{0306}\u{0300}"),
            'ẳ' => out.push_str("a\u{0306}\u{0309}"),
            'ẵ' => out.push_str("a\u{0306}\u{0303}"),
            'ặ' => out.push_str("a\u{0306}\u{0323}"),

            'Á' => out.push_str("A\u{0301}"),
            'À' => out.push_str("A\u{0300}"),
            'Ả' => out.push_str("A\u{0309}"),
            'Ã' => out.push_str("A\u{0303}"),
            'Ạ' => out.push_str("A\u{0323}"),
            'Â' => out.push_str("A\u{0302}"),
            'Ấ' => out.push_str("A\u{0302}\u{0301}"),
            'Ầ' => out.push_str("A\u{0302}\u{0300}"),
            'Ẩ' => out.push_str("A\u{0302}\u{0309}"),
            'Ẫ' => out.push_str("A\u{0302}\u{0303}"),
            'Ậ' => out.push_str("A\u{0302}\u{0323}"),
            'Ă' => out.push_str("A\u{0306}"),
            'Ắ' => out.push_str("A\u{0306}\u{0301}"),
            'Ằ' => out.push_str("A\u{0306}\u{0300}"),
            'Ẳ' => out.push_str("A\u{0306}\u{0309}"),
            'Ẵ' => out.push_str("A\u{0306}\u{0303}"),
            'Ặ' => out.push_str("A\u{0306}\u{0323}"),

            'é' => out.push_str("e\u{0301}"),
            'è' => out.push_str("e\u{0300}"),
            'ẻ' => out.push_str("e\u{0309}"),
            'ẽ' => out.push_str("e\u{0303}"),
            'ẹ' => out.push_str("e\u{0323}"),
            'ê' => out.push_str("e\u{0302}"),
            'ế' => out.push_str("e\u{0302}\u{0301}"),
            'ề' => out.push_str("e\u{0302}\u{0300}"),
            'ể' => out.push_str("e\u{0302}\u{0309}"),
            'ễ' => out.push_str("e\u{0302}\u{0303}"),
            'ệ' => out.push_str("e\u{0302}\u{0323}"),

            'É' => out.push_str("E\u{0301}"),
            'È' => out.push_str("E\u{0300}"),
            'Ẻ' => out.push_str("E\u{0309}"),
            'Ẽ' => out.push_str("E\u{0303}"),
            'Ẹ' => out.push_str("E\u{0323}"),
            'Ê' => out.push_str("E\u{0302}"),
            'Ế' => out.push_str("E\u{0302}\u{0301}"),
            'Ề' => out.push_str("E\u{0302}\u{0300}"),
            'Ể' => out.push_str("E\u{0302}\u{0309}"),
            'Ễ' => out.push_str("E\u{0302}\u{0303}"),
            'Ệ' => out.push_str("E\u{0302}\u{0323}"),

            'í' => out.push_str("i\u{0301}"),
            'ì' => out.push_str("i\u{0300}"),
            'ỉ' => out.push_str("i\u{0309}"),
            'ĩ' => out.push_str("i\u{0303}"),
            'ị' => out.push_str("i\u{0323}"),

            'Í' => out.push_str("I\u{0301}"),
            'Ì' => out.push_str("I\u{0300}"),
            'Ỉ' => out.push_str("I\u{0309}"),
            'Ĩ' => out.push_str("I\u{0303}"),
            'Ị' => out.push_str("I\u{0323}"),

            'ó' => out.push_str("o\u{0301}"),
            'ò' => out.push_str("o\u{0300}"),
            'ỏ' => out.push_str("o\u{0309}"),
            'õ' => out.push_str("o\u{0303}"),
            'ọ' => out.push_str("o\u{0323}"),
            'ô' => out.push_str("o\u{0302}"),
            'ố' => out.push_str("o\u{0302}\u{0301}"),
            'ồ' => out.push_str("o\u{0302}\u{0300}"),
            'ổ' => out.push_str("o\u{0302}\u{0309}"),
            'ỗ' => out.push_str("o\u{0302}\u{0303}"),
            'ộ' => out.push_str("o\u{0302}\u{0323}"),
            'ơ' => out.push_str("o\u{031B}"),
            'ớ' => out.push_str("o\u{031B}\u{0301}"),
            'ờ' => out.push_str("o\u{031B}\u{0300}"),
            'ở' => out.push_str("o\u{031B}\u{0309}"),
            'ỡ' => out.push_str("o\u{031B}\u{0303}"),
            'ợ' => out.push_str("o\u{031B}\u{0323}"),

            'Ó' => out.push_str("O\u{0301}"),
            'Ò' => out.push_str("O\u{0300}"),
            'Ỏ' => out.push_str("O\u{0309}"),
            'Õ' => out.push_str("O\u{0303}"),
            'Ọ' => out.push_str("O\u{0323}"),
            'Ô' => out.push_str("O\u{0302}"),
            'Ố' => out.push_str("O\u{0302}\u{0301}"),
            'Ồ' => out.push_str("O\u{0302}\u{0300}"),
            'Ổ' => out.push_str("O\u{0302}\u{0309}"),
            'Ỗ' => out.push_str("O\u{0302}\u{0303}"),
            'Ộ' => out.push_str("O\u{0302}\u{0323}"),
            'Ơ' => out.push_str("O\u{031B}"),
            'Ớ' => out.push_str("O\u{031B}\u{0301}"),
            'Ờ' => out.push_str("O\u{031B}\u{0300}"),
            'Ở' => out.push_str("O\u{031B}\u{0309}"),
            'Ỡ' => out.push_str("O\u{031B}\u{0303}"),
            'Ợ' => out.push_str("O\u{031B}\u{0323}"),

            'ú' => out.push_str("u\u{0301}"),
            'ù' => out.push_str("u\u{0300}"),
            'ủ' => out.push_str("u\u{0309}"),
            'ũ' => out.push_str("u\u{0303}"),
            'ụ' => out.push_str("u\u{0323}"),
            'ư' => out.push_str("u\u{031B}"),
            'ứ' => out.push_str("u\u{031B}\u{0301}"),
            'ừ' => out.push_str("u\u{031B}\u{0300}"),
            'ử' => out.push_str("u\u{031B}\u{0309}"),
            'ữ' => out.push_str("u\u{031B}\u{0303}"),
            'ự' => out.push_str("u\u{031B}\u{0323}"),

            'Ú' => out.push_str("U\u{0301}"),
            'Ù' => out.push_str("U\u{0300}"),
            'Ủ' => out.push_str("U\u{0309}"),
            'Ũ' => out.push_str("U\u{0303}"),
            'Ụ' => out.push_str("U\u{0323}"),
            'Ư' => out.push_str("U\u{031B}"),
            'Ứ' => out.push_str("U\u{031B}\u{0301}"),
            'Ừ' => out.push_str("U\u{031B}\u{0300}"),
            'Ử' => out.push_str("U\u{031B}\u{0309}"),
            'Ữ' => out.push_str("U\u{031B}\u{0303}"),
            'Ự' => out.push_str("U\u{031B}\u{0323}"),

            'ý' => out.push_str("y\u{0301}"),
            'ỳ' => out.push_str("y\u{0300}"),
            'ỷ' => out.push_str("y\u{0309}"),
            'ỹ' => out.push_str("y\u{0303}"),
            'ỵ' => out.push_str("y\u{0323}"),

            'Ý' => out.push_str("Y\u{0301}"),
            'Ỳ' => out.push_str("Y\u{0300}"),
            'Ỷ' => out.push_str("Y\u{0309}"),
            'Ỹ' => out.push_str("Y\u{0303}"),
            'Ỵ' => out.push_str("Y\u{0323}"),

            other => out.push(other),
        }
    }
    out
}

