use std::collections::HashMap;

use crate::design::Module;

// ─── Public API ───────────────────────────────────────────────────────────────

/// Resolved parameter environment: name → integer value.
#[derive(Debug, Clone, Default)]
pub struct ParamEnv {
    values: HashMap<String, i64>,
}

impl ParamEnv {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a ParamEnv from a module's default parameter declarations.
    /// Earlier params are visible to later ones (sequential evaluation).
    pub fn from_module(module: &Module) -> Self {
        let mut env = Self::new();
        for p in &module.params {
            if let Some(val) = evaluate_expr(p.value.trim(), &env.values) {
                env.values.insert(p.name.clone(), val);
            }
        }
        env
    }

    /// Apply overrides (e.g. from -P or parent instance bindings).
    pub fn with_overrides(mut self, overrides: &[(String, i64)]) -> Self {
        for (k, v) in overrides {
            self.values.insert(k.clone(), *v);
        }
        self
    }

    pub fn get(&self, name: &str) -> Option<i64> {
        self.values.get(name).copied()
    }

    pub fn as_map(&self) -> &HashMap<String, i64> {
        &self.values
    }
}

/// Evaluate a SV constant-expression string against a parameter map.
/// Returns None if any identifier is unresolvable or the expression is malformed.
pub fn evaluate_expr(expr: &str, params: &HashMap<String, i64>) -> Option<i64> {
    let tokens = tokenize(expr.trim())?;
    let mut pos = 0usize;
    let result = parse_compare(&tokens, &mut pos, params)?;
    if pos == tokens.len() {
        Some(result)
    } else {
        None // leftover tokens
    }
}

/// $clog2 — ceiling of log2, minimum 1
pub fn evaluate_clog2(n: i64) -> i64 {
    if n <= 1 {
        1
    } else {
        let bits = 64 - (n - 1).leading_zeros();
        bits as i64
    }
}

// ─── Tokenizer ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(i64),
    Ident(String),
    SysFn(String), // $clog2 etc.
    // two-char operators
    LShift,   // <<
    RShift,   // >>
    Power,    // **
    Le,       // <=
    Ge,       // >=
    Eq,       // ==
    Ne,       // !=
    // single-char operators / punctuation
    Lt,       // <
    Gt,       // >
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Tilde,
    LParen,
    RParen,
    Comma,
    Colon,
}

fn tokenize(s: &str) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\r' | '\n' => {
                i += 1;
            }
            '0'..='9' => {
                let start = i;
                // Consume digits and common SV literal chars ('d, 'h, etc.)
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric()
                        || chars[i] == '\''
                        || chars[i] == '_'
                        || chars[i] == 'x'
                        || chars[i] == 'z'
                        || chars[i] == 'X'
                        || chars[i] == 'Z')
                {
                    i += 1;
                }
                let raw: String = chars[start..i].iter().collect();
                let val = parse_sv_literal(&raw)?;
                tokens.push(Token::Num(val));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(name));
            }
            '$' => {
                i += 1;
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                tokens.push(Token::SysFn(name));
            }
            '<' if i + 1 < chars.len() && chars[i + 1] == '<' => {
                tokens.push(Token::LShift);
                i += 2;
            }
            '<' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Le);
                i += 2;
            }
            '>' if i + 1 < chars.len() && chars[i + 1] == '>' => {
                tokens.push(Token::RShift);
                i += 2;
            }
            '>' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Ge);
                i += 2;
            }
            '=' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Eq);
                i += 2;
            }
            '!' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Ne);
                i += 2;
            }
            '*' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                tokens.push(Token::Power);
                i += 2;
            }
            '<' => { tokens.push(Token::Lt);      i += 1; }
            '>' => { tokens.push(Token::Gt);      i += 1; }
            '+' => { tokens.push(Token::Plus);    i += 1; }
            '-' => { tokens.push(Token::Minus);   i += 1; }
            '*' => { tokens.push(Token::Star);    i += 1; }
            '/' => { tokens.push(Token::Slash);   i += 1; }
            '%' => { tokens.push(Token::Percent); i += 1; }
            '&' => { tokens.push(Token::Amp);     i += 1; }
            '|' => { tokens.push(Token::Pipe);    i += 1; }
            '^' => { tokens.push(Token::Caret);   i += 1; }
            '~' => { tokens.push(Token::Tilde);   i += 1; }
            '(' => { tokens.push(Token::LParen);  i += 1; }
            ')' => { tokens.push(Token::RParen);  i += 1; }
            ',' => { tokens.push(Token::Comma);   i += 1; }
            ':' => { tokens.push(Token::Colon);   i += 1; }
            _ => { i += 1; } // skip unknown (e.g. '?' in ternary — not supported)
        }
    }

    Some(tokens)
}

