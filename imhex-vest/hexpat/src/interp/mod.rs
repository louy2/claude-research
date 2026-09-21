//! The hexpat evaluator.
//!
//! The runtime walks the [`Program`] the way ImHex does: type definitions are
//! registered, placements (`Type name @ address;`) evaluate a type against the
//! data at that address, and evaluating a struct executes its body statement
//! by statement, appending member patterns and advancing the cursor (`$`).
//!
//! Every byte read goes through [`crate::reader::Reader`], whose primitives
//! are `vest_lib` combinators.

mod builtins;
pub mod dump;
mod expr;
mod format;
mod value;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::ast::*;
use crate::error::{Error, Result};
use crate::reader::Reader;

pub use dump::{dump_json, dump_text};
pub use value::{format_float, Pattern, PatternKind, PatternRef, Value};

/// Result of executing a statement list.
#[derive(Debug, Clone)]
pub enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

/// A registered user-defined type.
#[derive(Debug)]
pub struct TypeDecl {
    pub full_name: String,
    pub namespace: Vec<String>,
    pub template_params: Vec<TemplateParam>,
    pub kind: TypeKind,
    /// Resolved enum entries, computed on first use.
    enum_cache: RefCell<Option<Rc<Vec<(String, i128, i128)>>>>,
}

#[derive(Debug)]
pub enum TypeKind {
    Struct(Rc<StructDef>),
    Union(Rc<StructDef>),
    Enum(Rc<EnumDef>),
    Bitfield(Rc<StructDef>),
    Alias(TypeRef, Vec<Attribute>),
    Forward,
}

/// A template binding: either a type or a value.
#[derive(Debug, Clone)]
pub enum Binding {
    Type(Instance),
    Value(Value),
}

/// A fully resolved type: a builtin, or a declaration plus template bindings.
#[derive(Debug, Clone)]
pub struct Instance {
    pub base: InstanceBase,
    pub endian: Option<Endian>,
    pub reference: bool,
    pub bindings: Vec<(String, Binding)>,
    /// Attributes accumulated through `using` aliases.
    pub alias_attrs: Vec<Attribute>,
    /// The name as written (for `type_name` of the pattern).
    pub display: String,
}

#[derive(Debug, Clone)]
pub enum InstanceBase {
    Builtin(BuiltinType),
    Decl(Rc<TypeDecl>),
}

impl Instance {
    fn builtin(b: BuiltinType, endian: Option<Endian>) -> Self {
        Instance {
            base: InstanceBase::Builtin(b),
            endian,
            reference: false,
            bindings: Vec::new(),
            alias_attrs: Vec::new(),
            display: b.name().to_string(),
        }
    }

    pub fn as_builtin(&self) -> Option<BuiltinType> {
        match self.base {
            InstanceBase::Builtin(b) => Some(b),
            _ => None,
        }
    }

    pub fn decl(&self) -> Option<&Rc<TypeDecl>> {
        match &self.base {
            InstanceBase::Decl(d) => Some(d),
            _ => None,
        }
    }

