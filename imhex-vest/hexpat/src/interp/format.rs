//! `std::format` / `std::print` format strings (the fmtlib subset used by
//! patterns): `{}`, `{0}`, `{:x}`, `{:08X}`, `{:#x}`, `{:>10}`, `{:.2f}`,
//! `{:b}`, `{:c}`, `{{` and `}}`.

use super::Value;
use crate::error::{Error, Result};

#[derive(Default)]
struct Spec {
    fill: char,
    align: Option<char>,
    alternate: bool,
    zero: bool,
    width: Option<usize>,
    precision: Option<usize>,
    kind: Option<char>,
    plus: bool,
}

fn parse_spec(s: &str) -> Spec {
    let mut spec = Spec { fill: ' ', ..Default::default() };
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    if chars.len() >= 2 && matches!(chars[1], '<' | '>' | '^') {
        spec.fill = chars[0];
        spec.align = Some(chars[1]);
        i = 2;
    } else if !chars.is_empty() && matches!(chars[0], '<' | '>' | '^') {
        spec.align = Some(chars[0]);
        i = 1;
    }
    if i < chars.len() && chars[i] == '+' {
        spec.plus = true;
        i += 1;
    }
    if i < chars.len() && chars[i] == '#' {
        spec.alternate = true;
        i += 1;
    }
    if i < chars.len() && chars[i] == '0' {
        spec.zero = true;
        i += 1;
    }
    let mut w = String::new();
    while i < chars.len() && chars[i].is_ascii_digit() {
        w.push(chars[i]);
        i += 1;
    }
    if !w.is_empty() {
        spec.width = w.parse().ok();
    }
    if i < chars.len() && chars[i] == '.' {
        i += 1;
        let mut p = String::new();
        while i < chars.len() && chars[i].is_ascii_digit() {
            p.push(chars[i]);
            i += 1;
        }
        spec.precision = p.parse().ok();
    }
    if i < chars.len() {
        spec.kind = Some(chars[i]);
    }
    spec
}

fn pad(s: String, spec: &Spec, numeric: bool) -> String {
    let Some(width) = spec.width else { return s };
    let len = s.chars().count();
    if len >= width {
        return s;
    }
    let fill_count = width - len;
    if spec.zero && numeric && spec.align.is_none() {
        // Zero padding goes after any sign / prefix.
        let (prefix, digits) = split_prefix(&s);
        return format!("{}{}{}", prefix, "0".repeat(fill_count), digits);
    }
    let align = spec.align.unwrap_or(if numeric { '>' } else { '<' });
    let fill: String = std::iter::repeat_n(spec.fill, fill_count).collect();
    match align {
        '<' => format!("{}{}", s, fill),
        '^' => {
            let left = fill_count / 2;
            let right = fill_count - left;
            format!("{}{}{}", std::iter::repeat_n(spec.fill, left).collect::<String>(), s, std::iter::repeat_n(spec.fill, right).collect::<String>())
        }
        _ => format!("{}{}", fill, s),
    }
}

fn split_prefix(s: &str) -> (&str, &str) {
    let mut idx = 0;
    let b = s.as_bytes();
    if !b.is_empty() && (b[0] == b'-' || b[0] == b'+') {
        idx = 1;
    }
    if b.len() >= idx + 2 && b[idx] == b'0' && matches!(b[idx + 1], b'x' | b'X' | b'b' | b'B' | b'o') {
        idx += 2;
    }
    (&s[..idx], &s[idx..])
}

fn format_int(v: i128, unsigned: u128, is_signed: bool, spec: &Spec) -> String {
    let neg = is_signed && v < 0;
    let mag: u128 = if is_signed { v.unsigned_abs() } else { unsigned };
    let body = match spec.kind {
        Some('x') => format!("{}{:x}", if spec.alternate { "0x" } else { "" }, mag),
        Some('X') => format!("{}{:X}", if spec.alternate { "0x" } else { "" }, mag),
        Some('b') => format!("{}{:b}", if spec.alternate { "0b" } else { "" }, mag),
        Some('o') => format!("{}{:o}", if spec.alternate { "0" } else { "" }, mag),
        Some('c') => return pad((mag as u8 as char).to_string(), spec, false),
        Some('f') | Some('e') | Some('g') => return format_float_spec(v as f64, spec),
        _ => mag.to_string(),
    };
    let sign = if neg { "-" } else if spec.plus { "+" } else { "" };
    pad(format!("{}{}", sign, body), spec, true)
}

