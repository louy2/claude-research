//! Abstract syntax tree for the ImHex pattern language.
//!
//! Produced by [`crate::lower`] from the tree-sitter concrete syntax tree.
//! Every node carries the 1-based line of its first token so diagnostics
//! can point back into the (preprocessed) source.

use std::fmt;

/// A whole pattern file.
#[derive(Debug, Clone, Default)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// Byte order of a multi-byte value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Endian {
    Little,
    Big,
}

/// The built-in value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinType {
    U8,
    U16,
    U24,
    U32,
    U48,
    U64,
    U96,
    U128,
    S8,
    S16,
    S24,
    S32,
    S48,
    S64,
    S96,
    S128,
    Float,
    Double,
    Bool,
    Char,
    Char16,
    Str,
    Auto,
}

impl BuiltinType {
    pub fn from_name(name: &str) -> Option<Self> {
        use BuiltinType::*;
        Some(match name {
            "u8" => U8,
            "u16" => U16,
            "u24" => U24,
            "u32" => U32,
            "u48" => U48,
            "u64" => U64,
            "u96" => U96,
            "u128" => U128,
            "s8" => S8,
            "s16" => S16,
            "s24" => S24,
            "s32" => S32,
            "s48" => S48,
            "s64" => S64,
            "s96" => S96,
            "s128" => S128,
            "float" => Float,
            "double" => Double,
            "bool" => Bool,
            "char" => Char,
            "char16" => Char16,
            "str" => Str,
            "auto" => Auto,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        use BuiltinType::*;
        match self {
            U8 => "u8",
            U16 => "u16",
            U24 => "u24",
            U32 => "u32",
            U48 => "u48",
            U64 => "u64",
            U96 => "u96",
            U128 => "u128",
            S8 => "s8",
            S16 => "s16",
            S24 => "s24",
            S32 => "s32",
            S48 => "s48",
            S64 => "s64",
            S96 => "s96",
            S128 => "s128",
            Float => "float",
            Double => "double",
            Bool => "bool",
            Char => "char",
            Char16 => "char16",
            Str => "str",
            Auto => "auto",
        }
    }

    /// Size on the wire in bytes, or `None` for `str` and `auto`.
    pub fn size(self) -> Option<u64> {
        use BuiltinType::*;
        Some(match self {
            U8 | S8 | Bool | Char => 1,
            U16 | S16 | Char16 => 2,
            U24 | S24 => 3,
            U32 | S32 | Float => 4,
            U48 | S48 => 6,
            U64 | S64 | Double => 8,
            U96 | S96 => 12,
            U128 | S128 => 16,
            Str | Auto => return None,
        })
    }

    pub fn is_unsigned(self) -> bool {
        use BuiltinType::*;
        matches!(self, U8 | U16 | U24 | U32 | U48 | U64 | U96 | U128)
    }

    pub fn is_signed(self) -> bool {
        use BuiltinType::*;
        matches!(self, S8 | S16 | S24 | S32 | S48 | S64 | S96 | S128)
    }

    pub fn is_float(self) -> bool {
        matches!(self, BuiltinType::Float | BuiltinType::Double)
    }
}

impl fmt::Display for BuiltinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A reference to a type: `[ref] [le|be] base`.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeRef {
    pub reference: bool,
    pub endian: Option<Endian>,
    pub base: TypeBase,
}

impl TypeRef {
    pub fn builtin(b: BuiltinType) -> Self {
        TypeRef { reference: false, endian: None, base: TypeBase::Builtin(b) }
    }

    pub fn custom(name: &str) -> Self {
        TypeRef {
            reference: false,
            endian: None,
            base: TypeBase::Custom { name: vec![name.to_string()], args: Vec::new() },
        }
    }
}

impl fmt::Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.reference {
            f.write_str("ref ")?;
        }
        match self.endian {
            Some(Endian::Little) => f.write_str("le ")?,
            Some(Endian::Big) => f.write_str("be ")?,
            None => {}
        }
        match &self.base {
            TypeBase::Builtin(b) => write!(f, "{}", b),
            TypeBase::Custom { name, args } => {
                write!(f, "{}", name.join("::"))?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            f.write_str(", ")?;
                        }
                        match a {
                            TemplateArg::Type(t) => write!(f, "{}", t)?,
                            TemplateArg::Expr(_) => f.write_str("<expr>")?,
                        }
                    }
                    f.write_str(">")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeBase {
    Builtin(BuiltinType),
    /// A user-defined (possibly namespaced, possibly templated) type.
    Custom { name: Vec<String>, args: Vec<TemplateArg> },
}

/// A template argument. A bare identifier is parsed as a type; the evaluator
/// falls back to a value if no such type exists.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateArg {
    Type(TypeRef),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateParam {
    pub name: String,
    /// `true` for `T`, `false` for `auto N`.
    pub is_type: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Vec<String>,
    pub args: Vec<Expr>,
}

impl Attribute {
    pub fn is(&self, name: &str) -> bool {
        self.name.join("::") == name
    }
}

/// `@ address [in section]`
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub address: Expr,
    pub section: Option<Expr>,
}

/// The size part of an array declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum ArraySize {
    /// `T x[]` — read until a zero element.
    Unsized,
    /// `T x[n]`
    Fixed(Expr),
    /// `T x[while(cond)]`
    While(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub line: u32,
    pub constant: bool,
    pub ty: TypeRef,
    /// Empty for anonymous members such as `u8;` or `std::mem::AlignTo<4>;`.
    pub name: String,
    pub placement: Option<Placement>,
    pub value: Option<Expr>,
    pub in_var: bool,
    pub out_var: bool,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayDecl {
    pub line: u32,
    pub constant: bool,
    pub ty: TypeRef,
    pub name: String,
    pub size: ArraySize,
    pub placement: Option<Placement>,
    pub init: Option<Vec<Expr>>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerDecl {
    pub line: u32,
    pub ty: TypeRef,
    pub name: String,
    /// `Some` for pointer-to-array declarations `T *p[n] : u32`.
    pub array: Option<ArraySize>,
    pub pointer_ty: TypeRef,
    pub placement: Option<Placement>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub line: u32,
    pub name: String,
    pub template_params: Vec<TemplateParam>,
    pub parents: Vec<TypeRef>,
    pub body: Vec<Stmt>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumEntry {
    pub name: String,
    pub value: Option<Expr>,
    /// `Name = a ... b`
    pub end: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub line: u32,
    pub name: String,
    pub ty: TypeRef,
    pub entries: Vec<EnumEntry>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub ty: TypeRef,
    pub name: String,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub line: u32,
    pub name: String,
    pub params: Vec<Param>,
    /// `auto ... name`
    pub pack: Option<String>,
    pub body: Vec<Stmt>,
}

/// One arm of a `match`.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    /// One pattern per matched value.
    pub patterns: Vec<CasePattern>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CasePattern {
    Wildcard,
    /// Alternatives; each is a single value or an inclusive range.
    Alternatives(Vec<(Expr, Option<Expr>)>),
}

/// A bit-sized entry inside a `bitfield`.
#[derive(Debug, Clone, PartialEq)]
pub struct BitfieldField {
    pub line: u32,
    /// `"$padding$"` for `padding : n`.
    pub name: String,
    pub size: Expr,
    pub signed: bool,
    /// Set for `Type name : n` (for example `bool f : 1` or an enum).
    pub ty: Option<TypeRef>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Neg,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Xor,
}

impl BinaryOp {
    pub fn from_str(s: &str) -> Option<Self> {
        use BinaryOp::*;
        Some(match s {
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
            "%" => Rem,
            "<<" => Shl,
            ">>" => Shr,
            "&" => BitAnd,
            "|" => BitOr,
            "^" => BitXor,
            "==" => Eq,
            "!=" => Ne,
            "<" => Lt,
            ">" => Gt,
            "<=" => Le,
            ">=" => Ge,
            "&&" => And,
            "||" => Or,
            "^^" => Xor,
            _ => return None,
        })
    }

    pub fn symbol(self) -> &'static str {
        use BinaryOp::*;
        match self {
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            Rem => "%",
            Shl => "<<",
            Shr => ">>",
            BitAnd => "&",
            BitOr => "|",
            BitXor => "^",
            Eq => "==",
            Ne => "!=",
            Lt => "<",
            Gt => ">",
            Le => "<=",
            Ge => ">=",
            And => "&&",
            Or => "||",
            Xor => "^^",
        }
    }
}

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Unsigned(u128),
    Signed(i128),
    Float(f64),
    Bool(bool),
    Char(u8),
    String(String),
}

/// Argument of `sizeof` / `addressof` / `typenameof`.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeOrExpr {
    Type(TypeRef),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    /// A plain identifier: variable, pattern, template parameter or constant.
    Ident(String),
    /// `A::B::c` — an enum value, or a namespaced variable/type.
    Scoped(Vec<String>),
    Dollar,
    Null,
    Parent,
    This,
    Unary { op: UnaryOp, operand: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Ternary { cond: Box<Expr>, then: Box<Expr>, otherwise: Box<Expr> },
    /// `u32(x)`, `be u16(x)`
    Cast { ty: TypeRef, value: Box<Expr> },
    /// `x as T`
    Reinterpret { value: Box<Expr>, ty: TypeRef },
    Call { name: Vec<String>, args: Vec<Expr>, line: u32 },
    Member { object: Box<Expr>, member: String },
    Index { object: Box<Expr>, index: Box<Expr> },
    SizeOf(TypeOrExpr),
    AddressOf(TypeOrExpr),
    TypeNameOf(TypeOrExpr),
}

impl Expr {
    pub fn unsigned(v: u128) -> Self {
        Expr::Literal(Literal::Unsigned(v))
    }
}

/// An assignable location.
#[derive(Debug, Clone, PartialEq)]
pub enum LValue {
    Ident(String),
    Dollar,
    /// `a.b.c`, `parent.x`, `this.x`, `a[i].b`
    Path(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Import { line: u32, path: String, alias: Option<String>, all: bool },
    Using { line: u32, name: String, template_params: Vec<TemplateParam>, ty: TypeRef, attrs: Vec<Attribute> },
    UsingForward { line: u32, name: String, template_params: Vec<TemplateParam> },
    Namespace { line: u32, name: Vec<String>, auto: bool, body: Vec<Stmt> },
    Struct(StructDef),
    Union(StructDef),
    Enum(EnumDef),
    Bitfield(StructDef),
    Function(FunctionDef),
    Var(VarDecl),
    Array(ArrayDecl),
    Pointer(PointerDecl),
    Padding { line: u32, size: ArraySize, attrs: Vec<Attribute> },
    BitfieldField(BitfieldField),
    Assign { line: u32, target: LValue, value: Expr },
    Expr { line: u32, expr: Expr },
    If { line: u32, cond: Expr, then: Vec<Stmt>, otherwise: Vec<Stmt> },
    Match { line: u32, values: Vec<Expr>, cases: Vec<MatchCase> },
    TryCatch { line: u32, body: Vec<Stmt>, handler: Vec<Stmt> },
    While { line: u32, cond: Expr, body: Vec<Stmt> },
    For { line: u32, init: Box<Stmt>, cond: Expr, update: Box<Stmt>, body: Vec<Stmt> },
    Return { line: u32, value: Option<Expr> },
    Break { line: u32 },
    Continue { line: u32 },
}

impl Stmt {
    pub fn line(&self) -> u32 {
        match self {
            Stmt::Import { line, .. }
            | Stmt::Using { line, .. }
            | Stmt::UsingForward { line, .. }
            | Stmt::Namespace { line, .. }
            | Stmt::Padding { line, .. }
            | Stmt::Assign { line, .. }
            | Stmt::Expr { line, .. }
            | Stmt::If { line, .. }
            | Stmt::Match { line, .. }
            | Stmt::TryCatch { line, .. }
            | Stmt::While { line, .. }
            | Stmt::For { line, .. }
            | Stmt::Return { line, .. }
            | Stmt::Break { line }
            | Stmt::Continue { line } => *line,
            Stmt::Struct(s) | Stmt::Union(s) | Stmt::Bitfield(s) => s.line,
            Stmt::Enum(e) => e.line,
            Stmt::Function(f) => f.line,
            Stmt::Var(v) => v.line,
            Stmt::Array(a) => a.line,
            Stmt::Pointer(p) => p.line,
            Stmt::BitfieldField(b) => b.line,
        }
    }
}