/// Parse a SV numeric literal: 123, 8'd255, 8'hFF, 8'b1010, 'b1, 'h1F, 1'b1, etc.
fn parse_sv_literal(s: &str) -> Option<i64> {
    if let Some(tick) = s.find('\'') {
        let after = &s[tick + 1..];
        if after.is_empty() {
            return None;
        }
        let radix_char = after.chars().next()?;
        let digits: String = after[1..]
            .chars()
            .filter(|c| *c != '_')
            .collect();
        match radix_char.to_ascii_lowercase() {
            'd' => i64::from_str_radix(&digits, 10).ok(),
            'h' => i64::from_str_radix(&digits, 16).ok(),
            'b' => i64::from_str_radix(&digits, 2).ok(),
            'o' => i64::from_str_radix(&digits, 8).ok(),
            _ => None,
        }
    } else {
        let clean: String = s.chars().filter(|c| *c != '_').collect();
        clean.parse().ok()
    }
}

// ─── Recursive-descent parser ─────────────────────────────────────────────────
// Precedence (low → high):
//   |   ^   &   shift   add   mul   power   unary   primary

/// Handles relational and equality operators (lowest precedence above bitwise).
fn parse_compare(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    let mut v = parse_or(tokens, pos, params)?;
    loop {
        match peek(tokens, *pos) {
            Some(Token::Lt) => { *pos += 1; let r = parse_or(tokens, pos, params)?; v = (v < r) as i64; }
            Some(Token::Gt) => { *pos += 1; let r = parse_or(tokens, pos, params)?; v = (v > r) as i64; }
            Some(Token::Le) => { *pos += 1; let r = parse_or(tokens, pos, params)?; v = (v <= r) as i64; }
            Some(Token::Ge) => { *pos += 1; let r = parse_or(tokens, pos, params)?; v = (v >= r) as i64; }
            Some(Token::Eq) => { *pos += 1; let r = parse_or(tokens, pos, params)?; v = (v == r) as i64; }
            Some(Token::Ne) => { *pos += 1; let r = parse_or(tokens, pos, params)?; v = (v != r) as i64; }
            _ => break,
        }
    }
    Some(v)
}

fn parse_or(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    let mut v = parse_xor(tokens, pos, params)?;
    while peek(tokens, *pos) == Some(&Token::Pipe) {
        *pos += 1;
        v |= parse_xor(tokens, pos, params)?;
    }
    Some(v)
}

fn parse_xor(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    let mut v = parse_and(tokens, pos, params)?;
    while peek(tokens, *pos) == Some(&Token::Caret) {
        *pos += 1;
        v ^= parse_and(tokens, pos, params)?;
    }
    Some(v)
}

fn parse_and(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    let mut v = parse_shift(tokens, pos, params)?;
    while peek(tokens, *pos) == Some(&Token::Amp) {
        *pos += 1;
        v &= parse_shift(tokens, pos, params)?;
    }
    Some(v)
}

fn parse_shift(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    let mut v = parse_add(tokens, pos, params)?;
    loop {
        match peek(tokens, *pos) {
            Some(Token::LShift) => {
                *pos += 1;
                let n = parse_add(tokens, pos, params)?;
                v = v.checked_shl(n.min(63) as u32)?;
            }
            Some(Token::RShift) => {
                *pos += 1;
                let n = parse_add(tokens, pos, params)?;
                v = v.checked_shr(n.min(63) as u32)?;
            }
            _ => break,
        }
    }
    Some(v)
}

