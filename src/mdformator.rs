use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

pub struct MdFormator {
    cache: CommonMarkCache,
    last_hash: u64,
}

impl MdFormator {
    pub fn new() -> Self {
        Self { cache: CommonMarkCache::default(), last_hash: 0 }
    }

    pub fn render_scrollable(&mut self, ui: &mut egui::Ui, markdown: &str) {
        let hash = hash_str(markdown);
        if hash != self.last_hash {
            self.cache = CommonMarkCache::default();
            self.last_hash = hash;
        }
        let processed = latex_preprocess(markdown);

        // Use plain `show` inside our own ScrollArea.
        // `show_scrollable` uses virtual-scroll split-points that can land mid-list,
        // causing egui_commonmark to receive Tag::Item without Tag::List → panic.
        egui::ScrollArea::vertical()
            .id_salt("md_preview_scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                CommonMarkViewer::new()
                    .show_alt_text_on_hover(true)
                    .default_implicit_uri_scheme("https://")
                    .syntax_theme_light("InspiredGitHub")
                    .syntax_theme_dark("base16-ocean.dark")
                    .show(ui, &mut self.cache, &processed);
            });
    }
}

impl Default for MdFormator {
    fn default() -> Self { Self::new() }
}

// ── LaTeX preprocessing ───────────────────────────────────────────────────────

fn latex_preprocess(input: &str) -> String {
    let s = replace_display_math(input);
    replace_inline_math(&s)
}

/// Replaces `$$...$$` blocks (multiline allowed) with a fenced ```math block.
fn replace_display_math(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("$$") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("$$") {
            let inner = rest[..end].trim();
            let converted = apply_symbol_map(inner);
            out.push_str("\n```math\n");
            out.push_str(&converted);
            out.push_str("\n```\n");
            rest = &rest[end + 2..];
        } else {
            out.push_str("$$");
            out.push_str(rest);
            return out;
        }
    }
    out.push_str(rest);
    out
}

/// Replaces `$...$` inline math — processes LINE BY LINE so corrupted spans
/// can never bleed across newlines into list items, footnotes, or code blocks.
/// Also skips backtick spans and currency patterns like $3.7 or $100.
fn replace_inline_math(input: &str) -> String {
    // Track fenced code block state across lines so we don't touch them.
    let mut in_fence = false;
    let mut out_lines: Vec<String> = Vec::with_capacity(input.lines().count());

    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            out_lines.push(line.to_string());
            continue;
        }
        if in_fence {
            out_lines.push(line.to_string());
            continue;
        }
        out_lines.push(process_math_on_line(line));
    }

    out_lines.join("\n")
}

fn process_math_on_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        // ── Skip backtick spans verbatim ────────────────────────────────
        if chars[i] == '`' {
            // Count consecutive opening ticks (support `` and ``` spans)
            let tick_start = i;
            while i < n && chars[i] == '`' { i += 1; }
            let tick_len = i - tick_start;
            for _ in 0..tick_len { out.push('`'); }

            // Copy content until matching closing run of same length
            let mut close_buf = String::new();
            while i < n {
                if chars[i] == '`' {
                    let close_start = i;
                    while i < n && chars[i] == '`' { i += 1; }
                    let close_len = i - close_start;
                    if close_len == tick_len {
                        for _ in 0..close_len { out.push('`'); }
                        close_buf.clear();
                        break;
                    } else {
                        // Wrong number of ticks — push them to buffer and keep looking
                        for _ in 0..close_len { close_buf.push('`'); }
                    }
                } else {
                    close_buf.push(chars[i]);
                    out.push(chars[i]);
                    i += 1;
                }
            }
            continue;
        }

        // ── Math expression detection ────────────────────────────────────
        if chars[i] == '$' && (i == 0 || chars[i - 1] != '\\') {
            let peek = if i + 1 < n { Some(chars[i + 1]) } else { None };

            // Skip currency: $3, $1,000, $-5 etc.
            let is_currency = peek.map(|c| c.is_ascii_digit() || c == '-').unwrap_or(false);

            if !is_currency {
                // Scan for closing $ on the SAME line only
                let start = i + 1;
                let mut j = start;
                while j < n {
                    if chars[j] == '$' && (j == 0 || chars[j - 1] != '\\') { break; }
                    j += 1;
                }

                if j < n && j > start {
                    let inner: String = chars[start..j].iter().collect();
                    let trimmed_inner = inner.trim();
                    if !trimmed_inner.is_empty() {
                        let converted = apply_symbol_map(trimmed_inner);
                        out.push('`');
                        out.push_str(&converted);
                        out.push('`');
                        i = j + 1;
                        continue;
                    }
                }
            }
            // Not valid math — emit $ literally
            out.push('$');
            i += 1;
            continue;
        }

        out.push(chars[i]);
        i += 1;
    }
    out
}