    /// True for types that occupy bytes as a single scalar (ints, floats,
    /// bools, chars and enums).
    pub fn is_scalar(&self) -> bool {
        match &self.base {
            InstanceBase::Builtin(b) => b.size().is_some(),
            InstanceBase::Decl(d) => matches!(d.kind, TypeKind::Enum(_)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Struct,
    Union,
    Bitfield,
    Function,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitfieldOrder {
    LeastToMost,
    MostToLeast,
}

#[derive(Debug, Clone)]
struct Slot {
    value: Value,
    ty: Option<Instance>,
    constant: bool,
}

#[derive(Debug)]
struct BitfieldState {
    base_offset: u64,
    /// Bits consumed so far.
    bits: u64,
    order: BitfieldOrder,
    /// Total size in bits when fixed by `bitfield_order`.
    total_bits: Option<u64>,
    endian: Endian,
    section: usize,
}

#[derive(Debug)]
struct Scope {
    kind: ScopeKind,
    vars: HashMap<String, Slot>,
    template_types: HashMap<String, Instance>,
    this: Option<PatternRef>,
    parent: Option<PatternRef>,
    array_index: Option<u64>,
    namespace: Vec<String>,
    /// For unions: start offset every member is read at, and the running end.
    union_start: u64,
    union_end: u64,
    bitfield: Option<Rc<RefCell<BitfieldState>>>,
}

impl Scope {
    fn new(kind: ScopeKind, namespace: Vec<String>) -> Self {
        Scope {
            kind,
            vars: HashMap::new(),
            template_types: HashMap::new(),
            this: None,
            parent: None,
            array_index: None,
            namespace,
            union_start: 0,
            union_end: 0,
            bitfield: None,
        }
    }
}

pub type BuiltinFn = fn(&mut Runtime, Vec<Value>) -> Result<Value>;

/// Types from the ImHex standard library that native builtins depend on.
const PRELUDE: &str = r#"
namespace std {
    namespace core {
        enum BitfieldOrder : u8 { LeftToRight = 0, RightToLeft = 1, MostToLeastSignificant = 0, LeastToMostSignificant = 1 };
    }
    namespace mem {
        enum Endian : u8 { Native = 0, Big = 1, Little = 2 };
        using Section = u128;
    }
    namespace time {
        struct Time { u8 sec; u8 min; u8 hour; u8 mday; u8 mon; s16 year; u8 wday; u16 yday; bool isdst; } [[sealed]];
        enum TimeZone : u8 { Local, UTC };
    }
}
namespace type {
    struct Magic<auto ExpectedValue> {
        char value[std::string::length(ExpectedValue)];
        std::assert(value == ExpectedValue, std::format("Invalid magic value! Expected \"{}\", got \"{}\"", ExpectedValue, value));
    } [[sealed]];
}
"#;

/// Execution limits to keep malformed inputs from running away.
#[derive(Debug, Clone)]
pub struct Limits {
    pub max_patterns: u64,
    pub max_array_entries: u64,
    pub max_loop_iterations: u64,
    pub max_recursion: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { max_patterns: 2_000_000, max_array_entries: 1_000_000, max_loop_iterations: 50_000_000, max_recursion: 256 }
    }
}

/// The evaluator state.
pub struct Runtime {
    sections: Vec<Vec<u8>>,
    pub cursor: u64,
    pub current_section: usize,
    pub default_endian: Endian,
    bitfield_order: BitfieldOrder,
    types: HashMap<String, Rc<TypeDecl>>,
    functions: HashMap<String, Rc<FunctionDef>>,
    builtins: HashMap<String, BuiltinFn>,
    scopes: Vec<Scope>,
    ns_aliases: HashMap<String, Vec<String>>,
    /// Patterns produced by top-level placements, in order.
    pub patterns: Vec<PatternRef>,
    /// Text written by `std::print`.
    pub console: Vec<String>,
    pub warnings: Vec<String>,
    include_paths: Vec<PathBuf>,
    imported: HashSet<String>,
    /// Pragmas seen (`endian`, `MIME`, ...).
    pub pragmas: Vec<(String, String)>,
    dry_run: usize,
    pattern_count: u64,
    pub limits: Limits,
    env_vars: HashMap<String, Value>,
    in_vars: HashMap<String, Value>,
    pub out_vars: HashMap<String, Value>,
    call_depth: usize,
    /// Extra include-directory search for `import a.b` style paths.
    defines: HashMap<String, String>,
    /// Set when a `break` ran inside a struct body, so that an enclosing
    /// array stops after the current element.
    pending_break: bool,
    pending_continue: bool,
    /// Print every statement and pattern creation to stderr (HEXPAT_TRACE=1).
    pub trace: bool,
    started: std::time::Instant,
}

impl Runtime {
    /// Creates a runtime over `data` (section 0).
    pub fn new(data: Vec<u8>) -> Self {
        let mut rt = Runtime {
            sections: vec![data],
            cursor: 0,
            current_section: 0,
            default_endian: Endian::Little,
            bitfield_order: BitfieldOrder::LeastToMost,
            types: HashMap::new(),
            functions: HashMap::new(),
            builtins: HashMap::new(),
            scopes: vec![Scope::new(ScopeKind::Global, Vec::new())],
            ns_aliases: HashMap::new(),
            patterns: Vec::new(),
            console: Vec::new(),
            warnings: Vec::new(),
            include_paths: Vec::new(),
            imported: HashSet::new(),
            pragmas: Vec::new(),
            dry_run: 0,
            pattern_count: 0,
            limits: Limits::default(),
            env_vars: HashMap::new(),
            in_vars: HashMap::new(),
            out_vars: HashMap::new(),
            call_depth: 0,
            defines: HashMap::new(),
            pending_break: false,
            pending_continue: false,
            trace: std::env::var_os("HEXPAT_TRACE").is_some(),
            started: std::time::Instant::now(),
        };
        builtins::register(&mut rt);
        // Declarations the native builtins refer to, in case the real
        // `std/*.pat` includes are not available. Importing the real files
        // later simply re-registers them with the same values.
        if let Ok(prelude) = crate::lower::parse_program(PRELUDE) {
            let _ = rt.hoist(&prelude.statements);
        }
        rt
    }

    pub fn add_include_path(&mut self, path: impl Into<PathBuf>) {
        self.include_paths.push(path.into());
    }

    pub fn set_env(&mut self, name: &str, value: Value) {
        self.env_vars.insert(name.to_string(), value);
    }

    pub fn set_in_variable(&mut self, name: &str, value: Value) {
        self.in_vars.insert(name.to_string(), value);
    }

    pub fn define(&mut self, name: &str, value: &str) {
        self.defines.insert(name.to_string(), value.to_string());
    }

    pub fn data(&self) -> &[u8] {
        &self.sections[0]
    }

    pub fn section_data(&self, section: usize) -> &[u8] {
        self.sections.get(section).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn data_size(&self) -> u64 {
        self.sections[0].len() as u64
    }

    fn reader(&self, section: usize) -> Reader<'_> {
        Reader::new(self.section_data(section))
    }

    // ------------------------------------------------------------------
    // Entry points
    // ------------------------------------------------------------------

    /// Preprocesses, parses and evaluates a pattern source file.
    pub fn run_source(&mut self, source: &str, base_dir: Option<&Path>) -> Result<()> {
        if let Some(dir) = base_dir {
            if !self.include_paths.iter().any(|p| p == dir) {
                self.include_paths.push(dir.to_path_buf());
            }
        }
        let program = self.load(source, base_dir)?;
        self.run(&program)
    }

    /// Preprocesses and parses without evaluating.
    pub fn load(&mut self, source: &str, base_dir: Option<&Path>) -> Result<Program> {
        let mut pp = crate::preprocess::Preprocessor::new();
        pp.include_paths = self.include_paths.clone();
        pp.defines = self.defines.clone();
        pp.defines.insert("__PL_UNIT_TESTS__".to_string(), String::new());
        let out = pp.run(source, base_dir)?;
        for (name, value) in &out.pragmas {
            self.apply_pragma(name, value)?;
        }
        self.pragmas.extend(out.pragmas.iter().cloned());
        crate::lower::parse_program(&out.source)
    }

    fn apply_pragma(&mut self, name: &str, value: &str) -> Result<()> {
        match name {
            "endian" => {
                self.default_endian = match value.trim() {
                    "big" => Endian::Big,
                    "little" => Endian::Little,
                    "native" => Endian::Little,
                    other => return Err(Error::new(format!("unknown endianness in pragma: {}", other))),
                };
            }
            "base_address" => {
                // Not modelled: the data always starts at address 0.
            }
            _ => {}
        }
        Ok(())
    }

    /// Evaluates a parsed program against the data.
    pub fn run(&mut self, program: &Program) -> Result<()> {
        // Types and functions are visible before their definition, as in the
        // reference implementation, which registers them while parsing.
        self.hoist(&program.statements)?;
        self.exec_stmts(&program.statements)?;
        // Like the reference runtime, a global `fn main()` runs afterwards.
        if self.functions.contains_key("main") {
            self.call_function(&["main".to_string()], Vec::new())?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Scopes
    // ------------------------------------------------------------------

    fn scope(&self) -> &Scope {
        self.scopes.last().expect("scope stack is never empty")
    }

    fn scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("scope stack is never empty")
    }

    fn push_scope(&mut self, kind: ScopeKind) {
        let ns = self.scope().namespace.clone();
        let mut s = Scope::new(kind, ns);
        if kind != ScopeKind::Function {
            // Nested scopes inherit `this`/`parent`/array index unless they
            // introduce their own.
            let cur = self.scope();
            s.this = cur.this.clone();
            s.parent = cur.parent.clone();
            s.array_index = cur.array_index;
            s.bitfield = cur.bitfield.clone();
        }
        self.scopes.push(s);
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        debug_assert!(!self.scopes.is_empty());
    }

    fn current_namespace(&self) -> Vec<String> {
        self.scope().namespace.clone()
    }

    fn this(&self) -> Option<PatternRef> {
        self.scope().this.clone()
    }

    fn lookup_slot(&self, name: &str) -> Option<&Slot> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.vars.get(name) {
                return Some(v);
            }
            if s.kind == ScopeKind::Function {
                break;
            }
        }
        self.scopes[0].vars.get(name)
    }

    fn lookup_slot_mut(&mut self, name: &str) -> Option<&mut Slot> {
        let mut idx = None;
        for (i, s) in self.scopes.iter().enumerate().rev() {
            if s.vars.contains_key(name) {
                idx = Some(i);
                break;
            }
            if s.kind == ScopeKind::Function {
                break;
            }
        }
        let i = idx.or_else(|| if self.scopes[0].vars.contains_key(name) { Some(0) } else { None })?;
        self.scopes[i].vars.get_mut(name)
    }

    fn lookup_template_type(&self, name: &str) -> Option<Instance> {
        for s in self.scopes.iter().rev() {
            if let Some(t) = s.template_types.get(name) {
                return Some(t.clone());
            }
            if s.kind == ScopeKind::Function {
                break;
            }
        }
        None
    }

    /// Finds a member of the innermost struct-like pattern being evaluated.
    fn lookup_member(&self, name: &str) -> Option<PatternRef> {
        for s in self.scopes.iter().rev() {
            if let Some(this) = &s.this {
                if let Some(m) = this.borrow().member(name) {
                    return Some(m);
                }
                return None;
            }
            if s.kind == ScopeKind::Function {
                break;
            }
        }
        None
    }

    pub fn array_index(&self) -> Option<u64> {
        for s in self.scopes.iter().rev() {
            if let Some(i) = s.array_index {
                return Some(i);
            }
        }
        None
    }

    // ------------------------------------------------------------------
    // Name resolution
    // ------------------------------------------------------------------

    fn candidate_names(&self, path: &[String]) -> Vec<String> {
        let mut path: Vec<String> = path.to_vec();
        if let Some(alias) = self.ns_aliases.get(&path[0]) {
            let mut p = alias.clone();
            p.extend(path.drain(1..));
            path = p;
        }
        let joined = path.join("::");
        let mut out = Vec::new();
        let ns = self.current_namespace();
        for i in (0..=ns.len()).rev() {
            let prefix = ns[..i].join("::");
            if prefix.is_empty() {
                out.push(joined.clone());
            } else {
                out.push(format!("{}::{}", prefix, joined));
            }
        }
        // Types defined in enclosing struct namespaces are visible too.
        for s in self.scopes.iter().rev() {
            for i in (1..=s.namespace.len()).rev() {
                let cand = format!("{}::{}", s.namespace[..i].join("::"), joined);
                if !out.contains(&cand) {
                    out.push(cand);
                }
            }
        }
        out
    }

    fn find_type(&self, path: &[String]) -> Option<Rc<TypeDecl>> {
        for c in self.candidate_names(path) {
            if let Some(t) = self.types.get(&c) {
                return Some(t.clone());
            }
        }
        None
    }

    pub fn find_function(&self, path: &[String]) -> Option<Rc<FunctionDef>> {
        for c in self.candidate_names(path) {
            if let Some(f) = self.functions.get(&c) {
                return Some(f.clone());
            }
        }
        None
    }

    fn find_builtin(&self, path: &[String]) -> Option<BuiltinFn> {
        for c in self.candidate_names(path) {
            if let Some(f) = self.builtins.get(&c) {
                return Some(*f);
            }
        }
        None
    }

    fn qualified(&self, name: &str) -> String {
        let ns = self.current_namespace();
        if ns.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", ns.join("::"), name)
        }
    }

    // ------------------------------------------------------------------
    // Type instantiation
    // ------------------------------------------------------------------

    /// Resolves a [`TypeRef`] (following aliases and binding template
    /// arguments) in the current scope.
    pub fn instantiate(&mut self, ty: &TypeRef) -> Result<Instance> {
        let mut inst = match &ty.base {
            TypeBase::Builtin(b) => Instance::builtin(*b, ty.endian),
            TypeBase::Custom { name, args } => {
                if name.len() == 1 {
                    if let Some(t) = self.lookup_template_type(&name[0]) {
                        let mut t = t;
                        if ty.endian.is_some() {
                            t.endian = ty.endian;
                        }
                        t.reference = ty.reference;
                        return Ok(t);
                    }
                }
                let decl = self
                    .find_type(name)
                    .ok_or_else(|| Error::new(format!("unknown type `{}`", name.join("::"))))?;
                self.instantiate_decl(decl, args, ty.endian, name.join("::"))?
            }
        };
        inst.reference = ty.reference;
        Ok(inst)
    }

    fn instantiate_decl(
        &mut self,
        decl: Rc<TypeDecl>,
        args: &[TemplateArg],
        endian: Option<Endian>,
        display: String,
    ) -> Result<Instance> {
        // Bind template arguments in the *current* scope.
        let mut bindings = Vec::new();
        if !decl.template_params.is_empty() {
            if args.len() > decl.template_params.len() {
                return Err(Error::new(format!(
                    "type `{}` takes {} template argument(s), {} given",
                    decl.full_name,
                    decl.template_params.len(),
                    args.len()
                )));
            }
            for (param, arg) in decl.template_params.iter().zip(args.iter()) {
                let b = if param.is_type {
                    match arg {
                        TemplateArg::Type(t) => Binding::Type(self.instantiate(t)?),
                        TemplateArg::Expr(Expr::Ident(n)) => Binding::Type(self.instantiate(&TypeRef::custom(n))?),
                        TemplateArg::Expr(Expr::Scoped(path)) => {
                            let tr = TypeRef { reference: false, endian: None, base: TypeBase::Custom { name: path.clone(), args: Vec::new() } };
                            Binding::Type(self.instantiate(&tr)?)
                        }
                        TemplateArg::Expr(_) => {
                            return Err(Error::new(format!(
                                "template parameter `{}` of `{}` expects a type",
                                param.name, decl.full_name
                            )))
                        }
                    }
                } else {
                    match arg {
                        TemplateArg::Expr(e) => Binding::Value(self.eval(e)?),
                        TemplateArg::Type(t) => {
                            // A bare identifier parsed as a type: treat as a value.
                            match &t.base {
                                TypeBase::Custom { name, args } if args.is_empty() => {
                                    let e = if name.len() == 1 {
                                        Expr::Ident(name[0].clone())
                                    } else {
                                        Expr::Scoped(name.clone())
                                    };
                                    Binding::Value(self.eval(&e)?)
                                }
                                _ => {
                                    return Err(Error::new(format!(
                                        "template parameter `{}` of `{}` expects a value",
                                        param.name, decl.full_name
                                    )))
                                }
                            }
                        }
                    }
                };
                bindings.push((param.name.clone(), b));
            }
        }
        match &decl.kind {
            TypeKind::Alias(target, attrs) => {
                // Resolve the alias target with the parameters bound.
                let mut scope = Scope::new(ScopeKind::Global, decl.namespace.clone());
                for (name, b) in &bindings {
                    match b {
                        Binding::Type(t) => {
                            scope.template_types.insert(name.clone(), t.clone());
                        }
                        Binding::Value(v) => {
                            scope.vars.insert(name.clone(), Slot { value: v.clone(), ty: None, constant: true });
                        }
                    }
                }
                let cur = self.scope();
                scope.this = cur.this.clone();
                scope.parent = cur.parent.clone();
                scope.array_index = cur.array_index;
                self.scopes.push(scope);
                let r = self.instantiate(target);
                self.pop_scope();
                let mut inst = r?;
                if endian.is_some() {
                    inst.endian = endian;
                }
                inst.alias_attrs.extend(attrs.iter().cloned());
                inst.display = display;
                Ok(inst)
            }
            TypeKind::Forward => Err(Error::new(format!("type `{}` is only forward-declared", decl.full_name))),
            _ => Ok(Instance {
                base: InstanceBase::Decl(decl),
                endian,
                reference: false,
                bindings,
                alias_attrs: Vec::new(),
                display,
            }),
        }
    }

    /// Size in bytes of a type when it is statically known.
    pub fn static_size(&mut self, inst: &Instance) -> Result<Option<u64>> {
        match &inst.base {
            InstanceBase::Builtin(b) => Ok(b.size()),
            InstanceBase::Decl(d) => match &d.kind {
                TypeKind::Enum(e) => {
                    let u = self.instantiate(&e.ty)?;
                    self.static_size(&u)
                }
                _ => Ok(None),
            },
        }
    }

    /// `sizeof(Type)`: evaluates the type at the cursor in dry-run mode.
    pub fn sizeof_type(&mut self, inst: &Instance) -> Result<u64> {
        if let Some(s) = self.static_size(inst)? {
            return Ok(s);
        }
        // Evaluate the type in a zero-filled scratch section (which grows on
        // demand), as the reference runtime does, so that `sizeof(T)` never
        // depends on the cursor position or the data.
        let saved_cursor = self.cursor;
        let saved_section = self.current_section;
        let saved_count = self.pattern_count;
        let scratch = self.new_section(0);
        self.cursor = 0;
        self.current_section = scratch;
        self.dry_run += 1;
        let endian = inst.endian.unwrap_or(self.default_endian);
        let r = self.create_pattern(inst, "", endian, scratch);
        self.dry_run -= 1;
        self.cursor = saved_cursor;
        self.current_section = saved_section;
        self.pattern_count = saved_count;
        if scratch + 1 == self.sections.len() {
            self.sections.pop();
        } else {
            self.sections[scratch] = Vec::new();
        }
        let p = r?;
        let size = p.borrow().size;
        Ok(size)
    }

    // ------------------------------------------------------------------
    // Statement execution
    // ------------------------------------------------------------------

    pub fn exec_stmts(&mut self, stmts: &[Stmt]) -> Result<Flow> {
        for s in stmts {
            let flow = self.exec_stmt(s).map_err(|e| e.with_location(s.line(), 1))?;
            if !matches!(flow, Flow::Normal) {
                return Ok(flow);
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Flow> {
        if self.trace {
            eprintln!("[trace] {:>7}ms {:indent$}line {}", self.started.elapsed().as_millis(), "", stmt.line(), indent = self.scopes.len());
        }
        match stmt {
            Stmt::Import { path, alias, all: _, .. } => {
                self.import(path, alias.as_deref())?;
            }
            Stmt::Using { name, template_params, ty, attrs, .. } => {
                let full = self.qualified(name);
                let decl = TypeDecl {
                    full_name: full.clone(),
                    namespace: self.current_namespace(),
                    template_params: template_params.clone(),
                    kind: TypeKind::Alias(ty.clone(), attrs.clone()),
                    enum_cache: RefCell::new(None),
                };
                self.types.insert(full, Rc::new(decl));
            }
            Stmt::UsingForward { name, template_params, .. } => {
                let full = self.qualified(name);
                if !self.types.contains_key(&full) {
                    let decl = TypeDecl {
                        full_name: full.clone(),
                        namespace: self.current_namespace(),
                        template_params: template_params.clone(),
                        kind: TypeKind::Forward,
                        enum_cache: RefCell::new(None),
                    };
                    self.types.insert(full, Rc::new(decl));
                }
            }
            Stmt::Namespace { name, body, .. } => {
                let mut ns = self.current_namespace();
                ns.extend(name.iter().cloned());
                let saved = std::mem::replace(&mut self.scope_mut().namespace, ns);
                let r = self.exec_stmts(body);
                self.scope_mut().namespace = saved;
                return r;
            }
            Stmt::Struct(def) => self.register_struct(def, "struct")?,
            Stmt::Union(def) => self.register_struct(def, "union")?,
            Stmt::Bitfield(def) => self.register_struct(def, "bitfield")?,
            Stmt::Enum(def) => {
                let full = self.qualified(&def.name);
                let decl = TypeDecl {
                    full_name: full.clone(),
                    namespace: self.current_namespace(),
                    template_params: Vec::new(),
                    kind: TypeKind::Enum(Rc::new(def.clone())),
                    enum_cache: RefCell::new(None),
                };
                self.types.insert(full, Rc::new(decl));
            }
            Stmt::Function(def) => {
                let full = self.qualified(&def.name);
                self.functions.insert(full, Rc::new(def.clone()));
            }
            Stmt::Var(decl) => return self.exec_var_decl(decl),
            Stmt::Array(decl) => return self.exec_array_decl(decl),
            Stmt::Pointer(decl) => self.exec_pointer_decl(decl)?,
            Stmt::Padding { size, attrs, .. } => self.exec_padding(size, attrs)?,
            Stmt::BitfieldField(field) => self.exec_bitfield_field(field)?,
            Stmt::Assign { target, value, .. } => {
                let v = self.eval(value)?;
                self.assign(target, v)?;
            }
            Stmt::Expr { expr, .. } => {
                self.eval(expr)?;
            }
            Stmt::If { cond, then, otherwise, .. } => {
                let c = self.eval(cond)?.as_bool()?;
                return if c { self.exec_stmts(then) } else { self.exec_stmts(otherwise) };
            }
            Stmt::Match { values, cases, .. } => return self.exec_match(values, cases),
            Stmt::TryCatch { body, handler, .. } => {
                let saved_cursor = self.cursor;
                match self.exec_stmts(body) {
                    Ok(flow) => return Ok(flow),
                    Err(_) => {
                        self.cursor = saved_cursor;
                        return self.exec_stmts(handler);
                    }
                }
            }
            Stmt::While { cond, body, .. } => {
                let mut iterations = 0u64;
                while self.eval(cond)?.as_bool()? {
                    iterations += 1;
                    if iterations > self.limits.max_loop_iterations {
                        return Err(Error::new("while loop exceeded the iteration limit"));
                    }
                    match self.exec_stmts(body)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        _ => {}
                    }
                }
            }
            Stmt::For { init, cond, update, body, .. } => {
                self.exec_stmt(init)?;
                let mut iterations = 0u64;
                while self.eval(cond)?.as_bool()? {
                    iterations += 1;
                    if iterations > self.limits.max_loop_iterations {
                        return Err(Error::new("for loop exceeded the iteration limit"));
                    }
                    match self.exec_stmts(body)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        _ => {}
                    }
                    self.exec_stmt(update)?;
                }
            }
            Stmt::Return { value, .. } => {
                let v = match value {
                    Some(e) => self.eval(e)?,
                    None => Value::Null,
                };
                return Ok(Flow::Return(v));
            }
            Stmt::Break { .. } => return Ok(Flow::Break),
            Stmt::Continue { .. } => return Ok(Flow::Continue),
        }
        Ok(Flow::Normal)
    }

    /// Registers every type and function definition in `stmts` (recursing
    /// into namespaces) without evaluating anything else.
    fn hoist(&mut self, stmts: &[Stmt]) -> Result<()> {
        for s in stmts {
            match s {
                Stmt::Namespace { name, body, .. } => {
                    let mut ns = self.current_namespace();
                    ns.extend(name.iter().cloned());
                    let saved = std::mem::replace(&mut self.scope_mut().namespace, ns);
                    let r = self.hoist(body);
                    self.scope_mut().namespace = saved;
                    r?;
                }
                Stmt::Struct(_) | Stmt::Union(_) | Stmt::Bitfield(_) | Stmt::Enum(_) | Stmt::Function(_) | Stmt::Using { .. } | Stmt::UsingForward { .. } => {
                    self.exec_stmt(s)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn register_struct(&mut self, def: &StructDef, kind: &str) -> Result<()> {
        let full = self.qualified(&def.name);
        let rc = Rc::new(def.clone());
        let decl = TypeDecl {
            full_name: full.clone(),
            namespace: self.current_namespace(),
            template_params: def.template_params.clone(),
            kind: match kind {
                "struct" => TypeKind::Struct(rc),
                "union" => TypeKind::Union(rc),
                _ => TypeKind::Bitfield(rc),
            },
            enum_cache: RefCell::new(None),
        };
        self.types.insert(full, Rc::new(decl));
        Ok(())
    }

    fn exec_match(&mut self, values: &[Expr], cases: &[MatchCase]) -> Result<Flow> {
        let vals: Vec<Value> = values.iter().map(|e| self.eval(e)).collect::<Result<_>>()?;
        let mut default: Option<&MatchCase> = None;
        for case in cases {
            if case.patterns.iter().all(|p| matches!(p, CasePattern::Wildcard)) {
                default = Some(case);
                continue;
            }
            let mut all = true;
            for (pat, val) in case.patterns.iter().zip(vals.iter()) {
                if !self.case_matches(pat, val)? {
                    all = false;
                    break;
                }
            }
            if all {
                return self.exec_stmts(&case.body);
            }
        }
        if let Some(d) = default {
            return self.exec_stmts(&d.body);
        }
        Ok(Flow::Normal)
    }

    fn case_matches(&mut self, pat: &CasePattern, val: &Value) -> Result<bool> {
        match pat {
            CasePattern::Wildcard => Ok(true),
            CasePattern::Alternatives(alts) => {
                for (a, b) in alts {
                    let av = self.eval(a)?;
                    let ok = match b {
                        None => expr::compare(val, &av, BinaryOp::Eq)?,
                        Some(b) => {
                            let bv = self.eval(b)?;
                            expr::compare(val, &av, BinaryOp::Ge)? && expr::compare(val, &bv, BinaryOp::Le)?
                        }
                    };
                    if ok {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    // ------------------------------------------------------------------
    // Imports
    // ------------------------------------------------------------------

    fn import(&mut self, path: &str, alias: Option<&str>) -> Result<()> {
        let key = path.trim_end_matches(".pat").to_string();
        let file = self.resolve_import(&key);
        let Some(file) = file else {
            // The standard library is available natively for the common
            // functions, so a missing include directory is not fatal.
            if key.starts_with("std/") || key.starts_with("std\\") || key == "type/magic" {
                if let Some(a) = alias {
                    let ns: Vec<String> = key.split('/').map(|s| s.to_string()).collect();
                    self.ns_aliases.insert(a.to_string(), ns);
                }
                return Ok(());
            }
            return Err(Error::new(format!("cannot resolve import `{}` (no include path contains it)", path)));
        };
        let canonical = file.canonicalize().unwrap_or(file.clone()).to_string_lossy().to_string();
        if let Some(a) = alias {
            let ns: Vec<String> = key.split(['/', '\\']).map(|s| s.to_string()).collect();
            self.ns_aliases.insert(a.to_string(), ns);
        }
        if self.imported.contains(&canonical) {
            return Ok(());
        }
        self.imported.insert(canonical);
        let source = std::fs::read_to_string(&file)
            .map_err(|e| Error::new(format!("cannot read import {}: {}", file.display(), e)))?;
        let program = self.load(&source, file.parent())?;
        // Imports evaluate at global scope, in the root namespace.
        let saved_ns = std::mem::take(&mut self.scopes[0].namespace);
        let saved_len = self.scopes.len();
        let saved_scopes: Vec<Scope> = self.scopes.drain(1..).collect();
        let r = self.hoist(&program.statements).and_then(|_| self.exec_stmts(&program.statements));
        self.scopes.truncate(1);
        self.scopes.extend(saved_scopes);
        debug_assert_eq!(self.scopes.len(), saved_len);
        self.scopes[0].namespace = saved_ns;
        r.map_err(|e| Error::new(format!("in import `{}`: {}", path, e)))?;
        Ok(())
    }

    fn resolve_import(&self, key: &str) -> Option<PathBuf> {
        let mut candidates = Vec::new();
        for inc in &self.include_paths {
            candidates.push(inc.join(format!("{}.pat", key)));
            candidates.push(inc.join(format!("{}.hexpat", key)));
            candidates.push(inc.join(key));
        }
        candidates.into_iter().find(|p| p.is_file())
    }

    // ------------------------------------------------------------------
    // Declarations
    // ------------------------------------------------------------------

    fn in_type_body(&self) -> bool {
        matches!(self.scope().kind, ScopeKind::Struct | ScopeKind::Union | ScopeKind::Bitfield)
    }

    fn exec_var_decl(&mut self, decl: &VarDecl) -> Result<Flow> {
        let inst = self.instantiate(&decl.ty)?;
        let is_scalar_builtin = matches!(inst.base, InstanceBase::Builtin(_));
        let has_init = decl.value.is_some();
        let scope_kind = self.scope().kind;

        // Local / global variables (not read from the data).
        let is_local = has_init
            || decl.in_var
            || decl.out_var
            || scope_kind == ScopeKind::Function
            || (scope_kind == ScopeKind::Global && decl.placement.is_none() && is_scalar_builtin);
        if is_local && decl.placement.is_none() {
            let value = if let Some(e) = &decl.value {
                let v = self.eval(e)?;
                self.coerce(&inst, v)?
            } else if decl.in_var {
                match self.in_vars.get(&decl.name) {
                    Some(v) => v.clone(),
                    None => self.default_value(&inst)?,
                }
            } else {
                self.default_value(&inst)?
            };
            if decl.out_var {
                self.out_vars.insert(decl.name.clone(), value.clone());
            }
            // Inside a type body a local becomes a (hidden unless exported)
            // member backed by a heap section, so that `parent.x` works.
            if self.in_type_body() && inst.is_scalar() && !matches!(value, Value::Pattern(_)) {
                if let Ok(size) = self.sizeof_type(&inst) {
                    let section = self.new_section(size);
                    let saved = (self.cursor, self.current_section);
                    self.cursor = 0;
                    self.current_section = section;
                    let endian = inst.endian.unwrap_or(self.default_endian);
                    let r = self.create_pattern(&inst, &decl.name, endian, section);
                    self.cursor = saved.0;
                    self.current_section = saved.1;
                    let p = r?;
                    self.write_pattern(&p, value.clone())?;
                    p.borrow_mut().hidden = !decl.attrs.iter().any(|a| a.is("export"));
                    self.apply_attributes(&p, &decl.attrs)?;
                    if let Some(this) = self.this() {
                        this.borrow_mut().push_member(p.clone());
                    }
                    self.scope_mut()
                        .vars
                        .insert(decl.name.clone(), Slot { value: Value::Pattern(p), ty: Some(inst), constant: decl.constant });
                    return Ok(Flow::Normal);
                }
            }
            self.scope_mut()
                .vars
                .insert(decl.name.clone(), Slot { value, ty: Some(inst), constant: decl.constant });
            return Ok(Flow::Normal);
        }

        // A view in a function body: `Type name @ addr;` reads the data but
        // does not add to the output.
        let endian = inst.endian.unwrap_or(self.default_endian);
        if scope_kind == ScopeKind::Function {
            let placement = decl.placement.as_ref().unwrap();
            let addr = self.eval(&placement.address)?.as_u64()?;
            let section = self.eval_section(placement.section.as_ref())?;
            let saved = self.cursor;
            self.cursor = addr;
            let r = self.create_pattern(&inst, &decl.name, endian, section);
            self.cursor = saved;
            let p = r?;
            self.apply_attributes(&p, &decl.attrs)?;
            self.scope_mut().vars.insert(
                decl.name.clone(),
                Slot { value: Value::Pattern(p), ty: Some(inst), constant: false },
            );
            return Ok(Flow::Normal);
        }

        // Member (inside a type body) or top-level placement.
        self.place_member(&inst, &decl.name, endian, decl.placement.as_ref(), &decl.attrs, |rt, inst, name, endian, section| {
            rt.create_pattern(inst, name, endian, section)
        })
    }

    /// Shared logic for declaring a pattern in a type body or at top level.
    fn place_member<F>(
        &mut self,
        inst: &Instance,
        name: &str,
        endian: Endian,
        placement: Option<&Placement>,
        attrs: &[Attribute],
        create: F,
    ) -> Result<Flow>
    where
        F: FnOnce(&mut Runtime, &Instance, &str, Endian, usize) -> Result<PatternRef>,
    {
        let scope_kind = self.scope().kind;
        let no_unique_address = attrs.iter().any(|a| a.is("no_unique_address"));
        let in_body = self.in_type_body();
        let flow_break = Rc::new(RefCell::new(false));

        let pattern = if let Some(pl) = placement {
            let addr = self.eval(&pl.address)?.as_u64()?;
            let section = self.eval_section(pl.section.as_ref())?;
            let saved_cursor = self.cursor;
            let saved_section = self.current_section;
            self.cursor = addr;
            self.current_section = section;
            let r = create(self, inst, name, endian, section);
            self.current_section = saved_section;
            if in_body || no_unique_address {
                self.cursor = saved_cursor;
            }
            r?
        } else {
            let section = self.current_section;
            if scope_kind == ScopeKind::Union {
                let start = self.scope().union_start;
                self.cursor = start;
            }
            let start = self.cursor;
            let r = create(self, inst, name, endian, section);
            let p = r?;
            if no_unique_address {
                self.cursor = start;
            }
            if scope_kind == ScopeKind::Union {
                let end = p.borrow().end();
                let s = self.scope_mut();
                s.union_end = s.union_end.max(end);
                self.cursor = start;
            }
            p
        };
        let _ = flow_break;
        self.apply_attributes(&pattern, attrs)?;
        if in_body {
            if let Some(this) = self.this() {
                let mut tb = this.borrow_mut();
                tb.push_member(pattern);
                // Keep `sizeof(this)` meaningful while the body runs.
                if scope_kind == ScopeKind::Struct && self.cursor > tb.offset && tb.section == self.current_section {
                    tb.size = self.cursor - tb.offset;
                }
            }
            // A `break` inside the member's body stops this body too, up to
            // the nearest enclosing array.
            if self.pending_break {
                return Ok(Flow::Break);
            }
            if self.pending_continue {
                return Ok(Flow::Continue);
            }
        } else if self.dry_run == 0 {
            self.pending_break = false;
            self.pending_continue = false;
            if scope_kind == ScopeKind::Function {
                self.scope_mut()
                    .vars
                    .insert(name.to_string(), Slot { value: Value::Pattern(pattern), ty: None, constant: false });
            } else {
                self.patterns.push(pattern);
            }
        }
        Ok(Flow::Normal)
    }

    fn eval_section(&mut self, section: Option<&Expr>) -> Result<usize> {
        match section {
            None => Ok(self.current_section),
            Some(e) => {
                let id = self.eval(e)?.as_u64()? as usize;
                if id >= self.sections.len() {
                    return Err(Error::new(format!("invalid section {}", id)));
                }
                Ok(id)
            }
        }
    }

    fn exec_array_decl(&mut self, decl: &ArrayDecl) -> Result<Flow> {
        let inst = self.instantiate(&decl.ty)?;
        let scope_kind = self.scope().kind;
        let endian = inst.endian.unwrap_or(self.default_endian);
        if decl.placement.is_none()
            && (decl.init.is_some() || scope_kind == ScopeKind::Function || (scope_kind == ScopeKind::Global && inst.as_builtin().is_some()))
        {
            // Local array of `str` (or another non-placeable type): a list value.
            if inst.as_builtin().map(|b| b.size().is_none()).unwrap_or(false) {
                let mut items = Vec::new();
                if let Some(init) = &decl.init {
                    for e in init {
                        items.push(self.eval(e)?);
                    }
                } else if let ArraySize::Fixed(e) = &decl.size {
                    let n = self.eval(e)?.as_u64()?;
                    let d = self.default_value(&inst)?;
                    items.resize(n as usize, d);
                }
                let list = Value::List(Rc::new(RefCell::new(items)));
                self.scope_mut().vars.insert(decl.name.clone(), Slot { value: list, ty: None, constant: decl.constant });
                return Ok(Flow::Normal);
            }
            // Local array variable: allocate a heap section.
            let count = match &decl.size {
                ArraySize::Fixed(e) => self.eval(e)?.as_u64()?,
                ArraySize::Unsized => decl.init.as_ref().map(|v| v.len() as u64).unwrap_or(0),
                ArraySize::While(_) => return Err(Error::new("while-sized arrays are not allowed for local variables")),
            };
            let elem = self.sizeof_type(&inst)?;
            let section = self.new_section(count * elem);
            let saved = (self.cursor, self.current_section);
            self.cursor = 0;
            self.current_section = section;
            let r = self.create_array(&inst, &decl.name, &ArraySize::Fixed(Expr::unsigned(count as u128)), endian, section);
            self.cursor = saved.0;
            self.current_section = saved.1;
            let p = r?;
            if let Some(init) = &decl.init {
                for (i, e) in init.iter().enumerate() {
                    let v = self.eval(e)?;
                    let elem = self.array_element(&p, i as u64)?;
                    self.write_pattern(&elem, v)?;
                }
            }
            self.scope_mut()
                .vars
                .insert(decl.name.clone(), Slot { value: Value::Pattern(p), ty: Some(inst), constant: decl.constant });
            return Ok(Flow::Normal);
        }
        if scope_kind == ScopeKind::Function {
            // View: `Type name[n] @ addr;`
            let pl = decl.placement.as_ref().unwrap();
            let addr = self.eval(&pl.address)?.as_u64()?;
            let section = self.eval_section(pl.section.as_ref())?;
            let saved = self.cursor;
            self.cursor = addr;
            let r = self.create_array(&inst, &decl.name, &decl.size, endian, section);
            self.cursor = saved;
            let p = r?;
            self.apply_attributes(&p, &decl.attrs)?;
            self.scope_mut().vars.insert(decl.name.clone(), Slot { value: Value::Pattern(p), ty: None, constant: false });
            return Ok(Flow::Normal);
        }
        let size = &decl.size;
        self.place_member(&inst, &decl.name, endian, decl.placement.as_ref(), &decl.attrs, |rt, inst, name, endian, section| {
            rt.create_array(inst, name, size, endian, section)
        })
    }

    fn exec_pointer_decl(&mut self, decl: &PointerDecl) -> Result<()> {
        let inst = self.instantiate(&decl.ty)?;
        let ptr_inst = self.instantiate(&decl.pointer_ty)?;
        let endian = inst.endian.unwrap_or(self.default_endian);
        let ptr_endian = ptr_inst.endian.unwrap_or(self.default_endian);
        let base_fn = decl.attrs.iter().find(|a| a.is("pointer_base")).and_then(|a| a.args.first().cloned());
        let array = decl.array.clone();
        self.place_member(&inst, &decl.name, endian, decl.placement.as_ref(), &decl.attrs, |rt, inst, name, endian, section| {
            rt.create_pointer(inst, &ptr_inst, name, endian, ptr_endian, section, array.as_ref(), base_fn.as_ref())
        })?;
        Ok(())
    }

    fn exec_padding(&mut self, size: &ArraySize, attrs: &[Attribute]) -> Result<()> {
        let start = self.cursor;
        let section = self.current_section;
        let n = match size {
            ArraySize::Fixed(e) => self.eval(e)?.as_u64()?,
            ArraySize::While(cond) => {
                let mut n = 0u64;
                loop {
                    if !self.eval(cond)?.as_bool()? {
                        break;
                    }
                    if self.cursor >= self.section_len(section) {
                        break;
                    }
                    self.cursor += 1;
                    n += 1;
                }
                self.cursor = start;
                n
            }
            ArraySize::Unsized => return Err(Error::new("padding needs a size")),
        };
        if self.scope().kind == ScopeKind::Bitfield {
            return Err(Error::new("byte padding inside a bitfield"));
        }
        let p = Pattern::new("", "padding", start, n, section, self.default_endian, PatternKind::Padding).shared();
        p.borrow_mut().hidden = true;
        self.apply_attributes(&p, attrs)?;
        self.cursor = start + n;
        if let Some(this) = self.this() {
            this.borrow_mut().push_member(p);
        }
        Ok(())
    }

    fn section_len(&self, section: usize) -> u64 {
        self.section_data(section).len() as u64
    }

    fn new_section(&mut self, size: u64) -> usize {
        self.sections.push(vec![0u8; size as usize]);
        self.sections.len() - 1
    }

    // ------------------------------------------------------------------
    // Pattern creation
    // ------------------------------------------------------------------

    fn count_pattern(&mut self) -> Result<()> {
        self.pattern_count += 1;
        if self.pattern_count > self.limits.max_patterns {
            return Err(Error::new("pattern limit exceeded"));
        }
        Ok(())
    }

    /// Creates a pattern of type `inst` named `name` at the cursor and
    /// advances the cursor past it.
    pub fn create_pattern(&mut self, inst: &Instance, name: &str, endian: Endian, section: usize) -> Result<PatternRef> {
        self.count_pattern()?;
        if self.trace {
            eprintln!("[trace] {:>7}ms {:indent$}create {} {} @ 0x{:X}", self.started.elapsed().as_millis(), "", inst.display, name, self.cursor, indent = self.scopes.len());
        }
        let endian = inst.endian.unwrap_or(endian);
        let p = match &inst.base {
            InstanceBase::Builtin(b) => self.create_builtin(*b, name, endian, section)?,
            InstanceBase::Decl(decl) => {
                let decl = decl.clone();
                match &decl.kind {
                    TypeKind::Struct(def) => self.create_struct(&decl, def.clone(), inst, name, endian, section, false)?,
                    TypeKind::Union(def) => self.create_struct(&decl, def.clone(), inst, name, endian, section, true)?,
                    TypeKind::Bitfield(def) => self.create_bitfield(&decl, def.clone(), inst, name, endian, section)?,
                    TypeKind::Enum(def) => self.create_enum(&decl, def.clone(), name, endian, section)?,
                    TypeKind::Alias(..) | TypeKind::Forward => unreachable!("aliases are resolved by instantiate"),
                }
            }
        };
        let applied_in_scope = matches!(&inst.base, InstanceBase::Decl(d) if matches!(d.kind, TypeKind::Struct(_) | TypeKind::Union(_) | TypeKind::Bitfield(_)));
        if !inst.alias_attrs.is_empty() && !applied_in_scope {
            let attrs = inst.alias_attrs.clone();
            self.apply_attributes(&p, &attrs)?;
        }
        if let InstanceBase::Decl(_) = inst.base {
            p.borrow_mut().type_name = inst.display.clone();
        }
        Ok(p)
    }

    fn check_bounds(&mut self, section: usize, offset: u64, size: u64) -> Result<()> {
        let len = self.section_len(section);
        if section != 0 && offset.checked_add(size).map(|end| end > len).unwrap_or(false) {
            // Heap sections grow on demand, like ImHex sections, within a
            // sanity limit so that a garbage length cannot allocate gigabytes.
            let end = offset + size;
            if end <= (16u64 << 20) {
                self.sections[section].resize(end as usize, 0);
                return Ok(());
            }
        }
        if offset.checked_add(size).map(|end| end > len).unwrap_or(true) {
            return Err(Error::new(format!(
                "cannot read {} byte(s) at address 0x{:X}: out of bounds (size 0x{:X})",
                size, offset, len
            )));
        }
        Ok(())
    }

    fn create_builtin(&mut self, b: BuiltinType, name: &str, endian: Endian, section: usize) -> Result<PatternRef> {
        let size = b.size().ok_or_else(|| Error::new(format!("type `{}` cannot be placed in memory", b)))?;
        let kind = match b {
            _ if b.is_unsigned() => PatternKind::Unsigned,
            _ if b.is_signed() => PatternKind::Signed,
            BuiltinType::Float | BuiltinType::Double => PatternKind::Float,
            BuiltinType::Bool => PatternKind::Bool,
            BuiltinType::Char => PatternKind::Char,
            BuiltinType::Char16 => PatternKind::Char16,
            _ => unreachable!(),
        };
        let offset = self.cursor;
        self.check_bounds(section, offset, size)?;
        let mut p = Pattern::new(name, b.name(), offset, size, section, endian, kind);
        p.builtin = Some(b);
        self.cursor = offset + size;
        Ok(p.shared())
    }

    fn enum_entries(&mut self, decl: &Rc<TypeDecl>, def: &EnumDef) -> Result<Rc<Vec<(String, i128, i128)>>> {
        if let Some(e) = decl.enum_cache.borrow().as_ref() {
            return Ok(e.clone());
        }
        // Entries may refer to earlier entries (`B = A + 1` or `E::A + 1`),
        // so publish the partial list while evaluating and expose the names
        // computed so far as constants.
        let mut entries: Vec<(String, i128, i128)> = Vec::new();
        let mut last: i128 = -1;
        self.push_scope(self.scope().kind);
        self.scope_mut().namespace = decl.namespace.clone();
        let mut result = Ok(());
        for e in &def.entries {
            *decl.enum_cache.borrow_mut() = Some(Rc::new(entries.clone()));
            let v = match &e.value {
                Some(expr) => match self.eval(expr).and_then(|v| v.as_i128()) {
                    Ok(v) => v,
                    Err(err) => {
                        result = Err(err);
                        break;
                    }
                },
                None => last + 1,
            };
            let end = match &e.end {
                Some(expr) => match self.eval(expr).and_then(|v| v.as_i128()) {
                    Ok(v) => v,
                    Err(err) => {
                        result = Err(err);
                        break;
                    }
                },
                None => v,
            };
            entries.push((e.name.clone(), v, end));
            self.scope_mut().vars.insert(e.name.clone(), Slot { value: Value::Signed(v), ty: None, constant: true });
            last = end;
        }
        self.pop_scope();
        if let Err(err) = result {
            *decl.enum_cache.borrow_mut() = None;
            return Err(err);
        }
        let rc = Rc::new(entries);
        *decl.enum_cache.borrow_mut() = Some(rc.clone());
        Ok(rc)
    }

    fn create_enum(&mut self, decl: &Rc<TypeDecl>, def: Rc<EnumDef>, name: &str, endian: Endian, section: usize) -> Result<PatternRef> {
        let underlying = self.instantiate(&def.ty)?;
        let Some(b) = underlying.as_builtin() else {
            // A user type (typically a variable-length integer with a
            // `[[transform]]`): evaluate it and freeze its value.
            let entries = self.enum_entries(decl, &def)?;
            let inner = self.create_pattern(&underlying, name, endian, section)?;
            let value = self.value_of(&inner)?;
            let (offset, size) = {
                let ib = inner.borrow();
                (ib.offset, ib.size)
            };
            let signed = matches!(value, Value::Signed(_));
            let mut p = Pattern::new(name, &decl.full_name, offset, size, section, endian, PatternKind::Enum { signed, entries });
            p.value_override = Some(match value {
                Value::Pattern(_) => return Err(Error::new(format!("enum `{}` underlying type has no integer value", decl.full_name))),
                v => v,
            });
            return Ok(p.shared());
        };
        let size = b.size().ok_or_else(|| Error::new("invalid enum underlying type"))?;
        let entries = self.enum_entries(decl, &def)?;
        let offset = self.cursor;
        self.check_bounds(section, offset, size)?;
        let mut p = Pattern::new(
            name,
            &decl.full_name,
            offset,
            size,
            section,
            endian,
            PatternKind::Enum { signed: b.is_signed(), entries },
        );
        p.builtin = Some(b);
        self.cursor = offset + size;
        Ok(p.shared())
    }

    fn push_type_scope(&mut self, decl: &Rc<TypeDecl>, inst: &Instance, kind: ScopeKind, this: PatternRef) {
        let parent = self.this();
        let array_index = self.scope().array_index;
        let mut scope = Scope::new(kind, decl.namespace.clone());
        scope.this = Some(this);
        scope.parent = parent;
        scope.array_index = array_index;
        for (name, b) in &inst.bindings {
            match b {
                Binding::Type(t) => {
                    scope.template_types.insert(name.clone(), t.clone());
                }
                Binding::Value(v) => {
                    scope.vars.insert(name.clone(), Slot { value: v.clone(), ty: None, constant: true });
                }
            }
        }
        self.scopes.push(scope);
    }

    fn create_struct(
        &mut self,
        decl: &Rc<TypeDecl>,
        def: Rc<StructDef>,
        inst: &Instance,
        name: &str,
        endian: Endian,
        section: usize,
        is_union: bool,
    ) -> Result<PatternRef> {
        let start = self.cursor;
        let kind = if is_union { PatternKind::Union { members: Vec::new() } } else { PatternKind::Struct { members: Vec::new() } };
        let pattern = Pattern::new(name, &decl.full_name, start, 0, section, endian, kind).shared();
        let scope_kind = if is_union { ScopeKind::Union } else { ScopeKind::Struct };
        self.push_type_scope(decl, inst, scope_kind, pattern.clone());
        self.scope_mut().union_start = start;
        self.scope_mut().union_end = start;
        let saved_endian = std::mem::replace(&mut self.default_endian, endian);
        let r = self.exec_struct_body(&def);
        self.default_endian = saved_endian;
        let flow = match r {
            Ok(f) => f,
            Err(e) => {
                self.pop_scope();
                return Err(e);
            }
        };
        if is_union {
            let end = self.scope().union_end;
            self.cursor = end;
        }
        let size = self.cursor.saturating_sub(start);
        pattern.borrow_mut().size = size;
        // Type-level attributes evaluate with `this` bound to the pattern.
        let mut attrs = def.attrs.clone();
        attrs.extend(inst.alias_attrs.iter().cloned());
        let r = self.apply_attributes(&pattern, &attrs);
        self.pop_scope();
        r?;
        // `break` / `continue` inside a struct propagate to an enclosing array.
        match flow {
            Flow::Break => self.pending_break = true,
            Flow::Continue => self.pending_continue = true,
            _ => {}
        }
        Ok(pattern)
    }

    fn exec_struct_body(&mut self, def: &StructDef) -> Result<Flow> {
        for parent in &def.parents {
            let pinst = self.instantiate(parent)?;
            let Some(pdecl) = pinst.decl().cloned() else { continue };
            let pdef = match &pdecl.kind {
                TypeKind::Struct(d) | TypeKind::Union(d) => d.clone(),
                _ => return Err(Error::new(format!("cannot inherit from `{}`", pdecl.full_name))),
            };
            // Bind the parent's template arguments alongside our own.
            for (name, b) in &pinst.bindings {
                match b {
                    Binding::Type(t) => {
                        self.scope_mut().template_types.insert(name.clone(), t.clone());
                    }
                    Binding::Value(v) => {
                        self.scope_mut().vars.insert(name.clone(), Slot { value: v.clone(), ty: None, constant: true });
                    }
                }
            }
            let saved_ns = std::mem::replace(&mut self.scope_mut().namespace, pdecl.namespace.clone());
            let flow = self.exec_struct_body(&pdef)?;
            self.scope_mut().namespace = saved_ns;
            if !matches!(flow, Flow::Normal) {
                return Ok(flow);
            }
        }
        self.exec_stmts(&def.body)
    }

    fn create_bitfield(
        &mut self,
        decl: &Rc<TypeDecl>,
        def: Rc<StructDef>,
        inst: &Instance,
        name: &str,
        endian: Endian,
        section: usize,
    ) -> Result<PatternRef> {
        // Nested bitfield: continue in the enclosing bit stream.
        let nested = self.scope().kind == ScopeKind::Bitfield;
        let parent_state = self.scope().bitfield.clone();
        let start = self.cursor;
        let pattern = Pattern::new(
            name,
            &decl.full_name,
            start,
            0,
            section,
            endian,
            PatternKind::Bitfield { fields: Vec::new(), bit_size: 0 },
        )
        .shared();
        let mut order = self.bitfield_order;
        let mut total_bits = None;
        for a in def.attrs.iter().chain(inst.alias_attrs.iter()) {
            if a.is("bitfield_order") && a.args.len() == 2 {
                let o = self.eval(&a.args[0])?.as_u64()?;
                order = if o == 0 { BitfieldOrder::MostToLeast } else { BitfieldOrder::LeastToMost };
                total_bits = Some(self.eval(&a.args[1])?.as_u64()?);
            }
        }
        let state = match (&parent_state, nested) {
            (Some(ps), true) => {
                pattern.borrow_mut().offset = ps.borrow().base_offset + ps.borrow().bits / 8;
                ps.clone()
            }
            _ => Rc::new(RefCell::new(BitfieldState {
                base_offset: start,
                bits: 0,
                order,
                total_bits,
                endian,
                section,
            })),
        };
        let start_bits = state.borrow().bits;
        self.push_type_scope(decl, inst, ScopeKind::Bitfield, pattern.clone());
        self.scope_mut().bitfield = Some(state.clone());
        let saved_endian = std::mem::replace(&mut self.default_endian, endian);
        let r = self.exec_struct_body(&def);
        self.default_endian = saved_endian;
        if let Err(e) = r {
            self.pop_scope();
            return Err(e);
        }
        let mut attrs = def.attrs.clone();
        attrs.extend(inst.alias_attrs.iter().cloned());
        let r = self.apply_attributes(&pattern, &attrs);
        self.pop_scope();
        r?;
        let used_bits = state.borrow().bits - start_bits;
        if nested {
            let mut pb = pattern.borrow_mut();
            let base = state.borrow().base_offset;
            let first_bit = start_bits;
            pb.offset = base + first_bit / 8;
            pb.size = ((first_bit + used_bits).div_ceil(8)).saturating_sub(first_bit / 8);
            if let PatternKind::Bitfield { bit_size, .. } = &mut pb.kind {
                *bit_size = used_bits;
            }
        } else {
            let total = state.borrow().total_bits.unwrap_or(used_bits);
            let size = total.div_ceil(8);
            self.check_bounds(section, start, size)?;
            let mut pb = pattern.borrow_mut();
            pb.size = size;
            if let PatternKind::Bitfield { bit_size, .. } = &mut pb.kind {
                *bit_size = total;
            }
            drop(pb);
            self.cursor = start + size;
        }
        Ok(pattern)
    }

    fn exec_bitfield_field(&mut self, field: &BitfieldField) -> Result<()> {
        let Some(state) = self.scope().bitfield.clone() else {
            return Err(Error::new("bit-sized field outside of a bitfield"));
        };
        let bits = self.eval(&field.size)?.as_u64()?;
        let (base_offset, running, order, total, endian, section) = {
            let s = state.borrow();
            (s.base_offset, s.bits, s.order, s.total_bits, s.endian, s.section)
        };
        let bit_offset = match order {
            BitfieldOrder::LeastToMost => running,
            BitfieldOrder::MostToLeast => {
                let total = total.ok_or_else(|| Error::new("MostToLeastSignificant bitfields need a fixed size"))?;
                if running + bits > total {
                    return Err(Error::new("bitfield overflows its declared size"));
                }
                total - running - bits
            }
        };
        let typed = match &field.ty {
            Some(t) => {
                let inst = self.instantiate(t)?;
                Some(match &inst.base {
                    InstanceBase::Builtin(BuiltinType::Bool) => Box::new(PatternKind::Bool),
                    InstanceBase::Builtin(b) if b.is_signed() => Box::new(PatternKind::Signed),
                    InstanceBase::Builtin(_) => Box::new(PatternKind::Unsigned),
                    InstanceBase::Decl(d) => match &d.kind {
                        TypeKind::Enum(e) => {
                            let entries = self.enum_entries(d, e)?;
                            let u = self.instantiate(&e.ty)?;
                            let signed = u.as_builtin().map(|b| b.is_signed()).unwrap_or(false);
                            Box::new(PatternKind::Enum { signed, entries })
                        }
                        _ => return Err(Error::new("only builtin and enum types can be bit-sized fields")),
                    },
                })
            }
            None => None,
        };
        let first_byte = base_offset + bit_offset / 8;
        let last_byte = base_offset + (bit_offset + bits.max(1) - 1) / 8;
        let type_name = match &field.ty {
            Some(t) => format!("{}", t),
            None if field.signed => "signed".to_string(),
            None => "unsigned".to_string(),
        };
        let mut p = Pattern::new(
            &field.name,
            &type_name,
            first_byte,
            last_byte - first_byte + 1,
            section,
            endian,
            PatternKind::BitfieldField { base_offset, bit_offset, bit_size: bits, signed: field.signed, typed },
        );
        if field.name == "$padding$" {
            p.hidden = true;
        }
        let p = p.shared();
        if !field.attrs.iter().any(|a| a.is("no_unique_address")) {
            state.borrow_mut().bits = running + bits;
        }
        self.apply_attributes(&p, &field.attrs)?;
        if let Some(this) = self.this() {
            this.borrow_mut().push_member(p);
        }
        Ok(())
    }

    /// Creates an array pattern at the cursor.
    pub fn create_array(&mut self, inst: &Instance, name: &str, size: &ArraySize, endian: Endian, section: usize) -> Result<PatternRef> {
        self.count_pattern()?;
        let endian = inst.endian.unwrap_or(endian);
        let start = self.cursor;
        let elem_size = self.static_size(inst)?;
        let is_char = matches!(inst.as_builtin(), Some(BuiltinType::Char));
        let is_char16 = matches!(inst.as_builtin(), Some(BuiltinType::Char16));

        if let Some(es) = elem_size {
            // Scalar element type: a static array (or a string).
            let mut count: u64;
            let mut trim_nul = false;
            match size {
                ArraySize::Fixed(e) => {
                    count = self.eval(e)?.as_u64()?;
                }
                ArraySize::Unsized => {
                    let reader = self.reader(section);
                    let zero = reader.find_zero(start, es);
                    match zero {
                        Some(pos) => count = (pos - start) / es + 1,
                        None => return Err(Error::new(format!("unterminated array `{}` at 0x{:X}", name, start))),
                    }
                    trim_nul = true;
                }
                ArraySize::While(cond) => {
                    count = 0;
                    loop {
                        if !self.eval(cond)?.as_bool()? {
                            break;
                        }
                        if count >= self.limits.max_array_entries {
                            return Err(Error::new("array entry limit exceeded"));
                        }
                        self.check_bounds(section, self.cursor, es)?;
                        self.cursor += es;
                        count += 1;
                    }
                    self.cursor = start;
                }
            }
            let total = count.checked_mul(es).ok_or_else(|| Error::new("array size overflow"))?;
            self.check_bounds(section, start, total)?;
            let kind = if is_char {
                PatternKind::String
            } else if is_char16 {
                PatternKind::WideString
            } else {
                let saved = self.cursor;
                let template = self.create_pattern(inst, "", endian, section)?;
                self.cursor = saved;
                PatternKind::StaticArray { template, count }
            };
            let type_name = format!("{}[{}]", inst.display, count);
            let mut p = Pattern::new(name, &type_name, start, total, section, endian, kind);
            if trim_nul {
                p.attributes.push(("$null_terminated$".to_string(), Vec::new()));
            }
            self.cursor = start + total;
            let _ = &mut count;
            return Ok(p.shared());
        }

        // Non-scalar elements: materialise each entry.
        let pattern = Pattern::new(name, &format!("{}[]", inst.display), start, 0, section, endian, PatternKind::Array { entries: Vec::new() }).shared();
        let mut index = 0u64;
        let mut stop = false;
        let limit = self.limits.max_array_entries;
        let mut count_limit: Option<u64> = None;
        match size {
            ArraySize::Fixed(e) => count_limit = Some(self.eval(e)?.as_u64()?),
            ArraySize::Unsized | ArraySize::While(_) => {}
        }
        loop {
            if let Some(n) = count_limit {
                if index >= n {
                    break;
                }
            } else if let ArraySize::While(cond) = size {
                if !self.eval(cond)?.as_bool()? {
                    break;
                }
            }
            if index >= limit {
                return Err(Error::new("array entry limit exceeded"));
            }
            self.push_scope(self.scope().kind);
            self.scope_mut().array_index = Some(index);
            self.pending_break = false;
            self.pending_continue = false;
            let r = self.create_pattern(inst, &format!("[{}]", index), endian, section);
            let broke = self.pending_break;
            let continued = self.pending_continue;
            self.pending_break = false;
            self.pending_continue = false;
            self.pop_scope();
            let entry = r?;
            if continued {
                // Like the reference runtime: `continue` discards the entry
                // and moves on (the bytes stay consumed).
                index += 1;
                continue;
            }
            entry.borrow_mut().array_index = Some(index);
            if matches!(size, ArraySize::Unsized) {
                // An unsized array ends with an entry whose bytes are all zero.
                let (off, sz, sec) = {
                    let e = entry.borrow();
                    (e.offset, e.size, e.section)
                };
                let zero = self.section_data(sec).get(off as usize..(off + sz) as usize).map(|b| b.iter().all(|x| *x == 0)).unwrap_or(true);
                if zero || self.cursor >= self.section_len(sec) {
                    stop = true;
                }
            }
            pattern.borrow_mut().push_member(entry);
            index += 1;
            if broke {
                stop = true;
            }
            if stop {
                break;
            }
        }
        let size_bytes = self.cursor.saturating_sub(start);
        {
            let mut pb = pattern.borrow_mut();
            pb.size = size_bytes;
            pb.type_name = format!("{}[{}]", inst.display, index);
        }
        Ok(pattern)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_pointer(
        &mut self,
        inst: &Instance,
        ptr_inst: &Instance,
        name: &str,
        endian: Endian,
        ptr_endian: Endian,
        section: usize,
        array: Option<&ArraySize>,
        base_fn: Option<&Expr>,
    ) -> Result<PatternRef> {
        let b = ptr_inst
            .as_builtin()
            .filter(|b| b.is_unsigned() || b.is_signed())
            .ok_or_else(|| Error::new("pointer size type must be an integer"))?;
        let psize = b.size().unwrap();
        let offset = self.cursor;
        self.check_bounds(section, offset, psize)?;
        let raw = if b.is_signed() {
            self.reader(section).signed(offset, psize, ptr_endian)? as i128
        } else {
            self.reader(section).unsigned(offset, psize, ptr_endian)? as i128
        };
        let mut address = raw;
        if let Some(f) = base_fn {
            let fname = self.eval(f)?.as_str()?;
            let path: Vec<String> = fname.split("::").map(|s| s.to_string()).collect();
            let base = self.call_function(&path, vec![Value::Signed(raw)])?.as_i128()?;
            address = base;
        }
        self.cursor = offset + psize;
        let after = self.cursor;
        if address < 0 {
            return Err(Error::new(format!("pointer `{}` resolves to a negative address", name)));
        }
        self.cursor = address as u64;
        let r = match array {
            Some(sz) => self.create_array(inst, name, sz, endian, section),
            None => self.create_pattern(inst, name, endian, section),
        };
        self.cursor = after;
        let pointee = r?;
        let mut p = Pattern::new(
            name,
            &format!("{}*", inst.display),
            offset,
            psize,
            section,
            ptr_endian,
            PatternKind::Pointer { pointee, address: address as u128 },
        );
        p.builtin = Some(b);
        Ok(p.shared())
    }

    // ------------------------------------------------------------------
    // Attributes
    // ------------------------------------------------------------------

    fn apply_attributes(&mut self, pattern: &PatternRef, attrs: &[Attribute]) -> Result<()> {
        if attrs.is_empty() {
            return Ok(());
        }
        // Top-level attributes such as `[[hex::visualize("image", this)]]`
        // refer to the pattern itself.
        let pushed = self.this().is_none();
        if pushed {
            self.push_scope(self.scope().kind);
            self.scope_mut().this = Some(pattern.clone());
        }
        let r = self.apply_attributes_inner(pattern, attrs);
        if pushed {
            self.pop_scope();
        }
        r
    }

    fn apply_attributes_inner(&mut self, pattern: &PatternRef, attrs: &[Attribute]) -> Result<()> {
        for a in attrs {
            let name = a.name.join("::");
            let mut args = Vec::new();
            match name.as_str() {
                "name" | "comment" | "color" | "format" | "format_read" | "format_write" | "transform" | "transform_entries"
                | "pointer_base" | "bitfield_order" | "fixed_size" | "single_color" | "format_entries" | "sealed" | "hidden"
                | "inline" | "no_unique_address" | "export" | "static" | "highlight_hidden" | "transform_read" => {
                    for e in &a.args {
                        args.push(self.eval(e)?);
                    }
                }
                _ => {
                    for e in &a.args {
                        args.push(self.eval(e).unwrap_or(Value::Null));
                    }
                }
            }
            let mut pb = pattern.borrow_mut();
            match name.as_str() {
                "name" => pb.display_name = args.first().map(|v| v.to_display_string()),
                "comment" => pb.comment = args.first().map(|v| v.to_display_string()),
                "hidden" => pb.hidden = true,
                "sealed" => pb.sealed = true,
                "inline" => pb.inline = true,
                "color" | "single_color" => {
                    if let Some(v) = args.first() {
                        pb.color = u32::from_str_radix(&v.to_display_string(), 16).ok();
                    }
                }
                "format" | "format_read" => pb.format_fn = args.first().map(|v| v.to_display_string()),
                "transform" | "transform_read" => pb.transform_fn = args.first().map(|v| v.to_display_string()),
                "fixed_size" => {
                    if let Some(v) = args.first() {
                        let n = v.as_u64()?;
                        if pb.size < n {
                            let extra = n - pb.size;
                            pb.size = n;
                            drop(pb);
                            self.cursor += extra;
                            pattern.borrow_mut().attributes.push((name, args));
                            continue;
                        }
                    }
                }
                _ => {}
            }
            pb.attributes.push((name, args));
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Values of patterns
    // ------------------------------------------------------------------

    /// The value a pattern denotes in an expression, honouring
    /// `[[transform("fn")]]` (which turns e.g. a LEB128 struct into an
    /// integer). Struct-like patterns without a transform evaluate to
    /// themselves.
    pub fn value_of(&mut self, p: &PatternRef) -> Result<Value> {
        // Suspend the transform while it runs: the function's own parameter
        // coercion reads the pattern again and must see the raw value.
        let transform = p.borrow_mut().transform_fn.take();
        if let Some(name) = transform {
            let path: Vec<String> = name.split("::").map(|s| s.to_string()).collect();
            let r = self.call_function(&path, vec![Value::Pattern(p.clone())]);
            p.borrow_mut().transform_fn = Some(name);
            return match r? {
                Value::Pattern(inner) if !Rc::ptr_eq(&inner, p) => self.value_of(&inner),
                other => Ok(other),
            };
        }
        self.pattern_value(p)
    }

    /// The raw value a pattern denotes when used in an expression.
    pub fn pattern_value(&self, p: &PatternRef) -> Result<Value> {
        let pb = p.borrow();
        if let Some(v) = &pb.value_override {
            return Ok(v.clone());
        }
        let reader = self.reader(pb.section);
        Ok(match &pb.kind {
            PatternKind::Unsigned => Value::Unsigned(reader.unsigned(pb.offset, pb.size, pb.endian)?),
            PatternKind::Signed => Value::Signed(reader.signed(pb.offset, pb.size, pb.endian)?),
            PatternKind::Float => Value::Float(if pb.size == 4 { reader.f32(pb.offset, pb.endian)? } else { reader.f64(pb.offset, pb.endian)? }),
            PatternKind::Bool => Value::Bool(reader.unsigned(pb.offset, 1, pb.endian)? != 0),
            PatternKind::Char => Value::Char(reader.unsigned(pb.offset, 1, pb.endian)? as u8),
            PatternKind::Char16 => Value::Char16(reader.unsigned(pb.offset, 2, pb.endian)? as u16),
            PatternKind::Enum { signed, .. } => {
                if *signed {
                    Value::Signed(reader.signed(pb.offset, pb.size, pb.endian)?)
                } else {
                    Value::Unsigned(reader.unsigned(pb.offset, pb.size, pb.endian)?)
                }
            }
            PatternKind::String => {
                let bytes = reader.bytes(pb.offset, pb.size)?;
                let bytes = if pb.has_attribute("$null_terminated$") {
                    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
                    &bytes[..end]
                } else {
                    bytes
                };
                Value::Str(crate::lower::bytes_to_string(bytes))
            }
            PatternKind::WideString => {
                let units = reader.unsigned_array(pb.offset, 2, pb.size / 2, pb.endian)?;
                let mut units: Vec<u16> = units.into_iter().map(|u| u as u16).collect();
                if pb.has_attribute("$null_terminated$") {
                    if let Some(end) = units.iter().position(|u| *u == 0) {
                        units.truncate(end);
                    }
                }
                Value::Str(String::from_utf16_lossy(&units))
            }
            PatternKind::BitfieldField { base_offset, bit_offset, bit_size, signed, typed } => {
                let raw = self.read_bits(pb.section, *base_offset, *bit_offset, *bit_size, pb.endian)?;
                match typed.as_deref() {
                    Some(PatternKind::Bool) => Value::Bool(raw != 0),
                    Some(PatternKind::Signed) => Value::Signed(crate::reader::sign_extend(raw, *bit_size)),
                    Some(PatternKind::Enum { signed: true, .. }) => Value::Signed(crate::reader::sign_extend(raw, *bit_size)),
                    _ if *signed => Value::Signed(crate::reader::sign_extend(raw, *bit_size)),
                    _ => Value::Unsigned(raw),
                }
            }
            PatternKind::Pointer { address, .. } => Value::Unsigned(*address),
            PatternKind::Padding => Value::Unsigned(0),
            PatternKind::Struct { .. }
            | PatternKind::Union { .. }
            | PatternKind::Array { .. }
            | PatternKind::StaticArray { .. }
            | PatternKind::Bitfield { .. } => Value::Pattern(p.clone()),
        })
    }

    /// Reads `bit_size` bits starting `bit_offset` bits into the integer
    /// formed by the bytes at `base_offset` (little-endian bit numbering
    /// for `Endian::Little`, most-significant-first for `Endian::Big`).
    fn read_bits(&self, section: usize, base_offset: u64, bit_offset: u64, bit_size: u64, endian: Endian) -> Result<u128> {
        let nbytes = ((bit_offset + bit_size).div_ceil(8)).max(1);
        let reader = self.reader(section);
        let bytes = reader.bytes(base_offset, nbytes)?;
        // Build the integer with the bitfield's byte order.
        let mut v: u128 = 0;
        match endian {
            Endian::Little => {
                for b in bytes.iter().rev() {
                    v = (v << 8) | *b as u128;
                }
                let mask = if bit_size >= 128 { u128::MAX } else { (1u128 << bit_size) - 1 };
                Ok((v >> bit_offset) & mask)
            }
            Endian::Big => {
                for b in bytes {
                    v = (v << 8) | *b as u128;
                }
                let total = nbytes * 8;
                let shift = total - bit_offset - bit_size;
                let mask = if bit_size >= 128 { u128::MAX } else { (1u128 << bit_size) - 1 };
                Ok((v >> shift) & mask)
            }
        }
    }

    /// Element `i` of an array pattern.
    pub fn array_element(&mut self, arr: &PatternRef, i: u64) -> Result<PatternRef> {
        let ab = arr.borrow();
        match &ab.kind {
            PatternKind::Array { entries } => entries
                .get(i as usize)
                .cloned()
                .ok_or_else(|| Error::new(format!("index {} out of bounds for array `{}` of {} entries", i, ab.name, entries.len()))),
            PatternKind::StaticArray { template, count } => {
                if i >= *count {
                    return Err(Error::new(format!("index {} out of bounds for array `{}` of {} entries", i, ab.name, count)));
                }
                let mut e = template.borrow().clone();
                e.offset = ab.offset + i * e.size;
                e.name = format!("[{}]", i);
                e.array_index = Some(i);
                e.section = ab.section;
                Ok(e.shared())
            }
            PatternKind::String | PatternKind::WideString => {
                let width = if matches!(ab.kind, PatternKind::String) { 1 } else { 2 };
                if i * width >= ab.size {
                    return Err(Error::new(format!("index {} out of bounds for `{}`", i, ab.name)));
                }
                let kind = if width == 1 { PatternKind::Char } else { PatternKind::Char16 };
                let mut e = Pattern::new(&format!("[{}]", i), if width == 1 { "char" } else { "char16" }, ab.offset + i * width, width, ab.section, ab.endian, kind);
                e.array_index = Some(i);
                Ok(e.shared())
            }
            _ => Err(Error::new(format!("`{}` is not an array", ab.name))),
        }
    }

    /// Writes `value` into the bytes of a scalar pattern (heap sections) or
    /// records it as an override (data section).
    pub fn write_pattern(&mut self, p: &PatternRef, value: Value) -> Result<()> {
        let (section, offset, size, endian, kind) = {
            let pb = p.borrow();
            (pb.section, pb.offset, pb.size, pb.endian, pb.kind.clone())
        };
        if section == 0 {
            p.borrow_mut().value_override = Some(value);
            return Ok(());
        }
        let bytes: Vec<u8> = match &kind {
            PatternKind::Float => {
                if size == 4 {
                    let bits = (value.as_f64()? as f32).to_bits() as u128;
                    int_bytes(bits, 4, endian)
                } else {
                    int_bytes(value.as_f64()?.to_bits() as u128, 8, endian)
                }
            }
            PatternKind::String => {
                let s = value.as_str()?;
                let mut b = s.into_bytes();
                b.resize(size as usize, 0);
                b
            }
            PatternKind::Struct { .. } | PatternKind::Union { .. } | PatternKind::Array { .. } | PatternKind::StaticArray { .. } | PatternKind::Bitfield { .. } => {
                let src = value.as_pattern()?;
                let sb = src.borrow();
                let data = self.reader(sb.section).bytes(sb.offset, sb.size.min(size))?.to_vec();
                let mut data = data;
                data.resize(size as usize, 0);
                data
            }
            _ => {
                let v = match &value {
                    Value::Float(f) => *f as i128 as u128,
                    other => other.as_i128()? as u128,
                };
                int_bytes(v, size, endian)
            }
        };
        let sec = &mut self.sections[section];
        let end = (offset + size) as usize;
        if end > sec.len() {
            return Err(Error::new("write out of bounds"));
        }
        sec[offset as usize..end].copy_from_slice(&bytes[..size as usize]);
        Ok(())
    }

    fn default_value(&mut self, inst: &Instance) -> Result<Value> {
        Ok(match &inst.base {
            InstanceBase::Builtin(b) => match b {
                BuiltinType::Str => Value::Str(String::new()),
                BuiltinType::Bool => Value::Bool(false),
                BuiltinType::Char => Value::Char(0),
                BuiltinType::Char16 => Value::Char16(0),
                BuiltinType::Float | BuiltinType::Double => Value::Float(0.0),
                BuiltinType::Auto => Value::Null,
                b if b.is_signed() => Value::Signed(0),
                _ => Value::Unsigned(0),
            },
            InstanceBase::Decl(d) => match &d.kind {
                TypeKind::Enum(_) => Value::Unsigned(0),
                _ => {
                    // Struct-typed local: allocate a zeroed heap section and
                    // evaluate the type into it.
                    let size = self.sizeof_type(inst)?;
                    let section = self.new_section(size);
                    let saved = (self.cursor, self.current_section);
                    self.cursor = 0;
                    self.current_section = section;
                    let endian = inst.endian.unwrap_or(self.default_endian);
                    let r = self.create_pattern(inst, "", endian, section);
                    self.cursor = saved.0;
                    self.current_section = saved.1;
                    Value::Pattern(r?)
                }
            },
        })
    }

    /// Converts a value to the representation of a declared type.
    pub fn coerce(&mut self, inst: &Instance, v: Value) -> Result<Value> {
        match &inst.base {
            InstanceBase::Builtin(b) => expr::cast_builtin(*b, v),
            InstanceBase::Decl(d) => match &d.kind {
                TypeKind::Enum(_) => Ok(v),
                _ => Ok(v),
            },
        }
    }

    // ------------------------------------------------------------------
    // Assignment
    // ------------------------------------------------------------------

    fn assign(&mut self, target: &LValue, value: Value) -> Result<()> {
        match target {
            LValue::Dollar => {
                self.cursor = value.as_u64()?;
                Ok(())
            }
            LValue::Ident(name) => {
                if let Some(slot) = self.lookup_slot_mut(name) {
                    if slot.constant {
                        return Err(Error::new(format!("cannot assign to constant `{}`", name)));
                    }
                    let ty = slot.ty.clone();
                    let v = match ty {
                        Some(t) => self.coerce(&t, value)?,
                        None => value,
                    };
                    let slot = self.lookup_slot_mut(name).unwrap();
                    slot.value = v.clone();
                    if self.out_vars.contains_key(name) {
                        self.out_vars.insert(name.clone(), v);
                    }
                    return Ok(());
                }
                if let Some(m) = self.lookup_member(name) {
                    return self.write_pattern(&m, value);
                }
                // Assigning to an undeclared name creates a global (ImHex
                // allows this for `out`-like usage in some patterns).
                self.scopes[0].vars.insert(name.clone(), Slot { value, ty: None, constant: false });
                Ok(())
            }
            LValue::Path(expr) => {
                if let Expr::Index { object, index } = expr {
                    if let Expr::Ident(name) = object.as_ref() {
                        if let Some(Value::List(items)) = self.lookup_slot(name).map(|s| s.value.clone()) {
                            let i = self.eval(index)?.as_u64()? as usize;
                            let mut items = items.borrow_mut();
                            if i >= items.len() {
                                return Err(Error::new("array index out of range"));
                            }
                            items[i] = value;
                            return Ok(());
                        }
                    }
                }
                let p = self.eval_pattern(expr)?;
                self.write_pattern(&p, value)
            }
        }
    }

    // ------------------------------------------------------------------
    // Functions
    // ------------------------------------------------------------------

    /// Calls a user-defined or builtin function by (possibly qualified) name.
    pub fn call_function(&mut self, path: &[String], args: Vec<Value>) -> Result<Value> {
        // Flatten parameter packs passed as arguments.
        let mut flat = Vec::new();
        for a in args {
            match a {
                Value::Pack(items) => flat.extend(items),
                other => flat.push(other),
            }
        }
        let args = flat;
        if let Some(f) = self.find_function(path) {
            return self.call_user_function(&f, args);
        }
        if let Some(b) = self.find_builtin(path) {
            return b(self, args);
        }
        Err(Error::new(format!("unknown function `{}`", path.join("::"))))
    }

    fn call_user_function(&mut self, f: &Rc<FunctionDef>, args: Vec<Value>) -> Result<Value> {
        if self.call_depth >= self.limits.max_recursion {
            return Err(Error::new(format!("recursion limit reached calling `{}`", f.name)));
        }
        let min_args = f.params.iter().filter(|p| p.default.is_none()).count();
        if args.len() < min_args || (f.pack.is_none() && args.len() > f.params.len()) {
            return Err(Error::new(format!(
                "function `{}` expects {} argument(s), {} given",
                f.name,
                f.params.len(),
                args.len()
            )));
        }
        // Resolve the defining namespace of the function for name lookup.
        let full = self
            .functions
            .iter()
            .find(|(_, v)| Rc::ptr_eq(v, f))
            .map(|(k, _)| k.clone())
            .unwrap_or_default();
        let ns: Vec<String> = {
            let mut parts: Vec<String> = full.split("::").map(|s| s.to_string()).collect();
            parts.pop();
            parts
        };
        let mut scope = Scope::new(ScopeKind::Function, ns);
        // Inside a function, `parent` denotes the pattern being evaluated at
        // the call site (the caller's `this`).
        scope.parent = self.this();
        self.scopes.push(scope);
        let bind = |rt: &mut Runtime, f: &Rc<FunctionDef>, args: Vec<Value>| -> Result<()> {
            let mut args = args.into_iter();
            for p in &f.params {
                let inst = rt.instantiate(&p.ty)?;
                let v = match args.next() {
                    Some(v) => {
                        if inst.reference || matches!(inst.as_builtin(), Some(BuiltinType::Auto)) {
                            v
                        } else {
                            match v {
                                Value::Pattern(ref pr) => {
                                    let sv = rt.value_of(pr)?;
                                    match sv {
                                        Value::Pattern(_) => sv,
                                        other => rt.coerce(&inst, other)?,
                                    }
                                }
                                other => rt.coerce(&inst, other)?,
                            }
                        }
                    }
                    None => match &p.default {
                        Some(d) => rt.eval(d)?,
                        None => Value::Null,
                    },
                };
                rt.scope_mut().vars.insert(p.name.clone(), Slot { value: v, ty: Some(inst), constant: false });
            }
            if let Some(pack) = &f.pack {
                let rest: Vec<Value> = args.collect();
                rt.scope_mut().vars.insert(pack.clone(), Slot { value: Value::Pack(rest), ty: None, constant: false });
            }
            Ok(())
        };
        if let Err(e) = bind(self, f, args) {
            self.pop_scope();
            return Err(e);
        }
        self.call_depth += 1;
        let r = self.exec_stmts(&f.body);
        self.call_depth -= 1;
        self.pop_scope();
        match r? {
            Flow::Return(v) => Ok(v),
            _ => Ok(Value::Null),
        }
    }

    /// Formats a pattern's value for display, honouring `[[format]]`.
    pub fn formatted_value(&mut self, p: &PatternRef) -> Result<String> {
        let fmt = p.borrow().format_fn.clone();
        if let Some(name) = fmt {
            let path: Vec<String> = name.split("::").map(|s| s.to_string()).collect();
            let v = self.call_function(&path, vec![Value::Pattern(p.clone())])?;
            return Ok(v.to_display_string());
        }
        dump::default_formatted_value(self, p)
    }
}

/// Encodes `v` as `size` bytes in the given byte order.
fn int_bytes(v: u128, size: u64, endian: Endian) -> Vec<u8> {
    let mut out = Vec::with_capacity(size as usize);
    for i in 0..size {
        out.push(((v >> (8 * i)) & 0xFF) as u8);
    }
    if endian == Endian::Big {
        out.reverse();
    }
    out
}

impl Runtime {
    // Bitfield order accessor for builtins.
    pub(crate) fn set_bitfield_order(&mut self, order: BitfieldOrder) {
        self.bitfield_order = order;
    }

    pub(crate) fn bitfield_order(&self) -> BitfieldOrder {
        self.bitfield_order
    }

    pub(crate) fn create_section_buffer(&mut self, size: u64) -> usize {
        self.new_section(size)
    }

    pub(crate) fn resize_section(&mut self, id: usize, size: u64) -> Result<()> {
        let s = self.sections.get_mut(id).ok_or_else(|| Error::new(format!("invalid section {}", id)))?;
        s.resize(size as usize, 0);
        Ok(())
    }

    pub(crate) fn section_mut(&mut self, id: usize) -> Result<&mut Vec<u8>> {
        self.sections.get_mut(id).ok_or_else(|| Error::new(format!("invalid section {}", id)))
    }

    pub(crate) fn is_dry_run(&self) -> bool {
        self.dry_run > 0
    }

    pub(crate) fn env_var(&self, name: &str) -> Option<Value> {
        self.env_vars.get(name).cloned()
    }

    pub(crate) fn bit_cursor(&self) -> u64 {
        self.scope().bitfield.as_ref().map(|s| s.borrow().bits).unwrap_or(0)
    }
}