fn parse_add(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    let mut v = parse_mul(tokens, pos, params)?;
    loop {
        match peek(tokens, *pos) {
            Some(Token::Plus) => {
                *pos += 1;
                v = v.checked_add(parse_mul(tokens, pos, params)?)?;
            }
            Some(Token::Minus) => {
                *pos += 1;
                v = v.checked_sub(parse_mul(tokens, pos, params)?)?;
            }
            _ => break,
        }
    }
    Some(v)
}

fn parse_mul(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    let mut v = parse_power(tokens, pos, params)?;
    loop {
        match peek(tokens, *pos) {
            Some(Token::Star) => {
                *pos += 1;
                v = v.checked_mul(parse_power(tokens, pos, params)?)?;
            }
            Some(Token::Slash) => {
                *pos += 1;
                let d = parse_power(tokens, pos, params)?;
                if d == 0 { return None; }
                v /= d;
            }
            Some(Token::Percent) => {
                *pos += 1;
                let d = parse_power(tokens, pos, params)?;
                if d == 0 { return None; }
                v %= d;
            }
            _ => break,
        }
    }
    Some(v)
}

fn parse_power(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    let base = parse_unary(tokens, pos, params)?;
    if peek(tokens, *pos) == Some(&Token::Power) {
        *pos += 1;
        let exp = parse_unary(tokens, pos, params)?;
        if exp < 0 { return None; }
        base.checked_pow(exp as u32)
    } else {
        Some(base)
    }
}

fn parse_unary(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    match peek(tokens, *pos) {
        Some(Token::Minus) => {
            *pos += 1;
            parse_unary(tokens, pos, params).map(|v| -v)
        }
        Some(Token::Tilde) => {
            *pos += 1;
            parse_unary(tokens, pos, params).map(|v| !v)
        }
        _ => parse_primary(tokens, pos, params),
    }
}

fn parse_primary(tokens: &[Token], pos: &mut usize, params: &HashMap<String, i64>) -> Option<i64> {
    match tokens.get(*pos)?.clone() {
        Token::Num(n) => {
            *pos += 1;
            Some(n)
        }
        Token::Ident(name) => {
            *pos += 1;
            params.get(name.as_str()).copied()
        }
        Token::SysFn(name) => {
            *pos += 1;
            match name.to_lowercase().as_str() {
                "clog2" => {
                    expect(tokens, pos, &Token::LParen)?;
                    let arg = parse_or(tokens, pos, params)?;
                    expect(tokens, pos, &Token::RParen)?;
                    Some(evaluate_clog2(arg))
                }
                "bits" | "size" | "width" => {
                    // $bits(expr) — not fully evaluable here; skip args and return None
                    None
                }
                _ => None,
            }
        }
        Token::LParen => {
            *pos += 1;
            let v = parse_or(tokens, pos, params)?;
            expect(tokens, pos, &Token::RParen)?;
            Some(v)
        }
        _ => None,
    }
}

fn peek(tokens: &[Token], pos: usize) -> Option<&Token> {
    tokens.get(pos)
}

