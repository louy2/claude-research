//! A static check that a pattern terminates on every input.
//!
//! The interpreter stops every evaluation at a limit (`interp::Limits`).
//! This pass asks the stronger question: which loops and recursions end
//! for a reason that can be read off the pattern? It builds a graph of
//! types and functions, and gives a verdict for every place where
//! evaluation can repeat:
//!
//! * a `while` or `for` statement,
//! * a `[while(...)]` or `[]` array,
//! * a cycle in the graph (a type or function that reaches itself).
//!
//! Three arguments are recognised:
//!
//! * **counter**: a variable moves by a constant step towards a bound the
//!   loop does not change;
//! * **cursor progress**: every round reads at least one byte and nothing
//!   moves the cursor back, so the distance to the end of the data shrinks
//!   (a read past the end is an error);
//! * **no cycle**: types that only reach other types need no argument, which
//!   is the rank argument of `interval-vest`.
//!
//! Where there is no static argument but the interpreter's measure checks
//! (`interp::Limits::check_measures`) decide the question for each input,
//! the verdict is "run-time check":
//!
//! * an array whose entries may be empty or step back: every entry that
//!   does not end the array must end past where it started;
//! * a cycle of types through a placement or a pointer, or one that may not
//!   read a byte between two instances: no type may be instantiated inside
//!   an instance of itself (same template arguments) at the same offset.
//!
//! Everything else is "not proven", and only the interpreter's limits end
//! it: loops without a recognisable counter, recursive functions, and
//! cycles that never put an instance at the cursor.
//!
//! The static arguments hold for evaluation with the measure checks on.
//! The checks keep the cursor inside the data (so progress is bounded) and
//! catch a counter that wraps around; without them `$ += 4` could walk past
//! the end without reading, and a `u8` counter could wrap below its bound.
//! Two things stay outside the measures and are left to the depth limit:
//! template arguments that take ever new values, and placements into
//! sections created along the way.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::ast::*;
use crate::error::{Error, Result};
use crate::interp::Runtime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Terminates; the string names the argument.
    Proven(String),
    /// No static argument; the interpreter's measure checks
    /// (`Limits::check_measures`) decide it for each input. The string says
    /// why there is no static argument.
    RuntimeCheck(String),
    NotProven(String),
}

/// One place where evaluation can repeat, with its verdict.
#[derive(Debug, Clone)]
pub struct Obligation {
    pub file: String,
    pub line: u32,
    /// For example "while loop in `Chunk`" or "cycle `Node` -> `Node`".
    pub what: String,
    pub verdict: Verdict,
}

#[derive(Debug, Default)]
pub struct Report {
    /// Obligations of everything reachable from the top-level statements.
    pub obligations: Vec<Obligation>,
    /// Names that could not be resolved and other limits of the analysis.
    pub notes: Vec<String>,
    pub reachable_types: usize,
    pub reachable_functions: usize,
}

impl Report {
    pub fn count(&self) -> (usize, usize, usize) {
        let mut c = (0, 0, 0);
        for o in &self.obligations {
            match o.verdict {
                Verdict::Proven(_) => c.0 += 1,
                Verdict::RuntimeCheck(_) => c.1 += 1,
                Verdict::NotProven(_) => c.2 += 1,
            }
        }
        c
    }

    /// True if every obligation is proven and nothing was left unresolved.
    pub fn all_proven(&self) -> bool {
        let (_, r, n) = self.count();
        r == 0 && n == 0 && self.notes.is_empty()
    }

    /// True if every obligation is proven or decided by a run-time measure
    /// check, and nothing was left unresolved: no evaluation of the pattern
    /// depends on the interpreter's limits to stop.
    pub fn guarded(&self) -> bool {
        self.count().2 == 0 && self.notes.is_empty()
    }
}

