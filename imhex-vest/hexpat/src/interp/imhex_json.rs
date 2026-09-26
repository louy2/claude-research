//! JSON export in the format of the reference implementation's `json`
//! formatter (`plcli format -f json`, `pl::gen::fmt::FormatterJson`).
//!
//! The layout is reproduced line for line:
//!
//! ```text
//! {
//!     "data": {
//!         "s": "\u0089PNG\r\n\u001a\n\u0000",
//!         "ua": 0,
//!         "arr": [
//!             1,
//!             2
//!         ]
//!     }
//! }
//! ```
//!
//! - structs, unions and bitfields are objects keyed by member name; sealed
//!   ones become a string (their formatted value);
//! - arrays are lists; `char[]` / `char16[]` arrays are strings;
//! - integers, floats and booleans are JSON literals unless the pattern has a
//!   `[[format]]` function, in which case the function's result is a string;
//! - characters and enums are strings (`"A"`, `"Kind::B"`, `"Kind::???"`);
//! - pointers are objects with one entry named `*(name)`;
//! - padding and hidden patterns are omitted;
//! - with metadata enabled, objects start with `__type`, `__address`,
//!   `__size`, `__color`, `__endian` and (if set) `__comment`.

use super::{PatternKind, PatternRef, Runtime, Value};

/// Renders the top-level patterns as reference-compatible JSON.
pub fn dump_json_imhex(rt: &mut Runtime, meta: bool) -> String {
    let mut w = Writer { out: String::new(), indent: 4, in_array: false, meta };
    let roots = rt.patterns.clone();
    for p in &roots {
        w.visit(rt, p);
    }
    w.pop_indent_trailing_comma();
    format!("{{\n{}}}", w.out)
}

struct Writer {
    out: String,
    indent: usize,
    in_array: bool,
    meta: bool,
}

impl Writer {
    fn add_line(&mut self, name: &str, text: &str, no_name: bool) {
        self.out.push_str(&" ".repeat(self.indent));
        if !no_name && !self.in_array {
            self.out.push_str(&format!("\"{}\": ", name));
        }
        self.out.push_str(text);
        self.out.push('\n');
        self.in_array = false;
    }

    fn push_indent(&mut self) {
        self.indent += 4;
    }

    /// Un-indents and removes the trailing comma of the last line of the
    /// block, exactly like the reference visitor.
    fn pop_indent_trailing_comma(&mut self) {
        self.indent = self.indent.saturating_sub(4);
        if self.out.ends_with(",\n") {
            self.out.truncate(self.out.len() - 2);
            self.out.push('\n');
        }
    }

    fn visit(&mut self, rt: &mut Runtime, p: &PatternRef) {
        let (hidden, kind, name) = {
            let pb = p.borrow();
            (pb.hidden, pb.kind.clone(), pb.name.clone())
        };
        if hidden {
            return;
        }
        match kind {
            PatternKind::Padding => {}
            PatternKind::Struct { .. } | PatternKind::Union { .. } | PatternKind::Bitfield { .. } => self.object(rt, p, &name),
            PatternKind::Array { .. } | PatternKind::StaticArray { .. } => self.array(rt, p, &name),
            PatternKind::Pointer { pointee, .. } => {
                self.add_line(&name, "{", false);
                self.push_indent();
                self.visit(rt, &pointee);
                self.pop_indent_trailing_comma();
                self.add_line("", "},", true);
            }
            PatternKind::String | PatternKind::WideString | PatternKind::Char | PatternKind::Char16 | PatternKind::Enum { .. } => {
                let s = to_string(rt, p);
                self.string_line(&name, &s);
            }
            PatternKind::Unsigned | PatternKind::Signed | PatternKind::Float | PatternKind::Bool | PatternKind::BitfieldField { .. } => {
                self.value(rt, p, &name)
            }
        }
    }

