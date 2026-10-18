//! Translation of the declarative subset of hexpat into ASN.1 plus ACN, the
//! input of [ASN1SCC](https://github.com/esa/asn1scc).
//!
//! ASN1SCC generates encoders and decoders from the pair; its Scala backend
//! annotates them for the Stainless verifier (runtime safety, exact encoded
//! sizes and, with `-stainlessInvertibility`, `decode(encode(v)) == v`).
//!
//! | hexpat                                   | ASN.1                                  | ACN                                         |
//! |------------------------------------------|----------------------------------------|---------------------------------------------|
//! | `u16` (little endian)                    | `INTEGER (0..65535)`                   | `[encoding pos-int, size 16, endianness little]` |
//! | `s32`                                    | `INTEGER (-2147483648..2147483647)`    | `[encoding twos-complement, size 32]`       |
//! | `float` / `double`                       | `REAL`                                 | `[encoding IEEE754-1985-32]` / `-64`        |
//! | `struct S { .. }`                        | `S ::= SEQUENCE { .. }`                | `S [] { .. }`                               |
//! | `enum E : u8 { .. }`                     | `E ::= INTEGER (0..255)` (open)        | `E [encoding pos-int, size 8]`              |
//! | `char x[4]` / `u8 x[4]`                  | `OCTET STRING (SIZE(4))`               | `[]`                                        |
//! | `u8 x[len]`, `T x[len]`                  | `OCTET STRING (SIZE(0..max))` / `SEQUENCE (SIZE(0..max)) OF T` | `len INTEGER [..]` inserted, `x [size len]` |
//! | `char s[]` (NUL terminated)              | `OCTET STRING (SIZE(0..max))`          | `[size null-terminated, termination-pattern '00'H]` |
//! | `type::Magic<"GIF">`                     | `magic INTEGER (4671814)`              | `[encoding pos-int, size 24]`               |
//! | `if (c) { T a; } else { U b; }`          | `a T OPTIONAL, b U OPTIONAL`           | `a [present-when (c)], b [present-when (!(c))]` |
//! | `match (f) { (1): ..; (_): ..; }`        | OPTIONAL components                    | `present-when (f == 1)` / `(!(f == 1))`     |
//! | `bitfield B { a : 3; b : 5; }`           | `SEQUENCE { b INTEGER (0..31), a INTEGER (0..7) }` | `[encoding pos-int, size 5]`, ...  |
//! | `padding[n]`                             | `padding-1 OCTET STRING (SIZE(n))`     | `[]`                                        |
//!
//! ACN only lets ACN-inserted fields determine sizes, and only plain
//! (non-optional) ASN.1 fields of the same SEQUENCE appear in `present-when`
//! expressions, so:
//!
//! - a hexpat field used as an array length becomes an ACN-inserted field.
//!   Its value is not part of the ASN.1 value; the encoder recomputes it from
//!   the array. When every array that depends on it is conditional, the value
//!   cannot be recovered for the other branches and the field is listed in
//!   [`Asn1Output::lossy`].
//! - a condition or length that refers to `child.field` inlines the
//!   components of `child` into the parent (named `child-field`).
//! - a `char[n]` field (n ≤ 8) compared with a string literal is read as an
//!   `n`-byte big-endian INTEGER so that the comparison can be expressed.
//! - variable sizes need an upper bound (ASN1SCC sizes every buffer for the
//!   worst case); it is the smaller of the length field's range and
//!   [`Asn1Options::max_len`].
//! - hexpat enums accept values without a name, so they become INTEGER types
//!   with the names in a comment rather than closed ENUMERATED types.
//! - a trailing `T x[while(!std::mem::eof())]` in the root struct has no ACN
//!   form in the Scala backend. It is left out of the root SEQUENCE and
//!   reported in [`Asn1Output::trailing`]; the caller decodes `T` repeatedly
//!   until the input ends.
//!
//! Everything else (placements, pointers, `$`, functions, unions, locals
//! that feed sizes, arithmetic in sizes, bit-level fields that straddle
//! little-endian bytes) is reported in [`Asn1Output::notes`], and a struct
//! that depends on it is left out.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;

use crate::ast::*;
use crate::error::{Error, Result};

/// Options for [`emit_asn1`].
#[derive(Debug, Clone)]
pub struct Asn1Options {
    /// ASN.1 module name (an upper-case identifier is derived from it).
    pub module: String,
    /// Root type override (hexpat name); default: the first top-level
    /// `Type name @ addr` placement.
    pub root: Option<String>,
    /// Upper bound for variable-size strings and arrays.
    pub max_len: u64,
    /// Encode `s16`/`s32`/`s64` as two's complement. Off by default: the
    /// Scala runtime of ASN1SCC has the proofs (`_pure` decoders and prefix
    /// lemmas) that `-stainlessInvertibility` needs only for 8-bit two's
    /// complement, so wider signed fields are read as unsigned integers of
    /// the same width.
    pub twos_complement: bool,
    /// Emit enums as INTEGER components of the SEQUENCEs that use them
    /// instead of named INTEGER types. ASN1SCC's invertibility lemmas for a
    /// standalone INTEGER type assignment do not follow from the generated
    /// contracts; inside a SEQUENCE they do.
    pub inline_enums: bool,
}

impl Default for Asn1Options {
    fn default() -> Self {
        Asn1Options { module: "Hexpat".into(), root: None, max_len: 1 << 20, twos_complement: false, inline_enums: false }
    }
}

/// Output of [`emit_asn1`].
#[derive(Debug, Default, Clone)]
pub struct Asn1Output {
    /// ASN.1 module name.
    pub module: String,
    /// The `.asn` source.
    pub asn: String,
    /// The `.acn` source.
    pub acn: String,
    /// ASN.1 name of the root type, if it has any component left.
    pub root: Option<String>,
    /// The root's trailing `while(!std::mem::eof())` array.
    pub trailing: Option<Trailing>,
    /// Every emitted type, for tools that compare decoded values with the
    /// hexpat interpreter.
    pub types: Vec<TypeMap>,
    /// Constructs that could not be translated, or were translated with a
    /// caveat.
    pub notes: Vec<String>,
    /// ACN-inserted fields whose value is not recoverable on re-encoding for
    /// some inputs (`Type.field: reason`).
    pub lossy: Vec<String>,
    /// hexpat names of the emitted declarations.
    pub emitted: Vec<String>,
    /// The `max_len` the output was generated with.
    pub max_len: u64,
}

/// A trailing `T x[while(!std::mem::eof())]` array of the root struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Trailing {
    /// hexpat member name.
    pub field: String,
    /// ASN.1 name of the element type.
    pub element: String,
}

/// An emitted ASN.1 type.
#[derive(Debug, Clone)]
pub struct TypeMap {
    /// ASN.1 type name.
    pub name: String,
    /// hexpat declaration it comes from.
    pub hexpat: String,
    /// `None` for SEQUENCE types; the value shape for aliases (enums).
    pub alias: Option<Shape>,
    /// SEQUENCE components in ASN.1 order (including ACN-only fields).
    pub components: Vec<ComponentMap>,
}

/// A SEQUENCE component and where its value lives in the hexpat tree.
#[derive(Debug, Clone)]
pub struct ComponentMap {
    /// ASN.1 identifier.
    pub name: String,
    /// Member path relative to the enclosing hexpat struct (`a` or `a.b`).
    pub path: String,
    pub role: Role,
    pub shape: Shape,
    pub optional: bool,
}

/// How a component appears in the decoded ASN.1 value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    /// An ASN.1 component.
    Field,
    /// An ACN-inserted length field (not in the ASN.1 value).
    Inserted,
    /// A `type::Magic`: a single-value INTEGER (or bytes when longer than 8).
    Magic,
    /// Padding bytes or bits (in the ASN.1 value, not shown by hexpat).
    Padding,
}

/// The shape of a decoded value.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Int,
    /// A hexpat signed integer of this many bits read as unsigned.
    Unsigned(u32),
    Real,
    Bytes,
    /// A `char[n]` read as an `n`-byte big-endian integer.
    CharsInt(u32),
    Ref(String),
    SeqOf(Box<Shape>),
    Null,
}

// ----------------------------------------------------------------------
// Internal model
// ----------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Size {
    Fixed(u64),
    /// Determined by an ACN-inserted component (by id).
    Det { field: String, max: u64 },
    NullTerminated { max: u64 },
}

