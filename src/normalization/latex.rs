use std::sync::LazyLock;

use regex::{Captures, Regex};

/// Converts LeetCode's plain math notation to inline LaTeX, e.g.
/// `0 <= nums[i] <= 10^9` → `$0 \leq \mathtt{nums[i]} \leq 10^{9}$`.
///
/// Deterministic on purpose: the model only decides *whether* to convert, so it can't alter a number.
/// ponytail: covers LeetCode's constraint notation only (`<= >= != == * % ^`, identifiers, indexing);
/// anything else passes through as written.
pub(super) fn to_latex(expr: &str) -> String {
    static TOKEN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(concat!(
            r"\s*(?<op><=|>=|!=|==|\*|%)\s*",
            r"|\^(?<sup>\{[^}]*\}|[A-Za-z0-9]+)",
            // An identifier with any member access / indexing: `n`, `nums`, `nums.length`, `grid[i][j]`.
            r"|(?<code>[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*|\[[^\]]*\])*)",
            r"|(?<special>[$&#_{}])",
        ))
        .expect("valid LaTeX token regex")
    });

    let body = TOKEN.replace_all(expr.trim(), |caps: &Captures| {
        if let Some(op) = caps.name("op") {
            let op = match op.as_str() {
                "<=" => r"\leq",
                ">=" => r"\geq",
                "!=" => r"\neq",
                "==" => "=",
                "*" => r"\times",
                _ => r"\bmod",
            };
            format!(" {op} ")
        } else if let Some(sup) = caps.name("sup") {
            let sup = sup.as_str();
            if sup.starts_with('{') { format!("^{sup}") } else { format!("^{{{sup}}}") }
        } else if let Some(code) = caps.name("code") {
            let code = code.as_str();
            // A lone letter is a math variable (`n`, the `i` in `i^{th}`); anything longer is code.
            if code.len() == 1 { code.to_owned() } else { format!(r"\mathtt{{{}}}", escape(code)) }
        } else {
            escape(&caps[0])
        }
    });

    format!("${}$", body.trim())
}

fn escape(text: &str) -> String {
    text.chars().fold(String::with_capacity(text.len()), |mut out, ch| {
        if matches!(ch, '$' | '&' | '#' | '_' | '%' | '{' | '}') {
            out.push('\\');
        }
        out.push(ch);
        out
    })
}

#[cfg(test)]
mod tests {
    use super::to_latex;

    #[test]
    fn converts_leetcode_notation() {
        for (input, expected) in [
            ("i^{th}", "$i^{th}$"),
            ("2 * 10^{5}", r"$2 \times 10^{5}$"),
            ("1 <= n, m <= 2 * 10^{5}", r"$1 \leq n, m \leq 2 \times 10^{5}$"),
            ("1<=n<=10^9", r"$1 \leq n \leq 10^{9}$"),
            ("2^{31} - 1", "$2^{31} - 1$"),
            ("0 <= nums[i] <= 10^9", r"$0 \leq \mathtt{nums[i]} \leq 10^{9}$"),
            ("1 <= nums.length <= 100", r"$1 \leq \mathtt{nums.length} \leq 100$"),
            ("0 <= queries[i][0] < n", r"$0 \leq \mathtt{queries[i][0]} < n$"),
            ("s[i] != t[i]", r"$\mathtt{s[i]} \neq \mathtt{t[i]}$"),
            ("nums[i] % 2 == 0", r"$\mathtt{nums[i]} \bmod 2 = 0$"),
            ("1 <= max_len <= 50", r"$1 \leq \mathtt{max\_len} \leq 50$"),
            ("x >= {1, 2}", r"$x \geq \{1, 2\}$"),
        ] {
            assert_eq!(to_latex(input), expected, "{input:?}");
        }
    }
}
