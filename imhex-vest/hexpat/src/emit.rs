//! Translation of the declarative subset of hexpat into the Vest DSL.
//!
//! The Vest DSL describes *formats* (what bytes mean) without an imperative
//! evaluator, so only the part of a pattern that is a pure format can be
//! translated:
//!
//! | hexpat                                   | Vest                                   |
//! |------------------------------------------|----------------------------------------|
//! | `struct S { u32 a; T b; }`               | `s = { a: u32, b: t, }`                |
//! | `u8 x[n]` / `char x[4]` / `T x[count]`   | `[u8; @n]` / `[u8; 4]` / `[t; @count]` |
//! | `enum E : u8 { A = 1, ... }`             | `e = enum { A = 1u8, ..., ... }`       |
//! | `if (f == C) { .. } else if .. else ..`  | `choose(@f) { C => .., _ => .. }`      |
//! | `match (f) { (C): ..; (_): ..; }`        | `choose(@f) { .. }`                    |
//! | `type::Magic<"GIF"> m;`                  | `const m: [u8; 3] = "GIF"`             |
//! | `padding[n]`                             | `_padN: [u8; n]`                       |
//! | `bitfield B { a : 3; b : 5; }`           | `b = bits { b: u5, a: u3, }`           |
//! | `T x[while(!std::mem::eof())]` (last)    | `Vec<t>`                               |
//! | `using A = B;`                           | `a = b`                                |
//!
//! Everything else (placements with `@`, pointers, `$` arithmetic, function
//! calls, unsized strings, unions, local variables that feed sizes) is
//! reported in [`VestOutput::notes`], and a struct that depends on it is left
//! out rather than emitted incorrectly.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::*;
use crate::error::{Error, Result};

/// Output of [`emit_vest`].
#[derive(Debug, Default)]
pub struct VestOutput {
    /// The `.vest` source.
    pub source: String,
    /// The Vest name of the root format (the first `Type name @ 0` placement
    /// or the `--root` override).
    pub root: Option<String>,
    /// Constructs that could not be translated.
    pub notes: Vec<String>,
    /// Structs that were emitted, by hexpat name.
    pub emitted: Vec<String>,
}

#[derive(Debug, Clone)]
enum Decl {
    Struct(StructDef),
    Union,
    Enum(EnumDef),
    Bitfield(StructDef),
    Alias(TypeRef),
}

struct Emitter {
    endian: Endian,
    decls: BTreeMap<String, Decl>,
    /// hexpat name -> vest name
    names: HashMap<String, String>,
    out: Vec<String>,
    notes: Vec<String>,
    emitted: HashSet<String>,
    in_progress: HashSet<String>,
    failed: HashSet<String>,
    order: Vec<String>,
    root_candidates: Vec<String>,
}

const VEST_RESERVED: &[&str] = &[
    "macro", "const", "enum", "choose", "wrap", "Option", "Vec", "Tail", "Nothing", "Never", "btc_varint", "uleb128",
];

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let",
    "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
    "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final",
    "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

/// Converts `BitmapFileHeader` / `chunk_t` / `ns::Name` into a snake_case
/// Vest identifier.
pub fn snake_case(name: &str) -> String {
    let mut out = String::new();
    let flat = name.replace("::", "_");
    let chars: Vec<char> = flat.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let prev_lower = i > 0 && (chars[i - 1].is_ascii_lowercase() || chars[i - 1].is_ascii_digit());
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_ascii_lowercase();
            if i > 0 && chars[i - 1] != '_' && (prev_lower || next_lower) {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(*c);
        }
    }
    let out = out.trim_matches('_').to_string();
    let out = if out.is_empty() { "unnamed".to_string() } else { out };
    if out.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return format!("f_{}", out);
    }
    out
}

fn field_name(name: &str) -> String {
    let mut n = name.to_string();
    if n.is_empty() {
        n = "field".to_string();
    }
    if RUST_KEYWORDS.contains(&n.as_str()) || VEST_RESERVED.contains(&n.as_str()) {
        n.push('_');
    }
    if n.starts_with('$') {
        n = n.trim_matches('$').to_string();
    }
    // `u8`-style names are reserved in Vest.
    if n.len() > 1 && (n.starts_with('u') || n.starts_with('i')) && n[1..].chars().all(|c| c.is_ascii_digit()) {
        n.push('_');
    }
    n
}

fn is_eof_condition(e: &Expr) -> bool {
    match e {
        Expr::Unary { op: UnaryOp::Not, operand } => matches!(
            operand.as_ref(),
            Expr::Call { name, args, .. } if args.is_empty() && name.join("::").ends_with("std::mem::eof")
        ),
        _ => false,
    }
}

/// A field of a struct being emitted.
struct Field {
    name: String,
    format: String,
    /// `const name: fmt = value`
    const_value: Option<String>,
    is_dep: bool,
}