    fn string_line(&mut self, name: &str, s: &str) {
        self.add_line(name, &format!("\"{}\",", json_escape(s)), false);
    }

    fn object(&mut self, rt: &mut Runtime, p: &PatternRef, name: &str) {
        let sealed = p.borrow().sealed;
        if sealed {
            self.value(rt, p, name);
            return;
        }
        self.add_line(name, "{", false);
        self.push_indent();
        if self.meta {
            for (k, v) in meta_information(rt, p) {
                self.add_line(&k, &format!("\"{}\",", json_escape(&v)), false);
            }
        }
        let members = p.borrow().members().to_vec();
        for m in &members {
            self.visit(rt, m);
        }
        self.pop_indent_trailing_comma();
        self.add_line("", "},", true);
    }

    fn array(&mut self, rt: &mut Runtime, p: &PatternRef, name: &str) {
        self.add_line(name, "[", false);
        self.push_indent();
        let count = p.borrow().entry_count().unwrap_or(0);
        for i in 0..count {
            let Ok(entry) = rt.array_element(p, i) else { break };
            self.in_array = true;
            self.visit(rt, &entry);
        }
        self.pop_indent_trailing_comma();
        self.add_line("", "],", true);
    }

    /// `formatValue`: a formatter function turns the value into a string,
    /// otherwise the value is a JSON literal.
    fn value(&mut self, rt: &mut Runtime, p: &PatternRef, name: &str) {
        let has_formatter = p.borrow().format_fn.is_some();
        if has_formatter {
            let s = to_string(rt, p);
            self.string_line(name, &s);
            return;
        }
        let text = match rt.value_of(p) {
            Ok(v) => literal(rt, &v),
            Err(e) => format!("\"<error: {}>\"", json_escape(&e.message)),
        };
        self.add_line(name, &format!("{},", text), false);
    }
}

/// `Pattern::toString()`: the formatter function's result, or the natural
/// string form of the pattern.
fn to_string(rt: &mut Runtime, p: &PatternRef) -> String {
    let has_formatter = p.borrow().format_fn.is_some();
    if has_formatter {
        return match rt.formatted_value(p) {
            Ok(s) => s,
            Err(e) => e.message,
        };
    }
    let kind = p.borrow().kind.clone();
    let v = match rt.value_of(p) {
        Ok(v) => v,
        Err(e) => return e.message,
    };
    match (&kind, &v) {
        (PatternKind::Char, Value::Char(c)) => encode_byte_string(&[*c]),
        (PatternKind::Enum { entries, .. }, v) => {
            let raw = v.as_i128().unwrap_or(0);
            let type_name = p.borrow().type_name.clone();
            match entries.iter().find(|(_, a, b)| raw >= *a && raw <= *b) {
                Some((n, _, _)) => format!("{}::{}", type_name, n),
                None => format!("{}::???", type_name),
            }
        }
        (PatternKind::Struct { .. } | PatternKind::Union { .. } | PatternKind::Bitfield { .. }, Value::Pattern(_)) => {
            // Base `Pattern::toString()` fallback for sealed objects without a formatter.
            let pb = p.borrow();
            format!("{} {} @ 0x{:X}", pb.type_name, pb.name, pb.offset)
        }
        (_, Value::Float(f)) => float_string(*f, p.borrow().size),
        (_, v) => v.to_display_string(),
    }
}

/// `formatLiteral`: JSON literal for a value.
fn literal(rt: &mut Runtime, v: &Value) -> String {
    match v {
        Value::Unsigned(u) => u.to_string(),
        Value::Signed(i) => i.to_string(),
        Value::Char(c) => format!("\"{}\"", json_escape(&(*c as char).to_string())),
        Value::Bool(b) => b.to_string(),
        Value::Float(f) => float_string(*f, 8),
        Value::Str(s) => format!("\"{}\"", json_escape(s)),
        Value::Pattern(inner) => {
            let s = to_string(rt, inner);
            format!("\"{}\"", json_escape(&s))
        }
        other => format!("\"{}\"", json_escape(&other.to_display_string())),
    }
}

