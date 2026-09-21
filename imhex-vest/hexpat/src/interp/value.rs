//! Runtime values and the pattern tree.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::ast::{BuiltinType, Endian};
use crate::error::{Error, Result};

/// Shared, mutable handle to a pattern node.
pub type PatternRef = Rc<RefCell<Pattern>>;

/// A runtime value.
#[derive(Debug, Clone)]
pub enum Value {
    Unsigned(u128),
    Signed(i128),
    Float(f64),
    Bool(bool),
    Char(u8),
    Char16(u16),
    /// Byte strings are kept as `String` (see [`crate::lower::bytes_to_string`]).
    Str(String),
    /// A struct, union, array, bitfield or pointer pattern (or a scalar
    /// pattern passed by reference).
    Pattern(PatternRef),
    /// A parameter pack.
    Pack(Vec<Value>),
    /// A local array of non-placeable values (for example `str names[3]`).
    List(Rc<RefCell<Vec<Value>>>),
    Null,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unsigned(_) => "unsigned",
            Value::Signed(_) => "signed",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Char(_) => "char",
            Value::Char16(_) => "char16",
            Value::Str(_) => "str",
            Value::Pattern(_) => "pattern",
            Value::Pack(_) => "pack",
            Value::List(_) => "list",
            Value::Null => "null",
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, Value::Signed(_))
    }

    pub fn as_u128(&self) -> Result<u128> {
        Ok(match self {
            Value::Unsigned(v) => *v,
            Value::Signed(v) => *v as u128,
            Value::Float(f) => *f as u128,
            Value::Bool(b) => *b as u128,
            Value::Char(c) => *c as u128,
            Value::Char16(c) => *c as u128,
            Value::Null => 0,
            Value::Str(s) => return Err(Error::new(format!("cannot use string \"{}\" as an integer", s))),
            Value::Pattern(p) => {
                return Err(Error::new(format!("cannot use pattern \"{}\" as an integer", p.borrow().name)))
            }
            Value::Pack(_) => return Err(Error::new("cannot use a parameter pack as an integer")),
            Value::List(_) => return Err(Error::new("cannot use an array as an integer")),
        })
    }

    pub fn as_i128(&self) -> Result<i128> {
        Ok(match self {
            Value::Unsigned(v) => *v as i128,
            Value::Signed(v) => *v,
            Value::Float(f) => *f as i128,
            _ => self.as_u128()? as i128,
        })
    }

    pub fn as_u64(&self) -> Result<u64> {
        let v = self.as_i128()?;
        if v < 0 {
            return Err(Error::new(format!("expected a non-negative value, got {}", v)));
        }
        u64::try_from(v).map_err(|_| Error::new(format!("value {} does not fit in 64 bits", v)))
    }

    pub fn as_f64(&self) -> Result<f64> {
        Ok(match self {
            Value::Float(f) => *f,
            Value::Signed(v) => *v as f64,
            Value::Unsigned(v) => *v as f64,
            _ => self.as_u128()? as f64,
        })
    }

    pub fn as_bool(&self) -> Result<bool> {
        Ok(match self {
            Value::Bool(b) => *b,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Pattern(_) => true,
            _ => self.as_u128()? != 0,
        })
    }

    pub fn as_str(&self) -> Result<String> {
        Ok(match self {
            Value::Str(s) => s.clone(),
            Value::Char(c) => (*c as char).to_string(),
            Value::Char16(c) => char::from_u32(*c as u32).unwrap_or('?').to_string(),
            other => other.to_display_string(),
        })
    }

    pub fn as_pattern(&self) -> Result<PatternRef> {
        match self {
            Value::Pattern(p) => Ok(p.clone()),
            other => Err(Error::new(format!("expected a pattern, got {}", other.type_name()))),
        }
    }

    /// Conversion used by `{}` in format strings and by `str(x)`.
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Unsigned(v) => v.to_string(),
            Value::Signed(v) => v.to_string(),
            Value::Float(f) => format_float(*f),
            Value::Bool(b) => b.to_string(),
            Value::Char(c) => (*c as char).to_string(),
            Value::Char16(c) => char::from_u32(*c as u32).unwrap_or('?').to_string(),
            Value::Str(s) => s.clone(),
            Value::Pattern(p) => p.borrow().name.clone(),
            Value::Pack(items) => items.iter().map(|v| v.to_display_string()).collect::<Vec<_>>().join(", "),
            Value::List(items) => format!("[ {} ]", items.borrow().iter().map(|v| v.to_display_string()).collect::<Vec<_>>().join(", ")),
            Value::Null => "null".to_string(),
        }
    }
}

/// Formats a float the way `fmt::format("{}")` does (no trailing zeros, no
/// exponent for normal magnitudes).
pub fn format_float(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if f == f.trunc() && f.abs() < 1e16 {
        return format!("{}", f as i128);
    }
    let s = format!("{}", f);
    s
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_display_string())
    }
}

