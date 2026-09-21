//! Rendering of the pattern tree as text or JSON.

use super::{PatternKind, PatternRef, Runtime, Value};
use crate::error::Result;

/// Options for the dumps.
#[derive(Debug, Clone, Default)]
pub struct DumpOptions {
    /// Call `[[format]]` functions instead of printing raw values.
    pub formatted: bool,
    /// Include hidden patterns.
    pub show_hidden: bool,
    /// Maximum number of array entries to print per array (0 = all).
    pub max_entries: usize,
}

/// Escapes non-printable bytes of a string as `\xHH`.
pub fn escape_string(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7F => out.push_str(&format!("\\x{:02X}", c as u32)),
            c if (c as u32) >= 0x80 && (c as u32) < 0x100 && !c.is_alphanumeric() => {
                out.push_str(&format!("\\x{:02X}", c as u32))
            }
            c => out.push(c),
        }
    }
    out
}

/// The value string ImHex would show without a `[[format]]` attribute.
pub fn default_formatted_value(rt: &mut Runtime, p: &PatternRef) -> Result<String> {
    let kind = p.borrow().kind.clone();
    Ok(match kind {
        PatternKind::Enum { entries, .. } => {
            let v = rt.pattern_value(p)?.as_i128()?;
            match entries.iter().find(|(_, a, b)| v >= *a && v <= *b) {
                Some((name, _, _)) => format!("{}::{}", p.borrow().type_name.rsplit("::").next().unwrap_or(""), name),
                None => format!("{} (invalid)", v),
            }
        }
        PatternKind::String | PatternKind::WideString => format!("\"{}\"", escape_string(&rt.pattern_value(p)?.as_str()?)),
        PatternKind::Char => {
            let v = rt.pattern_value(p)?;
            match v {
                Value::Char(c) => format!("'{}'", escape_string(&(c as char).to_string())),
                other => other.to_display_string(),
            }
        }
        PatternKind::Char16 => format!("'{}'", rt.pattern_value(p)?.to_display_string()),
        PatternKind::Unsigned | PatternKind::Signed => {
            let v = rt.pattern_value(p)?;
            match v {
                Value::Unsigned(u) => format!("{} (0x{:X})", u, u),
                Value::Signed(i) => {
                    if i < 0 {
                        format!("{}", i)
                    } else {
                        format!("{} (0x{:X})", i, i)
                    }
                }
                other => other.to_display_string(),
            }
        }
        PatternKind::BitfieldField { typed, .. } => {
            let v = rt.pattern_value(p)?;
            match (typed.as_deref(), v) {
                (Some(PatternKind::Enum { entries, .. }), v) => {
                    let iv = v.as_i128()?;
                    match entries.iter().find(|(_, a, b)| iv >= *a && iv <= *b) {
                        Some((name, _, _)) => name.clone(),
                        None => format!("{} (invalid)", iv),
                    }
                }
                (_, Value::Unsigned(u)) => format!("{} (0x{:X})", u, u),
                (_, other) => other.to_display_string(),
            }
        }
        PatternKind::Pointer { address, .. } => format!("*(0x{:X})", address),
        PatternKind::Struct { .. } | PatternKind::Union { .. } | PatternKind::Bitfield { .. } => {
            format!("{} {{ ... }}", p.borrow().type_name)
        }
        PatternKind::Array { .. } | PatternKind::StaticArray { .. } => {
            format!("{} [ ... ]", p.borrow().type_name)
        }
        PatternKind::Padding => String::new(),
        _ => rt.pattern_value(p)?.to_display_string(),
    })
}

fn value_string(rt: &mut Runtime, p: &PatternRef, opts: &DumpOptions) -> String {
    let r = if opts.formatted { rt.formatted_value(p) } else { default_formatted_value(rt, p) };
    match r {
        Ok(s) => s,
        Err(e) => format!("<error: {}>", e.message),
    }
}

/// Renders the pattern tree as indented text, one pattern per line:
/// `name: type @ 0xOFF [size] = value`.
pub fn dump_text(rt: &mut Runtime, opts: &DumpOptions) -> String {
    let mut out = String::new();
    let roots = rt.patterns.clone();
    for p in &roots {
        dump_text_node(rt, p, 0, opts, &mut out);
    }
    out
}