#[derive(Debug, Clone)]
enum Ty {
    /// `as_unsigned`: a hexpat signed field read as unsigned.
    Int { min: i128, max: i128, bits: u32, signed: bool, little: bool, as_unsigned: bool },
    Real { double: bool, little: bool },
    Octets(Size),
    SeqOf(Size, Box<Ty>),
    /// A named type, by hexpat key.
    Ref(String),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
enum Cond {
    Field(String),
    Int(i128),
    Bin(&'static str, Box<Cond>, Box<Cond>),
    Not(Box<Cond>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Field,
    Inserted,
    Magic,
    Padding,
}

#[derive(Debug, Clone)]
struct Comp {
    /// Unique id inside the struct; also the reference name used by sizes
    /// and conditions (`a`, `a.b` once inlined, `a#2` for a repeated name).
    id: String,
    /// hexpat member path relative to the struct.
    path: String,
    kind: Kind,
    ty: Ty,
    present: Option<Cond>,
    /// Bytes of a magic.
    magic: Vec<u8>,
    /// `char[n]` turned into an integer for a string comparison.
    chars_int: bool,
    /// Final ASN.1 name.
    asn: String,
}

impl Comp {
    fn new(id: &str, path: &str, kind: Kind, ty: Ty) -> Comp {
        Comp { id: id.into(), path: path.into(), kind, ty, present: None, magic: Vec::new(), chars_int: false, asn: String::new() }
    }
}

#[derive(Debug, Clone)]
enum Body {
    Seq(Vec<Comp>),
    Alias(Ty),
}

#[derive(Debug, Clone)]
struct TypeDef {
    key: String,
    asn: String,
    body: Body,
    comment: Vec<String>,
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
    max_len: u64,
    twos_complement: bool,
    inline_enums: bool,
    decls: BTreeMap<String, Decl>,
    order: Vec<String>,
    root_candidates: Vec<String>,
    types: Vec<TypeDef>,
    index: HashMap<String, usize>,
    in_progress: HashSet<String>,
    failed: HashSet<String>,
    notes: Vec<String>,
    lossy: Vec<String>,
    trailing: Option<(String, String)>,
    root_key: Option<String>,
}

/// Words that ASN1SCC rejects as identifiers (C and Ada keywords) or that
/// break the Scala, Rust or Python output.
const KEYWORDS: &[&str] = &[
    // C
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float",
    "for", "goto", "if", "int", "long", "register", "return", "short", "signed", "sizeof", "static", "struct", "switch",
    "typedef", "union", "unsigned", "void", "volatile", "while",
    // Ada
    "abort", "new", "abs", "elsif", "not", "reverse", "abstract", "end", "null", "accept", "entry", "select", "access",
    "exception", "of", "separate", "aliased", "exit", "or", "some", "all", "others", "subtype", "and", "out",
    "synchronized", "array", "function", "overriding", "at", "tagged", "generic", "package", "task", "begin", "pragma",
    "terminate", "body", "private", "then", "procedure", "type", "in", "protected", "constant", "interface", "until",
    "is", "raise", "use", "declare", "range", "delay", "limited", "record", "when", "delta", "loop", "rem", "digits",
    "renames", "with", "mod", "requeue", "xor",
    // Scala
    "catch", "class", "def", "export", "extends", "false", "final", "finally", "given", "implicit", "import", "lazy",
    "match", "object", "override", "sealed", "super", "throw", "trait", "true", "try", "val", "var", "yield",
    // Rust
    "as", "crate", "fn", "impl", "let", "move", "mut", "pub", "ref", "self", "trait", "where", "async", "await", "dyn",
    "become", "box", "macro", "priv", "typeof", "unsized", "virtual",
    // Python
    "assert", "del", "elif", "except", "from", "global", "lambda", "nonlocal", "pass", "none",
    // ASN1SCC runtime names
    "codec", "pval", "ret", "result",
];

/// Type names that collide with the generated runtime or the target
/// languages' standard types.
const RESERVED_TYPES: &[&str] = &[
    "Vector", "Option", "Some", "None", "List", "Either", "Left", "Right", "Byte", "Short", "Int", "Long", "Float",
    "Double", "Boolean", "Char", "String", "Unit", "Array", "Any", "Object", "BitStream", "Codec", "ACN", "UPER", "PER",
    "UByte", "UShort", "UInt", "ULong", "NullType", "ErrorCode", "Asn1Real", "Asn1Boolean", "OptionMut", "EitherMut",
    "SomeMut", "NoneMut", "LeftMut", "RightMut", "Integer", "Real", "Boolean", "Null", "Set", "Map", "Main",
];

/// Emits an ASN.1 + ACN translation of `program`.
pub fn emit_asn1(program: &Program, endian: Endian, opts: &Asn1Options) -> Result<Asn1Output> {
    let mut em = Emitter {
        endian,
        max_len: opts.max_len.max(1),
        twos_complement: opts.twos_complement,
        inline_enums: opts.inline_enums,
        decls: BTreeMap::new(),
        order: Vec::new(),
        root_candidates: Vec::new(),
        types: Vec::new(),
        index: HashMap::new(),
        in_progress: HashSet::new(),
        failed: HashSet::new(),
        notes: Vec::new(),
        lossy: Vec::new(),
        trailing: None,
        root_key: None,
    };
    em.collect(&program.statements, &[]);
    let root_name = match &opts.root {
        Some(r) => Some(r.clone()),
        None => em.root_candidates.first().cloned(),
    };
    if let Some(r) = &root_name {
        match em.lookup(&r.split("::").map(String::from).collect::<Vec<_>>()) {
            Some((full, Decl::Struct(_))) => {
                em.root_key = Some(full.clone());
                em.emit_decl(&full);
            }
            _ => em.note(format!("root type `{}` is not a struct definition", r)),
        }
    }
    for name in em.order.clone() {
        if em.inline_enums && matches!(em.decls.get(&name), Some(Decl::Enum(_))) {
            continue;
        }
        em.emit_decl(&name);
    }
    em.assign_names();
    em.check_lossy();
    Ok(em.render(opts))
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
                Stmt::Struct(d) if d.template_params.is_empty() => self.add(qualify(&d.name), Decl::Struct(d.clone())),
                Stmt::Union(d) => self.add(qualify(&d.name), Decl::Union),
                Stmt::Bitfield(d) if d.template_params.is_empty() => self.add(qualify(&d.name), Decl::Bitfield(d.clone())),
                Stmt::Enum(d) => self.add(qualify(&d.name), Decl::Enum(d.clone())),
                Stmt::Using { name, ty, template_params, .. } if template_params.is_empty() => {
                    self.add(qualify(name), Decl::Alias(ty.clone()))
                }
                Stmt::Var(v) if v.placement.is_some() && ns.is_empty() => {
                    if let TypeBase::Custom { name, .. } = &v.ty.base {
                        self.root_candidates.push(name.join("::"));
                    }
                }
                _ => {}
            }
        }
    }

    fn add(&mut self, name: String, decl: Decl) {
        if !self.decls.contains_key(&name) {
            self.order.push(name.clone());
        }
        self.decls.insert(name, decl);
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

    fn type_def(&self, key: &str) -> Option<&TypeDef> {
        self.index.get(key).map(|&i| &self.types[i])
    }

    /// Follows references to alias types (enums) down to their base type.
    fn resolved(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Ref(k) => match self.type_def(k) {
                Some(TypeDef { body: Body::Alias(inner), .. }) => self.resolved(inner),
                _ => ty.clone(),
            },
            other => other.clone(),
        }
    }

