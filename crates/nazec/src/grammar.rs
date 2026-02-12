//! Pest-to-GBNF grammar converter for AI constrained decoding.
//!
//! Reads the embedded `naze.pest` grammar, parses its PEG rules,
//! and converts them to GBNF (GGML BNF) format used by llama.cpp,
//! XGrammar, and SGLang for constrained token generation.

use crate::cli::GrammarFormat;

/// The Naze PEG grammar source, embedded at compile time.
const PEST_SOURCE: &str = include_str!("../../naze-parser/src/naze.pest");

// ---------------------------------------------------------------------------
// Pest grammar AST
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum RuleType {
    Normal,         // = { ... }
    Atomic,         // = @{ ... }
    Silent,         // = _{ ... }
    CompoundAtomic, // = ${ ... }
}

#[derive(Debug, Clone)]
struct PestRule {
    name: String,
    rule_type: RuleType,
    body: PestExpr,
}

#[derive(Debug, Clone, PartialEq)]
enum RepeatKind {
    ZeroOrMore,        // *
    OneOrMore,         // +
    Optional,          // ?
    Range(usize, Option<usize>), // {m,n} or {m,} or {m}
}

#[derive(Debug, Clone)]
enum PestExpr {
    Seq(Vec<PestExpr>),
    Choice(Vec<PestExpr>),
    Literal(String),
    Ident(String),
    Repeat(Box<PestExpr>, RepeatKind),
    Group(Box<PestExpr>),
    Neg(Box<PestExpr>),
}

// ---------------------------------------------------------------------------
// Pest grammar parser (recursive descent)
// ---------------------------------------------------------------------------