// ── Symbol conversion ─────────────────────────────────────────────────────────

fn apply_symbol_map(input: &str) -> String {
    let mut s = input.to_string();

    // ── \frac{num}{den} → (num/den) ──────────────────────────────────────
    loop {
        let Some(pos) = s.find("\\frac{") else { break };
        let after = &s[pos + 6..];
        let Some(e1) = after.find('}') else { break };
        let num = after[..e1].to_string();
        let rest = &after[e1 + 1..];
        if !rest.starts_with('{') { break; }
        let Some(e2) = rest[1..].find('}') else { break };
        let den = rest[1..e2 + 1].to_string();
        let full = format!("\\frac{{{num}}}{{{den}}}");
        let repl = format!("({num}/{den})");
        s = s.replacen(&full, &repl, 1);
    }

    // ── \sqrt[n]{x} → ⁿ√(x) ─────────────────────────────────────────────
    loop {
        let Some(pos) = s.find("\\sqrt[") else { break };
        let after = &s[pos + 6..];
        let Some(en) = after.find(']') else { break };
        let n = after[..en].to_string();
        let rest = &after[en + 1..];
        if !rest.starts_with('{') { break; }
        let Some(ex) = rest[1..].find('}') else { break };
        let x = rest[1..ex + 1].to_string();
        let sup = to_superscript(&n);
        let full = format!("\\sqrt[{n}]{{{x}}}");
        let repl = format!("{sup}√({x})");
        s = s.replacen(&full, &repl, 1);
    }

    // ── \sqrt{x} → √(x) ──────────────────────────────────────────────────
    loop {
        let Some(pos) = s.find("\\sqrt{") else { break };
        let after = &s[pos + 6..];
        let Some(e) = after.find('}') else { break };
        let x = after[..e].to_string();
        let full = format!("\\sqrt{{{x}}}");
        let repl = format!("√({x})");
        s = s.replacen(&full, &repl, 1);
    }

    // ── ^{...} and _{...} → Unicode super/subscript ───────────────────────
    loop {
        let Some(pos) = s.find("^{") else { break };
        let after = &s[pos + 2..];
        let Some(e) = after.find('}') else { break };
        let inner = after[..e].to_string();
        let full = format!("^{{{inner}}}");
        s = s.replacen(&full, &to_superscript(&inner), 1);
    }
    loop {
        let Some(pos) = s.find("_{") else { break };
        let after = &s[pos + 2..];
        let Some(e) = after.find('}') else { break };
        let inner = after[..e].to_string();
        let full = format!("_{{{inner}}}");
        s = s.replacen(&full, &to_subscript(&inner), 1);
    }
    s = replace_single_marker(&s, '^', to_superscript_char);
    s = replace_single_marker(&s, '_', to_subscript_char);

    // ── \text{...} → unwrap ───────────────────────────────────────────────
    loop {
        let Some(pos) = s.find("\\text{") else { break };
        let after = &s[pos + 6..];
        let Some(e) = after.find('}') else { break };
        let inner = after[..e].to_string();
        let full = format!("\\text{{{inner}}}");
        s = s.replacen(&full, &inner, 1);
    }

    // ── \mathbb{X} ────────────────────────────────────────────────────────
    for (src, dst) in [
        ("\\mathbb{R}", "ℝ"), ("\\mathbb{N}", "ℕ"), ("\\mathbb{Z}", "ℤ"),
        ("\\mathbb{C}", "ℂ"), ("\\mathbb{Q}", "ℚ"), ("\\mathbb{P}", "ℙ"),
    ] { s = s.replace(src, dst); }

    // ── \vec{x} → x⃗ ──────────────────────────────────────────────────────
    loop {
        let Some(pos) = s.find("\\vec{") else { break };
        let after = &s[pos + 5..];
        let Some(e) = after.find('}') else { break };
        let inner = after[..e].to_string();
        let full = format!("\\vec{{{inner}}}");
        let repl = format!("{inner}\u{20D7}");
        s = s.replacen(&full, &repl, 1);
    }

    // ── Symbol tables (ORDER MATTERS: longer/specific before shorter prefix) ─
    for (src, dst) in GREEK_MAP { s = s.replace(src, dst); }
    for (src, dst) in OPERATOR_MAP { s = s.replace(src, dst); }

    // ── Whitespace / misc cleanup ─────────────────────────────────────────
    s = s.replace("\\\\", " ");
    s = s.replace("\\,", "\u{202F}"); // narrow no-break space
    s = s.replace("\\;", " ");
    s = s.replace("\\quad", "  ");
    s = s.replace("\\qquad", "    ");
    s = s.replace("\\!", "");
    s = s.replace("\\cdot", "·");

    // Strip remaining unrecognised \command tokens
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            while chars.peek().map(|c| c.is_alphabetic()).unwrap_or(false) {
                chars.next();
            }
        } else {
            result.push(c);
        }
    }

    result.replace('{', "(").replace('}', ")")
}