/// Emits a `.vest` translation of `program`.
pub fn emit_vest(program: &Program, endian: Endian, root: Option<&str>) -> Result<VestOutput> {
    let mut em = Emitter {
        endian,
        decls: BTreeMap::new(),
        names: HashMap::new(),
        out: Vec::new(),
        notes: Vec::new(),
        emitted: HashSet::new(),
        in_progress: HashSet::new(),
        failed: HashSet::new(),
        order: Vec::new(),
        root_candidates: Vec::new(),
    };
    em.collect(&program.statements, &[]);
    // Emit everything reachable from the root first, then the rest.
    let root_name = match root {
        Some(r) => Some(r.to_string()),
        None => em.root_candidates.first().cloned(),
    };
    if let Some(r) = &root_name {
        if em.decls.contains_key(r) {
            em.emit_decl(r);
        } else {
            em.notes.push(format!("root type `{}` is not a struct definition", r));
        }
    }
    for name in em.order.clone() {
        em.emit_decl(&name);
    }
    let mut source = String::new();
    source.push_str("// Generated from an ImHex pattern by hexpat's Vest emitter.\n");
    if endian == Endian::Big {
        source.push_str("!BIG_ENDIAN\n");
    }
    source.push('\n');
    for d in &em.out {
        source.push_str(d);
        source.push('\n');
    }
    let root_vest = root_name.as_ref().filter(|r| em.emitted.contains(*r)).map(|r| em.vest_name(r));
    Ok(VestOutput {
        source,
        root: root_vest,
        notes: em.notes,
        emitted: em.emitted.into_iter().collect(),
    })
}

impl Emitter {
    fn collect(&mut self, stmts: &[Stmt], ns: &[String]) {
        for s in stmts {
            let qualify = |n: &str| if ns.is_empty() { n.to_string() } else { format!("{}::{}", ns.join("::"), n) };
            match s {
                Stmt::Namespace { name, body, .. } => {
                    let mut inner = ns.to_vec();
                    inner.extend(name.iter().cloned());
                    self.collect(body, &inner);
                }
                Stmt::Struct(d) => self.add(qualify(&d.name), Decl::Struct(d.clone())),
                Stmt::Union(d) => self.add(qualify(&d.name), Decl::Union),
                Stmt::Bitfield(d) => self.add(qualify(&d.name), Decl::Bitfield(d.clone())),
                Stmt::Enum(d) => self.add(qualify(&d.name), Decl::Enum(d.clone())),
                Stmt::Using { name, ty, template_params, .. } if template_params.is_empty() => {
                    self.add(qualify(name), Decl::Alias(ty.clone()))
                }
                Stmt::Var(v) if v.placement.is_some() && ns.is_empty() => {
                    if let TypeBase::Custom { name, .. } = &v.ty.base {
                        self.root_candidates.push(name.join("::"));
                    }
                }
                Stmt::Array(a) if a.placement.is_some() && ns.is_empty() => {
                    if let TypeBase::Custom { name, .. } = &a.ty.base {
                        self.root_candidates.push(name.join("::"));
                    }
                }
                _ => {}
            }
        }
    }

    fn add(&mut self, name: String, decl: Decl) {
        let vest = snake_case(&name);
        self.names.insert(name.clone(), vest);
        if !self.decls.contains_key(&name) {
            self.order.push(name.clone());
        }
        self.decls.insert(name, decl);
    }

    fn vest_name(&self, hexpat_name: &str) -> String {
        self.names.get(hexpat_name).cloned().unwrap_or_else(|| snake_case(hexpat_name))
    }

    fn note(&mut self, msg: String) {
        if !self.notes.contains(&msg) {
            self.notes.push(msg);
        }
    }

    fn lookup(&self, path: &[String]) -> Option<(String, Decl)> {
        let joined = path.join("::");
        if let Some(d) = self.decls.get(&joined) {
            return Some((joined, d.clone()));
        }
        // Namespace-insensitive fallback: match on the last component.
        let last = path.last()?;
        let mut hit = None;
        for (k, d) in &self.decls {
            if k.rsplit("::").next() == Some(last.as_str()) {
                if hit.is_some() {
                    return None;
                }
                hit = Some((k.clone(), d.clone()));
            }
        }
        hit
    }