    /// Emits a declaration once; false if it could not be emitted.
    fn emit_decl(&mut self, name: &str) -> bool {
        if self.index.contains_key(name) {
            return true;
        }
        if self.failed.contains(name) || self.in_progress.contains(name) {
            return false;
        }
        let Some(decl) = self.decls.get(name).cloned() else { return false };
        self.in_progress.insert(name.to_string());
        let result = match &decl {
            Decl::Struct(d) => self.struct_body(name, d),
            Decl::Bitfield(d) => self.bitfield_body(name, d).map(|b| (b, Vec::new())),
            Decl::Enum(d) => self.enum_body(name, d),
            Decl::Alias(t) => self.ty_of(t).map(|ty| (Body::Alias(ty), Vec::new())),
            Decl::Union => Err(Error::new("unions have no ACN equivalent")),
        };
        self.in_progress.remove(name);
        match result {
            Ok((body, comment)) => {
                self.index.insert(name.to_string(), self.types.len());
                self.types.push(TypeDef { key: name.to_string(), asn: String::new(), body, comment });
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

    fn int_ty(&self, bits: u32, signed: bool, endian: Option<Endian>) -> Result<Ty> {
        let little = endian.unwrap_or(self.endian) == Endian::Little && bits > 8;
        if little && !matches!(bits, 16 | 32 | 64) {
            return Err(Error::new(format!("{}-bit little-endian integers have no ACN encoding (only 16, 32 and 64)", bits)));
        }
        let (min, max) = if signed {
            (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
        } else {
            (0, (1i128 << bits) - 1)
        };
        Ok(Ty::Int { min, max, bits, signed, little, as_unsigned: false })
    }

    fn builtin_ty(&mut self, b: BuiltinType, endian: Option<Endian>) -> Result<Ty> {
        use BuiltinType::*;
        Ok(match b {
            U8 | Char | Bool => self.int_ty(8, false, endian)?,
            U16 | Char16 => self.int_ty(16, false, endian)?,
            U24 => self.int_ty(24, false, endian)?,
            U32 => self.int_ty(32, false, endian)?,
            U64 => self.int_ty(64, false, endian)?,
            S8 => self.int_ty(8, true, endian)?,
            S16 | S24 | S32 | S64 => {
                let bits = match b {
                    S16 => 16,
                    S24 => 24,
                    S32 => 32,
                    _ => 64,
                };
                if self.twos_complement {
                    self.int_ty(bits, true, endian)?
                } else {
                    self.note(format!(
                        "`{}` fields are read as unsigned {}-bit integers (ASN1SCC's Scala runtime proves two's complement only for 8 bits; see --twos-complement)",
                        b, bits
                    ));
                    match self.int_ty(bits, false, endian)? {
                        Ty::Int { min, max, bits, little, .. } => Ty::Int { min, max, bits, signed: false, little, as_unsigned: true },
                        other => other,
                    }
                }
            }
            Float | Double => {
                Ty::Real { double: b == Double, little: endian.unwrap_or(self.endian) == Endian::Little }
            }
            U48 | U96 | U128 | S48 | S96 | S128 => {
                return Err(Error::new(format!("`{}` is wider than ASN1SCC's 64-bit integers", b)))
            }
            Str | Auto => return Err(Error::new(format!("`{}` cannot be a wire field", b))),
        })
    }

    /// The type of a hexpat type reference (emitting named types first).
    fn ty_of(&mut self, t: &TypeRef) -> Result<Ty> {
        match &t.base {
            TypeBase::Builtin(b) => self.builtin_ty(*b, t.endian),
            TypeBase::Custom { name, args } => {
                let joined = name.join("::");
                if !args.is_empty() {
                    return Err(Error::new(format!("templated type `{}` is not supported", joined)));
                }
                let Some((full, decl)) = self.lookup(name) else {
                    return Err(Error::new(format!("unknown type `{}`", joined)));
                };
                if let (true, Decl::Enum(d)) = (self.inline_enums, &decl) {
                    let mut base = d.ty.clone();
                    if t.endian.is_some() {
                        base.endian = t.endian;
                    }
                    return self.ty_of(&base);
                }
                if let Decl::Alias(inner) = &decl {
                    // Aliases of builtins are inlined (with the use site's endianness).
                    if let TypeBase::Builtin(_) = inner.base {
                        let mut inner = inner.clone();
                        if t.endian.is_some() {
                            inner.endian = t.endian;
                        }
                        return self.ty_of(&inner);
                    }
                }
                if t.endian.is_some() && t.endian != Some(self.endian) {
                    return Err(Error::new(format!("`{}` is used with a byte order other than the file's", joined)));
                }
                if !self.emit_decl(&full) {
                    return Err(Error::new(format!("type `{}` could not be emitted", full)));
                }
                // Aliases of named types resolve to their target.
                if let Some(TypeDef { body: Body::Alias(Ty::Ref(target)), .. }) = self.type_def(&full) {
                    return Ok(Ty::Ref(target.clone()));
                }
                Ok(Ty::Ref(full))
            }
        }
    }

    fn enum_body(&mut self, name: &str, d: &EnumDef) -> Result<(Body, Vec<String>)> {
        let ty = self.ty_of(&d.ty)?;
        if !matches!(ty, Ty::Int { .. }) {
            return Err(Error::new("enum underlying type must be an integer"));
        }
        let mut entries = Vec::new();
        let mut last: i128 = -1;
        for e in &d.entries {
            let v = match &e.value {
                Some(x) => const_int(x).ok_or_else(|| Error::new(format!("enum entry `{}` has a non-constant value", e.name)))?,
                None => last + 1,
            };
            last = match &e.end {
                Some(x) => const_int(x).unwrap_or(v),
                None => v,
            };
            match &e.end {
                Some(_) => entries.push(format!("{} = {} ... {}", e.name, v, last)),
                None => entries.push(format!("{} = {}", e.name, v)),
            }
        }
        let _ = name;
        // hexpat enums accept any value, so the ASN.1 type is the full
        // integer range; the names are kept as a comment.
        let comment = vec![format!("enum: {}", entries.join(", "))];
        Ok((Body::Alias(ty), comment))
    }

    fn enum_value(&self, path: &[String]) -> Option<i128> {
        if path.len() < 2 {
            return None;
        }
        let (ename, entry) = path.split_at(path.len() - 1);
        let (_, decl) = self.lookup(ename)?;
        let Decl::Enum(d) = decl else { return None };
        let mut last: i128 = -1;
        for e in &d.entries {
            let v = match &e.value {
                Some(x) => const_int(x)?,
                None => last + 1,
            };
            if e.name == entry[0] {
                return Some(v);
            }
            last = e.end.as_ref().and_then(const_int).unwrap_or(v);
        }
        None
    }

    fn bitfield_body(&mut self, name: &str, d: &StructDef) -> Result<Body> {
        let mut msb_first = false;
        let mut declared_total: Option<u64> = None;
        for a in &d.attrs {
            if a.is("bitfield_order") && a.args.len() == 2 {
                if let Expr::Scoped(p) = &a.args[0] {
                    msb_first = p.last().map(|s| s == "MostToLeastSignificant").unwrap_or(false);
                }
                declared_total = const_int(&a.args[1]).map(|v| v as u64);
            }
        }
        let mut fields: Vec<Comp> = Vec::new();
        let mut pad = 0;
        for s in &d.body {
            match s {
                Stmt::BitfieldField(f) => {
                    let bits = const_int(&f.size).ok_or_else(|| Error::new("bitfield field size must be constant"))? as u32;
                    if bits == 0 || bits > 64 {
                        return Err(Error::new(format!("bitfield field `{}` has {} bits", f.name, bits)));
                    }
                    let (min, max) = if f.signed { (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1) } else { (0, (1i128 << bits) - 1) };
                    let ty = Ty::Int { min, max, bits, signed: f.signed, little: false, as_unsigned: false };
                    if f.name == "$padding$" || f.name == "padding" {
                        pad += 1;
                        let id = format!("padding{}", pad);
                        fields.push(Comp::new(&id, &id, Kind::Padding, ty));
                    } else {
                        fields.push(Comp::new(&f.name, &f.name, Kind::Field, ty));
                    }
                }
                _ => return Err(Error::new("bitfield contains statements other than bit fields")),
            }
        }
        let used: u64 = fields.iter().map(|c| if let Ty::Int { bits, .. } = c.ty { bits as u64 } else { 0 }).sum();
        let total = declared_total.unwrap_or(used.div_ceil(8) * 8);
        if used > total {
            return Err(Error::new("bitfield fields exceed the declared width"));
        }
        if total > 8 && self.endian == Endian::Little {
            return Err(Error::new(format!(
                "`{}` is a {}-bit bitfield in a little-endian file; ACN reads bits in stream order, so fields that straddle little-endian bytes have no encoding",
                name, total
            )));
        }
        let mut layout = Vec::new();
        let tail = if used < total {
            let bits = (total - used) as u32;
            Some(Comp::new("padding0", "padding0", Kind::Padding, Ty::Int { min: 0, max: (1i128 << bits) - 1, bits, signed: false, little: false, as_unsigned: false }))
        } else {
            None
        };
        // ACN reads bits most significant first; hexpat's default order is
        // least significant first, so the fields are reversed then.
        if msb_first {
            layout.extend(fields);
            layout.extend(tail);
        } else {
            layout.extend(tail);
            layout.extend(fields.into_iter().rev());
        }
        Ok(Body::Seq(layout))
    }

    // ------------------------------------------------------------------
    // Structs
    // ------------------------------------------------------------------

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

    fn struct_body(&mut self, name: &str, d: &StructDef) -> Result<(Body, Vec<String>)> {
        let mut body = Vec::new();
        self.flatten_parents(d, &mut body, 0)?;
        let mut sb = StructBuilder {
            owner: name.to_string(),
            comps: Vec::new(),
            is_root: self.root_key.as_deref() == Some(name),
            used_in_cond: HashSet::new(),
        };
        let n = body.len();
        for (i, s) in body.iter().enumerate() {
            self.stmt(&mut sb, s, None, i + 1 == n)?;
        }
        let mut comment = Vec::new();
        if sb.is_root {
            if let Some((field, _)) = &self.trailing {
                comment.push(format!("the trailing `{}` array (until the end of the input) is decoded by the caller", field));
            }
        }
        Ok((Body::Seq(sb.comps), comment))
    }

    fn stmt(&mut self, sb: &mut StructBuilder, s: &Stmt, cond: Option<&Cond>, is_last: bool) -> Result<()> {
        let owner = sb.owner.clone();
        match s {
            Stmt::Var(v) => {
                if v.placement.is_some() {
                    return Err(Error::new(format!("member `{}` is placed with `@`", v.name)));
                }
                if v.value.is_some() || v.in_var || v.out_var {
                    self.note(format!("`{}`: local variable `{}` skipped", owner, v.name));
                    return Ok(());
                }
                if v.attrs.iter().any(|a| a.is("no_unique_address")) {
                    self.note(format!("`{}`: `{}` has no_unique_address and is skipped", owner, v.name));
                    return Ok(());
                }
                if let TypeBase::Custom { name: tn, args } = &v.ty.base {
                    let joined = tn.join("::");
                    if joined == "type::Magic" || joined == "Magic" {
                        let Some(TemplateArg::Expr(Expr::Literal(Literal::String(m)))) = args.first() else {
                            return Err(Error::new("type::Magic needs a string argument"));
                        };
                        if cond.is_some() {
                            return Err(Error::new("type::Magic inside a condition"));
                        }
                        let fname = if v.name.is_empty() { "magic".to_string() } else { v.name.clone() };
                        let mut c = Comp::new(&sb.fresh_id(&fname), &fname, Kind::Magic, Ty::Null);
                        c.magic = m.as_bytes().to_vec();
                        if c.magic.len() > 8 {
                            self.note(format!("`{}`: the {}-byte magic `{}` is kept as bytes and not checked", owner, c.magic.len(), fname));
                        }
                        sb.comps.push(c);
                        return Ok(());
                    }
                }
                let ty = self.ty_of(&v.ty)?;
                let fname = if v.name.is_empty() { "anonymous".to_string() } else { v.name.clone() };
                let mut c = Comp::new(&sb.fresh_id(&fname), &fname, Kind::Field, ty);
                c.present = cond.cloned();
                sb.comps.push(c);
            }
            Stmt::Array(a) => {
                if a.placement.is_some() {
                    return Err(Error::new(format!("member `{}` is placed with `@`", a.name)));
                }
                if a.init.is_some() {
                    self.note(format!("`{}`: local array `{}` skipped", owner, a.name));
                    return Ok(());
                }
                let is_bytes = matches!(a.ty.base, TypeBase::Builtin(BuiltinType::U8) | TypeBase::Builtin(BuiltinType::Char));
                let size = match &a.size {
                    ArraySize::Fixed(e) => self.size_of(sb, e)?,
                    ArraySize::Unsized if is_bytes => Size::NullTerminated { max: self.max_len },
                    ArraySize::Unsized => return Err(Error::new(format!("unsized array `{}` of a non-byte type", a.name))),
                    ArraySize::While(c) if is_last && sb.is_root && cond.is_none() && is_eof_condition(c) => {
                        let ty = self.ty_of(&a.ty)?;
                        let Ty::Ref(key) = ty else {
                            return Err(Error::new(format!("trailing array `{}` must have a named element type", a.name)));
                        };
                        self.trailing = Some((a.name.clone(), key));
                        return Ok(());
                    }
                    ArraySize::While(_) => {
                        return Err(Error::new(format!(
                            "while-sized array `{}` (only a trailing `while(!std::mem::eof())` in the root struct translates)",
                            a.name
                        )))
                    }
                };
                let ty = if is_bytes { Ty::Octets(size) } else { Ty::SeqOf(size, Box::new(self.ty_of(&a.ty)?)) };
                let mut c = Comp::new(&sb.fresh_id(&a.name), &a.name, Kind::Field, ty);
                c.present = cond.cloned();
                sb.comps.push(c);
            }
            Stmt::Padding { size: ArraySize::Fixed(e), .. } => {
                let n = const_int(e).ok_or_else(|| Error::new("padding size must be constant"))?;
                if n < 0 {
                    return Err(Error::new("negative padding"));
                }
                let id = sb.fresh_id("padding");
                let mut c = Comp::new(&id, &id, Kind::Padding, Ty::Octets(Size::Fixed(n as u64)));
                c.present = cond.cloned();
                sb.comps.push(c);
            }
            Stmt::Padding { .. } => return Err(Error::new("while-sized padding")),
            Stmt::If { cond: c, then, otherwise, .. } => {
                let tc = self.cond_of(sb, c)?;
                let then_c = and(cond, tc.clone());
                let else_c = and(cond, Cond::Not(Box::new(tc)));
                for s in then {
                    self.stmt(sb, s, Some(&then_c), false)?;
                }
                for s in otherwise {
                    self.stmt(sb, s, Some(&else_c), false)?;
                }
            }
            Stmt::Match { values, cases, .. } => {
                if values.len() != 1 {
                    return Err(Error::new("match on several values"));
                }
                let subject = self.cond_of(sb, &values[0])?;
                let mut previous: Option<Cond> = None;
                for case in cases {
                    let mut case_c: Option<Cond> = None;
                    let mut wildcard = false;
                    for p in &case.patterns {
                        match p {
                            CasePattern::Wildcard => wildcard = true,
                            CasePattern::Alternatives(alts) => {
                                for (lo, hi) in alts {
                                    let lo_c = self.cond_of(sb, lo)?;
                                    let alt = match hi {
                                        None => Cond::Bin("==", Box::new(subject.clone()), Box::new(lo_c)),
                                        Some(h) => {
                                            let hi_c = self.cond_of(sb, h)?;
                                            Cond::Bin(
                                                "and",
                                                Box::new(Cond::Bin(">=", Box::new(subject.clone()), Box::new(lo_c))),
                                                Box::new(Cond::Bin("<=", Box::new(subject.clone()), Box::new(hi_c))),
                                            )
                                        }
                                    };
                                    case_c = Some(match case_c {
                                        None => alt,
                                        Some(prev) => Cond::Bin("or", Box::new(prev), Box::new(alt)),
                                    });
                                }
                            }
                        }
                    }
                    // A case applies when its pattern matches and no earlier case did.
                    let own = if wildcard { None } else { case_c.clone() };
                    let mut full = match (&previous, own) {
                        (None, Some(o)) => Some(o),
                        (Some(p), Some(o)) => Some(Cond::Bin("and", Box::new(Cond::Not(Box::new(p.clone()))), Box::new(o))),
                        (Some(p), None) => Some(Cond::Not(Box::new(p.clone()))),
                        (None, None) => None,
                    };
                    if let Some(cc) = cond {
                        full = Some(match full {
                            None => cc.clone(),
                            Some(f) => Cond::Bin("and", Box::new(cc.clone()), Box::new(f)),
                        });
                    }
                    for s in &case.body {
                        self.stmt(sb, s, full.as_ref(), false)?;
                    }
                    if !wildcard {
                        if let Some(cc) = case_c {
                            previous = Some(match previous {
                                None => cc,
                                Some(p) => Cond::Bin("or", Box::new(p), Box::new(cc)),
                            });
                        }
                    }
                }
            }
            Stmt::Expr { expr, .. } => {
                let what = match expr {
                    Expr::Call { name: n, .. } => n.join("::"),
                    _ => "expression".to_string(),
                };
                self.note(format!("`{}`: statement `{}` has no format meaning and is skipped", owner, what));
            }
            Stmt::Pointer(p) => return Err(Error::new(format!("pointer member `{}`", p.name))),
            Stmt::Assign { .. } => return Err(Error::new("assignment inside a struct body")),
            Stmt::BitfieldField(f) => return Err(Error::new(format!("bit field `{}` inside a struct", f.name))),
            Stmt::TryCatch { .. } | Stmt::While { .. } | Stmt::For { .. } | Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {
                return Err(Error::new("control flow inside a struct body"))
            }
            Stmt::Import { .. }
            | Stmt::Using { .. }
            | Stmt::UsingForward { .. }
            | Stmt::Namespace { .. }
            | Stmt::Struct(_)
            | Stmt::Union(_)
            | Stmt::Enum(_)
            | Stmt::Bitfield(_)
            | Stmt::Function(_) => return Err(Error::new("definition inside a struct body")),
        }
        Ok(())
    }

    /// Resolves `a` or `a.b.c` to a component id of `sb`, inlining the
    /// components of `a` (and `a.b`) into `sb` as needed.
    fn resolve_path(&mut self, sb: &mut StructBuilder, path: &[String]) -> Result<String> {
        let mut id = path[0].clone();
        let exists = |sb: &StructBuilder, id: &str| {
            let prefix = format!("{}.", id);
            sb.comps.iter().any(|c| c.id == id || c.id.starts_with(&prefix))
        };
        if !exists(sb, &id) {
            return Err(Error::new(format!("`{}` is not an earlier member of `{}`", id, sb.owner)));
        }
        for member in &path[1..] {
            // Already inlined by an earlier reference?
            if sb.comps.iter().any(|c| c.id == id) {
                self.inline(sb, &id)?;
            }
            id = format!("{}.{}", id, member);
            if !exists(sb, &id) {
                return Err(Error::new(format!("`{}` has no member `{}`", id.rsplit_once('.').unwrap().0, member)));
            }
        }
        if !sb.comps.iter().any(|c| c.id == id) {
            return Err(Error::new(format!("`{}` is a struct; conditions and sizes need a number", id)));
        }
        Ok(id)
    }

    /// Replaces the component `id` (of a SEQUENCE type) by that type's
    /// components, so that they can be referenced by ACN determinants.
    fn inline(&mut self, sb: &mut StructBuilder, id: &str) -> Result<()> {
        let pos = sb.comps.iter().position(|c| c.id == id).unwrap();
        let comp = sb.comps[pos].clone();
        let Ty::Ref(key) = &comp.ty else {
            return Err(Error::new(format!("`{}` is not a struct, so its members cannot be referenced", id)));
        };
        let Some(TypeDef { body: Body::Seq(children), .. }) = self.type_def(key).cloned() else {
            return Err(Error::new(format!("`{}` is not a struct, so its members cannot be referenced", id)));
        };
        let rename = |r: &str| format!("{}.{}", id, r);
        let mut inlined = Vec::new();
        for mut child in children {
            child.id = rename(&child.id);
            child.path = format!("{}.{}", comp.path, child.path);
            child.present = match (comp.present.clone(), child.present.map(|p| p.renamed(&rename))) {
                (None, x) => x,
                (Some(p), None) => Some(p),
                (Some(p), Some(x)) => Some(Cond::Bin("and", Box::new(p), Box::new(x))),
            };
            child.ty = child.ty.renamed(&rename);
            inlined.push(child);
        }
        sb.comps.splice(pos..=pos, inlined);
        Ok(())
    }

    /// The size of an array: a constant or an earlier member (which becomes
    /// an ACN-inserted field).
    fn size_of(&mut self, sb: &mut StructBuilder, e: &Expr) -> Result<Size> {
        if let Some(n) = const_int(e) {
            if n < 0 {
                return Err(Error::new("negative array size"));
            }
            return Ok(Size::Fixed(n as u64));
        }
        let path = match member_path(e) {
            Some(p) => p,
            None => {
                return Err(Error::new(
                    "array size is not a constant or a member (ACN size determinants cannot be expressions)",
                ))
            }
        };
        let id = self.resolve_path(sb, &path)?;
        let resolved = self.resolved(&sb.comps.iter().find(|c| c.id == id).unwrap().ty);
        let c = sb.comps.iter_mut().find(|c| c.id == id).unwrap();
        c.ty = resolved;
        let Ty::Int { max, min, .. } = c.ty else {
            return Err(Error::new(format!("size `{}` is not an integer", id)));
        };
        if c.present.is_some() {
            return Err(Error::new(format!("size `{}` is itself conditional", id)));
        }
        if sb.used_in_cond.contains(&id) {
            return Err(Error::new(format!("`{}` is used both as a size and in a condition (ACN allows only ASN.1 fields in conditions and only ACN fields as sizes)", id)));
        }
        if min < 0 {
            return Err(Error::new(format!("size `{}` is signed", id)));
        }
        c.kind = Kind::Inserted;
        let cap = (max as u128).min(self.max_len as u128) as u64;
        if (max as u128) > self.max_len as u128 {
            self.note(format!("`{}`: sizes taken from `{}` are capped at {} (ASN1SCC needs an upper bound)", sb.owner, id.replace('.', "-"), self.max_len));
        }
        Ok(Size::Det { field: id, max: cap })
    }

    /// Translates a condition into an ACN `present-when` expression.
    fn cond_of(&mut self, sb: &mut StructBuilder, e: &Expr) -> Result<Cond> {
        Ok(match e {
            Expr::Literal(Literal::Unsigned(v)) => Cond::Int(*v as i128),
            Expr::Literal(Literal::Signed(v)) => Cond::Int(*v),
            Expr::Literal(Literal::Char(c)) => Cond::Int(*c as i128),
            Expr::Literal(Literal::Bool(b)) => {
                Cond::Bin("==", Box::new(Cond::Int(1)), Box::new(Cond::Int(if *b { 1 } else { 0 })))
            }
            Expr::Scoped(path) => Cond::Int(
                self.enum_value(path).ok_or_else(|| Error::new(format!("`{}` is not an enum constant", path.join("::"))))?,
            ),
            Expr::Ident(_) | Expr::Member { .. } => {
                let path = member_path(e).ok_or_else(|| Error::new("unsupported member access in a condition"))?;
                let id = self.resolve_path(sb, &path)?;
                let c = sb.comps.iter().find(|c| c.id == id).unwrap();
                if c.present.is_some() {
                    return Err(Error::new(format!("condition refers to `{}`, which is itself conditional", path.join("."))));
                }
                match c.kind {
                    Kind::Field => {}
                    Kind::Inserted => {
                        return Err(Error::new(format!("`{}` is used both as a size and in a condition (ACN allows only ASN.1 fields in conditions and only ACN fields as sizes)", path.join("."))));
                    }
                    _ => return Err(Error::new(format!("condition refers to `{}`, which has no value", path.join(".")))),
                }
                if !matches!(self.resolved(&c.ty), Ty::Int { .. } | Ty::Octets(Size::Fixed(_)) | Ty::Real { .. }) {
                    return Err(Error::new(format!("condition refers to `{}`, which is not a number", path.join("."))));
                }
                sb.used_in_cond.insert(id.clone());
                Cond::Field(id)
            }
            Expr::Cast { value, .. } => self.cond_of(sb, value)?,
            Expr::Unary { op: UnaryOp::Not, operand } => Cond::Not(Box::new(self.cond_of(sb, operand)?)),
            Expr::Unary { op: UnaryOp::Neg, operand } => match self.cond_of(sb, operand)? {
                Cond::Int(v) => Cond::Int(-v),
                other => Cond::Bin("-", Box::new(Cond::Int(0)), Box::new(other)),
            },
            Expr::Binary { op, left, right } => {
                // `field == "str"` on a char array: compare it as an integer.
                if matches!(op, BinaryOp::Eq | BinaryOp::Ne) {
                    for (a, b) in [(left, right), (right, left)] {
                        if let Expr::Literal(Literal::String(s)) = b.as_ref() {
                            let f = self.cond_of(sb, a)?;
                            let Cond::Field(id) = &f else { return Err(Error::new("string comparison with a non-field")) };
                            let v = self.chars_as_int(sb, id, s.as_bytes())?;
                            let sym = if *op == BinaryOp::Eq { "==" } else { "!=" };
                            return Ok(Cond::Bin(sym, Box::new(f), Box::new(Cond::Int(v))));
                        }
                    }
                }
                let sym = match op {
                    BinaryOp::Eq => "==",
                    BinaryOp::Ne => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::Le => "<=",
                    BinaryOp::Gt => ">",
                    BinaryOp::Ge => ">=",
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Rem => "%",
                    _ => return Err(Error::new(format!("operator {} has no ACN equivalent", op.symbol()))),
                };
                let l = self.cond_of(sb, left)?;
                let r = self.cond_of(sb, right)?;
                Cond::Bin(sym, Box::new(l), Box::new(r))
            }
            _ => return Err(Error::new("condition is not translatable to an ACN expression")),
        })
    }

    /// Turns the `char[n]` component `id` into an `n`-byte big-endian
    /// integer and returns the value of `s` in that encoding.
    fn chars_as_int(&mut self, sb: &mut StructBuilder, id: &str, s: &[u8]) -> Result<i128> {
        let c = sb.comps.iter_mut().find(|c| c.id == id).unwrap();
        let n = match &c.ty {
            Ty::Octets(Size::Fixed(n)) => *n,
            Ty::Int { bits, .. } if c.chars_int => (*bits / 8) as u64,
            _ => return Err(Error::new(format!("`{}` is compared with a string but is not a fixed-size char array", id))),
        };
        if n == 0 || n > 8 {
            return Err(Error::new(format!("`{}` is compared with a string but is {} bytes long (at most 8 can be read as an integer)", id, n)));
        }
        if s.len() as u64 != n {
            return Err(Error::new(format!("`{}` has {} bytes but is compared with the {}-byte string {:?}", id, n, s.len(), String::from_utf8_lossy(s))));
        }
        let bits = (n * 8) as u32;
        c.ty = Ty::Int { min: 0, max: if bits == 64 { u64::MAX as i128 } else { (1i128 << bits) - 1 }, bits, signed: false, little: false, as_unsigned: false };
        c.chars_int = true;
        Ok(s.iter().fold(0i128, |acc, b| (acc << 8) | *b as i128))
    }

    // ------------------------------------------------------------------
    // Names
    // ------------------------------------------------------------------

    fn assign_names(&mut self) {
        // Components first: ASN1SCC rejects a component whose name equals a
        // type name ignoring case, so types are renamed around them.
        let mut component_names: HashSet<String> = HashSet::new();
        for t in &mut self.types {
            if let Body::Seq(comps) = &mut t.body {
                let mut used: HashSet<String> = HashSet::new();
                for c in comps.iter_mut() {
                    let base = field_ident(&c.id);
                    let mut name = base.clone();
                    let mut i = 2;
                    while used.contains(&name.to_lowercase()) {
                        name = format!("{}{}", base, i);
                        i += 1;
                    }
                    used.insert(name.to_lowercase());
                    component_names.insert(name.to_lowercase());
                    c.asn = name;
                }
            }
        }
        let mut used: HashSet<String> = HashSet::new();
        for t in &mut self.types {
            let base = type_ident(&t.key);
            let mut name = base.clone();
            let mut i = 2;
            loop {
                let lc = name.to_lowercase();
                let clash = used.contains(&lc)
                    || component_names.contains(&lc)
                    || RESERVED_TYPES.iter().any(|r| r.to_lowercase() == lc);
                if !clash {
                    break;
                }
                name = if i == 2 { format!("{}-T", base) } else { format!("{}-T{}", base, i - 1) };
                i += 1;
            }
            used.insert(name.to_lowercase());
            t.asn = name;
        }
    }

    fn asn_type_name(&self, key: &str) -> String {
        self.type_def(key).map(|t| t.asn.clone()).unwrap_or_else(|| type_ident(key))
    }

    /// Records inserted fields whose value is lost when a dependent is absent.
    fn check_lossy(&mut self) {
        let mut lossy = Vec::new();
        for t in &self.types {
            let Body::Seq(comps) = &t.body else { continue };
            for c in comps.iter().filter(|c| c.kind == Kind::Inserted) {
                let deps: Vec<&Comp> = comps.iter().filter(|d| d.ty.depends_on(&c.id)).collect();
                if !deps.is_empty() && deps.iter().all(|d| d.present.is_some()) {
                    let names: Vec<String> = deps.iter().map(|d| d.asn.clone()).collect();
                    lossy.push(format!(
                        "{}.{}: derived from {} on encoding and written as 0 when {} absent",
                        t.asn,
                        c.asn,
                        names.join(", "),
                        if names.len() == 1 { "it is" } else { "they are" }
                    ));
                }
            }
        }
        self.lossy = lossy;
    }

    // ------------------------------------------------------------------
    // Rendering
    // ------------------------------------------------------------------

    fn render(&self, opts: &Asn1Options) -> Asn1Output {
        // ASN1SCC also rejects a component or type named like the module.
        let mut module = module_ident(&opts.module);
        let taken = |m: &str| {
            let m = m.to_lowercase();
            self.types.iter().any(|t| {
                t.asn.to_lowercase() == m
                    || matches!(&t.body, Body::Seq(c) if c.iter().any(|c| c.asn.to_lowercase() == m))
            })
        };
        if taken(&module) {
            module = format!("{}-Module", module);
        }
        let mut asn = String::new();
        let mut acn = String::new();
        let _ = writeln!(asn, "-- Generated from an ImHex pattern by hexpat's ASN.1/ACN emitter.");
        let _ = writeln!(asn, "{} DEFINITIONS AUTOMATIC TAGS ::= BEGIN\n", module);
        let _ = writeln!(acn, "-- Generated from an ImHex pattern by hexpat's ASN.1/ACN emitter.");
        let _ = writeln!(acn, "{} DEFINITIONS ::= BEGIN\n", module);
        let mut maps = Vec::new();
        for t in &self.types {
            for c in &t.comment {
                let _ = writeln!(asn, "-- {}", c);
            }
            match &t.body {
                Body::Alias(ty) => {
                    let _ = writeln!(asn, "{} ::= {}\n", t.asn, self.asn_ty(ty));
                    let _ = writeln!(acn, "{} {}\n", t.asn, self.acn_spec(ty, &[], &|s| s.to_string()));
                    maps.push(TypeMap { name: t.asn.clone(), hexpat: t.key.clone(), alias: Some(self.shape(ty, false)), components: Vec::new() });
                }
                Body::Seq(comps) => {
                    let name_of = |id: &str| comps.iter().find(|c| c.id == id).map(|c| c.asn.clone()).unwrap_or_else(|| field_ident(id));
                    let asn_comps: Vec<&Comp> = comps.iter().filter(|c| c.kind != Kind::Inserted).collect();
                    if asn_comps.is_empty() {
                        let _ = writeln!(asn, "{} ::= SEQUENCE {{ }}\n", t.asn);
                    } else {
                        let _ = writeln!(asn, "{} ::= SEQUENCE {{", t.asn);
                        for (i, c) in asn_comps.iter().enumerate() {
                            let sep = if i + 1 < asn_comps.len() { "," } else { "" };
                            let opt = if c.present.is_some() { " OPTIONAL" } else { "" };
                            let note = if c.chars_int {
                                format!(" -- char[{}] read as a big-endian integer", if let Ty::Int { bits, .. } = c.ty { bits / 8 } else { 0 })
                            } else {
                                String::new()
                            };
                            let ty = match c.kind {
                                Kind::Magic => magic_asn(&c.magic),
                                _ => self.asn_ty(&c.ty),
                            };
                            let note = match c.kind {
                                Kind::Magic => format!(" -- magic {:?}", String::from_utf8_lossy(&c.magic)),
                                _ => note,
                            };
                            let _ = writeln!(asn, "    {} {}{}{}{}", c.asn, ty, opt, sep, note);
                        }
                        let _ = writeln!(asn, "}}\n");
                    }
                    if comps.is_empty() {
                        let _ = writeln!(acn, "{} [] {{ }}\n", t.asn);
                    } else {
                        let _ = writeln!(acn, "{} [] {{", t.asn);
                        for (i, c) in comps.iter().enumerate() {
                            let sep = if i + 1 < comps.len() { "," } else { "" };
                            let mut extra = Vec::new();
                            if let Some(p) = &c.present {
                                extra.push(format!("present-when ({})", render_cond(p, &name_of)));
                            }
                            let line = match c.kind {
                                Kind::Inserted => format!("{} INTEGER {}", c.asn, self.acn_spec(&c.ty, &extra, &name_of)),
                                Kind::Magic if c.magic.len() <= 8 => format!("{} [encoding pos-int, size {}]", c.asn, c.magic.len() * 8),
                                Kind::Magic => format!("{} []", c.asn),
                                Kind::Field | Kind::Padding => format!("{} {}", c.asn, self.acn_spec(&c.ty, &extra, &name_of)),
                            };
                            let _ = writeln!(acn, "    {}{}", line, sep);
                        }
                        let _ = writeln!(acn, "}}\n");
                    }
                    let components = comps
                        .iter()
                        .map(|c| ComponentMap {
                            name: c.asn.clone(),
                            path: c.path.clone(),
                            role: match c.kind {
                                Kind::Field => Role::Field,
                                Kind::Inserted => Role::Inserted,
                                Kind::Magic => Role::Magic,
                                Kind::Padding => Role::Padding,
                            },
                            shape: match c.kind {
                                Kind::Magic if c.magic.len() <= 8 => Shape::CharsInt(c.magic.len() as u32),
                                Kind::Magic => Shape::Bytes,
                                _ => self.shape(&c.ty, c.chars_int),
                            },
                            optional: c.present.is_some(),
                        })
                        .collect();
                    maps.push(TypeMap { name: t.asn.clone(), hexpat: t.key.clone(), alias: None, components });
                }
            }
        }
        let _ = writeln!(asn, "END");
        let _ = writeln!(acn, "END");
        let root = self
            .root_key
            .as_ref()
            .filter(|k| match self.type_def(k) {
                Some(TypeDef { body: Body::Seq(c), .. }) => !c.is_empty() || self.trailing.is_none(),
                Some(_) => true,
                None => false,
            })
            .map(|k| self.asn_type_name(k));
        Asn1Output {
            module,
            asn,
            acn,
            root,
            trailing: self.trailing.as_ref().map(|(f, k)| Trailing { field: f.clone(), element: self.asn_type_name(k) }),
            types: maps,
            notes: self.notes.clone(),
            lossy: self.lossy.clone(),
            emitted: self.types.iter().map(|t| t.key.clone()).collect(),
            max_len: self.max_len,
        }
    }

    fn asn_ty(&self, ty: &Ty) -> String {
        let size = |s: &Size| match s {
            Size::Fixed(n) => format!("SIZE({})", n),
            Size::Det { max, .. } | Size::NullTerminated { max } => format!("SIZE(0..{})", max),
        };
        match ty {
            Ty::Int { min, max, .. } => format!("INTEGER ({}..{})", min, max),
            Ty::Real { .. } => "REAL".into(),
            Ty::Octets(s) => format!("OCTET STRING ({})", size(s)),
            Ty::SeqOf(s, elem) => format!("SEQUENCE ({}) OF {}", size(s), self.asn_ty(elem)),
            Ty::Ref(k) => self.asn_type_name(k),
            Ty::Null => "NULL".into(),
        }
    }

    /// The ACN encoding spec of a type (`[props]`, plus the element spec of
    /// a SEQUENCE OF of builtins).
    fn acn_spec(&self, ty: &Ty, extra: &[String], name_of: &dyn Fn(&str) -> String) -> String {
        let mut props: Vec<String> = Vec::new();
        let mut children = String::new();
        let size_props = |s: &Size, props: &mut Vec<String>| match s {
            Size::Fixed(_) => {}
            Size::Det { field, .. } => props.push(format!("size {}", name_of(field))),
            Size::NullTerminated { .. } => {
                props.push("size null-terminated".into());
                props.push("termination-pattern '00'H".into());
            }
        };
        match ty {
            Ty::Int { bits, signed, little, .. } => {
                props.push(format!("encoding {}", if *signed { "twos-complement" } else { "pos-int" }));
                props.push(format!("size {}", bits));
                if *little {
                    props.push("endianness little".into());
                }
            }
            Ty::Real { double, little } => {
                props.push(format!("encoding IEEE754-1985-{}", if *double { 64 } else { 32 }));
                if *little {
                    props.push("endianness little".into());
                }
            }
            Ty::Octets(s) => size_props(s, &mut props),
            Ty::SeqOf(s, elem) => {
                size_props(s, &mut props);
                if !matches!(elem.as_ref(), Ty::Ref(_)) {
                    children = format!(" {{ {} }}", self.acn_spec(elem, &[], name_of));
                }
            }
            Ty::Ref(_) | Ty::Null => {}
        }
        // `present-when` first, as in the ACN manual's examples.
        let mut all: Vec<String> = extra.to_vec();
        all.extend(props);
        format!("[{}]{}", all.join(", "), children)
    }

    fn shape(&self, ty: &Ty, chars_int: bool) -> Shape {
        match ty {
            Ty::Int { bits, .. } if chars_int => Shape::CharsInt(bits / 8),
            Ty::Int { bits, as_unsigned: true, .. } => Shape::Unsigned(*bits),
            Ty::Int { .. } => Shape::Int,
            Ty::Real { .. } => Shape::Real,
            Ty::Octets(_) => Shape::Bytes,
            Ty::SeqOf(_, elem) => Shape::SeqOf(Box::new(self.shape(elem, false))),
            Ty::Ref(k) => Shape::Ref(self.asn_type_name(k)),
            Ty::Null => Shape::Null,
        }
    }
}

struct StructBuilder {
    owner: String,
    comps: Vec<Comp>,
    is_root: bool,
    /// Component ids used in `present-when` expressions.
    used_in_cond: HashSet<String>,
}

impl StructBuilder {
    fn fresh_id(&self, name: &str) -> String {
        if !self.comps.iter().any(|c| c.id == name) {
            return name.to_string();
        }
        let mut i = 2;
        loop {
            let id = format!("{}#{}", name, i);
            if !self.comps.iter().any(|c| c.id == id) {
                return id;
            }
            i += 1;
        }
    }
}

impl Cond {
    fn renamed(self, f: &dyn Fn(&str) -> String) -> Cond {
        match self {
            Cond::Field(id) => Cond::Field(f(&id)),
            Cond::Int(v) => Cond::Int(v),
            Cond::Bin(op, l, r) => Cond::Bin(op, Box::new(l.renamed(f)), Box::new(r.renamed(f))),
            Cond::Not(x) => Cond::Not(Box::new(x.renamed(f))),
        }
    }
}

impl Ty {
    fn renamed(self, f: &dyn Fn(&str) -> String) -> Ty {
        let size = |s: Size| match s {
            Size::Det { field, max } => Size::Det { field: f(&field), max },
            other => other,
        };
        match self {
            Ty::Octets(s) => Ty::Octets(size(s)),
            Ty::SeqOf(s, e) => Ty::SeqOf(size(s), Box::new(e.renamed(f))),
            other => other,
        }
    }

    fn depends_on(&self, id: &str) -> bool {
        match self {
            Ty::Octets(Size::Det { field, .. }) => field == id,
            Ty::SeqOf(s, e) => matches!(s, Size::Det { field, .. } if field == id) || e.depends_on(id),
            _ => false,
        }
    }
}

fn and(outer: Option<&Cond>, inner: Cond) -> Cond {
    match outer {
        None => inner,
        Some(o) => Cond::Bin("and", Box::new(o.clone()), Box::new(inner)),
    }
}

fn render_cond(c: &Cond, name_of: &dyn Fn(&str) -> String) -> String {
    match c {
        Cond::Field(id) => name_of(id),
        Cond::Int(v) => v.to_string(),
        Cond::Not(x) => format!("!({})", render_cond(x, name_of)),
        Cond::Bin(op, l, r) => {
            let wrap = |x: &Cond| match x {
                Cond::Bin(..) => format!("({})", render_cond(x, name_of)),
                _ => render_cond(x, name_of),
            };
            format!("{} {} {}", wrap(l), op, wrap(r))
        }
    }
}

/// A magic of at most 8 bytes is a single-value INTEGER read big-endian, so
/// that the decoder checks it (ACN `NULL [pattern ..]` fields have no
/// invertibility support in the Scala backend, and its OCTET STRING value
/// constraints generate C syntax); a longer one is kept unchecked.
fn magic_asn(bytes: &[u8]) -> String {
    if bytes.len() <= 8 {
        format!("INTEGER ({})", bytes.iter().fold(0u128, |acc, b| (acc << 8) | *b as u128))
    } else {
        format!("OCTET STRING (SIZE({}))", bytes.len())
    }
}

fn member_path(e: &Expr) -> Option<Vec<String>> {
    match e {
        Expr::Ident(n) => Some(vec![n.clone()]),
        Expr::Member { object, member } => {
            let mut p = member_path(object)?;
            p.push(member.clone());
            Some(p)
        }
        Expr::Cast { value, .. } => member_path(value),
        _ => None,
    }
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

/// Splits a hexpat name into ASN.1-legal words (letters and digits).
fn words(name: &str) -> Vec<String> {
    name.split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty()).map(String::from).collect()
}

/// An ASN.1 type reference: upper-case first letter, words joined by `-`.
pub fn type_ident(name: &str) -> String {
    let mut s = words(name).join("-");
    if s.is_empty() {
        s = "Unnamed".into();
    }
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        s = format!("T{}", s);
    }
    let mut chars: Vec<char> = s.chars().collect();
    chars[0] = chars[0].to_ascii_uppercase();
    chars.into_iter().collect()
}

/// An ASN.1 identifier for a component id (`a`, `a.b`, `a#2`).
pub fn field_ident(id: &str) -> String {
    let (base, suffix) = match id.split_once('#') {
        Some((b, n)) => (b, n),
        None => (id, ""),
    };
    let mut s = words(base).join("-");
    if s.is_empty() {
        s = "field".into();
    }
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        s = format!("f{}", s);
    }
    let mut chars: Vec<char> = s.chars().collect();
    // `IHDR` -> `ihdr`, `Width` -> `width`.
    let all_caps = chars.iter().take_while(|c| **c != '-').all(|c| !c.is_ascii_lowercase());
    if all_caps {
        for c in chars.iter_mut().take_while(|c| **c != '-') {
            *c = c.to_ascii_lowercase();
        }
    } else {
        chars[0] = chars[0].to_ascii_lowercase();
    }
    let mut s: String = chars.into_iter().collect();
    if KEYWORDS.contains(&s.to_lowercase().as_str()) {
        s.push_str("-field");
    }
    if !suffix.is_empty() {
        s = format!("{}{}", s, suffix);
    }
    s
}

/// An ASN.1 module reference.
pub fn module_ident(name: &str) -> String {
    let w: Vec<String> = words(name)
        .into_iter()
        .map(|w| {
            let mut c: Vec<char> = w.chars().collect();
            c[0] = c[0].to_ascii_uppercase();
            c.into_iter().collect()
        })
        .collect();
    let s = w.join("-");
    if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) {
        format!("M{}", s)
    } else {
        s
    }
}

// ----------------------------------------------------------------------
// Side outputs
// ----------------------------------------------------------------------

impl Asn1Output {
    /// The type map as JSON, for tools outside this crate.
    pub fn map_json(&self) -> String {
        fn esc(s: &str) -> String {
            let mut o = String::from("\"");
            for c in s.chars() {
                match c {
                    '"' => o.push_str("\\\""),
                    '\\' => o.push_str("\\\\"),
                    c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                    c => o.push(c),
                }
            }
            o.push('"');
            o
        }
        fn shape(s: &Shape) -> String {
            match s {
                Shape::Int => "\"int\"".into(),
                Shape::Unsigned(n) => format!("{{\"unsigned\": {}}}", n),
                Shape::Real => "\"real\"".into(),
                Shape::Bytes => "\"bytes\"".into(),
                Shape::Null => "\"null\"".into(),
                Shape::CharsInt(n) => format!("{{\"chars_int\": {}}}", n),
                Shape::Ref(t) => format!("{{\"ref\": {}}}", esc(t)),
                Shape::SeqOf(e) => format!("{{\"seq_of\": {}}}", shape(e)),
            }
        }
        let mut o = String::from("{\n");
        let _ = writeln!(o, "  \"module\": {},", esc(&self.module));
        let _ = writeln!(o, "  \"root\": {},", self.root.as_deref().map(esc).unwrap_or_else(|| "null".into()));
        match &self.trailing {
            Some(t) => {
                let _ = writeln!(o, "  \"trailing\": {{\"field\": {}, \"element\": {}}},", esc(&t.field), esc(&t.element));
            }
            None => {
                let _ = writeln!(o, "  \"trailing\": null,");
            }
        }
        let _ = writeln!(o, "  \"max_len\": {},", self.max_len);
        let list = |v: &[String]| v.iter().map(|s| esc(s)).collect::<Vec<_>>().join(", ");
        let _ = writeln!(o, "  \"notes\": [{}],", list(&self.notes));
        let _ = writeln!(o, "  \"lossy\": [{}],", list(&self.lossy));
        o.push_str("  \"types\": [\n");
        for (i, t) in self.types.iter().enumerate() {
            let _ = write!(o, "    {{\"name\": {}, \"hexpat\": {}", esc(&t.name), esc(&t.hexpat));
            if let Some(a) = &t.alias {
                let _ = write!(o, ", \"alias\": {}", shape(a));
            } else {
                o.push_str(", \"components\": [");
                for (j, c) in t.components.iter().enumerate() {
                    let role = match c.role {
                        Role::Field => "field",
                        Role::Inserted => "inserted",
                        Role::Magic => "magic",
                        Role::Padding => "padding",
                    };
                    let _ = write!(
                        o,
                        "{}\n      {{\"name\": {}, \"path\": {}, \"role\": \"{}\", \"shape\": {}, \"optional\": {}}}",
                        if j == 0 { "" } else { "," },
                        esc(&c.name),
                        esc(&c.path),
                        role,
                        shape(&c.shape),
                        c.optional
                    );
                }
                o.push_str("\n    ]");
            }
            o.push('}');
            o.push_str(if i + 1 < self.types.len() { ",\n" } else { "\n" });
        }
        o.push_str("  ]\n}\n");
        o
    }