// ── Greek letters (lowercase before uppercase to avoid partial matches) ───────

const GREEK_MAP: &[(&str, &str)] = &[
    ("\\alpha", "α"), ("\\beta", "β"), ("\\gamma", "γ"), ("\\delta", "δ"),
    ("\\varepsilon", "ε"), ("\\epsilon", "ε"), ("\\zeta", "ζ"), ("\\eta", "η"),
    ("\\vartheta", "ϑ"), ("\\theta", "θ"), ("\\iota", "ι"), ("\\kappa", "κ"),
    ("\\lambda", "λ"), ("\\mu", "μ"), ("\\nu", "ν"), ("\\xi", "ξ"),
    ("\\varpi", "ϖ"), ("\\pi", "π"), ("\\varrho", "ϱ"), ("\\rho", "ρ"),
    ("\\varsigma", "ς"), ("\\sigma", "σ"), ("\\tau", "τ"), ("\\upsilon", "υ"),
    ("\\varphi", "φ"), ("\\phi", "φ"), ("\\chi", "χ"), ("\\psi", "ψ"),
    ("\\omega", "ω"),
    ("\\Gamma", "Γ"), ("\\Delta", "Δ"), ("\\Theta", "Θ"), ("\\Lambda", "Λ"),
    ("\\Xi", "Ξ"), ("\\Pi", "Π"), ("\\Sigma", "Σ"), ("\\Upsilon", "Υ"),
    ("\\Phi", "Φ"), ("\\Psi", "Ψ"), ("\\Omega", "Ω"),
];