struct PestParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> PestParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.pos < self.input.len() {
                let ch = self.input.as_bytes()[self.pos];
                if ch == b' ' || ch == b'\t' || ch == b'\r' || ch == b'\n' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            // Skip // line comments
            if self.remaining().starts_with("//") {
                while self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b'\n' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn eat_char(&mut self, expected: char) -> bool {
        if self.remaining().starts_with(expected) {
            self.pos += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn eat_str(&mut self, s: &str) -> bool {
        if self.remaining().starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn parse_ident(&mut self) -> Option<String> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let ch = self.input.as_bytes()[self.pos];
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos > start {
            Some(self.input[start..self.pos].to_string())
        } else {
            None
        }
    }

    fn parse_string_literal(&mut self) -> Option<String> {
        if !self.eat_char('"') {
            return None;
        }
        let mut s = String::new();
        while self.pos < self.input.len() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b'"' {
                self.pos += 1;
                return Some(s);
            } else if ch == b'\\' {
                self.pos += 1;
                if self.pos < self.input.len() {
                    let esc = self.input.as_bytes()[self.pos];
                    match esc {
                        b'n' => s.push('\n'),
                        b'r' => s.push('\r'),
                        b't' => s.push('\t'),
                        b'\\' => s.push('\\'),
                        b'"' => s.push('"'),
                        b'{' => s.push('{'),
                        _ => {
                            s.push('\\');
                            s.push(esc as char);
                        }
                    }
                    self.pos += 1;
                }
            } else {
                s.push(ch as char);
                self.pos += 1;
            }
        }
        Some(s)
    }

    fn parse_repeat_range(&mut self) -> Option<RepeatKind> {
        if !self.eat_char('{') {
            return None;
        }
        self.skip_ws_and_comments();
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let m: usize = self.input[start..self.pos].parse().unwrap_or(0);
        self.skip_ws_and_comments();
        if self.eat_char(',') {
            self.skip_ws_and_comments();
            let start2 = self.pos;
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            let n_str = &self.input[start2..self.pos];
            self.skip_ws_and_comments();
            self.eat_char('}');
            if n_str.is_empty() {
                Some(RepeatKind::Range(m, None))
            } else {
                Some(RepeatKind::Range(m, Some(n_str.parse().unwrap_or(m))))
            }
        } else {
            self.eat_char('}');
            Some(RepeatKind::Range(m, Some(m)))
        }
    }

    /// Parse a single atom: literal, ident, group, or negation
    fn parse_atom(&mut self) -> Option<PestExpr> {
        self.skip_ws_and_comments();

        // Negation
        if self.eat_char('!') {
            let inner = self.parse_atom()?;
            return Some(PestExpr::Neg(Box::new(inner)));
        }

        // Positive lookahead (treat as no-op / pass through inner)
        if self.eat_char('&') {
            return self.parse_atom();
        }

        // Parenthesized group
        if self.eat_char('(') {
            let expr = self.parse_choice()?;
            self.skip_ws_and_comments();
            self.eat_char(')');
            return Some(PestExpr::Group(Box::new(expr)));
        }

        // String literal
        if self.peek_char() == Some('"') {
            let s = self.parse_string_literal()?;
            return Some(PestExpr::Literal(s));
        }

        // Identifier or builtin
        if let Some(id) = self.parse_ident() {
            return Some(PestExpr::Ident(id));
        }

        None
    }

    /// Parse an atom with optional postfix repetition
    fn parse_postfix(&mut self) -> Option<PestExpr> {
        let mut expr = self.parse_atom()?;
        loop {
            self.skip_ws_and_comments();
            if self.eat_char('*') {
                expr = PestExpr::Repeat(Box::new(expr), RepeatKind::ZeroOrMore);
            } else if self.eat_char('+') {
                expr = PestExpr::Repeat(Box::new(expr), RepeatKind::OneOrMore);
            } else if self.eat_char('?') {
                expr = PestExpr::Repeat(Box::new(expr), RepeatKind::Optional);
            } else if self.peek_char() == Some('{') {
                // Could be repeat range {m,n} — but only if followed by digits
                let saved = self.pos;
                if let Some(rk) = self.parse_repeat_range() {
                    expr = PestExpr::Repeat(Box::new(expr), rk);
                } else {
                    self.pos = saved;
                    break;
                }
            } else {
                break;
            }
        }
        Some(expr)
    }

    /// Parse a sequence: atoms joined by `~`
    fn parse_sequence(&mut self) -> Option<PestExpr> {
        let mut items = vec![self.parse_postfix()?];
        loop {
            self.skip_ws_and_comments();
            if self.eat_char('~') {
                if let Some(next) = self.parse_postfix() {
                    items.push(next);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if items.len() == 1 {
            Some(items.pop().unwrap())
        } else {
            Some(PestExpr::Seq(items))
        }
    }

    /// Parse a choice: sequences joined by `|`
    fn parse_choice(&mut self) -> Option<PestExpr> {
        let mut alts = vec![self.parse_sequence()?];
        loop {
            self.skip_ws_and_comments();
            if self.eat_char('|') {
                if let Some(next) = self.parse_sequence() {
                    alts.push(next);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if alts.len() == 1 {
            Some(alts.pop().unwrap())
        } else {
            Some(PestExpr::Choice(alts))
        }
    }

    /// Parse all rules from the pest grammar source.
    fn parse_rules(&mut self) -> Vec<PestRule> {
        let mut rules = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.pos >= self.input.len() {
                break;
            }
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            } else {
                // Skip to next line on parse failure
                while self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b'\n' {
                    self.pos += 1;
                }
                if self.pos < self.input.len() {
                    self.pos += 1;
                }
            }
        }
        rules
    }

    fn parse_rule(&mut self) -> Option<PestRule> {
        let name = self.parse_ident()?;
        self.skip_ws_and_comments();
        if !self.eat_char('=') {
            return None;
        }
        self.skip_ws_and_comments();

        // Determine rule type
        let rule_type = if self.eat_str("@{") {
            RuleType::Atomic
        } else if self.eat_str("_{") {
            RuleType::Silent
        } else if self.eat_str("${") {
            RuleType::CompoundAtomic
        } else if self.eat_char('{') {
            RuleType::Normal
        } else {
            return None;
        };

        let body = self.parse_choice().unwrap_or(PestExpr::Seq(vec![]));
        self.skip_ws_and_comments();
        self.eat_char('}');

        Some(PestRule {
            name,
            rule_type,
            body,
        })
    }
}

/// Parse the pest grammar source into a list of rules.
fn parse_pest(source: &str) -> Vec<PestRule> {
    PestParser::new(source).parse_rules()
}

// ---------------------------------------------------------------------------
// GBNF emitter
// ---------------------------------------------------------------------------

/// Convert pest rules to GBNF format string.
fn to_gbnf(rules: &[PestRule], include_test: bool) -> String {
    let mut out = String::new();
    out.push_str("# Naze grammar in GBNF format (auto-generated from naze.pest)\n");
    out.push_str("# For use with llama.cpp, XGrammar, SGLang constrained decoding\n\n");

    // Emit implicit whitespace rule
    out.push_str("ws ::= [ \\t\\r]*\n");
    out.push_str("nl ::= \"\\n\"\n\n");

    let test_rules = [
        "test_file", "test_use_stmt", "test_block", "flow_block",
        "test_step", "test_render", "test_click", "test_fill",
        "test_navigate", "test_wait", "test_assert", "assert_kind",
        "assert_text_visible", "assert_text_not_visible", "assert_page",
        "assert_state", "assert_emitted", "assert_a11y",
    ];

    // Map `file` to `root` for GBNF
    let mut first = true;
    for rule in rules {
        if !include_test && test_rules.contains(&rule.name.as_str()) {
            continue;
        }

        let gbnf_name = if rule.name == "file" && first {
            first = false;
            "root".to_string()
        } else {
            pest_name_to_gbnf(&rule.name)
        };

        let is_atomic = matches!(rule.rule_type, RuleType::Atomic | RuleType::CompoundAtomic)
            || is_silent_atomic_like(&rule.name);
        let body = expr_to_gbnf(&rule.body, is_atomic);

        out.push_str(&format!("{} ::= {}\n", gbnf_name, body));
    }

    out
}

/// Silent rules that behave as atomic building blocks (only used in atomic contexts).
/// These should NOT have implicit whitespace inserted.
fn is_silent_atomic_like(name: &str) -> bool {
    matches!(
        name,
        "ident_raw" | "WHITESPACE" | "NEWLINE_CHAR" | "string_part" | "ref_path_or_ident"
    )
}

/// Convert a pest rule name to GBNF convention (underscores → dashes, lowercase).
fn pest_name_to_gbnf(name: &str) -> String {
    name.replace('_', "-").to_lowercase()
}

/// Convert a pest expression to GBNF string.
fn expr_to_gbnf(expr: &PestExpr, atomic: bool) -> String {
    match expr {
        PestExpr::Literal(s) => literal_to_gbnf(s),

        PestExpr::Ident(id) => builtin_to_gbnf(id),

        PestExpr::Seq(items) => seq_to_gbnf(items, atomic),

        PestExpr::Choice(alts) => {
            let parts: Vec<String> = alts.iter().map(|e| expr_to_gbnf(e, atomic)).collect();
            if parts.len() == 1 {
                parts[0].clone()
            } else {
                format!("({})", parts.join(" | "))
            }
        }

        PestExpr::Repeat(inner, kind) => {
            let inner_s = expr_to_gbnf(inner, atomic);
            let needs_parens = matches!(
                inner.as_ref(),
                PestExpr::Seq(_) | PestExpr::Choice(_)
            );
            let base = if needs_parens {
                format!("({})", inner_s)
            } else {
                inner_s
            };
            match kind {
                RepeatKind::ZeroOrMore => format!("{}*", base),
                RepeatKind::OneOrMore => format!("{}+", base),
                RepeatKind::Optional => format!("{}?", base),
                RepeatKind::Range(m, Some(n)) if m == n => format!("{}{{{}}}", base, m),
                RepeatKind::Range(m, Some(n)) => format!("{}{{{},{}}}", base, m, n),
                RepeatKind::Range(m, None) => format!("{}{{{},}}", base, m),
            }
        }

        PestExpr::Group(inner) => {
            let s = expr_to_gbnf(inner, atomic);
            format!("({})", s)
        }

        PestExpr::Neg(inner) => {
            // Standalone negation (not collapsed by seq_to_gbnf)
            match inner.as_ref() {
                PestExpr::Literal(s) if s.len() == 1 => {
                    format!("[^{}]", escape_char_class_char(s.chars().next().unwrap()))
                }
                PestExpr::Ident(id) => match id.as_str() {
                    "NEWLINE_CHAR" | "NEWLINE" => "[^\\n]".to_string(),
                    _ => format!("/* !{} */", pest_name_to_gbnf(id)),
                },
                _ => "/* negation not converted */".to_string(),
            }
        }
    }
}

/// Convert a sequence, collapsing `!X ~ !Y ~ ... ~ ANY` into `[^XY...]`.
fn seq_to_gbnf(items: &[PestExpr], atomic: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < items.len() {
        // Skip SOI/EOI
        if matches!(&items[i], PestExpr::Ident(id) if id == "SOI" || id == "EOI") {
            i += 1;
            continue;
        }

        // Collapse !literal ~ !literal ~ ... ~ ANY → [^chars]
        if matches!(&items[i], PestExpr::Neg(_)) {
            let mut neg_chars = Vec::new();
            let mut j = i;
            while j < items.len() {
                match &items[j] {
                    PestExpr::Neg(inner) => {
                        match inner.as_ref() {
                            PestExpr::Literal(s) if s.len() == 1 => {
                                neg_chars.push(s.chars().next().unwrap());
                            }
                            PestExpr::Ident(id) if id == "NEWLINE_CHAR" || id == "NEWLINE" => {
                                neg_chars.push('\n');
                            }
                            _ => break,
                        }
                        j += 1;
                    }
                    _ => break,
                }
            }
            // If followed by ANY, this is a negated character class
            if j < items.len() && matches!(&items[j], PestExpr::Ident(id) if id == "ANY") {
                let escaped: String = neg_chars
                    .iter()
                    .map(|c| escape_char_class_char(*c))
                    .collect();
                parts.push(format!("[^{}]", escaped));
                i = j + 1; // skip past ANY
                continue;
            }
            // Not followed by ANY — emit each negation individually
            for c in &neg_chars {
                parts.push(format!("[^{}]", escape_char_class_char(*c)));
            }
            i = j;
            continue;
        }

        parts.push(expr_to_gbnf(&items[i], atomic));
        i += 1;
    }

    if parts.is_empty() {
        return "\"\"".to_string();
    }

    if atomic {
        parts.join(" ")
    } else {
        parts.join(" ws ")
    }
}

/// Escape a character for use inside a GBNF character class [...]
fn escape_char_class_char(c: char) -> String {
    match c {
        '"' => "\\\"".to_string(),
        '\\' => "\\\\".to_string(),
        ']' => "\\]".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        '{' => "\\{".to_string(),
        _ => c.to_string(),
    }
}

/// Convert a string literal to GBNF. Multi-char strings become "text".
fn literal_to_gbnf(s: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Map pest builtins to GBNF equivalents.
fn builtin_to_gbnf(id: &str) -> String {
    match id {
        "ASCII_DIGIT" => "[0-9]".to_string(),
        "ASCII_ALPHA" => "[a-zA-Z]".to_string(),
        "ASCII_ALPHANUMERIC" => "[a-zA-Z0-9]".to_string(),
        "ASCII_HEX_DIGIT" => "[0-9a-fA-F]".to_string(),
        "ANY" => "[^\\x00]".to_string(),
        "SOI" | "EOI" => "\"\"".to_string(), // no-op anchors
        "NEWLINE" | "NEWLINE_CHAR" => "nl".to_string(),
        "WHITESPACE" => "ws".to_string(),
        _ => pest_name_to_gbnf(id),
    }
}

// ---------------------------------------------------------------------------
// EBNF emitter (human-readable)
// ---------------------------------------------------------------------------

/// Convert pest rules to human-readable EBNF.
fn to_ebnf(rules: &[PestRule], include_test: bool) -> String {
    let mut out = String::new();
    out.push_str("(* Naze grammar in EBNF notation — auto-generated from naze.pest *)\n\n");

    let test_rules = [
        "test_file", "test_use_stmt", "test_block", "flow_block",
        "test_step", "test_render", "test_click", "test_fill",
        "test_navigate", "test_wait", "test_assert", "assert_kind",
        "assert_text_visible", "assert_text_not_visible", "assert_page",
        "assert_state", "assert_emitted", "assert_a11y",
    ];

    for rule in rules {
        if !include_test && test_rules.contains(&rule.name.as_str()) {
            continue;
        }

        let type_tag = match rule.rule_type {
            RuleType::Normal => "",
            RuleType::Atomic => " (* atomic *)",
            RuleType::Silent => " (* silent *)",
            RuleType::CompoundAtomic => " (* compound-atomic *)",
        };

        let body = expr_to_ebnf(&rule.body);
        out.push_str(&format!("{} = {} ;{}\n", rule.name, body, type_tag));
    }

    out
}

/// Convert a pest expression to EBNF string.
fn expr_to_ebnf(expr: &PestExpr) -> String {
    match expr {
        PestExpr::Literal(s) => {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            format!("\"{}\"", escaped)
        }

        PestExpr::Ident(id) => {
            match id.as_str() {
                "ASCII_DIGIT" => "DIGIT".to_string(),
                "ASCII_ALPHA" => "ALPHA".to_string(),
                "ASCII_ALPHANUMERIC" => "ALNUM".to_string(),
                "ASCII_HEX_DIGIT" => "HEX".to_string(),
                "ANY" => "ANY".to_string(),
                "SOI" => "SOI".to_string(),
                "EOI" => "EOI".to_string(),
                "NEWLINE" | "NEWLINE_CHAR" => "NEWLINE".to_string(),
                "WHITESPACE" => "WS".to_string(),
                _ => id.clone(),
            }
        }

        PestExpr::Seq(items) => {
            let parts: Vec<String> = items.iter().map(expr_to_ebnf).collect();
            parts.join(" , ")
        }

        PestExpr::Choice(alts) => {
            let parts: Vec<String> = alts.iter().map(expr_to_ebnf).collect();
            parts.join(" | ")
        }

        PestExpr::Repeat(inner, kind) => {
            let s = expr_to_ebnf(inner);
            match kind {
                RepeatKind::ZeroOrMore => format!("{{ {} }}", s),
                RepeatKind::OneOrMore => format!("{} , {{ {} }}", s, s),
                RepeatKind::Optional => format!("[ {} ]", s),
                RepeatKind::Range(m, n) => {
                    let n_str = match n {
                        Some(n) => n.to_string(),
                        None => "*".to_string(),
                    };
                    format!("{}{{{},{}}}", s, m, n_str)
                }
            }
        }

        PestExpr::Group(inner) => {
            let s = expr_to_ebnf(inner);
            format!("( {} )", s)
        }

        PestExpr::Neg(inner) => {
            let s = expr_to_ebnf(inner);
            format!("NOT({})", s)
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the grammar export command.
pub fn run(format: GrammarFormat, no_test: bool) -> Result<(), Box<dyn std::error::Error>> {
    let rules = parse_pest(PEST_SOURCE);
    if rules.is_empty() {
        return Err("failed to parse any grammar rules from naze.pest".into());
    }

    let include_test = !no_test;
    let output = match format {
        GrammarFormat::Gbnf => to_gbnf(&rules, include_test),
        GrammarFormat::Ebnf => to_ebnf(&rules, include_test),
    };

    print!("{}", output);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let rules = parse_pest(r#"bool_lit = { "true" | "false" }"#);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "bool_lit");
        assert_eq!(rules[0].rule_type, RuleType::Normal);
        match &rules[0].body {
            PestExpr::Choice(alts) => assert_eq!(alts.len(), 2),
            other => panic!("expected Choice, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_atomic_rule() {
        let rules = parse_pest(r#"ident = @{ ident_raw }"#);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].rule_type, RuleType::Atomic);
    }

    #[test]
    fn test_parse_silent_rule() {
        let rules = parse_pest(r#"WHITESPACE = _{ " " | "\t" }"#);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].rule_type, RuleType::Silent);
    }

    #[test]
    fn test_parse_compound_atomic_rule() {
        let rules = parse_pest(r#"string_lit = ${ "\"" ~ string_part* ~ "\"" }"#);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].rule_type, RuleType::CompoundAtomic);
    }

    #[test]
    fn test_parse_sequence() {
        let rules = parse_pest(r#"app_block = { "app" ~ string_lit ~ block }"#);
        assert_eq!(rules.len(), 1);
        match &rules[0].body {
            PestExpr::Seq(items) => assert_eq!(items.len(), 3),
            other => panic!("expected Seq, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_repetition() {
        let rules = parse_pest(r#"number_raw = @{ "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }"#);
        assert_eq!(rules.len(), 1);
        match &rules[0].body {
            PestExpr::Seq(items) => assert_eq!(items.len(), 3),
            other => panic!("expected Seq, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_repeat_range() {
        let src = "color_lit = @{ \"#\" ~ ASCII_HEX_DIGIT{3,8} }";
        let rules = parse_pest(src);
        assert_eq!(rules.len(), 1);
        match &rules[0].body {
            PestExpr::Seq(items) => {
                assert_eq!(items.len(), 2);
                match &items[1] {
                    PestExpr::Repeat(_, RepeatKind::Range(3, Some(8))) => {}
                    other => panic!("expected Range(3,8), got {:?}", other),
                }
            }
            other => panic!("expected Seq, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_negation() {
        let rules = parse_pest(r#"comment = @{ "--" ~ (!NEWLINE_CHAR ~ ANY)* }"#);
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_convert_simple_gbnf() {
        let rules = parse_pest(r#"bool_lit = { "true" | "false" }"#);
        let gbnf = to_gbnf(&rules, false);
        assert!(gbnf.contains("bool-lit ::= "));
        assert!(gbnf.contains("\"true\""));
        assert!(gbnf.contains("\"false\""));
    }

    #[test]
    fn test_convert_atomic_no_ws() {
        let rules = parse_pest(r#"ref_path = @{ ident_raw ~ ("." ~ ident_raw)+ }"#);
        let gbnf = to_gbnf(&rules, false);
        // Atomic rules should NOT have ws inserted between elements
        assert!(!gbnf.contains("ws ident-raw"));
        // Should have elements directly adjacent
        assert!(gbnf.contains("ident-raw"));
    }

    #[test]
    fn test_builtin_mapping() {
        let rules = parse_pest(r#"num = @{ ASCII_DIGIT+ }"#);
        let gbnf = to_gbnf(&rules, false);
        assert!(gbnf.contains("[0-9]"));
    }

    #[test]
    fn test_name_conversion() {
        assert_eq!(pest_name_to_gbnf("app_block"), "app-block");
        assert_eq!(pest_name_to_gbnf("WHITESPACE"), "whitespace");
        assert_eq!(pest_name_to_gbnf("ident"), "ident");
    }

    #[test]
    fn test_full_grammar_parses() {
        let rules = parse_pest(PEST_SOURCE);
        // The grammar has ~89 rules
        assert!(rules.len() > 50, "expected >50 rules, got {}", rules.len());
    }

    #[test]
    fn test_gbnf_has_root() {
        let rules = parse_pest(PEST_SOURCE);
        let gbnf = to_gbnf(&rules, false);
        assert!(gbnf.contains("root ::="), "GBNF must start with root rule");
    }

    #[test]
    fn test_gbnf_covers_key_constructs() {
        let rules = parse_pest(PEST_SOURCE);
        let gbnf = to_gbnf(&rules, true);
        // Key constructs should be present
        for expected in &[
            "app-block", "page-block", "component-def", "state-stmt",
            "let-stmt", "on-handler", "element", "expression", "value",
            "if-stmt", "each-stmt", "match-stmt",
        ] {
            assert!(
                gbnf.contains(&format!("{} ::=", expected)),
                "missing rule: {}",
                expected
            );
        }
    }

    #[test]
    fn test_gbnf_no_test_rules() {
        let rules = parse_pest(PEST_SOURCE);
        let gbnf = to_gbnf(&rules, false);
        assert!(!gbnf.contains("test-block ::="));
        assert!(!gbnf.contains("test-render ::="));
    }

    #[test]
    fn test_gbnf_with_test_rules() {
        let rules = parse_pest(PEST_SOURCE);
        let gbnf = to_gbnf(&rules, true);
        assert!(gbnf.contains("test-block ::="));
    }

    #[test]
    fn test_ebnf_output() {
        let rules = parse_pest(PEST_SOURCE);
        let ebnf = to_ebnf(&rules, false);
        assert!(ebnf.contains("file ="));
        assert!(ebnf.contains("app_block ="));
    }
}