    /// A Scala program for the ASN1SCC output: it decodes a file with the
    /// generated ACN decoders (the root, then the trailing elements until
    /// the input ends), prints the decoded values as JSON, re-encodes each
    /// value, and reports whether the bytes and values round-trip.
    pub fn scala_harness(&self) -> String {
        let scala = |t: &str| t.replace('-', "_");
        let mut o = String::new();
        o.push_str(SCALA_HARNESS_PRELUDE);
        let root = match &self.root {
            Some(r) => format!("Some(Unit({}))", scala_unit(&scala(r))),
            None => "None".into(),
        };
        let trailing = match &self.trailing {
            Some(t) => format!("Some(({:?}, Unit({})))", t.field, scala_unit(&scala(&t.element))),
            None => "None".into(),
        };
        let _ = writeln!(o, "  val root: Option[Unit] = {}", root);
        let _ = writeln!(o, "  val trailing: Option[(String, Unit)] = {}", trailing);
        o.push_str(SCALA_HARNESS_MAIN);
        o
    }
}

fn scala_unit(t: &str) -> String {
    format!(
        "{t:?}, {t}_REQUIRED_BYTES_FOR_ACN_ENCODING, (c: ACN) => {t}_ACN_Decode(c) match {{ case RightMut(v) => Right(v); case LeftMut(e) => Left(e) }}, (v: Any, c: ACN) => {t}_ACN_Encode(v.asInstanceOf[{t}], c, true)"
    )
}

const SCALA_HARNESS_PRELUDE: &str = r#"// Generated by hexpat emit-asn1: decodes a file with the ASN1SCC decoders,
// prints the values as JSON and checks that re-encoding reproduces them.
package asn1scala

import asn1src._

object HexpatHarness:
  /** A PDU: its name, worst-case encoded size, decoder and encoder. */
  case class Unit(name: String, maxBytes: Int, decode: ACN => scala.Either[Int, Any], encode: (Any, ACN) => stainless.lang.Either[Int, Int])

"#;

const SCALA_HARNESS_MAIN: &str = r#"
  def codecFor(bytes: Array[Byte], from: Int, size: Int): ACN =
    val codec = initACNCodec(size)
    val n = math.min(size, bytes.length - from)
    if n > 0 then System.arraycopy(bytes, from, codec.base.bitStream.buf, 0, n)
    codec