fn format_float_spec(f: f64, spec: &Spec) -> String {
    let s = match (spec.kind, spec.precision) {
        (Some('e'), Some(p)) => format!("{:.*e}", p, f),
        (Some('e'), None) => format!("{:e}", f),
        (_, Some(p)) => format!("{:.*}", p, f),
        (Some('f'), None) => format!("{:.6}", f),
        _ => super::format_float(f),
    };
    let s = if spec.plus && f >= 0.0 { format!("+{}", s) } else { s };
    pad(s, spec, true)
}

fn format_value(v: &Value, spec: &Spec) -> String {
    match v {
        Value::Unsigned(u) => format_int(*u as i128, *u, false, spec),
        Value::Signed(i) => format_int(*i, *i as u128, true, spec),
        Value::Char(c) => match spec.kind {
            Some('x') | Some('X') | Some('d') | Some('b') => format_int(*c as i128, *c as u128, false, spec),
            _ => pad((*c as char).to_string(), spec, false),
        },
        Value::Char16(c) => pad(char::from_u32(*c as u32).unwrap_or('?').to_string(), spec, false),
        Value::Bool(b) => match spec.kind {
            Some('d') | Some('x') | Some('X') => format_int(*b as i128, *b as u128, false, spec),
            _ => pad(b.to_string(), spec, false),
        },
        Value::Float(f) => format_float_spec(*f, spec),
        Value::Str(s) => {
            let s = match spec.precision {
                Some(p) => s.chars().take(p).collect(),
                None => s.clone(),
            };
            pad(s, spec, false)
        }
        other => pad(other.to_display_string(), spec, false),
    }
}

/// Renders a format string with positional arguments.
pub fn format(fmt: &str, args: &[Value]) -> Result<String> {
    let mut out = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    let mut next_arg = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '{' {
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                out.push('{');
                i += 2;
                continue;
            }
            let mut j = i + 1;
            let mut depth = 1;
            while j < chars.len() {
                if chars[j] == '{' {
                    depth += 1;
                } else if chars[j] == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            if j >= chars.len() {
                return Err(Error::new("unterminated `{` in format string"));
            }
            let inner: String = chars[i + 1..j].iter().collect();
            let (index_part, spec_part) = match inner.find(':') {
                Some(k) => (&inner[..k], &inner[k + 1..]),
                None => (inner.as_str(), ""),
            };
            let idx = if index_part.is_empty() {
                let k = next_arg;
                next_arg += 1;
                k
            } else {
                index_part.trim().parse::<usize>().map_err(|_| Error::new(format!("bad argument index `{}`", index_part)))?
            };
            let arg = args.get(idx).ok_or_else(|| Error::new(format!("format string refers to argument {} but only {} were given", idx, args.len())))?;
            let spec = parse_spec(spec_part);
            out.push_str(&format_value(arg, &spec));
            i = j + 1;
            continue;
        }
        if c == '}' && i + 1 < chars.len() && chars[i + 1] == '}' {
            out.push('}');
            i += 2;
            continue;
        }
        out.push(c);
        i += 1;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_fmtlib() {
        let args = [Value::Unsigned(255), Value::Str("ab".into()), Value::Signed(-3), Value::Float(1.5)];
        assert_eq!(format("{:02X}{:>4}{}", &args).unwrap(), "FF  ab-3");
        assert_eq!(format("{0:#x} {0:08b} {3:.2f} {{}}", &args).unwrap(), "0xff 11111111 1.50 {}");
        assert_eq!(format("{:x}", &[Value::Signed(-1)]).unwrap(), "-1");
    }
}