// ── Operators — CRITICAL: longer/prefixed entries MUST come before shorter ───
// Rule: if A is a prefix of B, B must appear first in this table.
//   \infty, \int, \oint  before  \in
//   \neg                 before  \ne
//   \subseteq            before  \subset
//   \supseteq            before  \supset
//   \lnot                before  \ln  (handled: \lnot appears before \ln below)
const OPERATOR_MAP: &[(&str, &str)] = &[
    // Arrows
    ("\\Rightarrow", "⇒"), ("\\Leftarrow", "⇐"), ("\\Leftrightarrow", "⟺"),
    ("\\rightarrow", "→"), ("\\leftarrow", "←"), ("\\leftrightarrow", "↔"),
    ("\\uparrow", "↑"), ("\\downarrow", "↓"), ("\\Uparrow", "⇑"), ("\\Downarrow", "⇓"),
    ("\\nearrow", "↗"), ("\\searrow", "↘"), ("\\mapsto", "↦"),
    ("\\implies", "⟹"), ("\\iff", "⟺"), ("\\to", "→"), ("\\gets", "←"),
    // Relations (longer forms before shorter prefix forms)
    ("\\leq", "≤"), ("\\geq", "≥"), ("\\le", "≤"), ("\\ge", "≥"),
    ("\\neq", "≠"), ("\\neg", "¬"), ("\\ne", "≠"),   // \neg before \ne ← fix
    ("\\approx", "≈"), ("\\equiv", "≡"), ("\\sim", "∼"), ("\\simeq", "≃"),
    ("\\cong", "≅"), ("\\propto", "∝"), ("\\ll", "≪"), ("\\gg", "≫"),
    // Set / logic — \infty, \int, \oint BEFORE \in  ← fix
    ("\\infty", "∞"), ("\\oint", "∮"), ("\\int", "∫"),   // all before \in
    ("\\notin", "∉"), ("\\ni", "∋"), ("\\in", "∈"),       // \notin/\ni before \in
    ("\\subseteq", "⊆"), ("\\supseteq", "⊇"),             // before \subset/\supset ← fix
    ("\\subset", "⊂"), ("\\supset", "⊃"),
    ("\\cup", "∪"), ("\\cap", "∩"), ("\\emptyset", "∅"), ("\\varnothing", "∅"),
    ("\\forall", "∀"), ("\\exists", "∃"), ("\\nexists", "∄"),
    ("\\land", "∧"), ("\\lor", "∨"), ("\\lnot", "¬"),     // \lnot before \ln below ← fix
    ("\\oplus", "⊕"), ("\\otimes", "⊗"), ("\\ominus", "⊖"),
    // Arithmetic
    ("\\times", "×"), ("\\div", "÷"), ("\\pm", "±"), ("\\mp", "∓"),
    ("\\cdots", "⋯"), ("\\ldots", "…"), ("\\vdots", "⋮"), ("\\ddots", "⋱"),
    // Calculus — \lim, \ln, \log, \sum, \prod (after \lnot is already done above)
    ("\\partial", "∂"), ("\\nabla", "∇"),
    ("\\sum", "∑"), ("\\prod", "∏"),
    ("\\lim", "lim"), ("\\ln", "ln"), ("\\log", "log"), ("\\exp", "exp"),
    ("\\sin", "sin"), ("\\cos", "cos"), ("\\tan", "tan"),
    // Misc symbols
    ("\\hbar", "ℏ"), ("\\ell", "ℓ"), ("\\Re", "ℜ"), ("\\Im", "ℑ"),
    ("\\aleph", "ℵ"), ("\\wp", "℘"),
    ("\\angle", "∠"), ("\\perp", "⊥"), ("\\parallel", "∥"),
    ("\\therefore", "∴"), ("\\because", "∵"),
    ("\\checkmark", "✓"), ("\\dagger", "†"), ("\\ddagger", "‡"),
    ("\\langle", "⟨"), ("\\rangle", "⟩"),
    ("\\lfloor", "⌊"), ("\\rfloor", "⌋"),
    ("\\lceil", "⌈"), ("\\rceil", "⌉"),
];

// ── Unicode super/subscript helpers ──────────────────────────────────────────

fn to_superscript(s: &str) -> String { s.chars().map(to_superscript_char).collect() }
fn to_subscript(s: &str) -> String   { s.chars().map(to_subscript_char).collect() }

fn to_superscript_char(c: char) -> char {
    match c {
        '0'=>'⁰','1'=>'¹','2'=>'²','3'=>'³','4'=>'⁴',
        '5'=>'⁵','6'=>'⁶','7'=>'⁷','8'=>'⁸','9'=>'⁹',
        '+'=>'⁺','-'=>'⁻','='=>'⁼','('=>'⁽',')'=>'⁾',
        'n'=>'ⁿ','i'=>'ⁱ','a'=>'ᵃ','b'=>'ᵇ','c'=>'ᶜ','d'=>'ᵈ',
        'e'=>'ᵉ','f'=>'ᶠ','g'=>'ᵍ','h'=>'ʰ','j'=>'ʲ','k'=>'ᵏ',
        'l'=>'ˡ','m'=>'ᵐ','o'=>'ᵒ','p'=>'ᵖ','r'=>'ʳ','s'=>'ˢ',
        't'=>'ᵗ','u'=>'ᵘ','v'=>'ᵛ','w'=>'ʷ','x'=>'ˣ','y'=>'ʸ','z'=>'ᶻ',
        other => other,
    }
}

fn to_subscript_char(c: char) -> char {
    match c {
        '0'=>'₀','1'=>'₁','2'=>'₂','3'=>'₃','4'=>'₄',
        '5'=>'₅','6'=>'₆','7'=>'₇','8'=>'₈','9'=>'₉',
        '+'=>'₊','-'=>'₋','='=>'₌','('=>'₍',')'=>'₎',
        'a'=>'ₐ','e'=>'ₑ','i'=>'ᵢ','o'=>'ₒ','r'=>'ᵣ',
        'u'=>'ᵤ','v'=>'ᵥ','x'=>'ₓ','n'=>'ₙ','m'=>'ₘ',
        other => other,
    }
}

fn replace_single_marker(s: &str, marker: char, map: fn(char) -> char) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == marker {
            match chars.peek() {
                Some(&next) if !next.is_whitespace() && next != '{' && next != '(' => {
                    out.push(map(next));
                    chars.next();
                }
                _ => out.push(c),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn hash_str(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