  def json(x: Any, sb: StringBuilder): scala.Unit = x match
    case null => sb.append("null")
    case b: Boolean => sb.append(b)
    case l: Long => sb.append(l)
    case i: Int => sb.append(i)
    case b: Byte => sb.append(b & 0xFF)
    case d: Double => if d.isNaN || d.isInfinite then sb.append('"').append(d).append('"') else sb.append(d)
    case SomeMut(v) => json(v, sb)
    case NoneMut() => sb.append("null")
    case v: asn1scala.Vector[?] => seq(v.toScala, sb)
    case p: Product =>
      val names = p.productElementNames.toList
      if names == List("nCount", "arr") then
        val n = p.productElement(0) match
          case l: Long => l.toInt
          case i: Int => i
          case _ => 0
        seq(p.productElement(1).asInstanceOf[asn1scala.Vector[?]].toScala.take(n), sb)
      else if names == List("arr") then
        seq(p.productElement(0).asInstanceOf[asn1scala.Vector[?]].toScala, sb)
      else
        sb.append('{')
        for ((name, i) <- names.zipWithIndex) do
          if i > 0 then sb.append(", ")
          sb.append('"').append(name).append("\": ")
          json(p.productElement(i), sb)
        sb.append('}')
    case other => sb.append('"').append(other.toString.replace("\\", "\\\\").replace("\"", "\\\"")).append('"')