fn dump_text_node(rt: &mut Runtime, p: &PatternRef, depth: usize, opts: &DumpOptions, out: &mut String) {
    let (name, type_name, offset, size, hidden, kind, inline, section) = {
        let pb = p.borrow();
        (
            pb.display_name.clone().unwrap_or_else(|| pb.name.clone()),
            pb.type_name.clone(),
            pb.offset,
            pb.size,
            pb.hidden,
            pb.kind.clone(),
            pb.inline,
            pb.section,
        )
    };
    if hidden && !opts.show_hidden {
        return;
    }
    if !inline {
        let value = value_string(rt, p, opts);
        let indent = "  ".repeat(depth);
        let sec = if section == 0 { String::new() } else { format!(" (section {})", section) };
        out.push_str(&format!("{}{}: {} @ 0x{:X} [{}]{}", indent, name, type_name, offset, size, sec));
        if !value.is_empty() {
            out.push_str(&format!(" = {}", value));
        }
        out.push('\n');
    }
    let child_depth = if inline { depth } else { depth + 1 };
    match kind {
        PatternKind::Struct { members } | PatternKind::Union { members } => {
            for m in &members {
                dump_text_node(rt, m, child_depth, opts, out);
            }
        }
        PatternKind::Bitfield { fields, .. } => {
            for f in &fields {
                dump_text_node(rt, f, child_depth, opts, out);
            }
        }
        PatternKind::Array { entries } => {
            let limit = if opts.max_entries == 0 { entries.len() } else { opts.max_entries.min(entries.len()) };
            for e in entries.iter().take(limit) {
                dump_text_node(rt, e, child_depth, opts, out);
            }
            if limit < entries.len() {
                out.push_str(&format!("{}... ({} more)\n", "  ".repeat(child_depth), entries.len() - limit));
            }
        }
        PatternKind::StaticArray { count, .. } => {
            let limit = if opts.max_entries == 0 { count as usize } else { opts.max_entries.min(count as usize) };
            for i in 0..limit {
                if let Ok(e) = rt.array_element(p, i as u64) {
                    dump_text_node(rt, &e, child_depth, opts, out);
                }
            }
            if (limit as u64) < count {
                out.push_str(&format!("{}... ({} more)\n", "  ".repeat(child_depth), count - limit as u64));
            }
        }
        PatternKind::Pointer { pointee, .. } => {
            dump_text_node(rt, &pointee, child_depth, opts, out);
        }
        _ => {}
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Renders the pattern tree as JSON.
pub fn dump_json(rt: &mut Runtime, opts: &DumpOptions) -> String {
    let mut out = String::from("[");
    let roots = rt.patterns.clone();
    let mut first = true;
    for p in &roots {
        if p.borrow().hidden && !opts.show_hidden {
            continue;
        }
        if !first {
            out.push(',');
        }
        first = false;
        dump_json_node(rt, p, opts, &mut out);
    }
    out.push(']');
    out
}

fn dump_json_node(rt: &mut Runtime, p: &PatternRef, opts: &DumpOptions, out: &mut String) {
    let (name, type_name, offset, size, kind) = {
        let pb = p.borrow();
        (pb.display_name.clone().unwrap_or_else(|| pb.name.clone()), pb.type_name.clone(), pb.offset, pb.size, pb.kind.clone())
    };
    out.push_str(&format!(
        "{{\"name\":{},\"type\":{},\"offset\":{},\"size\":{}",
        json_escape(&name),
        json_escape(&type_name),
        offset,
        size
    ));
    let scalar = match &kind {
        PatternKind::Struct { .. } | PatternKind::Union { .. } | PatternKind::Bitfield { .. } | PatternKind::Array { .. } | PatternKind::StaticArray { .. } | PatternKind::Pointer { .. } => false,
        _ => true,
    };
    if scalar {
        match rt.pattern_value(p) {
            Ok(Value::Unsigned(u)) => out.push_str(&format!(",\"value\":{}", u)),
            Ok(Value::Signed(i)) => out.push_str(&format!(",\"value\":{}", i)),
            Ok(Value::Float(f)) if f.is_finite() => out.push_str(&format!(",\"value\":{}", f)),
            Ok(Value::Bool(b)) => out.push_str(&format!(",\"value\":{}", b)),
            Ok(v) => out.push_str(&format!(",\"value\":{}", json_escape(&v.to_display_string()))),
            Err(_) => {}
        }
        if opts.formatted {
            let s = value_string(rt, p, opts);
            out.push_str(&format!(",\"formatted\":{}", json_escape(&s)));
        }
    }
    let children: Vec<PatternRef> = match &kind {
        PatternKind::Struct { members } | PatternKind::Union { members } => members.clone(),
        PatternKind::Bitfield { fields, .. } => fields.clone(),
        PatternKind::Array { entries } => entries.clone(),
        PatternKind::StaticArray { count, .. } => {
            let limit = if opts.max_entries == 0 { *count as usize } else { opts.max_entries.min(*count as usize) };
            (0..limit).filter_map(|i| rt.array_element(p, i as u64).ok()).collect()
        }
        PatternKind::Pointer { pointee, .. } => vec![pointee.clone()],
        _ => Vec::new(),
    };
    if !children.is_empty() || !scalar {
        out.push_str(",\"children\":[");
        let mut first = true;
        for c in &children {
            if c.borrow().hidden && !opts.show_hidden {
                continue;
            }
            if !first {
                out.push(',');
            }
            first = false;
            dump_json_node(rt, c, opts, out);
        }
        out.push(']');
    }
    out.push('}');
}