fn expect(tokens: &[Token], pos: &mut usize, expected: &Token) -> Option<()> {
    if tokens.get(*pos) == Some(expected) {
        *pos += 1;
        Some(())
    } else {
        None
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn params(pairs: &[(&str, i64)]) -> HashMap<String, i64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_literals() {
        let p = HashMap::new();
        assert_eq!(evaluate_expr("8", &p), Some(8));
        assert_eq!(evaluate_expr("255", &p), Some(255));
        assert_eq!(evaluate_expr("8'd255", &p), Some(255));
        assert_eq!(evaluate_expr("8'hFF", &p), Some(255));
        assert_eq!(evaluate_expr("8'b11111111", &p), Some(255));
        assert_eq!(evaluate_expr("1'b0", &p), Some(0));
        assert_eq!(evaluate_expr("1'b1", &p), Some(1));
    }

    #[test]
    fn test_arithmetic() {
        let p = HashMap::new();
        assert_eq!(evaluate_expr("3 + 4", &p), Some(7));
        assert_eq!(evaluate_expr("10 - 3", &p), Some(7));
        assert_eq!(evaluate_expr("3 * 4", &p), Some(12));
        assert_eq!(evaluate_expr("12 / 4", &p), Some(3));
        assert_eq!(evaluate_expr("10 % 3", &p), Some(1));
        assert_eq!(evaluate_expr("2 ** 8", &p), Some(256));
    }

    #[test]
    fn test_shifts() {
        let p = HashMap::new();
        assert_eq!(evaluate_expr("1 << 3", &p), Some(8));
        assert_eq!(evaluate_expr("16 >> 2", &p), Some(4));
    }

    #[test]
    fn test_bitwise() {
        let p = HashMap::new();
        assert_eq!(evaluate_expr("0xF0 & 0xFF", &p), None); // 0x prefix not supported
        assert_eq!(evaluate_expr("8'hF0 & 8'hFF", &p), Some(0xF0));
        assert_eq!(evaluate_expr("4'b1010 | 4'b0101", &p), Some(0xF));
        assert_eq!(evaluate_expr("4'b1010 ^ 4'b1100", &p), Some(6));
    }

    #[test]
    fn test_params() {
        let p = params(&[("WIDTH", 8), ("DEPTH", 16)]);
        assert_eq!(evaluate_expr("WIDTH", &p), Some(8));
        assert_eq!(evaluate_expr("WIDTH - 1", &p), Some(7));
        assert_eq!(evaluate_expr("WIDTH * DEPTH", &p), Some(128));
        assert_eq!(evaluate_expr("WIDTH + 0", &p), Some(8));
    }

    #[test]
    fn test_clog2() {
        let p = params(&[("DEPTH", 16), ("N", 256)]);
        assert_eq!(evaluate_expr("$clog2(16)", &p), Some(4));
        assert_eq!(evaluate_expr("$clog2(256)", &p), Some(8));
        assert_eq!(evaluate_expr("$clog2(DEPTH)", &p), Some(4));
        assert_eq!(evaluate_expr("$clog2(N)", &p), Some(8));
        // Edge cases
        assert_eq!(evaluate_clog2(1), 1);
        assert_eq!(evaluate_clog2(2), 1);
        assert_eq!(evaluate_clog2(3), 2);
        assert_eq!(evaluate_clog2(4), 2);
        assert_eq!(evaluate_clog2(5), 3);
    }

    #[test]
    fn test_nested_expr() {
        let p = params(&[("WIDTH", 8), ("ADDR_W", 4)]);
        // [ADDR_W:0] → width = ADDR_W + 1 = 5
        assert_eq!(evaluate_expr("ADDR_W + 1", &p), Some(5));
        // [WIDTH-1:0] → width = WIDTH - 1 - 0 + 1 = WIDTH = 8
        assert_eq!(evaluate_expr("WIDTH - 1", &p), Some(7));
    }

    #[test]
    fn test_unresolvable() {
        let p = HashMap::new();
        // Unknown identifier returns None
        assert_eq!(evaluate_expr("UNKNOWN", &p), None);
        assert_eq!(evaluate_expr("WIDTH - 1", &p), None);
    }

    #[test]
    fn test_clog2_addr_w() {
        // ADDR_W = $clog2(DEPTH) where DEPTH=16 → 4
        let p = params(&[("DEPTH", 16)]);
        assert_eq!(evaluate_expr("$clog2(DEPTH)", &p), Some(4));
    }

    #[test]
    fn test_comparison_operators() {
        let p = params(&[("N", 4), ("I", 3)]);
        // relational
        assert_eq!(evaluate_expr("3 < 4", &p),  Some(1));
        assert_eq!(evaluate_expr("4 < 3", &p),  Some(0));
        assert_eq!(evaluate_expr("3 <= 3", &p), Some(1));
        assert_eq!(evaluate_expr("3 >= 4", &p), Some(0));
        assert_eq!(evaluate_expr("4 > 3", &p),  Some(1));
        // equality
        assert_eq!(evaluate_expr("3 == 3", &p), Some(1));
        assert_eq!(evaluate_expr("3 != 4", &p), Some(1));
        assert_eq!(evaluate_expr("3 == 4", &p), Some(0));
        // with params
        assert_eq!(evaluate_expr("I < N", &p),  Some(1));
        assert_eq!(evaluate_expr("I < N - 1", &p), Some(0)); // 3 < 3 == 0
        assert_eq!(evaluate_expr("I <= N - 1", &p), Some(1)); // 3 <= 3 == 1
    }
}