  def seq(xs: Seq[Any], sb: StringBuilder): scala.Unit =
    sb.append('[')
    for ((x, i) <- xs.zipWithIndex) do
      if i > 0 then sb.append(", ")
      json(x, sb)
    sb.append(']')

  /** Decodes one PDU at `from`; appends its report and returns the next offset. */
  def one(u: Unit, bytes: Array[Byte], from: Int, sb: StringBuilder): Option[Int] =
    val size = math.max(u.maxBytes, 1)
    val codec = codecFor(bytes, from, size)
    sb.append("{\"type\": \"").append(u.name).append("\", \"offset\": ").append(from)
    u.decode(codec) match
      case scala.Left(err) =>
        sb.append(", \"error\": ").append(err).append('}')
        None
      case scala.Right(value) =>
        val bits = codec.base.bitStream.bitIndex
        val end = from + ((bits + 7) / 8).toInt
        sb.append(", \"bits\": ").append(bits)
        if end > bytes.length then
          sb.append(", \"error\": \"read past the end of the input\"}")
          return None
        sb.append(", \"value\": ")
        json(value, sb)
        val enc = initACNCodec(size)
        u.encode(value, enc) match
          case stainless.lang.Left(err) =>
            sb.append(", \"encodeError\": ").append(err)
          case stainless.lang.Right(_) =>
            val encBits = enc.base.bitStream.bitIndex
            val out = enc.base.bitStream.buf.take(((encBits + 7) / 8).toInt)
            val sameBytes = encBits == bits && out.sameElements(bytes.slice(from, end))
            val again = u.decode(codecFor(out, 0, size))
            val sameValue = again match
              case scala.Right(v2) => v2 == value
              case scala.Left(_) => false
            sb.append(", \"bytesRoundTrip\": ").append(sameBytes)
            sb.append(", \"valueRoundTrip\": ").append(sameValue)
        sb.append('}')
        Some(end)