/// Analyses the pattern in `source`. `path` names it in the report and
/// gives the directory for relative includes.
pub fn analyze(source: &str, path: &Path, include_paths: &[PathBuf]) -> Result<Report> {
    let mut rt = Runtime::new(Vec::new());
    for p in include_paths {
        rt.add_include_path(p);
    }
    if let Some(dir) = path.parent() {
        rt.add_include_path(dir);
    }
    let mut an = An::new(rt);
    let prelude = crate::lower::parse_program(crate::interp::PRELUDE)?;
    let f = an.file("<prelude>".to_string());
    an.collect(&prelude.statements, f, &[], false)?;
    let program = an.rt.load(source, path.parent())?;
    let f = an.file(path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
    an.collect(&program.statements, f, &[], true)?;
    Ok(an.run())
}

// ---------------------------------------------------------------------------
// Declarations and the graph

enum TKind {
    Struct(Rc<StructDef>),
    Union(Rc<StructDef>),
    Bitfield(Rc<StructDef>),
    Enum(Rc<EnumDef>),
    Alias(TypeRef, Vec<Attribute>),
    Forward,
}

struct TypeDecl {
    full: String,
    ns: Vec<String>,
    file: usize,
    line: u32,
    params: Vec<TemplateParam>,
    kind: TKind,
}

struct FuncDecl {
    full: String,
    ns: Vec<String>,
    file: usize,
    def: Rc<FunctionDef>,
}

struct MainStmt {
    file: usize,
    ns: Vec<String>,
    stmt: Stmt,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EdgeKind {
    /// Instantiated at the cursor. `progress`: at least one byte of the
    /// source type has been read before.
    Inline { progress: bool },
    /// Instantiated at an address taken from a placement or a pointer.
    Jump,
    Call,
    /// Evaluated away from the cursor, in a section of its own: `sizeof`,
    /// a cast, a local variable.
    Scratch,
    /// A parent type or the target of an alias: resolved, and (for a
    /// parent) run in the body of the type that names it.
    Resolve,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CtxKind {
    Struct,
    Union,
    Bitfield,
    Func,
    Main,
    /// An alias or an enum: no body.
    Other,
}

#[derive(Clone)]
struct Ctx {
    node: usize,
    ns: Vec<String>,
    params: Vec<TemplateParam>,
    kind: CtxKind,
    file: usize,
}

impl Ctx {
    fn reads(&self) -> bool {
        matches!(self.kind, CtxKind::Struct | CtxKind::Union | CtxKind::Bitfield)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Only compute sizes.
    Min,
    /// Record edges and local effects.
    Edges,
    /// Judge loops; the graph and its closures are complete.
    Loops,
}

/// Bits consumed by a statement list, as lower bounds. `None`: no path.
#[derive(Clone, Copy, Default)]
struct Flow {
    /// Paths that reach the end.
    end: Option<u64>,
    /// Paths that leave through `continue` or `return`.
    exit: Option<u64>,
    /// Something in the list may move the cursor back.
    rewind: bool,
}

fn min_opt(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (x, None) | (None, x) => x,
    }
}

fn add_opt(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    Some(a?.saturating_add(b?))
}

/// The cursor after a member of `b` bits. Members of a union all start at
/// the same offset, so the union is as long as its longest member.
fn advance(cx: &Ctx, a: Option<u64>, b: Option<u64>) -> Option<u64> {
    if cx.kind == CtxKind::Union {
        Some(a?.max(b?))
    } else {
        add_opt(a, b)
    }
}

/// What a template type parameter is bound to at one instantiation.
#[derive(Clone, Copy)]
enum ArgTy {
    Type(usize),
    /// A builtin type of this many bits.
    Builtin(u64),
    /// An argument the analysis could not resolve.
    Unknown,
}

enum FuncRef {
    User(usize),
    Builtin(String),
    Unknown,
}

struct An {
    rt: Runtime,
    files: Vec<String>,
    imported: HashSet<String>,
    aliases: HashMap<String, Vec<String>>,
    types: Vec<TypeDecl>,
    type_ix: HashMap<String, usize>,
    funcs: Vec<FuncDecl>,
    func_ix: HashMap<String, usize>,
    main: Vec<MainStmt>,
    edges: Vec<Vec<(usize, EdgeKind)>>,
    /// How a template type parameter (type, index) is used in its own body.
    param_use: HashMap<(usize, usize), Vec<EdgeKind>>,
    /// Parameters passed on to another template.
    forwards: HashMap<(usize, usize), Vec<(usize, usize)>>,
    /// (template, parameter index, argument type) for every instantiation.
    arg_edges: Vec<(usize, usize, ArgTy)>,
    /// (user, template): the user computes a value argument of the template.
    computed_args: HashSet<(usize, usize)>,
    /// Every type a template type parameter is bound to, after `run`.
    param_args: HashMap<(usize, usize), Vec<ArgTy>>,
    rewind_local: Vec<bool>,
    wnl_local: Vec<bool>,
    rewind: Vec<bool>,
    wnl: Vec<bool>,
    min_memo: HashMap<usize, Flow>,
    min_stack: Vec<usize>,
    /// The lowest stack position whose placeholder the current computation used.
    min_low: usize,
    pure_memo: HashMap<usize, bool>,
    /// (node the note is about, text); `usize::MAX` for the whole pattern.
    notes: BTreeSet<(usize, String)>,
    obligations: Vec<Obligation>,
}

impl An {
    fn new(rt: Runtime) -> Self {
        An {
            rt,
            files: Vec::new(),
            imported: HashSet::new(),
            aliases: HashMap::new(),
            types: Vec::new(),
            type_ix: HashMap::new(),
            funcs: Vec::new(),
            func_ix: HashMap::new(),
            main: Vec::new(),
            edges: Vec::new(),
            param_use: HashMap::new(),
            forwards: HashMap::new(),
            arg_edges: Vec::new(),
            param_args: HashMap::new(),
            computed_args: HashSet::new(),
            rewind_local: Vec::new(),
            wnl_local: Vec::new(),
            rewind: Vec::new(),
            wnl: Vec::new(),
            min_memo: HashMap::new(),
            min_stack: Vec::new(),
            min_low: usize::MAX,
            pure_memo: HashMap::new(),
            notes: BTreeSet::new(),
            obligations: Vec::new(),
        }
    }

    fn file(&mut self, name: String) -> usize {
        self.files.push(name);
        self.files.len() - 1
    }

    // ---- collection ------------------------------------------------------

    fn collect(&mut self, stmts: &[Stmt], file: usize, ns: &[String], top: bool) -> Result<()> {
        for s in stmts {
            match s {
                Stmt::Namespace { name, body, .. } => {
                    let mut inner = ns.to_vec();
                    inner.extend(name.iter().cloned());
                    self.collect(body, file, &inner, top)?;
                }
                Stmt::Import { path, alias, .. } => self.import(path, alias.as_deref())?,
                Stmt::Struct(_)
                | Stmt::Union(_)
                | Stmt::Bitfield(_)
                | Stmt::Enum(_)
                | Stmt::Using { .. }
                | Stmt::UsingForward { .. }
                | Stmt::Function(_) => self.register(s, file, ns),
                other => {
                    if top {
                        self.main.push(MainStmt { file, ns: ns.to_vec(), stmt: other.clone() });
                    }
                }
            }
        }
        Ok(())
    }

    fn register(&mut self, s: &Stmt, file: usize, ns: &[String]) {
        let qualify = |name: &str| {
            let mut p = ns.to_vec();
            p.push(name.to_string());
            p.join("::")
        };
        let add_type = |an: &mut An, name: &str, line: u32, params: &[TemplateParam], kind: TKind| {
            let full = qualify(name);
            let decl = TypeDecl { full: full.clone(), ns: ns.to_vec(), file, line, params: params.to_vec(), kind };
            match an.type_ix.get(&full) {
                Some(&i) => {
                    // A definition replaces a forward declaration, not the reverse.
                    if !matches!(decl.kind, TKind::Forward) {
                        an.types[i] = decl;
                    }
                }
                None => {
                    an.type_ix.insert(full, an.types.len());
                    an.types.push(decl);
                }
            }
        };
        match s {
            Stmt::Struct(d) => add_type(self, &d.name, d.line, &d.template_params, TKind::Struct(Rc::new(d.clone()))),
            Stmt::Union(d) => add_type(self, &d.name, d.line, &d.template_params, TKind::Union(Rc::new(d.clone()))),
            Stmt::Bitfield(d) => add_type(self, &d.name, d.line, &d.template_params, TKind::Bitfield(Rc::new(d.clone()))),
            Stmt::Enum(d) => add_type(self, &d.name, d.line, &[], TKind::Enum(Rc::new(d.clone()))),
            Stmt::Using { line, name, template_params, ty, attrs } => {
                add_type(self, name, *line, template_params, TKind::Alias(ty.clone(), attrs.clone()))
            }
            Stmt::UsingForward { line, name, template_params } => add_type(self, name, *line, template_params, TKind::Forward),
            Stmt::Function(f) => {
                let full = qualify(&f.name);
                let decl = FuncDecl { full: full.clone(), ns: ns.to_vec(), file, def: Rc::new(f.clone()) };
                match self.func_ix.get(&full) {
                    Some(&i) => self.funcs[i] = decl,
                    None => {
                        self.func_ix.insert(full, self.funcs.len());
                        self.funcs.push(decl);
                    }
                }
            }
            _ => {}
        }
        // Types and functions declared inside a body.
        let body: &[Stmt] = match s {
            Stmt::Struct(d) | Stmt::Union(d) | Stmt::Bitfield(d) => &d.body,
            Stmt::Function(f) => &f.body,
            _ => &[],
        };
        let mut nested = Vec::new();
        for_each_stmt(body, &mut |st| {
            if matches!(
                st,
                Stmt::Struct(_) | Stmt::Union(_) | Stmt::Bitfield(_) | Stmt::Enum(_) | Stmt::Using { .. } | Stmt::Function(_)
            ) {
                nested.push(st.clone());
            }
        });
        for st in &nested {
            self.register(st, file, ns);
        }
    }

    fn import(&mut self, path: &str, alias: Option<&str>) -> Result<()> {
        let key = path.trim_end_matches(".pat").to_string();
        if let Some(a) = alias {
            self.aliases.insert(a.to_string(), key.split(['/', '\\']).map(|s| s.to_string()).collect());
        }
        let Some(file) = self.rt.resolve_import(&key) else {
            if !(key.starts_with("std/") || key == "type/magic") {
                self.notes.insert((usize::MAX, format!("import `{}` not found; what it declares is not analysed", path)));
            }
            return Ok(());
        };
        let canonical = file.canonicalize().unwrap_or(file.clone()).to_string_lossy().to_string();
        if !self.imported.insert(canonical) {
            return Ok(());
        }
        let source = std::fs::read_to_string(&file).map_err(|e| Error::new(format!("cannot read import {}: {}", file.display(), e)))?;
        let program = self.rt.load(&source, file.parent()).map_err(|e| Error::new(format!("in import `{}`: {}", path, e)))?;
        let f = self.file(format!("{}.pat", key));
        self.collect(&program.statements, f, &[], true)
    }

    // ---- names -----------------------------------------------------------

    fn candidates(&self, ns: &[String], path: &[String]) -> Vec<String> {
        let mut path = path.to_vec();
        if let Some(alias) = self.aliases.get(&path[0]) {
            let mut p = alias.clone();
            p.extend(path.drain(1..));
            path = p;
        }
        let joined = path.join("::");
        (0..=ns.len())
            .rev()
            .map(|i| if i == 0 { joined.clone() } else { format!("{}::{}", ns[..i].join("::"), joined) })
            .collect()
    }

    fn resolve_type(&self, ns: &[String], path: &[String]) -> Option<usize> {
        self.candidates(ns, path).iter().find_map(|c| self.type_ix.get(c).copied())
    }

    fn resolve_func(&self, ns: &[String], path: &[String]) -> FuncRef {
        let cands = self.candidates(ns, path);
        if let Some(i) = cands.iter().find_map(|c| self.func_ix.get(c).copied()) {
            return FuncRef::User(i);
        }
        match cands.iter().find(|c| self.rt.has_builtin(c)) {
            Some(c) => FuncRef::Builtin(c.trim_start_matches("builtin::").to_string()),
            None => FuncRef::Unknown,
        }
    }

    fn func_node(&self, f: usize) -> usize {
        self.types.len() + f
    }

    fn main_node(&self) -> usize {
        self.types.len() + self.funcs.len()
    }

    fn node_name(&self, n: usize) -> String {
        if n < self.types.len() {
            self.types[n].full.clone()
        } else if n < self.main_node() {
            format!("{}()", self.funcs[n - self.types.len()].full)
        } else {
            "the top level".to_string()
        }
    }

    fn ctx_of(&self, node: usize) -> Ctx {
        if node < self.types.len() {
            let t = &self.types[node];
            let kind = match t.kind {
                TKind::Struct(_) => CtxKind::Struct,
                TKind::Union(_) => CtxKind::Union,
                TKind::Bitfield(_) => CtxKind::Bitfield,
                _ => CtxKind::Other,
            };
            Ctx { node, ns: t.ns.clone(), params: t.params.clone(), kind, file: t.file }
        } else {
            let f = &self.funcs[node - self.types.len()];
            Ctx { node, ns: f.ns.clone(), params: Vec::new(), kind: CtxKind::Func, file: f.file }
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, kind: EdgeKind) {
        if !self.edges[from].contains(&(to, kind)) {
            self.edges[from].push((to, kind));
        }
    }

    // ---- sizes -----------------------------------------------------------

    /// Lower bounds, in bits, on what instantiating a type consumes.
    fn type_min(&mut self, t: usize) -> Flow {
        if let Some(f) = self.min_memo.get(&t) {
            return *f;
        }
        if let Some(pos) = self.min_stack.iter().position(|x| *x == t) {
            // A type inside itself: zero is a valid lower bound. Whatever is
            // computed from it holds only while `t` is being computed.
            self.min_low = self.min_low.min(pos);
            return Flow { end: Some(0), exit: None, rewind: false };
        }
        let pos = self.min_stack.len();
        self.min_stack.push(t);
        let outer_low = std::mem::replace(&mut self.min_low, usize::MAX);
        let cx = self.ctx_of(t);
        let flow = match &self.types[t].kind {
            TKind::Struct(d) | TKind::Bitfield(d) | TKind::Union(d) => {
                let d = d.clone();
                let mut f = self.walk_type_body(&cx, &d, Phase::Min);
                if cx.kind == CtxKind::Bitfield {
                    f.end = f.end.map(|b| b.div_ceil(8) * 8);
                    f.exit = f.exit.map(|b| b.div_ceil(8) * 8);
                }
                f
            }
            TKind::Enum(d) => {
                let ty = d.ty.clone();
                self.ref_min(&cx, &ty)
            }
            TKind::Alias(ty, _) => {
                let ty = ty.clone();
                self.ref_min(&cx, &ty)
            }
            TKind::Forward => Flow { end: Some(0), exit: None, rewind: false },
        };
        self.min_stack.pop();
        // Keep the result only if it did not use the placeholder of a type
        // further down the stack; otherwise it is recomputed when asked for
        // outside that type.
        if self.min_low >= pos {
            self.min_memo.insert(t, flow);
        }
        self.min_low = outer_low.min(if self.min_low >= pos { usize::MAX } else { self.min_low });
        flow
    }

    fn ref_min(&mut self, cx: &Ctx, ty: &TypeRef) -> Flow {
        match &ty.base {
            TypeBase::Builtin(b) => Flow { end: Some(b.size().unwrap_or(0) * 8), exit: None, rewind: false },
            TypeBase::Custom { name, .. } => {
                if name.len() == 1 && cx.params.iter().any(|p| p.name == name[0]) {
                    return Flow { end: Some(0), exit: None, rewind: false };
                }
                match self.resolve_type(&cx.ns, name) {
                    Some(t) => self.type_min(t),
                    None => Flow { end: Some(0), exit: None, rewind: false },
                }
            }
        }
    }

    /// Parents first, then the body, as the interpreter runs them.
    fn walk_type_body(&mut self, cx: &Ctx, d: &StructDef, ph: Phase) -> Flow {
        let mut reach = Some(0u64);
        let mut exit = None;
        let mut rewind = false;
        for parent in &d.parents {
            let target = self.type_ref(cx, parent, EdgeKind::Resolve, ph);
            let m = self.ref_min(cx, parent);
            exit = min_opt(exit, add_opt(reach, m.exit));
            reach = add_opt(reach, m.end);
            if ph == Phase::Loops {
                rewind |= target.map(|t| self.rewind[t]).unwrap_or(false);
            }
        }
        rewind |= self.attrs(cx, &d.attrs, ph);
        let f = self.walk(cx, &d.body, reach, ph);
        Flow { end: f.end, exit: min_opt(exit, f.exit), rewind: rewind || f.rewind }
    }

    // ---- the walker ------------------------------------------------------

    /// A reference to a type from `cx`. Returns the type's node.
    fn type_ref(&mut self, cx: &Ctx, ty: &TypeRef, kind: EdgeKind, ph: Phase) -> Option<usize> {
        let TypeBase::Custom { name, args } = &ty.base else { return None };
        if name.len() == 1 {
            if let Some(i) = cx.params.iter().position(|p| p.is_type && p.name == name[0]) {
                if ph == Phase::Edges {
                    self.param_use.entry((cx.node, i)).or_default().push(kind);
                }
                return None;
            }
        }
        let Some(t) = self.resolve_type(&cx.ns, name) else {
            if ph == Phase::Edges {
                self.notes.insert((cx.node, format!("unknown type `{}`", name.join("::"))));
            }
            return None;
        };
        if ph == Phase::Edges {
            self.add_edge(cx.node, t, kind);
        }
        if ph != Phase::Min {
            self.template_args(cx, t, args, ph);
        }
        Some(t)
    }

    fn template_args(&mut self, cx: &Ctx, t: usize, args: &[TemplateArg], ph: Phase) {
        for (j, arg) in args.iter().enumerate() {
            let is_type = self.types[t].params.get(j).map(|p| p.is_type).unwrap_or(false);
            if !is_type {
                if let TemplateArg::Expr(e) = arg {
                    self.expr(cx, e, ph);
                    let mut computed = false;
                    for_each_expr(e, &mut |x| {
                        computed |= matches!(x, Expr::Binary { .. } | Expr::Unary { .. } | Expr::Call { .. } | Expr::Ternary { .. } | Expr::Dollar)
                    });
                    if computed && ph == Phase::Edges {
                        self.computed_args.insert((cx.node, t));
                    }
                }
                continue;
            }
            let arg_ty = match arg {
                TemplateArg::Type(ty) => ty.clone(),
                TemplateArg::Expr(Expr::Ident(n)) => TypeRef::custom(n),
                TemplateArg::Expr(Expr::Scoped(p)) => {
                    TypeRef { reference: false, endian: None, base: TypeBase::Custom { name: p.clone(), args: Vec::new() } }
                }
                TemplateArg::Expr(_) => {
                    if ph == Phase::Edges {
                        self.arg_edges.push((t, j, ArgTy::Unknown));
                    }
                    continue;
                }
            };
            let (name, inner) = match &arg_ty.base {
                TypeBase::Custom { name, args } => (name, args),
                TypeBase::Builtin(b) => {
                    if ph == Phase::Edges {
                        self.arg_edges.push((t, j, ArgTy::Builtin(b.size().unwrap_or(0) * 8)));
                    }
                    continue;
                }
            };
            if name.len() == 1 {
                if let Some(k) = cx.params.iter().position(|p| p.is_type && p.name == name[0]) {
                    if ph == Phase::Edges {
                        self.forwards.entry((cx.node, k)).or_default().push((t, j));
                    }
                    continue;
                }
            }
            match self.resolve_type(&cx.ns, name) {
                Some(b) => {
                    if ph == Phase::Edges {
                        self.arg_edges.push((t, j, ArgTy::Type(b)));
                    }
                    self.template_args(cx, b, inner, ph);
                }
                None => {
                    if ph == Phase::Edges {
                        self.arg_edges.push((t, j, ArgTy::Unknown));
                    }
                }
            }
        }
    }

    /// Calls and type uses inside an expression. Returns true (in the
    /// `Loops` phase) if something called may move the cursor back.
    fn expr(&mut self, cx: &Ctx, e: &Expr, ph: Phase) -> bool {
        if ph == Phase::Min {
            return false;
        }
        let mut calls: Vec<(Vec<String>, Vec<Expr>)> = Vec::new();
        let mut type_uses: Vec<TypeRef> = Vec::new();
        for_each_expr(e, &mut |x| match x {
            Expr::Call { name, args, .. } => calls.push((name.clone(), args.clone())),
            Expr::SizeOf(TypeOrExpr::Type(t))
            | Expr::AddressOf(TypeOrExpr::Type(t))
            | Expr::TypeNameOf(TypeOrExpr::Type(t))
            | Expr::Cast { ty: t, .. }
            | Expr::Reinterpret { ty: t, .. } => type_uses.push(t.clone()),
            _ => {}
        });
        let mut rewind = false;
        for t in &type_uses {
            // `sizeof(x)` parses `x` as a type; a name that is not one is a
            // variable.
            if let TypeBase::Custom { name, .. } = &t.base {
                let param = name.len() == 1 && cx.params.iter().any(|p| p.name == name[0]);
                if !param && self.resolve_type(&cx.ns, name).is_none() {
                    continue;
                }
            }
            // Evaluated in a scratch section; the cursor is put back.
            self.type_ref(cx, t, EdgeKind::Scratch, ph);
        }
        for (name, args) in &calls {
            rewind |= self.call(cx, name, ph);
            // std::core::execute_function("name", ...) calls by name.
            if name.last().map(|s| s == "execute_function").unwrap_or(false) {
                match args.first() {
                    Some(Expr::Literal(Literal::String(s))) => {
                        let path: Vec<String> = s.split("::").map(|p| p.to_string()).collect();
                        rewind |= self.call(cx, &path, ph);
                    }
                    _ => {
                        // The library wrapper passes its own parameter on;
                        // its callers are followed above.
                        let wrapper = self.node_name(cx.node).ends_with("execute_function()");
                        if ph == Phase::Edges && !wrapper {
                            self.notes.insert((cx.node, "a function is called by a computed name; its body is not followed".to_string()));
                        }
                    }
                }
            }
        }
        rewind
    }

    fn call(&mut self, cx: &Ctx, name: &[String], ph: Phase) -> bool {
        match self.resolve_func(&cx.ns, name) {
            FuncRef::User(f) => {
                let n = self.func_node(f);
                if ph == Phase::Edges {
                    self.add_edge(cx.node, n, EdgeKind::Call);
                }
                ph == Phase::Loops && self.rewind[n]
            }
            FuncRef::Builtin(_) => false,
            FuncRef::Unknown => {
                // Natives this interpreter lacks are still leaves.
                if ph == Phase::Edges && name.first().map(|s| s != "builtin").unwrap_or(true) {
                    self.notes.insert((cx.node, format!("unknown function `{}`", name.join("::"))));
                }
                false
            }
        }
    }

    fn opt_expr(&mut self, cx: &Ctx, e: Option<&Expr>, ph: Phase) -> bool {
        e.map(|e| self.expr(cx, e, ph)).unwrap_or(false)
    }

    /// Attribute arguments; a string that names a function is a call
    /// (`format`, `transform`, `pointer_base`, ...).
    fn attrs(&mut self, cx: &Ctx, attrs: &[Attribute], ph: Phase) -> bool {
        if ph == Phase::Min {
            return false;
        }
        let mut rewind = false;
        for a in attrs {
            for arg in &a.args {
                if let Expr::Literal(Literal::String(s)) = arg {
                    let path: Vec<String> = s.split("::").map(|p| p.to_string()).collect();
                    if !path.iter().any(|p| p.is_empty()) {
                        if let FuncRef::User(_) = self.resolve_func(&cx.ns, &path) {
                            rewind |= self.call(cx, &path, ph);
                        }
                    }
                }
                rewind |= self.expr(cx, arg, ph);
            }
        }
        rewind
    }

    fn placement(&mut self, cx: &Ctx, p: Option<&Placement>, ph: Phase) -> bool {
        match p {
            Some(p) => self.expr(cx, &p.address, ph) | self.opt_expr(cx, p.section.as_ref(), ph),
            None => false,
        }
    }

    fn walk(&mut self, cx: &Ctx, stmts: &[Stmt], mut reach: Option<u64>, ph: Phase) -> Flow {
        let mut exit: Option<u64> = None;
        let mut rewind = false;
        let reads = cx.reads();
        for s in stmts {
            let progress = cx.kind == CtxKind::Struct && reach.map(|r| r >= 8).unwrap_or(false);
            match s {
                Stmt::Var(d) => {
                    let local = d.value.is_some() || d.in_var || d.out_var || !reads;
                    let kind = if d.placement.is_some() {
                        EdgeKind::Jump
                    } else if d.value.is_some() || cx.kind == CtxKind::Func {
                        EdgeKind::Scratch
                    } else {
                        EdgeKind::Inline { progress }
                    };
                    let target = self.type_ref(cx, &d.ty, kind, ph);
                    rewind |= self.placement(cx, d.placement.as_ref(), ph);
                    rewind |= self.opt_expr(cx, d.value.as_ref(), ph);
                    rewind |= self.attrs(cx, &d.attrs, ph);
                    if d.placement.is_none() {
                        if ph == Phase::Loops {
                            rewind |= target.map(|t| self.rewind[t]).unwrap_or(false);
                        }
                        if !local && !d.attrs.iter().any(|a| a.is("no_unique_address")) {
                            let m = self.ref_min(cx, &d.ty);
                            exit = min_opt(exit, advance(cx, reach, m.exit));
                            reach = advance(cx, reach, m.end);
                        }
                    }
                }
                Stmt::Array(d) => {
                    let local = d.init.is_some() || !reads;
                    let kind = if d.placement.is_some() {
                        EdgeKind::Jump
                    } else if d.init.is_some() || cx.kind == CtxKind::Func {
                        EdgeKind::Scratch
                    } else {
                        EdgeKind::Inline { progress }
                    };
                    let target = self.type_ref(cx, &d.ty, kind, ph);
                    rewind |= self.placement(cx, d.placement.as_ref(), ph);
                    rewind |= self.attrs(cx, &d.attrs, ph);
                    let cond_rewind = match &d.size {
                        ArraySize::Fixed(e) | ArraySize::While(e) => self.expr(cx, e, ph),
                        ArraySize::Unsized => false,
                    };
                    rewind |= cond_rewind;
                    for e in d.init.iter().flatten() {
                        rewind |= self.expr(cx, e, ph);
                    }
                    let m = self.ref_min(cx, &d.ty);
                    let entry = min_opt(m.end, m.exit);
                    let target_rewind = ph == Phase::Loops && target.map(|t| self.rewind[t]).unwrap_or(false);
                    if d.placement.is_none() {
                        rewind |= target_rewind;
                    }
                    if ph == Phase::Loops && !matches!(d.size, ArraySize::Fixed(_)) {
                        // The condition cannot move the array's cursor: the
                        // interpreter puts it back.
                        let verdict = self.array_verdict(cx, &d.ty, entry, target_rewind);
                        self.oblige(cx, d.line, format!("array `{}`", d.name), verdict);
                    }
                    if d.placement.is_none() && !local && !d.attrs.iter().any(|a| a.is("no_unique_address")) {
                        let count = match &d.size {
                            ArraySize::Fixed(e) => literal(e).unwrap_or(0),
                            ArraySize::Unsized => 1,
                            ArraySize::While(_) => 0,
                        };
                        // An entry that breaks ends the array; one that
                        // continues is dropped. Neither leaves this body.
                        reach = advance(cx, reach, Some(entry.unwrap_or(0).saturating_mul(count)));
                    }
                }
                Stmt::Pointer(d) => {
                    self.type_ref(cx, &d.ty, EdgeKind::Jump, ph);
                    rewind |= self.placement(cx, d.placement.as_ref(), ph);
                    rewind |= self.attrs(cx, &d.attrs, ph);
                    if let Some(ArraySize::Fixed(e) | ArraySize::While(e)) = &d.array {
                        rewind |= self.expr(cx, e, ph);
                    }
                    if d.placement.is_none() && reads {
                        let m = self.ref_min(cx, &d.pointer_ty);
                        reach = advance(cx, reach, m.end);
                    }
                }
                Stmt::Padding { line, size, attrs } => {
                    rewind |= self.attrs(cx, attrs, ph);
                    let mut bits = 0;
                    match size {
                        ArraySize::Fixed(e) => match literal(e) {
                            Some(n) => bits = n.saturating_mul(8),
                            None => rewind |= self.expr(cx, e, ph),
                        },
                        ArraySize::While(e) => {
                            let r = self.expr(cx, e, ph);
                            rewind |= r;
                            if ph == Phase::Loops {
                                let v = Verdict::Proven("each step skips one byte".to_string());
                                self.oblige(cx, *line, "padding".to_string(), v);
                            }
                        }
                        ArraySize::Unsized => {}
                    }
                    if reads {
                        reach = advance(cx, reach, Some(bits));
                    }
                }
                Stmt::BitfieldField(d) => {
                    if let Some(t) = &d.ty {
                        self.type_ref(cx, t, EdgeKind::Inline { progress: false }, ph);
                    }
                    rewind |= self.attrs(cx, &d.attrs, ph);
                    match literal(&d.size) {
                        Some(n) => reach = advance(cx, reach, Some(n)),
                        None => rewind |= self.expr(cx, &d.size, ph),
                    }
                }
                Stmt::Assign { target, value, .. } => {
                    rewind |= self.expr(cx, value, ph);
                    match target {
                        LValue::Dollar => match forward_step(value) {
                            Some(n) => {
                                if reads {
                                    reach = advance(cx, reach, Some(n.saturating_mul(8)));
                                }
                            }
                            None => {
                                rewind = true;
                                if ph == Phase::Edges {
                                    self.rewind_local[cx.node] = true;
                                }
                            }
                        },
                        LValue::Path(e) => rewind |= self.expr(cx, e, ph),
                        LValue::Ident(_) => {}
                    }
                }
                Stmt::Expr { expr, .. } => rewind |= self.expr(cx, expr, ph),
                Stmt::If { cond, then, otherwise, .. } => {
                    rewind |= self.expr(cx, cond, ph);
                    let a = self.walk(cx, then, reach, ph);
                    let b = self.walk(cx, otherwise, reach, ph);
                    reach = min_opt(a.end, b.end);
                    exit = min_opt(exit, min_opt(a.exit, b.exit));
                    rewind |= a.rewind | b.rewind;
                }
                Stmt::Match { values, cases, .. } => {
                    for v in values {
                        rewind |= self.expr(cx, v, ph);
                    }
                    let mut catch_all = false;
                    let mut end = None;
                    for c in cases {
                        catch_all |= c.patterns.iter().all(|p| matches!(p, CasePattern::Wildcard));
                        for p in &c.patterns {
                            if let CasePattern::Alternatives(alts) = p {
                                for (lo, hi) in alts {
                                    rewind |= self.expr(cx, lo, ph);
                                    rewind |= self.opt_expr(cx, hi.as_ref(), ph);
                                }
                            }
                        }
                        let f = self.walk(cx, &c.body, reach, ph);
                        end = min_opt(end, f.end);
                        exit = min_opt(exit, f.exit);
                        rewind |= f.rewind;
                    }
                    // Without a wildcard arm, no arm may match.
                    reach = if catch_all { end } else { min_opt(end, reach) };
                }
                Stmt::TryCatch { body, handler, .. } => {
                    // The body may fail anywhere; the handler then starts
                    // from the cursor the body started at.
                    let a = self.walk(cx, body, reach, ph);
                    let b = self.walk(cx, handler, reach, ph);
                    reach = min_opt(a.end, b.end);
                    exit = min_opt(exit, min_opt(a.exit, b.exit));
                    rewind |= a.rewind | b.rewind;
                }
                Stmt::While { line, cond, body } => {
                    let cond_rewind = self.expr(cx, cond, ph);
                    let f = self.walk(cx, body, reach, ph);
                    if ph == Phase::Loops {
                        let v = self.loop_verdict(cx, cond, None, body, reach, &f, cond_rewind);
                        self.oblige(cx, *line, "while loop".to_string(), v);
                    }
                    // The loop may run zero times.
                    exit = min_opt(exit, f.exit);
                    rewind |= f.rewind | cond_rewind;
                }
                Stmt::For { line, init, cond, update, body } => {
                    let i = self.walk(cx, std::slice::from_ref(init), reach, ph);
                    let cond_rewind = self.expr(cx, cond, ph);
                    let f = self.walk(cx, body, reach, ph);
                    let u = self.walk(cx, std::slice::from_ref(update), reach, ph);
                    if ph == Phase::Loops {
                        let v = self.loop_verdict(cx, cond, Some(update), body, reach, &f, cond_rewind | u.rewind);
                        self.oblige(cx, *line, "for loop".to_string(), v);
                    }
                    exit = min_opt(exit, f.exit);
                    rewind |= i.rewind | f.rewind | u.rewind | cond_rewind;
                }
                Stmt::Return { value, .. } => {
                    rewind |= self.opt_expr(cx, value.as_ref(), ph);
                    exit = min_opt(exit, reach);
                    reach = None;
                }
                Stmt::Continue { .. } => {
                    exit = min_opt(exit, reach);
                    reach = None;
                }
                Stmt::Break { .. } => reach = None,
                Stmt::Import { .. }
                | Stmt::Using { .. }
                | Stmt::UsingForward { .. }
                | Stmt::Namespace { .. }
                | Stmt::Struct(_)
                | Stmt::Union(_)
                | Stmt::Enum(_)
                | Stmt::Bitfield(_)
                | Stmt::Function(_) => {}
            }
        }
        Flow { end: reach, exit, rewind }
    }

    fn walk_node(&mut self, node: usize, ph: Phase) {
        if node == self.main_node() {
            let main: Vec<(usize, Vec<String>, Stmt)> = self.main.iter().map(|m| (m.file, m.ns.clone(), m.stmt.clone())).collect();
            for (file, ns, stmt) in main {
                let cx = Ctx { node, ns, params: Vec::new(), kind: CtxKind::Main, file };
                self.walk(&cx, std::slice::from_ref(&stmt), Some(0), ph);
            }
            return;
        }
        let cx = self.ctx_of(node);
        if node >= self.types.len() {
            let def = self.funcs[node - self.types.len()].def.clone();
            for p in &def.params {
                self.opt_expr(&cx, p.default.as_ref(), ph);
            }
            self.walk(&cx, &def.body, Some(0), ph);
            return;
        }
        match &self.types[node].kind {
            TKind::Struct(d) | TKind::Bitfield(d) | TKind::Union(d) => {
                let d = d.clone();
                self.walk_type_body(&cx, &d, ph);
            }
            TKind::Enum(d) => {
                let d = d.clone();
                self.type_ref(&cx, &d.ty, EdgeKind::Inline { progress: false }, ph);
                self.attrs(&cx, &d.attrs, ph);
                for e in &d.entries {
                    self.opt_expr(&cx, e.value.as_ref(), ph);
                    self.opt_expr(&cx, e.end.as_ref(), ph);
                }
            }
            TKind::Alias(ty, attrs) => {
                let (ty, attrs) = (ty.clone(), attrs.clone());
                self.type_ref(&cx, &ty, EdgeKind::Resolve, ph);
                self.attrs(&cx, &attrs, ph);
            }
            TKind::Forward => {}
        }
    }

    // ---- verdicts --------------------------------------------------------

    fn oblige(&mut self, cx: &Ctx, line: u32, what: String, verdict: Verdict) {
        let what = format!("{} in {}", what, self.node_name(cx.node));
        self.obligations.push(Obligation { file: self.files[cx.file].clone(), line, what, verdict });
    }

    fn array_verdict(&mut self, cx: &Ctx, ty: &TypeRef, entry: Option<u64>, rewind: bool) -> Verdict {
        if let TypeBase::Custom { name, .. } = &ty.base {
            if let Some(i) = cx.params.iter().position(|p| name.len() == 1 && p.name == name[0]) {
                // Judge the array for every type the parameter is bound to.
                let args = self.param_args.get(&(cx.node, i)).cloned().unwrap_or_default();
                if args.is_empty() || rewind {
                    return Verdict::RuntimeCheck("the entry type is a template parameter, so its size is unknown".to_string());
                }
                let mut least = u64::MAX;
                for a in args {
                    let (bits, name) = match a {
                        ArgTy::Builtin(bits) => (Some(bits), "a builtin type".to_string()),
                        ArgTy::Type(b) if self.rewind[b] => (Some(0), self.node_name(b)),
                        ArgTy::Type(b) => {
                            let m = self.type_min(b);
                            (min_opt(m.end, m.exit), self.node_name(b))
                        }
                        ArgTy::Unknown => (Some(0), "a type that was not resolved".to_string()),
                    };
                    match bits {
                        Some(bits) if bits < 8 => {
                            return Verdict::RuntimeCheck(format!(
                                "the entry type is a template parameter; with {} an entry can be empty or move the cursor back",
                                name
                            ));
                        }
                        Some(bits) => least = least.min(bits),
                        None => {}
                    }
                }
                return Verdict::Proven(if least == u64::MAX {
                    "every entry ends the array, for every type the template is used with".to_string()
                } else {
                    format!("each entry reads at least {} byte(s), for every type the template is used with", least / 8)
                });
            }
        }
        if rewind {
            return Verdict::RuntimeCheck("an entry may move the cursor back".to_string());
        }
        match entry {
            None => Verdict::Proven("every entry ends the array".to_string()),
            Some(bits) if bits >= 8 => Verdict::Proven(format!("each entry reads at least {} byte(s)", bits / 8)),
            Some(_) => Verdict::RuntimeCheck("an entry can be empty".to_string()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn loop_verdict(&mut self, cx: &Ctx, cond: &Expr, update: Option<&Stmt>, body: &[Stmt], reach: Option<u64>, f: &Flow, cond_rewind: bool) -> Verdict {
        let why = match self.counter_argument(cx, cond, update, body) {
            Ok(arg) => return Verdict::Proven(arg),
            Err(why) => why,
        };
        if cx.kind == CtxKind::Struct {
            if let Some(start) = reach {
                let step = min_opt(f.end, f.exit).map(|m| m.saturating_sub(start));
                if !f.rewind && !cond_rewind {
                    match step {
                        None => return Verdict::Proven("the body always leaves the loop".to_string()),
                        Some(bits) if bits >= 8 => {
                            return Verdict::Proven(format!("each round reads at least {} byte(s)", bits / 8));
                        }
                        Some(_) => {}
                    }
                }
                let read = if f.rewind || cond_rewind { "the cursor may move back" } else { "a round can read nothing" };
                return Verdict::NotProven(format!("{}; {}", why, read));
            }
        }
        Verdict::NotProven(why)
    }

    /// Nodes referenced directly by a statement list (types used, functions
    /// called), with the kind of each reference.
    fn subtree_refs(&mut self, cx: &Ctx, stmts: &[Stmt], extra: &[&Expr]) -> Vec<(usize, EdgeKind)> {
        let saved: Vec<Vec<(usize, EdgeKind)>> = std::mem::take(&mut self.edges);
        let saved_notes = self.notes.clone();
        let saved_uses = (self.param_use.clone(), self.forwards.clone(), self.arg_edges.len(), self.rewind_local.clone());
        self.edges = vec![Vec::new(); saved.len()];
        // Re-run the edge recorder on the fragment alone.
        self.walk(cx, stmts, Some(0), Phase::Edges);
        for e in extra {
            self.expr(cx, e, Phase::Edges);
        }
        let out = std::mem::take(&mut self.edges[cx.node]);
        self.edges = saved;
        self.notes = saved_notes;
        self.param_use = saved_uses.0;
        self.forwards = saved_uses.1;
        self.arg_edges.truncate(saved_uses.2);
        self.rewind_local = saved_uses.3;
        out
    }

    /// The counter argument: a variable (or `$`) moves by a constant step
    /// towards a bound that the loop does not change.
    fn counter_argument(&mut self, cx: &Ctx, cond: &Expr, update: Option<&Stmt>, body: &[Stmt]) -> std::result::Result<String, String> {
        let candidates = counter_candidates(update, body);
        let mut assigned: Vec<LValue> = Vec::new();
        for_each_stmt(body, &mut |s| {
            if let Stmt::Assign { target, .. } = s {
                assigned.push(target.clone());
            }
        });
        if candidates.is_empty() {
            return Err("no counter with a constant step".to_string());
        }
        // What the loop can touch.
        let mut all: Vec<Stmt> = body.to_vec();
        all.extend(update.cloned());
        let refs = self.subtree_refs(cx, &all, &[cond]);
        if let Some((n, _)) = refs.iter().find(|(n, _)| self.wnl[*n]) {
            return Err(format!("the loop uses {}, which may assign variables it does not own", self.node_name(*n)));
        }
        let path_assign = assigned.iter().any(|a| matches!(a, LValue::Path(_)));
        let mut assigned_names: HashSet<String> = HashSet::new();
        for a in &assigned {
            if let LValue::Ident(n) = a {
                assigned_names.insert(n.clone());
            }
        }
        if let Some(Stmt::Assign { target: LValue::Ident(n), .. }) = update {
            assigned_names.insert(n.clone());
        }
        let mut why = "the condition does not bound the counter".to_string();
        for (counter, up, step) in &candidates {
            let counter_expr = match counter {
                LValue::Ident(n) => Expr::Ident(n.clone()),
                LValue::Dollar => Expr::Dollar,
                LValue::Path(_) => continue,
            };
            if *counter == LValue::Dollar {
                // `$` also moves when something is read; that only helps a
                // counter that goes up.
                if !*up {
                    continue;
                }
                if refs.iter().any(|(n, k)| !matches!(k, EdgeKind::Jump | EdgeKind::Scratch) && self.rewind[*n]) {
                    why = "something in the loop may move the cursor back".to_string();
                    continue;
                }
            }
            for bound in counter_bounds(cond, counter, *up) {
                match self.invariant(cx, bound, &counter_expr, &assigned_names, path_assign) {
                    Ok(()) => {
                        let name = match counter {
                            LValue::Ident(n) => format!("`{}`", n),
                            _ => "the cursor".to_string(),
                        };
                        return Ok(format!(
                            "{} moves {} by {} towards a bound the loop does not change",
                            name,
                            if *up { "up" } else { "down" },
                            step
                        ));
                    }
                    Err(e) => why = e,
                }
            }
        }
        Err(why)
    }

    /// Is `bound` the same in every round?
    fn invariant(&mut self, cx: &Ctx, bound: &Expr, counter: &Expr, assigned: &HashSet<String>, path_assign: bool) -> std::result::Result<(), String> {
        let mut problem: Option<String> = None;
        let mut calls: Vec<Vec<String>> = Vec::new();
        for_each_expr(bound, &mut |x| {
            if problem.is_some() {
                return;
            }
            match x {
                _ if x == counter => problem = Some("the bound depends on the counter".to_string()),
                Expr::Dollar => problem = Some("the bound depends on the cursor".to_string()),
                Expr::Ident(n) if assigned.contains(n) => problem = Some(format!("the loop assigns `{}`, which the bound uses", n)),
                Expr::Member { .. } | Expr::Index { .. } | Expr::Parent | Expr::This if path_assign => {
                    problem = Some("the loop assigns to a member, and the bound reads one".to_string())
                }
                Expr::Call { name, .. } => calls.push(name.clone()),
                _ => {}
            }
        });
        if let Some(p) = problem {
            return Err(p);
        }
        for name in calls {
            let ok = match self.resolve_func(&cx.ns, &name) {
                FuncRef::User(f) => self.arg_pure(f),
                FuncRef::Builtin(b) => pure_builtin(&b),
                FuncRef::Unknown => false,
            };
            if !ok {
                return Err(format!("the bound calls `{}`, whose result may change", name.join("::")));
            }
        }
        Ok(())
    }

    /// A function whose result depends only on its arguments.
    fn arg_pure(&mut self, f: usize) -> bool {
        if let Some(p) = self.pure_memo.get(&f) {
            return *p;
        }
        // Assume purity while checking, so that recursion does not count
        // against a function.
        self.pure_memo.insert(f, true);
        let def = self.funcs[f].def.clone();
        let ns = self.funcs[f].ns.clone();
        let node = self.func_node(f);
        let mut pure = !self.wnl_local[node] && !self.rewind_local[node];
        let mut locals: HashSet<String> = def.params.iter().map(|p| p.name.clone()).collect();
        locals.extend(def.pack.iter().cloned());
        let mut calls: Vec<Vec<String>> = Vec::new();
        let mut reads: Vec<String> = Vec::new();
        for_each_stmt(&def.body, &mut |s| {
            match s {
                Stmt::Var(d) => {
                    locals.insert(d.name.clone());
                    pure &= d.placement.is_none() && matches!(d.ty.base, TypeBase::Builtin(_));
                }
                Stmt::Array(d) => {
                    locals.insert(d.name.clone());
                    pure &= d.placement.is_none() && matches!(d.ty.base, TypeBase::Builtin(_));
                }
                Stmt::Pointer(_) | Stmt::Padding { .. } => pure = false,
                _ => {}
            }
            for e in stmt_exprs(s) {
                for_each_expr(e, &mut |x| match x {
                    Expr::Dollar | Expr::Parent | Expr::This => pure = false,
                    Expr::Ident(n) => reads.push(n.clone()),
                    Expr::Call { name, .. } => calls.push(name.clone()),
                    _ => {}
                });
            }
        });
        pure &= reads.iter().all(|n| locals.contains(n));
        for name in calls {
            if !pure {
                break;
            }
            pure &= match self.resolve_func(&ns, &name) {
                FuncRef::User(g) => self.arg_pure(g),
                FuncRef::Builtin(b) => pure_builtin(&b),
                FuncRef::Unknown => false,
            };
        }
        self.pure_memo.insert(f, pure);
        pure
    }

    // ---- the whole analysis ----------------------------------------------

    fn run(mut self) -> Report {
        let n = self.main_node() + 1;
        self.edges = vec![Vec::new(); n];
        self.rewind_local = vec![false; n];
        self.wnl_local = vec![false; n];
        for node in 0..n {
            self.walk_node(node, Phase::Edges);
        }
        // Template arguments: the argument type is instantiated where the
        // template uses its parameter.
        let arg_edges = std::mem::take(&mut self.arg_edges);
        for (t, j, b) in arg_edges {
            let mut seen = HashSet::new();
            let mut stack = vec![(t, j)];
            while let Some(site) = stack.pop() {
                if !seen.insert(site) {
                    continue;
                }
                self.param_args.entry(site).or_default().push(b);
                if let ArgTy::Type(b) = b {
                    for kind in self.param_use.get(&site).cloned().unwrap_or_default() {
                        self.add_edge(site.0, b, kind);
                    }
                }
                stack.extend(self.forwards.get(&site).cloned().unwrap_or_default());
            }
        }
        // Assignments to variables a body does not own.
        for node in 0..n - 1 {
            self.wnl_local[node] = self.writes_foreign(node);
        }
        // Closures: `rewind` follows what runs at the same cursor,
        // `wnl` follows everything.
        self.rewind = self.rewind_local.clone();
        self.wnl = self.wnl_local.clone();
        loop {
            let mut changed = false;
            for a in 0..n {
                for &(b, kind) in &self.edges[a] {
                    if !matches!(kind, EdgeKind::Jump | EdgeKind::Scratch) && self.rewind[b] && !self.rewind[a] {
                        self.rewind[a] = true;
                        changed = true;
                    }
                    if self.wnl[b] && !self.wnl[a] {
                        self.wnl[a] = true;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        // Reachability from the top level.
        let mut reachable = vec![false; n];
        let mut stack = vec![self.main_node()];
        while let Some(a) = stack.pop() {
            if std::mem::replace(&mut reachable[a], true) {
                continue;
            }
            stack.extend(self.edges[a].iter().map(|e| e.0));
        }
        for (node, _) in reachable.iter().enumerate().filter(|(_, r)| **r) {
            self.walk_node(node, Phase::Loops);
        }
        self.cycles(&reachable);
        let mut obligations = std::mem::take(&mut self.obligations);
        obligations.sort_by(|a, b| (a.file.as_str(), a.line, a.what.as_str()).cmp(&(b.file.as_str(), b.line, b.what.as_str())));
        Report {
            obligations,
            notes: {
                let texts: BTreeSet<String> =
                    self.notes.iter().filter(|(n, _)| *n == usize::MAX || reachable[*n]).map(|(_, t)| t.clone()).collect();
                texts.into_iter().collect()
            },
            reachable_types: (0..self.types.len()).filter(|i| reachable[*i]).count(),
            reachable_functions: (self.types.len()..n - 1).filter(|i| reachable[*i]).count(),
        }
    }

    /// Does the body of `node` assign to a variable it does not declare,
    /// to a `ref` parameter, or to a member of a pattern?
    fn writes_foreign(&self, node: usize) -> bool {
        let (body, mut owned): (Vec<Stmt>, HashSet<String>) = if node < self.types.len() {
            match &self.types[node].kind {
                TKind::Struct(d) | TKind::Union(d) | TKind::Bitfield(d) => (d.body.clone(), HashSet::new()),
                _ => return false,
            }
        } else {
            let def = &self.funcs[node - self.types.len()].def;
            (def.body.clone(), def.params.iter().filter(|p| !p.ty.reference).map(|p| p.name.clone()).collect())
        };
        let mut targets = Vec::new();
        for_each_stmt(&body, &mut |s| match s {
            Stmt::Var(d) => {
                owned.insert(d.name.clone());
            }
            Stmt::Array(d) => {
                owned.insert(d.name.clone());
            }
            Stmt::Pointer(d) => {
                owned.insert(d.name.clone());
            }
            Stmt::Assign { target, .. } => targets.push(target.clone()),
            _ => {}
        });
        targets.iter().any(|t| match t {
            LValue::Ident(n) => !owned.contains(n),
            LValue::Path(_) => true,
            LValue::Dollar => false,
        })
    }

    /// One obligation per cycle class (strongly connected component).
    fn cycles(&mut self, reachable: &[bool]) {
        let n = reachable.len();
        let comps = sccs(n, &self.edges);
        let mut comp_of = vec![0usize; n];
        for (i, c) in comps.iter().enumerate() {
            for &v in c {
                comp_of[v] = i;
            }
        }
        for (ci, comp) in comps.iter().enumerate() {
            let cyclic = comp.len() > 1 || self.edges[comp[0]].iter().any(|e| e.0 == comp[0]);
            if !cyclic || !reachable[comp[0]] {
                continue;
            }
            let mut members: Vec<usize> = comp.clone();
            members.sort();
            let internal: Vec<(usize, usize, EdgeKind)> = members
                .iter()
                .flat_map(|&a| self.edges[a].iter().filter(|e| comp_of[e.0] == ci).map(move |e| (a, e.0, e.1)))
                .collect();
            let names: Vec<String> = members.iter().take(4).map(|m| format!("`{}`", self.node_name(*m))).collect();
            let more = if members.len() > 4 { format!(" and {} more", members.len() - 4) } else { String::new() };
            let what = format!("cycle through {}{}", names.join(", "), more);
            let has_func = members.iter().any(|m| *m >= self.types.len());
            let all_func = members.iter().all(|m| *m >= self.types.len());
            // The run-time rule for a cycle of types: no type is
            // instantiated inside an instance of itself (same template
            // arguments) at the same offset. It decides every cycle whose
            // rounds each put an instance at the cursor or at a placement.
            let resolve: Vec<(usize, usize)> = internal.iter().filter(|e| e.2 == EdgeKind::Resolve).map(|e| (e.0, e.1)).collect();
            let verdict = if all_func {
                Verdict::NotProven("recursive function; no decreasing argument was looked for".to_string())
            } else if has_func {
                Verdict::NotProven("types and functions that reach each other; only the depth limit ends this".to_string())
            } else if has_cycle(&members, &resolve) {
                Verdict::NotProven("a type inherits from itself or is an alias of itself".to_string())
            } else if internal.iter().any(|e| e.2 == EdgeKind::Scratch) {
                Verdict::NotProven(
                    "the cycle evaluates a type away from the cursor (`sizeof`, a cast, a local); only the depth limit ends this".to_string(),
                )
            } else if internal.iter().any(|e| self.computed_args.contains(&(e.0, e.1))) {
                Verdict::NotProven(
                    "a template argument is computed anew at each level, so no instance repeats; only the depth limit ends this".to_string(),
                )
            } else {
                // Every cycle of inline edges needs one edge with progress.
                let stuck: Vec<(usize, usize)> = internal
                    .iter()
                    .filter(|e| matches!(e.2, EdgeKind::Inline { progress: false } | EdgeKind::Resolve))
                    .map(|e| (e.0, e.1))
                    .collect();
                let jump = internal.iter().any(|e| e.2 == EdgeKind::Jump);
                if has_cycle(&members, &stuck) {
                    Verdict::RuntimeCheck("a type can reach itself without reading a byte first".to_string())
                } else if members.iter().any(|m| self.rewind[*m]) {
                    Verdict::RuntimeCheck("something in the cycle may move the cursor back".to_string())
                } else if jump {
                    Verdict::RuntimeCheck("the cycle goes through a placement or a pointer, whose target comes from the data".to_string())
                } else {
                    Verdict::Proven("at least one byte is read between a type and its next instance".to_string())
                }
            };
            let first = members[0];
            let (file, line) = if first < self.types.len() {
                (self.types[first].file, self.types[first].line)
            } else {
                let f = &self.funcs[first - self.types.len()];
                (f.file, f.def.line)
            };
            self.obligations.push(Obligation { file: self.files[file].clone(), line, what, verdict });
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers on the syntax tree

/// A non-negative integer literal (plain integers are lowered as signed).
fn literal(e: &Expr) -> Option<u64> {
    match e {
        Expr::Literal(Literal::Unsigned(n)) => u64::try_from(*n).ok(),
        Expr::Literal(Literal::Signed(n)) => u64::try_from(*n).ok(),
        _ => None,
    }
}

/// The counters of a loop: a variable or `$` that every completed round
/// moves by a constant step. Returns (counter, goes up, step).
fn counter_candidates(update: Option<&Stmt>, body: &[Stmt]) -> Vec<(LValue, bool, u128)> {
    let mut candidates = Vec::new();
    let mut assigned: Vec<LValue> = Vec::new();
    for_each_stmt(body, &mut |s| {
        if let Stmt::Assign { target, .. } = s {
            assigned.push(target.clone());
        }
    });
    let assigned_in_body = |t: &LValue| assigned.iter().filter(|a| *a == t).count();
    // A `for` loop: the update statement, if the body leaves the counter alone.
    if let Some(Stmt::Assign { target, value, .. }) = update {
        if let Some((up, step)) = step_of(target, value) {
            if assigned_in_body(target) == 0 {
                candidates.push((target.clone(), up, step));
            }
        }
    }
    // A `while` loop: a step at the top level of the body that is the only
    // assignment to the counter and that no `continue` can skip.
    if update.is_none() && !has_continue(body) {
        for s in body {
            if let Stmt::Assign { target, value, .. } = s {
                if let Some((up, step)) = step_of(target, value) {
                    if assigned_in_body(target) == 1 {
                        candidates.push((target.clone(), up, step));
                    }
                }
            }
        }
    }
    candidates
}

/// The bounds of `counter` in `cond`: the other side of every conjunct that
/// compares the counter in the direction it moves.
fn counter_bounds<'a>(cond: &'a Expr, counter: &LValue, up: bool) -> Vec<&'a Expr> {
    let counter_expr = match counter {
        LValue::Ident(n) => Expr::Ident(n.clone()),
        LValue::Dollar => Expr::Dollar,
        LValue::Path(_) => return Vec::new(),
    };
    let mut conjuncts = Vec::new();
    split_and(cond, &mut conjuncts);
    let mut out = Vec::new();
    for c in conjuncts {
        let Expr::Binary { op, left, right } = c else { continue };
        let (op, bound) = if **left == counter_expr {
            (*op, &**right)
        } else if **right == counter_expr {
            (mirror(*op), &**left)
        } else {
            continue;
        };
        let towards = if up { matches!(op, BinaryOp::Lt | BinaryOp::Le) } else { matches!(op, BinaryOp::Gt | BinaryOp::Ge) };
        if towards {
            out.push(bound);
        }
    }
    out
}

/// The counters of a loop that its condition bounds, with their direction
/// (true: up). The static argument relies on each of them moving in every
/// round; the interpreter checks at run time that they do, which catches a
/// counter that wraps around.
pub(crate) fn loop_counters(cond: &Expr, update: Option<&Stmt>, body: &[Stmt]) -> Vec<(LValue, bool)> {
    counter_candidates(update, body)
        .into_iter()
        .filter(|(c, up, _)| !counter_bounds(cond, c, *up).is_empty())
        .map(|(c, up, _)| (c, up))
        .collect()
}

/// `$ + n` or `n + $` with a literal `n`: the cursor moves forward.
fn forward_step(value: &Expr) -> Option<u64> {
    match value {
        Expr::Binary { op: BinaryOp::Add, left, right } => match (&**left, &**right) {
            (Expr::Dollar, n) | (n, Expr::Dollar) => literal(n),
            _ => None,
        },
        _ => None,
    }
}

/// `x = x + n` or `x = x - n` with a literal `n > 0`: (goes up, n).
fn step_of(target: &LValue, value: &Expr) -> Option<(bool, u128)> {
    let me = match target {
        LValue::Ident(n) => Expr::Ident(n.clone()),
        LValue::Dollar => Expr::Dollar,
        LValue::Path(_) => return None,
    };
    let Expr::Binary { op, left, right } = value else { return None };
    let lit = |e: &Expr| literal(e).filter(|n| *n > 0).map(|n| n as u128);
    match op {
        BinaryOp::Add if **left == me => lit(right).map(|n| (true, n)),
        BinaryOp::Add if **right == me => lit(left).map(|n| (true, n)),
        BinaryOp::Sub if **left == me => lit(right).map(|n| (false, n)),
        _ => None,
    }
}

fn mirror(op: BinaryOp) -> BinaryOp {
    match op {
        BinaryOp::Lt => BinaryOp::Gt,
        BinaryOp::Gt => BinaryOp::Lt,
        BinaryOp::Le => BinaryOp::Ge,
        BinaryOp::Ge => BinaryOp::Le,
        other => other,
    }
}

fn split_and<'a>(e: &'a Expr, out: &mut Vec<&'a Expr>) {
    match e {
        Expr::Binary { op: BinaryOp::And, left, right } => {
            split_and(left, out);
            split_and(right, out);
        }
        other => out.push(other),
    }
}

/// Native functions whose result depends only on their arguments and the
/// (fixed) data.
fn pure_builtin(name: &str) -> bool {
    name.starts_with("std::string::")
        || name.starts_with("std::math::")
        || matches!(name, "std::mem::size" | "std::mem::base_address" | "std::core::member_count" | "std::sizeof_pack")
}

/// A `continue` that belongs to this loop (not to a nested one).
fn has_continue(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::Continue { .. } => true,
        Stmt::If { then, otherwise, .. } => has_continue(then) || has_continue(otherwise),
        Stmt::Match { cases, .. } => cases.iter().any(|c| has_continue(&c.body)),
        Stmt::TryCatch { body, handler, .. } => has_continue(body) || has_continue(handler),
        _ => false,
    })
}

/// Every statement in the list and in nested blocks (not in nested
/// declarations).
fn for_each_stmt(stmts: &[Stmt], f: &mut dyn FnMut(&Stmt)) {
    for s in stmts {
        f(s);
        match s {
            Stmt::If { then, otherwise, .. } => {
                for_each_stmt(then, f);
                for_each_stmt(otherwise, f);
            }
            Stmt::Match { cases, .. } => {
                for c in cases {
                    for_each_stmt(&c.body, f);
                }
            }
            Stmt::TryCatch { body, handler, .. } => {
                for_each_stmt(body, f);
                for_each_stmt(handler, f);
            }
            Stmt::While { body, .. } => for_each_stmt(body, f),
            Stmt::For { init, update, body, .. } => {
                for_each_stmt(std::slice::from_ref(init), f);
                for_each_stmt(std::slice::from_ref(update), f);
                for_each_stmt(body, f);
            }
            _ => {}
        }
    }
}

/// The expressions a statement evaluates itself (not those of nested
/// statements).
fn stmt_exprs(s: &Stmt) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = Vec::new();
    fn placement<'a>(p: &'a Option<Placement>, out: &mut Vec<&'a Expr>) {
        if let Some(p) = p {
            out.push(&p.address);
            out.extend(p.section.iter());
        }
    }
    fn size<'a>(s: &'a ArraySize, out: &mut Vec<&'a Expr>) {
        if let ArraySize::Fixed(e) | ArraySize::While(e) = s {
            out.push(e);
        }
    }
    fn attrs<'a>(a: &'a [Attribute], out: &mut Vec<&'a Expr>) {
        out.extend(a.iter().flat_map(|a| a.args.iter()));
    }
    match s {
        Stmt::Var(d) => {
            placement(&d.placement, &mut out);
            out.extend(d.value.iter());
            attrs(&d.attrs, &mut out);
        }
        Stmt::Array(d) => {
            placement(&d.placement, &mut out);
            size(&d.size, &mut out);
            out.extend(d.init.iter().flatten());
            attrs(&d.attrs, &mut out);
        }
        Stmt::Pointer(d) => {
            placement(&d.placement, &mut out);
            if let Some(a) = &d.array {
                size(a, &mut out);
            }
            attrs(&d.attrs, &mut out);
        }
        Stmt::Padding { size: sz, attrs: a, .. } => {
            size(sz, &mut out);
            attrs(a, &mut out);
        }
        Stmt::BitfieldField(d) => {
            out.push(&d.size);
            attrs(&d.attrs, &mut out);
        }
        Stmt::Assign { target, value, .. } => {
            if let LValue::Path(e) = target {
                out.push(e);
            }
            out.push(value);
        }
        Stmt::Expr { expr, .. } => out.push(expr),
        Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::For { cond, .. } => out.push(cond),
        Stmt::Match { values, cases, .. } => {
            out.extend(values.iter());
            for c in cases {
                for p in &c.patterns {
                    if let CasePattern::Alternatives(alts) = p {
                        for (lo, hi) in alts {
                            out.push(lo);
                            out.extend(hi.iter());
                        }
                    }
                }
            }
        }
        Stmt::Return { value, .. } => out.extend(value.iter()),
        _ => {}
    }
    out
}

/// Every sub-expression, outermost first.
fn for_each_expr(e: &Expr, f: &mut dyn FnMut(&Expr)) {
    f(e);
    let toe = |t: &TypeOrExpr, f: &mut dyn FnMut(&Expr)| {
        if let TypeOrExpr::Expr(x) = t {
            for_each_expr(x, f);
        }
    };
    match e {
        Expr::Unary { operand, .. } => for_each_expr(operand, f),
        Expr::Binary { left, right, .. } => {
            for_each_expr(left, f);
            for_each_expr(right, f);
        }
        Expr::Ternary { cond, then, otherwise } => {
            for_each_expr(cond, f);
            for_each_expr(then, f);
            for_each_expr(otherwise, f);
        }
        Expr::Cast { value, .. } | Expr::Reinterpret { value, .. } => for_each_expr(value, f),
        Expr::Call { args, .. } => {
            for a in args {
                for_each_expr(a, f);
            }
        }
        Expr::Member { object, .. } => for_each_expr(object, f),
        Expr::Index { object, index } => {
            for_each_expr(object, f);
            for_each_expr(index, f);
        }
        Expr::SizeOf(t) | Expr::AddressOf(t) | Expr::TypeNameOf(t) => toe(t, f),
        Expr::Literal(_) | Expr::Ident(_) | Expr::Scoped(_) | Expr::Dollar | Expr::Null | Expr::Parent | Expr::This => {}
    }
}

/// Strongly connected components (Tarjan, iterative).
fn sccs(n: usize, edges: &[Vec<(usize, EdgeKind)>]) -> Vec<Vec<usize>> {
    const NONE: usize = usize::MAX;
    let mut index = vec![NONE; n];
    let mut low = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut next = 0usize;
    let mut out = Vec::new();
    for root in 0..n {
        if index[root] != NONE {
            continue;
        }
        let mut work: Vec<(usize, usize)> = vec![(root, 0)];
        while let Some(&mut (v, ref mut i)) = work.last_mut() {
            if *i == 0 {
                index[v] = next;
                low[v] = next;
                next += 1;
                stack.push(v);
                on_stack[v] = true;
            }
            if *i < edges[v].len() {
                let w = edges[v][*i].0;
                *i += 1;
                if index[w] == NONE {
                    work.push((w, 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                work.pop();
                if let Some(&(parent, _)) = work.last() {
                    low[parent] = low[parent].min(low[v]);
                }
                if low[v] == index[v] {
                    let mut comp = Vec::new();
                    loop {
                        let w = stack.pop().unwrap();
                        on_stack[w] = false;
                        comp.push(w);
                        if w == v {
                            break;
                        }
                    }
                    out.push(comp);
                }
            }
        }
    }
    out
}

/// Does the graph on `nodes` with `edges` contain a cycle?
fn has_cycle(nodes: &[usize], edges: &[(usize, usize)]) -> bool {
    let mut indegree: HashMap<usize, usize> = nodes.iter().map(|n| (*n, 0)).collect();
    for (_, b) in edges {
        *indegree.get_mut(b).unwrap() += 1;
    }
    let mut ready: Vec<usize> = nodes.iter().copied().filter(|n| indegree[n] == 0).collect();
    let mut removed = 0;
    while let Some(a) = ready.pop() {
        removed += 1;
        for (x, b) in edges {
            if *x == a {
                let d = indegree.get_mut(b).unwrap();
                *d -= 1;
                if *d == 0 {
                    ready.push(*b);
                }
            }
        }
    }
    removed != nodes.len()
}