/// The kind of a pattern node.
#[derive(Debug, Clone)]
pub enum PatternKind {
    Unsigned,
    Signed,
    Float,
    Bool,
    Char,
    Char16,
    /// An enum; entries are `(name, first, last)` on the underlying integer.
    Enum { signed: bool, entries: Rc<Vec<(String, i128, i128)>> },
    /// `char[...]`
    String,
    /// `char16[...]`
    WideString,
    Struct { members: Vec<PatternRef> },
    Union { members: Vec<PatternRef> },
    /// An array whose elements are individual patterns.
    Array { entries: Vec<PatternRef> },
    /// A homogeneous array of scalars: element `i` lives at `offset + i * template.size`.
    StaticArray { template: PatternRef, count: u64 },
    Bitfield { fields: Vec<PatternRef>, bit_size: u64 },
    /// A field inside a bitfield: `bit_offset` counts from bit 0 of the
    /// bitfield's first byte in the bitfield's integer representation.
    BitfieldField {
        base_offset: u64,
        bit_offset: u64,
        bit_size: u64,
        signed: bool,
        /// Interpretation of the extracted bits.
        typed: Option<Box<PatternKind>>,
    },
    /// A pointer: the pattern's own bytes hold the address, `pointee` is
    /// the pattern it points to.
    Pointer { pointee: PatternRef, address: u128 },
    Padding,
}

impl PatternKind {
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            PatternKind::Unsigned
                | PatternKind::Signed
                | PatternKind::Float
                | PatternKind::Bool
                | PatternKind::Char
                | PatternKind::Char16
                | PatternKind::Enum { .. }
        )
    }
}

/// A node in the pattern tree.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub display_name: Option<String>,
    pub type_name: String,
    pub offset: u64,
    pub size: u64,
    pub section: usize,
    pub endian: Endian,
    pub kind: PatternKind,
    pub comment: Option<String>,
    pub hidden: bool,
    pub sealed: bool,
    pub inline: bool,
    pub color: Option<u32>,
    /// `[[format("fn")]]` / `[[format_read("fn")]]`
    pub format_fn: Option<String>,
    /// `[[transform("fn")]]`
    pub transform_fn: Option<String>,
    /// Attributes as `(name, evaluated arguments)`.
    pub attributes: Vec<(String, Vec<Value>)>,
    pub array_index: Option<u64>,
    /// A value assigned by pattern code (`x.field = 3`) that shadows the bytes.
    pub value_override: Option<Value>,
    /// For scalar patterns of a builtin type.
    pub builtin: Option<BuiltinType>,
}

impl Pattern {
    pub fn new(name: &str, type_name: &str, offset: u64, size: u64, section: usize, endian: Endian, kind: PatternKind) -> Self {
        Pattern {
            name: name.to_string(),
            display_name: None,
            type_name: type_name.to_string(),
            offset,
            size,
            section,
            endian,
            kind,
            comment: None,
            hidden: false,
            sealed: false,
            inline: false,
            color: None,
            format_fn: None,
            transform_fn: None,
            attributes: Vec::new(),
            array_index: None,
            value_override: None,
            builtin: None,
        }
    }

    pub fn shared(self) -> PatternRef {
        Rc::new(RefCell::new(self))
    }

    pub fn end(&self) -> u64 {
        self.offset + self.size
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|(n, _)| n == name)
    }

    /// Looks up a direct member by name (structs, unions, bitfields).
    pub fn member(&self, name: &str) -> Option<PatternRef> {
        let members = match &self.kind {
            PatternKind::Struct { members } | PatternKind::Union { members } => members,
            PatternKind::Bitfield { fields, .. } => fields,
            _ => return None,
        };
        for m in members.iter().rev() {
            let mb = m.borrow();
            if mb.name == name {
                return Some(m.clone());
            }
            // Members of inlined nested patterns are visible in the parent.
            if mb.inline {
                if let Some(inner) = mb.member(name) {
                    return Some(inner);
                }
            }
        }
        None
    }

    pub fn members(&self) -> &[PatternRef] {
        match &self.kind {
            PatternKind::Struct { members } | PatternKind::Union { members } => members,
            PatternKind::Bitfield { fields, .. } => fields,
            PatternKind::Array { entries } => entries,
            _ => &[],
        }
    }

    pub fn push_member(&mut self, member: PatternRef) {
        match &mut self.kind {
            PatternKind::Struct { members } | PatternKind::Union { members } => members.push(member),
            PatternKind::Bitfield { fields, .. } => fields.push(member),
            PatternKind::Array { entries } => entries.push(member),
            _ => {}
        }
    }

    /// Number of array entries (materialised or static).
    pub fn entry_count(&self) -> Option<u64> {
        match &self.kind {
            PatternKind::Array { entries } => Some(entries.len() as u64),
            PatternKind::StaticArray { count, .. } => Some(*count),
            PatternKind::String => Some(self.size),
            PatternKind::WideString => Some(self.size / 2),
            _ => None,
        }
    }
}