  def main(args: Array[String]): scala.Unit =
    val bytes = java.nio.file.Files.readAllBytes(java.nio.file.Paths.get(args(0)))
    val sb = StringBuilder()
    sb.append("{\"length\": ").append(bytes.length)
    var offset = 0
    var ok = true
    root.foreach { u =>
      sb.append(", \"root\": ")
      one(u, bytes, 0, sb) match
        case Some(end) => offset = end
        case None => ok = false
    }
    trailing.foreach { (field, u) =>
      sb.append(", \"trailingField\": \"").append(field).append("\", \"trailing\": [")
      var first = true
      while ok && offset < bytes.length do
        if !first then sb.append(", ")
        first = false
        one(u, bytes, offset, sb) match
          case Some(end) if end > offset => offset = end
          case _ => ok = false
      sb.append(']')
    }
    sb.append(", \"consumed\": ").append(offset).append(", \"ok\": ").append(ok).append('}')
    println(sb.toString)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn emit(src: &str, endian: Endian) -> Asn1Output {
        let program = crate::parse_program(src).unwrap();
        emit_asn1(&program, endian, &Asn1Options { module: "test".into(), root: None, max_len: 1000, twos_complement: true, inline_enums: false }).unwrap()
    }

    #[test]
    fn identifiers() {
        assert_eq!(type_ident("BitmapFileHeader"), "BitmapFileHeader");
        assert_eq!(type_ident("chunk_t"), "Chunk-t");
        assert_eq!(type_ident("ns::Name"), "Ns-Name");
        assert_eq!(field_ident("IHDR"), "ihdr");
        assert_eq!(field_ident("Width"), "width");
        assert_eq!(field_ident("type"), "type-field");
        assert_eq!(field_ident("header#2"), "header2");
        assert_eq!(field_ident("ident.class"), "ident-class");
        assert_eq!(field_ident("$padding$"), "padding");
        assert_eq!(module_ident("gzip"), "Gzip");
    }