/// fmtlib's `{}` for a float (`float` for 4-byte patterns).
fn float_string(f: f64, size: u64) -> String {
    if size == 4 {
        fmt_shortest(f as f32 as f64, true)
    } else {
        fmt_shortest(f, false)
    }
}

fn fmt_shortest(f: f64, single: bool) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let plain = if single { format!("{}", f as f32) } else { format!("{}", f) };
    // fmtlib switches to exponent notation outside roughly [1e-4, 1e16).
    let mag = f.abs();
    if mag != 0.0 && (mag >= 1e16 || mag < 1e-4) {
        let e = if single { format!("{:e}", f as f32) } else { format!("{:e}", f) };
        // Rust prints `1e20`; fmtlib prints `1e+20` and pads two exponent digits.
        if let Some((m, exp)) = e.split_once('e') {
            let (sign, digits) = match exp.strip_prefix('-') {
                Some(d) => ("-", d),
                None => ("+", exp),
            };
            return format!("{}e{}{:0>2}", m, sign, digits);
        }
        return e;
    }
    plain
}

/// `hlp::encodeByteString`: printable bytes verbatim, C escapes otherwise.
pub fn encode_byte_string(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &b in bytes {
        match b {
            0x20..=0x7E if b != b'\\' => out.push(b as char),
            b'\\' => out.push('\\'),
            0x07 => out.push_str("\\a"),
            0x08 => out.push_str("\\b"),
            0x0C => out.push_str("\\f"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x0B => out.push_str("\\v"),
            other => out.push_str(&format!("\\x{:02X}", other)),
        }
    }
    out
}

/// The reference's JSON string escaping. Strings that came from raw bytes
/// map invalid UTF-8 bytes to U+0080..U+00FF (see
/// [`crate::lower::bytes_to_string`]); those are written as `\u00XX`, which
/// is what the reference emits for an invalid byte.
pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7F => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if (0x80..0x100).contains(&(c as u32)) => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if (c as u32) > 0xFFFF => {
                let code = c as u32 - 0x10000;
                out.push_str(&format!("\\u{:04x}\\u{:04x}", 0xD800 + ((code >> 10) & 0x3FF), 0xDC00 + (code & 0x3FF)));
            }
            c => out.push(c),
        }
    }
    out
}

/// `FormatterPatternVisitor::getMetaInformation`.
fn meta_information(rt: &Runtime, p: &PatternRef) -> Vec<(String, String)> {
    let pb = p.borrow();
    let _ = rt;
    let mut m = vec![
        ("__type".to_string(), pb.type_name.clone()),
        ("__address".to_string(), pb.offset.to_string()),
        ("__size".to_string(), pb.size.to_string()),
        ("__color".to_string(), format!("#{:08X}", pb.color.unwrap_or(0))),
        ("__endian".to_string(), if pb.endian == crate::ast::Endian::Little { "little".to_string() } else { "big".to_string() }),
    ];
    if let Some(c) = &pb.comment {
        if !c.is_empty() {
            m.push(("__comment".to_string(), c.clone()));
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_like_the_reference() {
        assert_eq!(json_escape("a\"b\\c\n\u{89}\u{1a}\u{7f}"), "a\\\"b\\\\c\\n\\u0089\\u001a\\u007f");
        assert_eq!(json_escape("😀"), "\\ud83d\\ude00");
        assert_eq!(encode_byte_string(&[b'A', 0x89, b'\n', b'\\']), "A\\x89\\n\\");
        assert_eq!(fmt_shortest(1.5, false), "1.5");
        assert_eq!(fmt_shortest(100.0, false), "100");
        assert_eq!(fmt_shortest(1e20, false), "1e+20");
        assert_eq!(fmt_shortest(0.1f32 as f64, true), "0.1");
    }
}
