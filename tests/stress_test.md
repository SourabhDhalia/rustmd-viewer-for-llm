# 🔍 Advanced Markdown Stress Test

> **Purpose:** Exercise every rendering path — text formatting, math, code, tables, lists, Unicode, footnotes, and edge cases.

---

## 1. Text Formatting

**Bold**, _italic_, ~~strikethrough~~, `inline code`, ***bold italic***, ___bold italic alt___.

Combined: **_bold italic_**, ~~**bold strikethrough**~~, `~~code not struck~~`.

---

## 2. Block Quotes

> Simple blockquote.

> **Nested:**
> > Level 2 blockquote.
> > > Level 3 blockquote.

> Multi-paragraph blockquote.
>
> Second paragraph inside the same quote.

---

## 3. Code Blocks (Syntax Highlighted)

```rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    let msg = greet("India");
    println!("{}", msg);
}
```

```python
from math import pi, e, sqrt

def euler_identity() -> complex:
    return e**(1j * pi) + 1  # Should equal ~0

print(f"π ≈ {pi:.10f}")
print(f"e ≈ {e:.10f}")
print(f"√2 ≈ {sqrt(2):.10f}")
```

```json
{
  "country": "India",
  "population": 1440000000,
  "math": "e^{iπ} + 1 = 0",
  "symbols": ["π", "∞", "∑", "∫", "√", "Δ", "Ω"]
}
```

```bash
echo "Hello World"
cargo build --release 2>&1 | grep -E "error|warning"
```

---

## 4. Math — Inline (LaTeX → Unicode Converted)

Greek: $\alpha$, $\beta$, $\gamma$, $\delta$, $\epsilon$, $\zeta$, $\eta$, $\theta$, $\lambda$, $\mu$, $\pi$, $\sigma$, $\omega$

Uppercase: $\Delta$, $\Gamma$, $\Omega$, $\Sigma$, $\Pi$, $\Phi$, $\Psi$

Operators: $\sum_{i=0}^{n}$, $\int_{a}^{b}$, $\sqrt{2}$, $\sqrt[3]{x}$

Relations: $\leq$, $\geq$, $\neq$, $\approx$, $\equiv$, $\infty$

Arrows: $\rightarrow$, $\leftarrow$, $\Rightarrow$, $\Leftarrow$, $\leftrightarrow$

Sets: $\in$, $\notin$, $\subset$, $\cup$, $\cap$, $\forall$, $\exists$

Logic: $\land$, $\lor$, $\neg$, $\oplus$, $\otimes$

Number sets: $\mathbb{R}$, $\mathbb{N}$, $\mathbb{Z}$, $\mathbb{C}$

Fractions: $\frac{1}{2}$, $\frac{\pi}{e}$, $\frac{a+b}{c-d}$

Euler's identity: $e^{i\pi} + 1 = 0$

GDP approximation: $GDP \approx \sum C_i$

---

## 5. Math — Display Block

$$
f(x) = \int_{0}^{1} x^t \, dt = \frac{x - 1}{\ln(x)}
$$

$$
\begin{aligned}
E &= mc^2 \\
F &= ma \\
PV &= nRT
\end{aligned}
$$

$$
\lim_{x \to \infty} \frac{1}{x} = 0
$$

---

## 6. Tables

### Simple Table
| Name    | Country | Score |
|---------|---------|-------|
| Alice   | India   | 98    |
| Bob     | USA     | 87    |
| Charlie | UK      | 92    |

### Aligned Table
| Left      | Center      | Right |
|:----------|:-----------:|------:|
| `code`    | **bold**    | 1.0   |
| _italic_  | ~~struck~~  | 2.5   |
| plain     | mixed       | 100   |

---

## 7. Lists

### Unordered (Nested)
- Level 1 item A
  - Level 2 item A.1
    - Level 3 item A.1.a
    - Level 3 item A.1.b
  - Level 2 item A.2
- Level 1 item B
- Level 1 item C

### Ordered (Nested)
1. First
   1. First sub
   2. Second sub
2. Second
3. Third

### Task List
- [x] Implement basic rendering
- [x] Add syntax highlighting
- [x] Fix double scroll
- [ ] Full LaTeX rendering
- [ ] Mermaid diagrams
- [ ] Export to HTML

---

## 8. Footnotes

India is the world's largest democracy.[^democracy]

It has a GDP of approximately $3.7 trillion.[^gdp]

[^democracy]: As of 2024, India holds the largest democratic elections in human history.
[^gdp]: World Bank nominal GDP figure for 2023.

---

## 9. Unicode — Emojis & Symbols

**Emojis:** ✨ 🌍 🇮🇳 📊 📈 🔢 📐 ⚙️ 🏭 🚀 🏆 💡 🔬 🔭

**Arrows (Unicode, no LaTeX):** ⇀ ↔ ⇋ ⇒ ➝ ← → ↑ ↓ ↗ ↘ ⟹ ⟺

**Math (plain Unicode):** π ∞ ∑ ∫ √ Δ Ω ≈ ≠ ≤ ≥ ∈ ∉ ∀ ∃ ⊂ ∪ ∩ ∧ ∨ ¬ ⊕

**Diacritics:** É ß ę ñ ü ö ä ç â ê î ô û

**Typographic:** © ™ ® • … — – « » „ " " ' '

---

## 10. Code Inline Edge Cases

`$not_math$` vs actual math: $\pi$

`**not bold**` inside code stays literal.

HTML entity display: `&nbsp;` `&copy;` `&trade;` `&amp;`

Escaped chars: \*not italic\*, \**not bold\**, \`not code\`

---

## 11. Horizontal Rules

Three hyphens:

---

Three asterisks:

***

Three underscores:

___

---

## 12. Links & Autolinks

[Named link](https://www.rust-lang.org)

[Link with title](https://www.rust-lang.org "Rust Language")

Autolink: <https://example.com>

Reference link: [Rust][rust-ref]

[rust-ref]: https://www.rust-lang.org

---

## 13. Special Edge Cases

Empty blockquote:
>

Consecutive code fences:

```
outer block
```

Deeply nested blockquote + list:
> - item one
>   - nested
> - item two

Long unbroken word (no-wrap test):
`aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`

---

## 14. Stress: All Inline Formatting Together

**Bold _italic_ ~~struck~~ `code`** end.

_Italic **bold** ~~struck~~ `code`_ end.

~~Struck **bold** _italic_ `code`~~ end.

`code **not bold** _not italic_ ~~not struck~~` end.

---

> **Rendering Score Card**
>
> | Feature              | Status       |
> |----------------------|--------------|
> | Bold / Italic        | ✅ Native    |
> | Strikethrough        | ✅ Native    |
> | Code highlighting    | ✅ syntect   |
> | Tables               | ✅ Native    |
> | Footnotes            | ✅ Native    |
> | Task lists           | ✅ Native    |
> | Unicode / Emojis     | ✅ Native    |
> | LaTeX inline ($)     | ⚠️ Preprocessed → Unicode |
> | LaTeX block ($$)     | ⚠️ Shown as code block   |
> | Mermaid diagrams     | ❌ Not supported          |
> | Raw HTML             | ❌ Stripped               |