    #[test]
    fn sizes_become_inserted_fields() {
        let out = emit(
            "struct Msg { u16 len; u8 payload[len]; s32 x; char tag[4]; }; Msg m @ 0;",
            Endian::Little,
        );
        assert_eq!(out.root.as_deref(), Some("Msg"));
        assert!(out.asn.contains("Msg ::= SEQUENCE {\n    payload OCTET STRING (SIZE(0..1000)),\n    x INTEGER (-2147483648..2147483647),\n    tag OCTET STRING (SIZE(4))\n}"), "{}", out.asn);
        assert!(out.acn.contains("    len INTEGER [encoding pos-int, size 16, endianness little],\n    payload [size len],\n    x [encoding twos-complement, size 32, endianness little],\n    tag []\n"), "{}", out.acn);
        assert!(out.notes.iter().any(|n| n.contains("capped at 1000")), "{:?}", out.notes);
    }

    #[test]
    fn conditions_become_optional_components() {
        let out = emit(
            r#"enum Kind : u8 { A = 1, B = 2 };
               struct Body { u8 x; };
               struct Msg {
                   Kind kind;
                   char type[4];
                   if (kind == Kind::A) { Body body; } else if (kind == Kind::B) { u32 word; }
                   if (type == "IHDR") { u8 ihdr; }
               };
               Msg m @ 0;"#,
            Endian::Big,
        );
        assert!(out.asn.contains("Kind-T ::= INTEGER (0..255)"), "{}", out.asn);
        assert!(out.asn.contains("    body-field Body OPTIONAL,\n    word INTEGER (0..4294967295) OPTIONAL,"), "{}", out.asn);
        assert!(out.asn.contains("type-field INTEGER (0..4294967295), -- char[4] read as a big-endian integer"), "{}", out.asn);
        assert!(out.acn.contains("body-field [present-when (kind == 1)]"), "{}", out.acn);
        assert!(out.acn.contains("word [present-when (!(kind == 1) and (kind == 2)), encoding pos-int, size 32]"), "{}", out.acn);
        assert!(out.acn.contains("ihdr [present-when (type-field == 1229472850), encoding pos-int, size 8]"), "{}", out.acn);
    }

    #[test]
    fn nested_references_inline_the_child() {
        let out = emit(
            "struct Header { u16 reserved; u16 count; }; struct Entry { u8 w; }; struct Ico { Header header; Entry entries[header.count]; }; Ico ico @ 0;",
            Endian::Little,
        );
        assert!(out.asn.contains("Ico ::= SEQUENCE {\n    header-reserved INTEGER (0..65535),\n    entries SEQUENCE (SIZE(0..1000)) OF Entry\n}"), "{}", out.asn);
        assert!(out.acn.contains("header-count INTEGER [encoding pos-int, size 16, endianness little],\n    entries [size header-count]"), "{}", out.acn);
        let ico = out.types.iter().find(|t| t.name == "Ico").unwrap();
        assert_eq!(ico.components[1].path, "header.count");
        assert_eq!(ico.components[1].role, Role::Inserted);
    }

    #[test]
    fn magic_bitfields_padding_and_trailing_arrays() {
        let out = emit(
            r#"import std.mem;
               bitfield F { a : 3; b : 1; padding : 4; };
               struct Item { u8 v; };
               struct File { type::Magic<"AB"> magic; F flags; padding[2]; Item items[while(!std::mem::eof())]; };
               File f @ 0;"#,
            Endian::Little,
        );
        assert!(out.asn.contains("    magic INTEGER (16706), -- magic \"AB\""), "{}", out.asn);
        assert!(out.acn.contains("    magic [encoding pos-int, size 16],"), "{}", out.acn);
        assert!(out.asn.contains("F ::= SEQUENCE {\n    padding1 INTEGER (0..15),\n    b INTEGER (0..1),\n    a INTEGER (0..7)\n}"), "{}", out.asn);
        assert!(out.asn.contains("    padding OCTET STRING (SIZE(2))"), "{}", out.asn);
        assert_eq!(out.trailing, Some(Trailing { field: "items".into(), element: "Item".into() }));
        assert_eq!(out.root.as_deref(), Some("File"));
    }

    #[test]
    fn lossy_lengths_are_reported() {
        let out = emit(
            r#"struct Chunk { u32 length; char type[4]; if (type == "IHDR") { u8 w; } else { u8 data[length]; } };
               Chunk c @ 0;"#,
            Endian::Big,
        );
        assert_eq!(out.lossy.len(), 1, "{:?}", out.lossy);
        assert!(out.lossy[0].starts_with("Chunk.length: derived from data"), "{:?}", out.lossy);
    }

    #[test]
    fn type_names_avoid_component_names() {
        let out = emit("struct Header { u8 a; }; struct Gif { Header header; }; Gif g @ 0;", Endian::Little);
        assert!(out.asn.contains("Header-T ::= SEQUENCE"), "{}", out.asn);
        assert!(out.asn.contains("header Header-T"), "{}", out.asn);
    }

    #[test]
    fn enums_can_be_inlined() {
        let program = crate::parse_program("enum E : u16 { A = 1 }; struct S { E e; }; S s @ 0;").unwrap();
        let opts = Asn1Options { module: "t".into(), inline_enums: true, ..Default::default() };
        let out = emit_asn1(&program, Endian::Big, &opts).unwrap();
        assert!(out.asn.contains("S ::= SEQUENCE {\n    e INTEGER (0..65535)\n}"), "{}", out.asn);
        assert!(!out.asn.contains("E ::="), "{}", out.asn);
    }

    #[test]
    fn untranslatable_constructs_are_notes() {
        let out = emit("struct S { u8 n; u8 data[n * 2]; }; S s @ 0;", Endian::Little);
        assert!(out.root.is_none());
        assert!(out.notes.iter().any(|n| n.contains("cannot be expressions")), "{:?}", out.notes);
    }
}