    /// Emits a declaration (once). Returns false if it could not be emitted.
    fn emit_decl(&mut self, name: &str) -> bool {
        if self.emitted.contains(name) {
            return true;
        }
        if self.failed.contains(name) || self.in_progress.contains(name) {
            return false;
        }
        let Some(decl) = self.decls.get(name).cloned() else { return false };
        self.in_progress.insert(name.to_string());
        let result = match &decl {
            Decl::Struct(d) => self.emit_struct(name, d),
            Decl::Enum(d) => self.emit_enum(name, d),
            Decl::Bitfield(d) => self.emit_bitfield(name, d),
            Decl::Alias(t) => match self.format_of(t, name) {
                Ok(f) => {
                    let v = self.vest_name(name);
                    self.out.push(format!("{} = {}\n", v, f));
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Decl::Union => Err(Error::new("unions have no Vest equivalent")),
        };
        self.in_progress.remove(name);
        match result {
            Ok(()) => {
                self.emitted.insert(name.to_string());
                true
            }
            Err(e) => {
                self.failed.insert(name.to_string());
                self.note(format!("`{}` not emitted: {}", name, e.message));
                false
            }
        }
    }

    // ------------------------------------------------------------------
    // Types
    // ------------------------------------------------------------------

    /// The Vest format for a hexpat type, emitting the declaration it
    /// refers to first.
    fn format_of(&mut self, ty: &TypeRef, ctx: &str) -> Result<String> {
        if let Some(e) = ty.endian {
            if e != self.endian {
                return Err(Error::new(format!(
                    "field endianness `{}` differs from the file's byte order (Vest byte order is file-global)",
                    if e == Endian::Big { "be" } else { "le" }
                )));
            }
        }
        match &ty.base {
            TypeBase::Builtin(b) => builtin_format(*b, &mut self.notes, ctx),
            TypeBase::Custom { name, args } => {
                let joined = name.join("::");
                match joined.as_str() {
                    "type::Magic" | "Magic" => {
                        // Handled by the caller as a const field.
                        Err(Error::new("type::Magic must be a struct field"))
                    }
                    "std::mem::Bytes" | "Bytes" => {
                        let Some(TemplateArg::Expr(e)) = args.first() else {
                            return Err(Error::new("std::mem::Bytes needs a size"));
                        };
                        let len = self.length_expr(e, &[])?;
                        Ok(format!("[u8; {}]", len))
                    }
                    _ => {
                        if !args.is_empty() {
                            return Err(Error::new(format!("templated type `{}` is not supported", joined)));
                        }
                        let Some((full, decl)) = self.lookup(name) else {
                            return Err(Error::new(format!("unknown type `{}`", joined)));
                        };
                        if let Decl::Alias(t) = &decl {
                            // Builtin aliases are inlined; others are emitted.
                            if let TypeBase::Builtin(_) = t.base {
                                return self.format_of(t, ctx);
                            }
                        }
                        if !self.emit_decl(&full) {
                            return Err(Error::new(format!("type `{}` could not be emitted", full)));
                        }
                        Ok(self.vest_name(&full))
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Expressions used as lengths and choose patterns
    // ------------------------------------------------------------------

    /// Translates a length expression over literals and earlier fields.
    fn length_expr(&mut self, e: &Expr, fields: &[String]) -> Result<String> {
        Ok(match e {
            Expr::Literal(Literal::Unsigned(v)) => v.to_string(),
            Expr::Literal(Literal::Signed(v)) if *v >= 0 => v.to_string(),
            Expr::Literal(Literal::Char(c)) => (*c as u32).to_string(),
            Expr::Ident(name) => {
                if fields.iter().any(|f| f == name) {
                    format!("@{}", field_name(name))
                } else {
                    return Err(Error::new(format!("length refers to `{}`, which is not an earlier field", name)));
                }
            }
            Expr::Member { object, member } => {
                let base = self.length_expr(object, fields)?;
                if !base.starts_with('@') {
                    return Err(Error::new("member access in a length must start at a field"));
                }
                format!("{}.{}", base, field_name(member))
            }
            Expr::Binary { op, left, right } => {
                let l = self.length_expr(left, fields)?;
                let r = self.length_expr(right, fields)?;
                let sym = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    _ => return Err(Error::new(format!("operator {} is not allowed in a length", op.symbol()))),
                };
                format!("({} {} {})", l, sym, r)
            }
            Expr::SizeOf(TypeOrExpr::Type(t)) => {
                let f = self.format_of(t, "sizeof")?;
                format!("|{}|", f)
            }
            Expr::Cast { value, .. } => self.length_expr(value, fields)?,
            _ => return Err(Error::new("length expression is not translatable")),
        })
    }

    /// A constant pattern for `choose` arms: integer, enum variant or bytes.
    fn const_pattern(&mut self, e: &Expr) -> Result<String> {
        Ok(match e {
            Expr::Literal(Literal::Unsigned(v)) => v.to_string(),
            Expr::Literal(Literal::Signed(v)) => v.to_string(),
            Expr::Literal(Literal::Char(c)) => format!("'{}'", *c as char),
            Expr::Literal(Literal::String(s)) => byte_list(s.as_bytes()),
            Expr::Scoped(path) if path.len() >= 2 => path[path.len() - 1].clone(),
            Expr::Cast { value, .. } => self.const_pattern(value)?,
            _ => return Err(Error::new("condition compares against a non-constant")),
        })
    }

    // ------------------------------------------------------------------
    // Declarations
    // ------------------------------------------------------------------

    fn emit_enum(&mut self, name: &str, d: &EnumDef) -> Result<()> {
        let TypeBase::Builtin(b) = d.ty.base else { return Err(Error::new("enum underlying type must be builtin")) };
        let suffix = match b {
            BuiltinType::U8 => "u8",
            BuiltinType::U16 => "u16",
            BuiltinType::U24 => "u24",
            BuiltinType::U32 => "u32",
            BuiltinType::U64 => "u64",
            other => return Err(Error::new(format!("enum backing type `{}` is not supported by Vest", other))),
        };
        let mut lines = Vec::new();
        let mut last: i128 = -1;
        let mut seen = HashSet::new();
        for e in &d.entries {
            let v = match &e.value {
                Some(x) => const_int(x).ok_or_else(|| Error::new(format!("enum entry `{}` has a non-constant value", e.name)))?,
                None => last + 1,
            };
            if e.end.is_some() {
                self.note(format!("enum `{}`: entry `{}` is a range; only its first value is kept", name, e.name));
            }
            last = match &e.end {
                Some(x) => const_int(x).unwrap_or(v),
                None => v,
            };
            if v < 0 {
                return Err(Error::new("negative enum values are not supported"));
            }
            let max = match b {
                BuiltinType::U8 => 0xFF,
                BuiltinType::U16 => 0xFFFF,
                BuiltinType::U24 => 0xFF_FFFF,
                BuiltinType::U32 => 0xFFFF_FFFF,
                _ => i128::MAX,
            };
            if v == max {
                // Vest's open enums append an `Unknown` variant after the last
                // entry, so the maximum value cannot be a named entry.
                self.note(format!("enum `{}`: entry `{}` uses the maximum value {} and is dropped (open enums reserve it)", name, e.name, v));
                continue;
            }
            if seen.insert(v) {
                // `Unknown` is the variant Vest adds to open enums.
                let mut ename = field_name(&e.name);
                if ename == "Unknown" {
                    ename.push('_');
                }
                lines.push(format!("    {} = {}{},", ename, v, suffix));
            } else {
                self.note(format!("enum `{}`: duplicate value {} for `{}` dropped", name, v, e.name));
            }
        }
        let v = self.vest_name(name);
        // hexpat enums accept unknown values, so the Vest enum is open.
        self.out.push(format!("{} = enum {{\n{}\n    ...\n}}\n", v, lines.join("\n")));
        Ok(())
    }

    fn emit_bitfield(&mut self, name: &str, d: &StructDef) -> Result<()> {
        let mut fields: Vec<(String, u64)> = Vec::new();
        let mut order_msb_first = false;
        let mut declared_total: Option<u64> = None;
        for a in &d.attrs {
            if a.is("bitfield_order") && a.args.len() == 2 {
                if let Expr::Scoped(p) = &a.args[0] {
                    order_msb_first = p.last().map(|s| s == "MostToLeastSignificant").unwrap_or(false);
                }
                declared_total = const_int(&a.args[1]).map(|v| v as u64);
            }
        }
        let mut pad = 0;
        for s in &d.body {
            match s {
                Stmt::BitfieldField(f) => {
                    let bits = const_int(&f.size).ok_or_else(|| Error::new("bitfield field size must be constant"))? as u64;
                    let fname = if f.name == "$padding$" {
                        pad += 1;
                        format!("_pad{}", pad)
                    } else {
                        field_name(&f.name)
                    };
                    if f.signed {
                        self.note(format!("bitfield `{}`: signed field `{}` emitted as unsigned bits", name, f.name));
                    }
                    fields.push((fname, bits));
                }
                _ => return Err(Error::new("bitfield contains statements other than bit fields")),
            }
        }
        let used: u64 = fields.iter().map(|(_, b)| b).sum();
        let total = declared_total.unwrap_or(used.div_ceil(8) * 8);
        if !matches!(total, 8 | 16 | 24 | 32 | 64) {
            return Err(Error::new(format!("bitfield width {} bits is not 8/16/24/32/64", total)));
        }
        if used > total {
            return Err(Error::new("bitfield fields exceed the declared width"));
        }
        // Vest lays fields out from the most significant bit; hexpat's
        // default is least-significant first, so reverse in that case.
        let mut layout: Vec<(String, u64)> = Vec::new();
        if order_msb_first {
            layout.extend(fields);
            if used < total {
                layout.push(("_tail".to_string(), total - used));
            }
        } else {
            if used < total {
                layout.push(("_tail".to_string(), total - used));
            }
            layout.extend(fields.into_iter().rev());
        }
        let v = self.vest_name(name);
        let body: Vec<String> = layout.iter().map(|(n, b)| format!("    {}: u{},", n, b)).collect();
        self.out.push(format!("{} = bits {{\n{}\n}}\n", v, body.join("\n")));
        Ok(())
    }

    fn emit_struct(&mut self, name: &str, d: &StructDef) -> Result<()> {
        let mut fields: Vec<Field> = Vec::new();
        let mut body: Vec<Stmt> = Vec::new();
        self.flatten_parents(d, &mut body, 0)?;
        // Names referenced by later lengths/conditions must be `@` deps.
        let mut deps: HashSet<String> = HashSet::new();
        for s in &body {
            collect_refs(s, &mut deps);
        }
        let mut pad = 0;
        let n = body.len();
        for (i, s) in body.iter().enumerate() {
            let is_last = i + 1 == n;
            match s {
                Stmt::Var(v) => {
                    if v.placement.is_some() {
                        return Err(Error::new(format!("member `{}` is placed with `@`", v.name)));
                    }
                    if v.value.is_some() || v.in_var || v.out_var {
                        if deps.contains(&v.name) {
                            return Err(Error::new(format!("local variable `{}` feeds a later size or condition", v.name)));
                        }
                        self.note(format!("`{}`: local variable `{}` skipped", name, v.name));
                        continue;
                    }
                    if v.attrs.iter().any(|a| a.is("no_unique_address")) {
                        self.note(format!("`{}`: `{}` has no_unique_address and is skipped", name, v.name));
                        continue;
                    }
                    if let TypeBase::Custom { name: tn, args } = &v.ty.base {
                        let joined = tn.join("::");
                        if joined == "type::Magic" || joined == "Magic" {
                            let Some(TemplateArg::Expr(Expr::Literal(Literal::String(s)))) = args.first() else {
                                return Err(Error::new("type::Magic needs a string argument"));
                            };
                            fields.push(Field {
                                name: field_name(if v.name.is_empty() { "magic" } else { &v.name }),
                                format: format!("[u8; {}]", s.len()),
                                const_value: Some(byte_list(s.as_bytes())),
                                is_dep: false,
                            });
                            continue;
                        }
                    }
                    let fmt = self.format_of(&v.ty, name)?;
                    let fname = if v.name.is_empty() {
                        pad += 1;
                        format!("_anon{}", pad)
                    } else {
                        field_name(&v.name)
                    };
                    fields.push(Field { name: fname, format: fmt, const_value: None, is_dep: deps.contains(&v.name) });
                }
                Stmt::Array(a) => {
                    if a.placement.is_some() {
                        return Err(Error::new(format!("member `{}` is placed with `@`", a.name)));
                    }
                    if a.init.is_some() {
                        self.note(format!("`{}`: local array `{}` skipped", name, a.name));
                        continue;
                    }
                    if a.attrs.iter().any(|x| x.is("no_unique_address")) {
                        self.note(format!("`{}`: `{}` has no_unique_address and is skipped", name, a.name));
                        continue;
                    }
                    let is_bytes = matches!(a.ty.base, TypeBase::Builtin(BuiltinType::U8) | TypeBase::Builtin(BuiltinType::Char));
                    let elem = self.format_of(&a.ty, name)?;
                    let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                    let raw_names: Vec<String> = body[..i].iter().filter_map(stmt_name).collect();
                    let _ = field_names;
                    let fmt = match &a.size {
                        ArraySize::Fixed(e) => {
                            let len = self.length_expr(e, &raw_names)?;
                            if is_bytes {
                                format!("[u8; {}]", len)
                            } else {
                                format!("[{}; {}]", elem, len)
                            }
                        }
                        ArraySize::While(cond) if is_last && is_eof_condition(cond) => format!("Vec<{}>", elem),
                        ArraySize::While(_) => {
                            return Err(Error::new(format!("while-sized array `{}` (only a trailing `while(!std::mem::eof())` translates)", a.name)))
                        }
                        ArraySize::Unsized => {
                            return Err(Error::new(format!("unsized (null-terminated) array `{}`", a.name)));
                        }
                    };
                    fields.push(Field { name: field_name(&a.name), format: fmt, const_value: None, is_dep: deps.contains(&a.name) });
                }
                Stmt::Padding { size: ArraySize::Fixed(e), .. } => {
                    let raw_names: Vec<String> = body[..i].iter().filter_map(stmt_name).collect();
                    let len = self.length_expr(e, &raw_names)?;
                    pad += 1;
                    fields.push(Field { name: format!("_pad{}", pad), format: format!("[u8; {}]", len), const_value: None, is_dep: false });
                }
                Stmt::Padding { .. } => return Err(Error::new("while-sized padding")),
                Stmt::If { cond, then, otherwise, .. } => {
                    let raw_names: Vec<String> = body[..i].iter().filter_map(stmt_name).collect();
                    let f = self.emit_choice(name, cond, then, otherwise, &raw_names, &mut pad)?;
                    fields.push(f);
                }
                Stmt::Match { values, cases, .. } => {
                    if values.len() != 1 {
                        return Err(Error::new("match on several values"));
                    }
                    let raw_names: Vec<String> = body[..i].iter().filter_map(stmt_name).collect();
                    let f = self.emit_match(name, &values[0], cases, &raw_names, &mut pad)?;
                    fields.push(f);
                }
                Stmt::Expr { expr, .. } => {
                    let what = match expr {
                        Expr::Call { name: n, .. } => n.join("::"),
                        _ => "expression".to_string(),
                    };
                    self.note(format!("`{}`: statement `{}` has no format meaning and is skipped", name, what));
                }
                Stmt::Pointer(p) => return Err(Error::new(format!("pointer member `{}`", p.name))),
                Stmt::Assign { .. } => return Err(Error::new("assignment inside a struct body")),
                Stmt::BitfieldField(f) => return Err(Error::new(format!("bit field `{}` inside a struct", f.name))),
                Stmt::TryCatch { .. } | Stmt::While { .. } | Stmt::For { .. } | Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {
                    return Err(Error::new("control flow inside a struct body"))
                }
                Stmt::Import { .. } | Stmt::Using { .. } | Stmt::UsingForward { .. } | Stmt::Namespace { .. } | Stmt::Struct(_) | Stmt::Union(_) | Stmt::Enum(_) | Stmt::Bitfield(_) | Stmt::Function(_) => {
                    return Err(Error::new("definition inside a struct body"))
                }
            }
        }
        if fields.is_empty() {
            return Err(Error::new("struct has no translatable fields"));
        }
        let v = self.vest_name(name);
        let mut lines = Vec::new();
        for f in &fields {
            match &f.const_value {
                Some(c) => lines.push(format!("    const {}: {} = {},", f.name, f.format, c)),
                None => lines.push(format!("    {}{}: {},", if f.is_dep { "@" } else { "" }, f.name, f.format)),
            }
        }
        self.out.push(format!("{} = {{\n{}\n}}\n", v, lines.join("\n")));
        Ok(())
    }

    /// Appends the members of `d`, preceded by those of its parents
    /// (recursively), to `out`.
    fn flatten_parents(&mut self, d: &StructDef, out: &mut Vec<Stmt>, depth: usize) -> Result<()> {
        if depth > 16 {
            return Err(Error::new("inheritance chain too deep"));
        }
        for p in &d.parents {
            let TypeBase::Custom { name: pn, .. } = &p.base else { return Err(Error::new("bad parent type")) };
            match self.lookup(pn) {
                Some((_, Decl::Struct(pd))) => self.flatten_parents(&pd, out, depth + 1)?,
                _ => return Err(Error::new(format!("parent `{}` is not a struct", pn.join("::")))),
            }
        }
        out.extend(d.body.iter().cloned());
        Ok(())
    }

    /// `if (dep == C) {..} else if (dep == D) {..} else {..}` -> choose.
    fn emit_choice(&mut self, owner: &str, cond: &Expr, then: &[Stmt], otherwise: &[Stmt], names: &[String], pad: &mut usize) -> Result<Field> {
        let mut arms: Vec<(Vec<String>, Vec<Stmt>)> = Vec::new();
        let mut dep: Option<Expr> = None;
        let mut cur_cond = cond.clone();
        let mut cur_then = then.to_vec();
        let mut cur_else = otherwise.to_vec();
        let default;
        loop {
            let (d, pats) = split_condition(&cur_cond)?;
            match &dep {
                None => dep = Some(d),
                Some(existing) if *existing == d => {}
                Some(_) => return Err(Error::new("if/else chain tests different fields")),
            }
            let mut pat_strings = Vec::new();
            for p in pats {
                pat_strings.push(self.const_pattern(&p)?);
            }
            arms.push((pat_strings, cur_then.clone()));
            if cur_else.len() == 1 {
                if let Stmt::If { cond: c2, then: t2, otherwise: o2, .. } = &cur_else[0] {
                    cur_cond = c2.clone();
                    cur_then = t2.clone();
                    cur_else = o2.clone();
                    continue;
                }
            }
            default = cur_else;
            break;
        }
        let dep = dep.unwrap();
        self.build_choose(owner, &dep, arms, Some(default), names, pad)
    }

    fn emit_match(&mut self, owner: &str, value: &Expr, cases: &[MatchCase], names: &[String], pad: &mut usize) -> Result<Field> {
        let mut arms = Vec::new();
        let mut default = None;
        for c in cases {
            match &c.patterns[0] {
                CasePattern::Wildcard => default = Some(c.body.clone()),
                CasePattern::Alternatives(alts) => {
                    let mut pats = Vec::new();
                    for (a, b) in alts {
                        if b.is_some() {
                            return Err(Error::new("range patterns in match are not translatable to choose"));
                        }
                        pats.push(self.const_pattern(a)?);
                    }
                    arms.push((pats, c.body.clone()));
                }
            }
        }
        self.build_choose(owner, value, arms, default, names, pad)
    }

    fn build_choose(
        &mut self,
        owner: &str,
        dep: &Expr,
        arms: Vec<(Vec<String>, Vec<Stmt>)>,
        default: Option<Vec<Stmt>>,
        names: &[String],
        pad: &mut usize,
    ) -> Result<Field> {
        let dep_str = self.length_expr(dep, names)?;
        if !dep_str.starts_with('@') {
            return Err(Error::new("condition does not test an earlier field"));
        }
        // The dependency's format decides how patterns are written.
        let dep_field = dep_str.trim_start_matches('@').split('.').next().unwrap().to_string();
        let mut lines = Vec::new();
        let mut arm_index = 0;
        let mut chosen_name: Option<String> = None;
        for (pats, body) in &arms {
            arm_index += 1;
            let fmt = self.branch_format(owner, body, &dep_field, arm_index, names, pad)?;
            if let Some(n) = branch_single_name(body) {
                chosen_name.get_or_insert(n);
            }
            for p in pats {
                lines.push(format!("        {} => {},", p, fmt));
            }
        }
        let default_fmt = match &default {
            Some(body) if !body.is_empty() => {
                arm_index += 1;
                self.branch_format(owner, body, &dep_field, arm_index, names, pad)?
            }
            _ => "Nothing".to_string(),
        };
        lines.push(format!("        _ => {},", default_fmt));
        let name = chosen_name.map(|n| field_name(&n)).unwrap_or_else(|| format!("{}_body", dep_field));
        Ok(Field { name, format: format!("choose({}) {{\n{}\n    }}", dep_str, lines.join("\n")), const_value: None, is_dep: false })
    }

    /// The format of one branch body: a single member's format, or a
    /// synthesized struct for several members.
    fn branch_format(&mut self, owner: &str, body: &[Stmt], dep_field: &str, index: usize, names: &[String], pad: &mut usize) -> Result<String> {
        let members: Vec<&Stmt> = body.iter().filter(|s| !matches!(s, Stmt::Expr { .. })).collect();
        if members.is_empty() {
            return Ok("Nothing".to_string());
        }
        if members.len() == 1 {
            if let Some(f) = self.simple_member_format(members[0], names)? {
                return Ok(f);
            }
        }
        // Synthesize a struct for the branch.
        let synth_name = format!("{}_{}_{}", owner, dep_field, index);
        let def = StructDef { line: 0, name: synth_name.clone(), template_params: Vec::new(), parents: Vec::new(), body: body.to_vec(), attrs: Vec::new() };
        self.add(synth_name.clone(), Decl::Struct(def));
        let _ = pad;
        if !self.emit_decl(&synth_name) {
            return Err(Error::new(format!("branch of `{}` could not be emitted", owner)));
        }
        Ok(self.vest_name(&synth_name))
    }

    fn simple_member_format(&mut self, s: &Stmt, names: &[String]) -> Result<Option<String>> {
        Ok(match s {
            Stmt::Var(v) if v.placement.is_none() && v.value.is_none() && !v.in_var && !v.out_var => {
                if let TypeBase::Custom { name: tn, .. } = &v.ty.base {
                    if tn.join("::").ends_with("Magic") {
                        return Ok(None);
                    }
                }
                Some(self.format_of(&v.ty, &v.name)?)
            }
            Stmt::Array(a) if a.placement.is_none() && a.init.is_none() => {
                let is_bytes = matches!(a.ty.base, TypeBase::Builtin(BuiltinType::U8) | TypeBase::Builtin(BuiltinType::Char));
                let elem = self.format_of(&a.ty, &a.name)?;
                match &a.size {
                    ArraySize::Fixed(e) => {
                        let len = self.length_expr(e, names)?;
                        Some(if is_bytes { format!("[u8; {}]", len) } else { format!("[{}; {}]", elem, len) })
                    }
                    _ => None,
                }
            }
            _ => None,
        })
    }
}

fn builtin_format(b: BuiltinType, notes: &mut Vec<String>, ctx: &str) -> Result<String> {
    use BuiltinType::*;
    Ok(match b {
        U8 | Char | Bool => "u8".to_string(),
        U16 | Char16 => "u16".to_string(),
        U24 => "u24".to_string(),
        U32 => "u32".to_string(),
        U64 => "u64".to_string(),
        S8 | S16 | S24 | S32 | S64 => {
            let note = format!("`{}`: signed `{}` emitted as its unsigned width (Vest DSL has no signed integers yet)", ctx, b);
            if !notes.contains(&note) {
                notes.push(note);
            }
            match b {
                S8 => "u8",
                S16 => "u16",
                S24 => "u24",
                S32 => "u32",
                _ => "u64",
            }
            .to_string()
        }
        Float | Double => {
            let note = format!("`{}`: `{}` emitted as raw {} bits", ctx, b, if b == Float { 32 } else { 64 });
            if !notes.contains(&note) {
                notes.push(note);
            }
            if b == Float { "u32".to_string() } else { "u64".to_string() }
        }
        U48 | U96 | U128 | S48 | S96 | S128 => return Err(Error::new(format!("`{}` has no Vest equivalent", b))),
        Str | Auto => return Err(Error::new(format!("`{}` cannot be a wire field", b))),
    })
}

fn byte_list(bytes: &[u8]) -> String {
    let items: Vec<String> = bytes.iter().map(|b| format!("0x{:02X}", b)).collect();
    format!("[{}]", items.join(", "))
}

fn const_int(e: &Expr) -> Option<i128> {
    match e {
        Expr::Literal(Literal::Unsigned(v)) => Some(*v as i128),
        Expr::Literal(Literal::Signed(v)) => Some(*v),
        Expr::Literal(Literal::Char(c)) => Some(*c as i128),
        Expr::Unary { op: UnaryOp::Neg, operand } => const_int(operand).map(|v| -v),
        Expr::Binary { op, left, right } => {
            let l = const_int(left)?;
            let r = const_int(right)?;
            Some(match op {
                BinaryOp::Add => l + r,
                BinaryOp::Sub => l - r,
                BinaryOp::Mul => l * r,
                BinaryOp::Shl => l << r,
                BinaryOp::Shr => l >> r,
                BinaryOp::BitOr => l | r,
                BinaryOp::BitAnd => l & r,
                _ => return None,
            })
        }
        Expr::Cast { value, .. } => const_int(value),
        _ => None,
    }
}

fn stmt_name(s: &Stmt) -> Option<String> {
    match s {
        Stmt::Var(v) if v.placement.is_none() && !v.name.is_empty() => Some(v.name.clone()),
        Stmt::Array(a) if a.placement.is_none() => Some(a.name.clone()),
        _ => None,
    }
}

/// The single member name of a branch body, if it has exactly one member.
fn branch_single_name(body: &[Stmt]) -> Option<String> {
    let members: Vec<&Stmt> = body.iter().filter(|s| !matches!(s, Stmt::Expr { .. })).collect();
    if members.len() == 1 {
        stmt_name(members[0])
    } else {
        None
    }
}

/// Splits `dep == C1 || dep == C2` into (dep, [C1, C2]).
fn split_condition(cond: &Expr) -> Result<(Expr, Vec<Expr>)> {
    match cond {
        Expr::Binary { op: BinaryOp::Eq, left, right } => {
            if is_const(right) {
                Ok((left.as_ref().clone(), vec![right.as_ref().clone()]))
            } else if is_const(left) {
                Ok((right.as_ref().clone(), vec![left.as_ref().clone()]))
            } else {
                Err(Error::new("condition is not `field == constant`"))
            }
        }
        Expr::Binary { op: BinaryOp::Or, left, right } => {
            let (dl, mut pl) = split_condition(left)?;
            let (dr, pr) = split_condition(right)?;
            if dl != dr {
                return Err(Error::new("`||` condition tests different fields"));
            }
            pl.extend(pr);
            Ok((dl, pl))
        }
        _ => Err(Error::new("condition is not `field == constant`")),
    }
}

fn is_const(e: &Expr) -> bool {
    match e {
        Expr::Literal(_) => true,
        Expr::Scoped(p) => p.len() >= 2,
        Expr::Cast { value, .. } => is_const(value),
        _ => false,
    }
}

/// Collects identifiers used in sizes and conditions of a statement.
fn collect_refs(s: &Stmt, out: &mut HashSet<String>) {
    fn expr_refs(e: &Expr, out: &mut HashSet<String>) {
        match e {
            Expr::Ident(n) => {
                out.insert(n.clone());
            }
            Expr::Member { object, .. } => expr_refs(object, out),
            Expr::Binary { left, right, .. } => {
                expr_refs(left, out);
                expr_refs(right, out);
            }
            Expr::Unary { operand, .. } => expr_refs(operand, out),
            Expr::Cast { value, .. } => expr_refs(value, out),
            Expr::Ternary { cond, then, otherwise } => {
                expr_refs(cond, out);
                expr_refs(then, out);
                expr_refs(otherwise, out);
            }
            _ => {}
        }
    }
    match s {
        Stmt::Array(a) => {
            if let ArraySize::Fixed(e) = &a.size {
                expr_refs(e, out);
            }
        }
        Stmt::Padding { size: ArraySize::Fixed(e), .. } => expr_refs(e, out),
        Stmt::If { cond, then, otherwise, .. } => {
            expr_refs(cond, out);
            for s in then.iter().chain(otherwise.iter()) {
                collect_refs(s, out);
            }
        }
        Stmt::Match { values, cases, .. } => {
            for v in values {
                expr_refs(v, out);
            }
            for c in cases {
                for s in &c.body {
                    collect_refs(s, out);
                }
            }
        }
        Stmt::Var(v) => {
            if let TypeBase::Custom { args, .. } = &v.ty.base {
                for a in args {
                    if let TemplateArg::Expr(e) = a {
                        expr_refs(e, out);
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_names() {
        assert_eq!(snake_case("BitmapFileHeader"), "bitmap_file_header");
        assert_eq!(snake_case("chunk_t"), "chunk_t");
        assert_eq!(snake_case("IHDR"), "ihdr");
        assert_eq!(snake_case("type::GUID"), "type_guid");
        assert_eq!(snake_case("Elf64_Ehdr"), "elf64_ehdr");
    }

    #[test]
    fn emits_struct_with_dependency_and_choose() {
        let src = r#"
            enum Kind : u8 { A = 1, B = 2 };
            struct Body { u8 x; u16 y; };
            struct Msg {
                Kind kind;
                u16 len;
                u8 payload[len];
                if (kind == Kind::A) { Body body; } else if (kind == Kind::B) { u32 word; }
                padding[2];
            };
            Msg m @ 0;
        "#;
        let program = crate::parse_program(src).unwrap();
        let out = emit_vest(&program, Endian::Little, None).unwrap();
        assert_eq!(out.root.as_deref(), Some("msg"));
        assert!(out.source.contains("kind = enum {\n    A = 1u8,\n    B = 2u8,\n    ...\n}"), "{}", out.source);
        assert!(out.source.contains("@len: u16,"), "{}", out.source);
        assert!(out.source.contains("payload: [u8; @len],"), "{}", out.source);
        assert!(out.source.contains("choose(@kind) {\n        A => body,\n        B => u32,\n        _ => Nothing,\n    }"), "{}", out.source);
        assert!(out.source.contains("_pad1: [u8; 2],"), "{}", out.source);
    }

    #[test]
    fn bitfields_are_reversed_for_lsb_first() {
        let src = "bitfield F { a : 3; b : 1; padding : 4; }; F f @ 0;";
        let program = crate::parse_program(src).unwrap();
        let out = emit_vest(&program, Endian::Little, Some("F")).unwrap();
        assert!(out.source.contains("f = bits {\n    _pad1: u4,\n    b: u1,\n    a: u3,\n}"), "{}", out.source);
    }
}
