//! `hexpat lint`: checks for pattern authors.
//!
//! From the pattern alone:
//!
//! * `duplicate-member`: a name declared twice on one path through a type
//!   body, or once inside a loop. Both members exist, lookups take the
//!   last one, and an export has the key twice.
//! * `narrow-counter`: a loop counter of a small integer type whose
//!   condition does not keep it inside that type, so it wraps.
//! * `narrow-offset`, `narrowing`: an offset, a literal or a wider field
//!   stored in an 8- or 16-bit variable.
//! * `missing-case`: an `if`/`else if` chain or a `match` that tests one
//!   value against enum constants and has no branch for some of them.
//! * `termination`: what the static termination check could not prove.
//! * `unused`: types and functions the pattern declares and never uses.
//! * `unresolved`: names that do not resolve.
//!
//! With a data file ([`lint_data`]), after evaluating the pattern on it:
//!
//! * `coverage`: the bytes no pattern describes;
//! * `overlap`: bytes described by more than one pattern;
//! * `enum-value`: an enum field whose value is not one of its entries.
//!
//! Warnings are things that are wrong on some input; notes are things to
//! look at. The checks are syntactic and do not know which paths can
//! really happen.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::error::Result;
use crate::interp::{PatternKind, PatternRef, Runtime};
use crate::termination::{counter_candidates, for_each_expr, for_each_stmt, literal, min_opt, mirror, split_and, An, Phase, TKind, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Warning,
    Note,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub file: String,
    /// 0 when the finding is about the whole file.
    pub line: u32,
    pub level: Level,
    /// The name of the check, for example `duplicate-member`.
    pub check: &'static str,
    pub message: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = if self.level == Level::Warning { "warning" } else { "note" };
        if self.line == 0 {
            write!(f, "{}: {}[{}]: {}", self.file, level, self.check, self.message)
        } else {
            write!(f, "{}:{}: {}[{}]: {}", self.file, self.line, level, self.check, self.message)
        }
    }
}

/// Lints the pattern in `source`. `all_files` also reports findings in the
/// files it imports; by default only the pattern's own file is reported.
pub fn lint(source: &str, path: &Path, include_paths: &[PathBuf], all_files: bool) -> Result<Vec<Finding>> {
    let mut an = An::load(source, path, include_paths)?;
    let report = an.run();
    let mut l = Linter { an, all_files, findings: Vec::new(), globals: HashMap::new(), pending_cases: Vec::new() };
    let root = l.an.files[l.an.root_file].clone();
    for o in &report.obligations {
        if !all_files && o.file != root {
            continue;
        }
        let (level, why) = match &o.verdict {
            Verdict::Proven(_) => continue,
            Verdict::RuntimeCheck(w) => (Level::Note, format!("{}: {} (decided at run time)", o.what, w)),
            Verdict::NotProven(w) => (Level::Warning, format!("{}: not proven to end: {}", o.what, w)),
        };
        l.findings.push(Finding { file: o.file.clone(), line: o.line, level, check: "termination", message: why });
    }
    for n in &report.notes {
        l.findings.push(Finding { file: root.clone(), line: 0, level: Level::Warning, check: "unresolved", message: n.clone() });
    }
    l.run();
    let mut findings = l.findings;
    findings.sort_by(|a, b| (a.file.as_str(), a.line, a.check, a.message.as_str()).cmp(&(b.file.as_str(), b.line, b.check, b.message.as_str())));
    findings.dedup_by(|a, b| a.file == b.file && a.line == b.line && a.check == b.check && a.message == b.message);
    Ok(findings)
}

struct Linter {
    an: An,
    all_files: bool,
    findings: Vec<Finding>,
    /// Types of the variables declared at the top level.
    globals: HashMap<String, TypeRef>,
    /// `missing-case` findings of the type being checked:
    /// (file, line, type, the type can be empty because of it, message).
    pending_cases: Vec<(usize, u32, usize, bool, String)>,
}

/// Names and types visible in one body.
struct Env {
    ns: Vec<String>,
    vars: HashMap<String, TypeRef>,
}

impl Linter {
    fn selected(&self, file: usize) -> bool {
        file == self.an.root_file || (self.all_files && self.an.files[file] != "<prelude>")
    }

    fn report(&mut self, file: usize, line: u32, level: Level, check: &'static str, message: String) {
        self.findings.push(Finding { file: self.an.files[file].clone(), line, level, check, message });
    }

    fn run(&mut self) {
        let main: Vec<(usize, Vec<String>, Stmt)> = self.an.main.iter().map(|m| (m.file, m.ns.clone(), m.stmt.clone())).collect();
        for (_, _, s) in &main {
            for_each_stmt(std::slice::from_ref(s), &mut |st| {
                if let Stmt::Var(d) = st {
                    self.globals.entry(d.name.clone()).or_insert_with(|| d.ty.clone());
                }
            });
        }
        let types = self.an.types.len();
        let reachable = self.an.reachable.clone();
        for node in 0..types {
            if !reachable[node] || !self.selected(self.an.types[node].file) {
                continue;
            }
            let (file, ns) = (self.an.types[node].file, self.an.types[node].ns.clone());
            let def = match &self.an.types[node].kind {
                TKind::Struct(d) | TKind::Union(d) | TKind::Bitfield(d) => d.clone(),
                _ => continue,
            };
            let mut declared: Declared =
                self.inherited(node, &mut HashSet::new()).into_iter().map(|(n, l)| (n, vec![(l, Vec::new())])).collect();
            let mut dups = Dups::new();
            self.duplicates(&def.body, &mut declared, &mut Vec::new(), false, &mut dups);
            let mut dups: Vec<_> = dups.into_iter().collect();
            dups.sort_by_key(|(_, (_, lines))| lines[0]);
            for (name, (first, lines)) in dups {
                let more = match lines.len() {
                    1 => String::new(),
                    n => {
                        let shown: Vec<String> = lines[1..].iter().take(4).map(|l| l.to_string()).collect();
                        let dots = if n - 1 > 4 { ", ..." } else { "" };
                        format!(" (and at {} more place(s): line {}{})", n - 1, shown.join(", "), dots)
                    }
                };
                let m = if first == 0 {
                    format!("`{}` is declared inside a loop; every round adds a member of that name{}", name, more)
                } else {
                    format!(
                        "`{}` may already be declared (line {}); both members exist and a lookup takes the last{}",
                        name, first, more
                    )
                };
                self.report(file, lines[0], Level::Warning, "duplicate-member", m);
            }
            let env = self.env(&ns, &def.body, &[]);
            self.narrow(file, &env, &def.body);
            let can_be_empty = {
                let m = self.an.type_min(node);
                min_opt(m.end, m.exit) == Some(0)
            };
            self.pending_cases.clear();
            self.cases(file, &ns, &def.body, node, can_be_empty);
            // A warning if this dispatch alone decides whether the type
            // reads anything; with several, each may cover for the other.
            let pending = std::mem::take(&mut self.pending_cases);
            let single = pending.len() == 1;
            for (file, line, owner, empties, message) in pending {
                if empties && single {
                    let m = format!("{}; `{}` then reads nothing", message, self.an.types[owner].full);
                    self.report(file, line, Level::Warning, "missing-case", m);
                } else {
                    self.report(file, line, Level::Note, "missing-case", message);
                }
            }
        }
        for f in 0..self.an.funcs.len() {
            if !reachable[types + f] || !self.selected(self.an.funcs[f].file) {
                continue;
            }
            let (file, ns, def) = (self.an.funcs[f].file, self.an.funcs[f].ns.clone(), self.an.funcs[f].def.clone());
            let env = self.env(&ns, &def.body, &def.params);
            self.narrow(file, &env, &def.body);
        }
        for (file, ns, s) in &main {
            if self.selected(*file) {
                let env = Env { ns: ns.clone(), vars: HashMap::new() };
                self.narrow(*file, &env, std::slice::from_ref(s));
            }
        }
        self.unused(&reachable, !main.is_empty());
    }

    // ---- duplicate members -------------------------------------------------

    /// The names a type inherits from its parents: name -> line.
    fn inherited(&mut self, node: usize, busy: &mut HashSet<usize>) -> HashMap<String, u32> {
        let mut out = HashMap::new();
        if !busy.insert(node) {
            return out;
        }
        let (ns, def) = match &self.an.types[node].kind {
            TKind::Struct(d) | TKind::Union(d) | TKind::Bitfield(d) => (self.an.types[node].ns.clone(), d.clone()),
            _ => return out,
        };
        for parent in &def.parents {
            let TypeBase::Custom { name, .. } = &parent.base else { continue };
            let Some(p) = self.an.resolve_type(&ns, name) else { continue };
            out.extend(self.inherited(p, busy));
            if let TKind::Struct(pd) | TKind::Union(pd) | TKind::Bitfield(pd) = &self.an.types[p].kind {
                let pd = pd.clone();
                let mut names = HashMap::new();
                collect_names(&pd.body, &mut names);
                out.extend(names);
            }
        }
        out
    }

    /// Walks a type body. `declared` holds the names that may already exist
    /// when a statement runs, each with the conditions it was declared
    /// under; `guards` are the conditions that hold here. Returns false if
    /// the list cannot fall through (it ends in `break`, `continue` or
    /// `return`).
    fn duplicates(&mut self, stmts: &[Stmt], declared: &mut Declared, guards: &mut Vec<Guard>, in_loop: bool, out: &mut Dups) -> bool {
        for s in stmts {
            let decl: Option<(&str, u32, bool)> = match s {
                Stmt::Var(d) if !d.in_var && !d.out_var => Some((&d.name, d.line, d.value.is_none())),
                Stmt::Array(d) => Some((&d.name, d.line, d.init.is_none())),
                Stmt::Pointer(d) => Some((&d.name, d.line, true)),
                Stmt::BitfieldField(d) => Some((&d.name, d.line, true)),
                _ => None,
            };
            if let Some((name, line, reads)) = decl {
                if !name.is_empty() && !name.starts_with('$') {
                    // An earlier declaration under `c` cannot meet this one
                    // under `!c`.
                    let earlier = declared.get(name).and_then(|ds| ds.iter().find(|(_, g)| !exclusive(g, guards)));
                    if let Some((first, _)) = earlier {
                        out.entry(name.to_string()).or_insert_with(|| (*first, Vec::new())).1.push(line);
                    } else if in_loop && reads {
                        out.entry(name.to_string()).or_insert_with(|| (0, Vec::new())).1.push(line);
                    }
                    declared.entry(name.to_string()).or_default().push((line, guards.clone()));
                }
            }
            match s {
                Stmt::If { cond, then, otherwise, .. } => {
                    let (c, positive) = strip_not(cond);
                    let mut a = declared.clone();
                    let mut b = declared.clone();
                    guards.push((c.clone(), positive));
                    let fa = self.duplicates(then, &mut a, guards, in_loop, out);
                    guards.pop();
                    guards.push((c.clone(), !positive));
                    let fb = self.duplicates(otherwise, &mut b, guards, in_loop, out);
                    guards.pop();
                    merge(declared, [(a, fa), (b, fb)]);
                }
                Stmt::Match { cases, .. } => {
                    let mut arms = Vec::new();
                    for c in cases {
                        let mut a = declared.clone();
                        let f = self.duplicates(&c.body, &mut a, guards, in_loop, out);
                        arms.push((a, f));
                    }
                    merge(declared, arms);
                }
                Stmt::TryCatch { body, handler, .. } => {
                    let mut a = declared.clone();
                    let fa = self.duplicates(body, &mut a, guards, in_loop, out);
                    // The handler runs after the body failed somewhere: what
                    // all but its last statement declare may exist.
                    let mut b = declared.clone();
                    if let Some((_, before_last)) = body.split_last() {
                        let mut names = HashMap::new();
                        collect_names(before_last, &mut names);
                        for (n, l) in names {
                            b.entry(n).or_default().push((l, guards.clone()));
                        }
                    }
                    let fb = self.duplicates(handler, &mut b, guards, in_loop, out);
                    merge(declared, [(a, fa), (b, fb)]);
                }
                Stmt::While { body, .. } | Stmt::For { body, .. } => {
                    let mut a = declared.clone();
                    self.duplicates(body, &mut a, guards, true, out);
                    merge(declared, [(a, true)]);
                }
                Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Return { .. } => return false,
                _ => {}
            }
        }
        true
    }

    // ---- narrow types ------------------------------------------------------

    fn env(&self, ns: &[String], body: &[Stmt], params: &[Param]) -> Env {
        let mut vars: HashMap<String, TypeRef> = params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect();
        for_each_stmt(body, &mut |s| match s {
            Stmt::Var(d) => {
                vars.entry(d.name.clone()).or_insert_with(|| d.ty.clone());
            }
            Stmt::BitfieldField(d) => {
                // A field of n bits: remember it as the smallest type that holds it.
                if let Some(bits) = literal(&d.size) {
                    let b = match bits {
                        0..=8 => BuiltinType::U8,
                        9..=16 => BuiltinType::U16,
                        17..=32 => BuiltinType::U32,
                        _ => BuiltinType::U64,
                    };
                    vars.entry(d.name.clone()).or_insert_with(|| TypeRef::builtin(b));
                }
            }
            _ => {}
        });
        Env { ns: ns.to_vec(), vars }
    }

    /// The builtin type behind a type reference (through aliases and enums).
    fn builtin(&self, ns: &[String], ty: &TypeRef) -> Option<BuiltinType> {
        let mut ty = ty.clone();
        let mut ns = ns.to_vec();
        for _ in 0..16 {
            match &ty.base {
                TypeBase::Builtin(b) => return Some(*b),
                TypeBase::Custom { name, .. } => {
                    let t = self.an.resolve_type(&ns, name)?;
                    let decl = &self.an.types[t];
                    let next = match &decl.kind {
                        TKind::Alias(target, _) => target.clone(),
                        TKind::Enum(e) => e.ty.clone(),
                        _ => return None,
                    };
                    ns = decl.ns.clone();
                    ty = next;
                }
            }
        }
        None
    }

    fn var_type(&self, env: &Env, name: &str) -> Option<TypeRef> {
        env.vars.get(name).or_else(|| self.globals.get(name)).cloned()
    }

    /// The largest value an expression can take, if it can be told.
    fn upper(&self, env: &Env, e: &Expr) -> Option<u128> {
        let int_max = |b: BuiltinType| int_max(b);
        match e {
            Expr::Literal(Literal::Unsigned(n)) => Some(*n),
            Expr::Literal(Literal::Signed(n)) => u128::try_from(*n).ok(),
            Expr::Literal(Literal::Char(_)) => Some(255),
            Expr::Literal(Literal::Bool(_)) => Some(1),
            Expr::Ident(n) => int_max(self.builtin(&env.ns, &self.var_type(env, n)?)?),
            Expr::Member { object, member } => {
                // `x.field`, where `x` is a variable of a struct type.
                let Expr::Ident(obj) = &**object else { return None };
                let ty = self.var_type(env, obj)?;
                let TypeBase::Custom { name, .. } = &ty.base else { return None };
                let t = self.an.resolve_type(&env.ns, name)?;
                let (TKind::Struct(d) | TKind::Union(d) | TKind::Bitfield(d)) = &self.an.types[t].kind else { return None };
                let inner = self.env(&self.an.types[t].ns, &d.body, &[]);
                int_max(self.builtin(&inner.ns, inner.vars.get(member)?)?)
            }
            Expr::Dollar | Expr::SizeOf(_) | Expr::AddressOf(_) => Some(u64::MAX as u128),
            // `$[n]` is the byte at n.
            Expr::Index { object, .. } if **object == Expr::Dollar => Some(255),
            Expr::Call { name, args, .. } => match name.last().map(|s| s.as_str()) {
                Some("read_unsigned") => {
                    let bytes = args.get(1).and_then(literal)?;
                    (bytes <= 16).then(|| if bytes == 16 { u128::MAX } else { (1u128 << (8 * bytes)) - 1 })
                }
                Some("size" | "base_address") => Some(u64::MAX as u128),
                _ => None,
            },
            Expr::Cast { ty, value } => {
                let m = int_max(self.builtin(&env.ns, ty)?)?;
                Some(self.upper(env, value).map(|v| v.min(m)).unwrap_or(m))
            }
            Expr::Unary { op: UnaryOp::Plus, operand } => self.upper(env, operand),
            Expr::Ternary { then, otherwise, .. } => Some(self.upper(env, then)?.max(self.upper(env, otherwise)?)),
            Expr::Binary { op, left, right } => {
                let (a, b) = (self.upper(env, left), self.upper(env, right));
                match op {
                    BinaryOp::Add => Some(a?.saturating_add(b?)),
                    BinaryOp::Mul => Some(a?.saturating_mul(b?)),
                    BinaryOp::Sub | BinaryOp::Div => a,
                    BinaryOp::Rem => b.map(|m| m.saturating_sub(1)).or(a),
                    BinaryOp::BitAnd => match (a, b) {
                        (Some(x), Some(y)) => Some(x.min(y)),
                        (x, None) | (None, x) => x,
                    },
                    BinaryOp::BitOr | BinaryOp::BitXor => {
                        let m = a?.max(b?);
                        Some(if m == u128::MAX { m } else { (m + 1).checked_next_power_of_two().map(|p| p - 1).unwrap_or(u128::MAX) })
                    }
                    BinaryOp::Shr => Some(a? >> literal(right)?.min(127)),
                    BinaryOp::Shl => {
                        let k = literal(right)?.min(127) as u32;
                        Some(a?.checked_shl(k).filter(|v| v >> k == a.unwrap()).unwrap_or(u128::MAX))
                    }
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge | BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => Some(1),
                }
            }
            _ => None,
        }
    }

    /// Loop counters and variables that are too small for what they hold.
    fn narrow(&mut self, file: usize, env: &Env, stmts: &[Stmt]) {
        let mut found: Vec<(u32, &'static str, Level, String)> = Vec::new();
        for_each_stmt(stmts, &mut |s| match s {
            Stmt::While { line, cond, body } => self.counter(env, *line, None, cond, None, body, &mut found),
            Stmt::For { line, init, cond, update, body } => self.counter(env, *line, Some(init), cond, Some(update), body, &mut found),
            Stmt::Var(d) => {
                if let Some(v) = &d.value {
                    self.store(env, d.line, &d.name, &d.ty, v, &mut found);
                }
            }
            Stmt::Assign { line, target: LValue::Ident(n), value } => {
                // `x = x + 1` is the business of the counter check.
                let mut reads_itself = false;
                for_each_expr(value, &mut |x| reads_itself |= matches!(x, Expr::Ident(i) if i == n));
                if !reads_itself {
                    if let Some(ty) = self.var_type(env, n) {
                        self.store(env, *line, n, &ty, value, &mut found);
                    }
                }
            }
            _ => {}
        });
        for (line, check, level, message) in found {
            self.report(file, line, level, check, message);
        }
    }

    /// A value stored in an 8- or 16-bit integer variable.
    fn store(&self, env: &Env, line: u32, name: &str, ty: &TypeRef, value: &Expr, found: &mut Vec<(u32, &'static str, Level, String)>) {
        let Some(b) = self.builtin(&env.ns, ty) else { return };
        let Some(max) = int_max(b) else { return };
        if b.size().unwrap_or(8) > 2 || b == BuiltinType::Bool || b == BuiltinType::Char {
            return;
        }
        let offset = offset_valued(value);
        let upper = self.upper(env, value);
        if offset && upper.map(|u| u > max).unwrap_or(true) {
            found.push((
                line,
                "narrow-offset",
                Level::Warning,
                format!("`{}` is {} but holds an offset; an offset above {} is cut off", name, b, max),
            ));
            return;
        }
        match value {
            Expr::Literal(_) if upper.map(|u| u > max).unwrap_or(false) => {
                found.push((line, "narrowing", Level::Warning, format!("`{}` is {} and cannot hold {}", name, b, upper.unwrap())));
            }
            Expr::Ident(_) | Expr::Member { .. } if upper.map(|u| u > max).unwrap_or(false) => {
                found.push((
                    line,
                    "narrowing",
                    Level::Note,
                    format!("`{}` is {} but is set from a value that can be as large as {}; the rest is cut off", name, b, upper.unwrap()),
                ));
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn counter(&self, env: &Env, line: u32, init: Option<&Stmt>, cond: &Expr, update: Option<&Stmt>, body: &[Stmt], found: &mut Vec<(u32, &'static str, Level, String)>) {
        for (counter, up, step) in counter_candidates(update, body) {
            let LValue::Ident(name) = &counter else { continue };
            let me = Expr::Ident(name.clone());
            let mut read = false;
            for_each_expr(cond, &mut |x| read |= *x == me);
            if !read {
                continue;
            }
            let ty = match init {
                Some(Stmt::Var(d)) if d.name == *name => Some(d.ty.clone()),
                _ => self.var_type(env, name),
            };
            let Some(b) = ty.and_then(|t| self.builtin(&env.ns, &t)) else { continue };
            let Some(max) = int_max(b) else { continue };
            let narrow = b.size().unwrap_or(8) <= 2;
            let mut conjuncts = Vec::new();
            split_and(cond, &mut conjuncts);
            let bounds: Vec<(BinaryOp, &Expr)> = conjuncts
                .iter()
                .filter_map(|c| match c {
                    Expr::Binary { op, left, right } if **left == me => Some((*op, &**right)),
                    Expr::Binary { op, left, right } if **right == me => Some((mirror(*op), &**left)),
                    _ => None,
                })
                .collect();
            if up {
                // `i < B` is safe if the last value plus the step fits.
                let mut safe = false;
                let mut worst: Option<u128> = None;
                let mut literal_bound = false;
                for (op, bound) in &bounds {
                    let slack = match op {
                        BinaryOp::Lt => step - 1,
                        BinaryOp::Le => step,
                        _ => continue,
                    };
                    match self.upper(env, bound) {
                        Some(u) if u.saturating_add(slack) <= max => safe = true,
                        Some(u) => {
                            worst = Some(worst.map_or(u, |w: u128| w.max(u)));
                            literal_bound |= matches!(bound, Expr::Literal(_));
                        }
                        None => {}
                    }
                }
                if !safe && (narrow || literal_bound) {
                    let detail = match worst {
                        Some(u) if u < u32::MAX as u128 * 2 => format!("its bound can be as large as {}", u),
                        _ => "the condition does not keep it below that".to_string(),
                    };
                    found.push((
                        line,
                        "narrow-counter",
                        Level::Warning,
                        format!("loop counter `{}` is {} and wraps at {}, but {}; the loop then never ends", name, b, max, detail),
                    ));
                }
            } else if b.is_unsigned() {
                // Counting down to zero with an unsigned counter.
                let never_false = bounds.iter().any(|(op, bound)| *op == BinaryOp::Ge && literal(bound) == Some(0));
                if never_false {
                    found.push((
                        line,
                        "narrow-counter",
                        Level::Warning,
                        format!("loop counter `{}` is unsigned, so `{} >= 0` always holds; below 0 it wraps to {}", name, name, max),
                    ));
                }
            }
        }
    }

    // ---- enum dispatch -----------------------------------------------------

    /// `E::A` -> (the enum's node, "A").
    fn enum_constant(&self, ns: &[String], e: &Expr) -> Option<(usize, String)> {
        let Expr::Scoped(path) = e else { return None };
        let (entry, ty) = path.split_last()?;
        if ty.is_empty() {
            return None;
        }
        let mut t = self.an.resolve_type(ns, ty)?;
        for _ in 0..16 {
            match &self.an.types[t].kind {
                TKind::Enum(def) => return def.entries.iter().any(|x| x.name == *entry).then(|| (t, entry.clone())),
                TKind::Alias(target, _) => {
                    let TypeBase::Custom { name, .. } = &target.base else { return None };
                    t = self.an.resolve_type(&self.an.types[t].ns, name)?;
                }
                _ => return None,
            }
        }
        None
    }

    /// `x == E::A`, `E::A == x` or several of them joined by `||`:
    /// (x, the enum, the entries tested).
    fn enum_test(&self, ns: &[String], cond: &Expr) -> Option<(Expr, usize, Vec<String>)> {
        match cond {
            Expr::Binary { op: BinaryOp::Or, left, right } => {
                let (x, e, mut names) = self.enum_test(ns, left)?;
                let (y, f, more) = self.enum_test(ns, right)?;
                (x == y && e == f).then(|| {
                    names.extend(more);
                    (x, e, names)
                })
            }
            Expr::Binary { op: BinaryOp::Eq, left, right } => {
                if let Some((e, n)) = self.enum_constant(ns, right) {
                    Some(((**left).clone(), e, vec![n]))
                } else {
                    let (e, n) = self.enum_constant(ns, left)?;
                    Some(((**right).clone(), e, vec![n]))
                }
            }
            _ => None,
        }
    }

    /// Do all the branches read at least a byte?
    fn all_read(&mut self, node: usize, bodies: &[&[Stmt]]) -> bool {
        let cx = self.an.ctx_of(node);
        bodies.iter().all(|b| {
            let f = self.an.walk(&cx, b, Some(0), Phase::Min);
            min_opt(f.end, f.exit).map(|bits| bits >= 8).unwrap_or(true)
        })
    }

    fn cases(&mut self, file: usize, ns: &[String], stmts: &[Stmt], owner: usize, can_be_empty: bool) {
        for s in stmts {
            match s {
                Stmt::If { line, .. } => {
                    // Collect the `else if` chain that starts here.
                    let mut arms: Vec<&Expr> = Vec::new();
                    let mut bodies: Vec<&[Stmt]> = Vec::new();
                    let mut cur = s;
                    let has_else = loop {
                        let Stmt::If { cond, then, otherwise, .. } = cur else { unreachable!() };
                        arms.push(cond);
                        bodies.push(then);
                        self.cases(file, ns, then, owner, can_be_empty);
                        match otherwise.as_slice() {
                            [] => break false,
                            [next @ Stmt::If { .. }] => cur = next,
                            other => {
                                self.cases(file, ns, other, owner, can_be_empty);
                                break true;
                            }
                        }
                    };
                    if !has_else && arms.len() >= 2 {
                        let tests: Option<Vec<_>> = arms.iter().map(|c| self.enum_test(ns, c)).collect();
                        if let Some(tests) = tests {
                            let same = tests.iter().all(|t| t.0 == tests[0].0 && t.1 == tests[0].1);
                            if same {
                                let covered: Vec<String> = tests.iter().flat_map(|t| t.2.iter().cloned()).collect();
                                let empties = can_be_empty && self.all_read(owner, &bodies);
                                self.missing(file, *line, tests[0].1, &covered, "`else`", owner, empties);
                            }
                        }
                    }
                }
                Stmt::Match { line, values, cases } => {
                    for c in cases {
                        self.cases(file, ns, &c.body, owner, can_be_empty);
                    }
                    let wildcard = cases.iter().any(|c| c.patterns.iter().all(|p| matches!(p, CasePattern::Wildcard)));
                    if values.len() == 1 && !wildcard && cases.len() >= 2 {
                        let mut covered = Vec::new();
                        let mut the_enum = None;
                        let mut ok = true;
                        for c in cases {
                            match c.patterns.as_slice() {
                                [CasePattern::Alternatives(alts)] => {
                                    for (lo, hi) in alts {
                                        match (self.enum_constant(ns, lo), hi) {
                                            (Some((e, n)), None) if the_enum.is_none() || the_enum == Some(e) => {
                                                the_enum = Some(e);
                                                covered.push(n);
                                            }
                                            _ => ok = false,
                                        }
                                    }
                                }
                                _ => ok = false,
                            }
                        }
                        if let (true, Some(e)) = (ok, the_enum) {
                            let bodies: Vec<&[Stmt]> = cases.iter().map(|c| c.body.as_slice()).collect();
                            let empties = can_be_empty && self.all_read(owner, &bodies);
                            self.missing(file, *line, e, &covered, "`(_)` arm", owner, empties);
                        }
                    }
                }
                Stmt::TryCatch { body, handler, .. } => {
                    self.cases(file, ns, body, owner, can_be_empty);
                    self.cases(file, ns, handler, owner, can_be_empty);
                }
                Stmt::While { body, .. } | Stmt::For { body, .. } => self.cases(file, ns, body, owner, can_be_empty),
                _ => {}
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn missing(&mut self, file: usize, line: u32, the_enum: usize, covered: &[String], fallback: &str, owner: usize, can_be_empty: bool) {
        let def = match &self.an.types[the_enum].kind {
            TKind::Enum(def) => def.clone(),
            _ => return,
        };
        let values = enum_values(&def);
        let value_of = |name: &str| values.iter().find(|v| v.0 == name).and_then(|v| v.1);
        let covered_values: Vec<(i128, i128)> = covered.iter().filter_map(|c| value_of(c)).collect();
        let missing: Vec<&str> = values
            .iter()
            .filter(|(name, v)| !covered.iter().any(|c| c == name) && !v.map(|v| covered_values.contains(&v)).unwrap_or(false))
            .map(|(name, _)| name.as_str())
            .collect();
        if missing.is_empty() {
            return;
        }
        let shown: Vec<String> = missing.iter().take(4).map(|m| format!("`{}`", m)).collect();
        let more = if missing.len() > 4 { format!(" and {} more", missing.len() - 4) } else { String::new() };
        let enum_name = self.an.types[the_enum].full.clone();
        let message = format!("no branch for {}{} of `{}` and no {}", shown.join(", "), more, enum_name, fallback);
        self.pending_cases.push((file, line, owner, can_be_empty, message));
    }

    // ---- unused ------------------------------------------------------------

    fn unused(&mut self, reachable: &[bool], has_main: bool) {
        // A file that places nothing is a library; everything in it is unused here.
        if !has_main {
            return;
        }
        let types = self.an.types.len();
        let mut found = Vec::new();
        // `#include` splices library text into the pattern's own file.
        let library = |ns: &[String]| matches!(ns.first().map(|s| s.as_str()), Some("std" | "type" | "hex" | "builtin"));
        for (i, t) in self.an.types.iter().enumerate() {
            if t.file == self.an.root_file && !reachable[i] && !matches!(t.kind, TKind::Forward) && !library(&t.ns) {
                found.push((t.line, format!("type `{}` is never used", t.full)));
            }
        }
        for (i, f) in self.an.funcs.iter().enumerate() {
            if f.file == self.an.root_file && !reachable[types + i] && !library(&f.ns) {
                found.push((f.def.line, format!("function `{}` is never called", f.full)));
            }
        }
        let root = self.an.root_file;
        for (line, m) in found {
            self.report(root, line, Level::Note, "unused", m);
        }
    }
}

/// A branch condition and whether it holds.
type Guard = (Expr, bool);
/// name -> every declaration that may be alive: (line, the guards it is under).
type Declared = HashMap<String, Vec<(u32, Vec<Guard>)>>;
/// name -> (line of the earlier declaration, or 0 for "inside a loop"; lines of the later ones).
type Dups = HashMap<String, (u32, Vec<u32>)>;

/// Can the two sets of branch conditions not hold together? True if one
/// has `c` and the other `!c`, or if they put one expression into ranges
/// that do not meet (`v < 600` against `v >= 600`).
fn exclusive(a: &[Guard], b: &[Guard]) -> bool {
    if a.iter().any(|(c, p)| b.iter().any(|(c2, p2)| c == c2 && p != p2)) {
        return true;
    }
    let (ra, rb) = (ranges(a), ranges(b));
    ra.iter().any(|(e, (lo, hi))| rb.iter().any(|(e2, (lo2, hi2))| e == e2 && (*lo).max(*lo2) > (*hi).min(*hi2)))
}

/// The range each expression is confined to by comparisons with literals.
fn ranges(guards: &[Guard]) -> Vec<(Expr, (i128, i128))> {
    fn lit(e: &Expr) -> Option<i128> {
        match e {
            Expr::Literal(Literal::Unsigned(n)) => i128::try_from(*n).ok(),
            Expr::Literal(Literal::Signed(n)) => Some(*n),
            _ => None,
        }
    }
    // (expression, comparison with the literal on the right, literal)
    fn atom(e: &Expr) -> Option<(Expr, BinaryOp, i128)> {
        let Expr::Binary { op, left, right } = e else { return None };
        if let Some(n) = lit(right) {
            Some(((**left).clone(), *op, n))
        } else {
            lit(left).map(|n| ((**right).clone(), mirror(*op), n))
        }
    }
    let mut out: Vec<(Expr, (i128, i128))> = Vec::new();
    let mut confine = |e: Expr, lo: i128, hi: i128| match out.iter_mut().find(|(x, _)| *x == e) {
        Some((_, r)) => *r = (r.0.max(lo), r.1.min(hi)),
        None => out.push((e, (lo, hi))),
    };
    for (cond, holds) in guards {
        let mut conjuncts = Vec::new();
        split_and(cond, &mut conjuncts);
        // The negation of a conjunction says nothing about one conjunct.
        if !holds && conjuncts.len() != 1 {
            continue;
        }
        for c in conjuncts {
            let Some((e, op, n)) = atom(c) else { continue };
            let op = match (holds, op) {
                (true, op) => op,
                (false, BinaryOp::Lt) => BinaryOp::Ge,
                (false, BinaryOp::Le) => BinaryOp::Gt,
                (false, BinaryOp::Gt) => BinaryOp::Le,
                (false, BinaryOp::Ge) => BinaryOp::Lt,
                (false, _) => continue,
            };
            match op {
                BinaryOp::Lt => confine(e, i128::MIN, n - 1),
                BinaryOp::Le => confine(e, i128::MIN, n),
                BinaryOp::Gt => confine(e, n + 1, i128::MAX),
                BinaryOp::Ge => confine(e, n, i128::MAX),
                BinaryOp::Eq => confine(e, n, n),
                _ => {}
            }
        }
    }
    out
}

/// `!c` -> (c, false); `c` -> (c, true).
fn strip_not(e: &Expr) -> (&Expr, bool) {
    match e {
        Expr::Unary { op: UnaryOp::Not, operand } => {
            let (c, p) = strip_not(operand);
            (c, !p)
        }
        other => (other, true),
    }
}

/// Is the value of the expression a position in the data? `$[n]` and
/// `read_unsigned($, 2)` are data, and `$ - addressof(this)` is a length.
fn offset_valued(e: &Expr) -> bool {
    match e {
        Expr::Dollar | Expr::AddressOf(_) => true,
        Expr::Call { name, .. } => {
            let last = name.last().map(|s| s.as_str()).unwrap_or("");
            name.iter().any(|p| p == "mem") && (matches!(last, "size" | "base_address") || last.starts_with("find_"))
        }
        Expr::Binary { op: BinaryOp::Sub, left, right } => offset_valued(left) && !offset_valued(right),
        Expr::Binary { op: BinaryOp::Add | BinaryOp::Mul | BinaryOp::BitOr, left, right } => offset_valued(left) || offset_valued(right),
        Expr::Cast { value, .. } => offset_valued(value),
        Expr::Ternary { then, otherwise, .. } => offset_valued(then) || offset_valued(otherwise),
        _ => false,
    }
}

/// The names a statement list may declare, on any path.
fn collect_names(stmts: &[Stmt], out: &mut HashMap<String, u32>) {
    for_each_stmt(stmts, &mut |s| {
        let (name, line) = match s {
            Stmt::Var(d) if !d.in_var && !d.out_var => (&d.name, d.line),
            Stmt::Array(d) => (&d.name, d.line),
            Stmt::Pointer(d) => (&d.name, d.line),
            Stmt::BitfieldField(d) => (&d.name, d.line),
            _ => return,
        };
        if !name.is_empty() && !name.starts_with('$') {
            out.entry(name.clone()).or_insert(line);
        }
    });
}

/// After branches: a declaration may be alive if a branch that falls
/// through made it.
fn merge(declared: &mut Declared, branches: impl IntoIterator<Item = (Declared, bool)>) {
    for (names, falls_through) in branches {
        if falls_through {
            for (n, ds) in names {
                let mine = declared.entry(n).or_default();
                for d in ds {
                    if !mine.iter().any(|m| m.0 == d.0) {
                        mine.push(d);
                    }
                }
            }
        }
    }
}

fn int_max(b: BuiltinType) -> Option<u128> {
    let bits = b.size()? * 8;
    if b.is_unsigned() || matches!(b, BuiltinType::Char | BuiltinType::Char16) {
        Some(if bits >= 128 { u128::MAX } else { (1u128 << bits) - 1 })
    } else if b.is_signed() {
        Some((1u128 << (bits - 1)) - 1)
    } else if b == BuiltinType::Bool {
        Some(1)
    } else {
        None
    }
}

/// The entries of an enum with their values, where they are literals:
/// (name, (first, last)).
fn enum_values(def: &EnumDef) -> Vec<(String, Option<(i128, i128)>)> {
    let lit = |e: &Expr| match e {
        Expr::Literal(Literal::Unsigned(n)) => i128::try_from(*n).ok(),
        Expr::Literal(Literal::Signed(n)) => Some(*n),
        Expr::Literal(Literal::Char(c)) => Some(*c as i128),
        _ => None,
    };
    let mut next: Option<i128> = Some(0);
    let mut out = Vec::new();
    for e in &def.entries {
        let first = match &e.value {
            Some(v) => lit(v),
            None => next,
        };
        let last = match &e.end {
            Some(v) => lit(v),
            None => first,
        };
        out.push((e.name.clone(), first.zip(last)));
        next = last.map(|l| l + 1);
    }
    out
}

// ---------------------------------------------------------------------------
// Checks on an evaluated file

/// Coverage, overlap and enum values of the patterns in `rt`, which has
/// evaluated a pattern on a data file named `file`.
pub fn lint_data(rt: &Runtime, file: &str) -> Vec<Finding> {
    let mut leaves: Vec<(u64, u64, String)> = Vec::new();
    let mut bad_enums: Vec<String> = Vec::new();
    for p in &rt.patterns {
        let name = p.borrow().name.clone();
        collect_leaves(rt, p, name, &mut leaves, &mut bad_enums, 0);
    }
    let len = rt.data_len();
    let mut out = Vec::new();
    let finding = |level, check, message| Finding { file: file.to_string(), line: 0, level, check, message };

    // Bytes no pattern describes.
    leaves.sort_by(|a, b| (a.0, std::cmp::Reverse(a.1)).cmp(&(b.0, std::cmp::Reverse(b.1))));
    let mut gaps: Vec<(u64, u64)> = Vec::new();
    let mut overlaps: Vec<(u64, u64, String, String)> = Vec::new();
    let mut overlap_bytes = 0u64;
    let mut end = 0u64;
    let mut end_name = String::new();
    for (start, size, name) in &leaves {
        let stop = start.saturating_add(*size).min(len);
        if *start > end {
            gaps.push((end, *start));
        } else if *start < end && stop > *start {
            let shared = end.min(stop) - start;
            overlap_bytes += shared;
            overlaps.push((*start, start + shared, end_name.clone(), name.clone()));
        }
        if stop > end {
            end = stop;
            end_name = name.clone();
        }
    }
    if end < len {
        gaps.push((end, len));
    }
    let missing: u64 = gaps.iter().map(|g| g.1 - g.0).sum();
    let range = |a: u64, b: u64| if b - a == 1 { format!("0x{:X}", a) } else { format!("0x{:X}-0x{:X}", a, b - 1) };
    if len > 0 {
        let pct = (len - missing) as f64 * 100.0 / len as f64;
        let mut m = format!("{} of {} bytes are described ({:.1}%)", len - missing, len, pct);
        if !gaps.is_empty() {
            let shown: Vec<String> = gaps.iter().take(6).map(|g| format!("{} ({} bytes)", range(g.0, g.1), g.1 - g.0)).collect();
            let more = if gaps.len() > 6 { format!(" and {} more ranges", gaps.len() - 6) } else { String::new() };
            m.push_str(&format!("; not described: {}{}", shown.join(", "), more));
        }
        out.push(finding(Level::Note, "coverage", m));
    }
    if !overlaps.is_empty() {
        // One line per pattern that lies over others, widest first: a view
        // of the whole file would otherwise list every field under it.
        let mut by_cover: Vec<(String, u64, usize, (u64, u64, String))> = Vec::new();
        for (a, b, cover, under) in &overlaps {
            match by_cover.iter_mut().find(|c| c.0 == *cover) {
                Some(c) => {
                    c.1 += b - a;
                    c.2 += 1;
                }
                None => by_cover.push((cover.clone(), b - a, 1, (*a, *b, under.clone()))),
            }
        }
        by_cover.sort_by(|x, y| y.1.cmp(&x.1));
        let shown: Vec<String> = by_cover
            .iter()
            .take(6)
            .map(|(cover, bytes, count, (a, b, under))| {
                if *count == 1 {
                    format!("`{}` and `{}` share {}", cover, under, range(*a, *b))
                } else {
                    format!("`{}` shares {} byte(s) with {} other patterns (first `{}` at {})", cover, bytes, count, under, range(*a, *b))
                }
            })
            .collect();
        let more = if by_cover.len() > 6 { format!("; and {} more", by_cover.len() - 6) } else { String::new() };
        out.push(finding(
            Level::Note,
            "overlap",
            format!("{} byte(s) are described more than once: {}{}", overlap_bytes, shown.join("; "), more),
        ));
    }
    let more = bad_enums.len().saturating_sub(8);
    for m in bad_enums.into_iter().take(8) {
        out.push(finding(Level::Warning, "enum-value", m));
    }
    if more > 0 {
        out.push(finding(Level::Warning, "enum-value", format!("{} more enum fields hold a value that is not an entry", more)));
    }
    out
}

/// The patterns that stand for bytes of the data: scalars, strings, arrays
/// of scalars, padding, bitfields and unions as a whole.
fn collect_leaves(rt: &Runtime, p: &PatternRef, path: String, leaves: &mut Vec<(u64, u64, String)>, bad_enums: &mut Vec<String>, depth: usize) {
    if depth > 512 {
        return;
    }
    let pb = p.borrow();
    let child = |name: &str| if name.starts_with('[') { format!("{}{}", path, name) } else { format!("{}.{}", path, name) };
    match &pb.kind {
        PatternKind::Struct { members } => {
            for m in members {
                let name = m.borrow().name.clone();
                collect_leaves(rt, m, child(&name), leaves, bad_enums, depth + 1);
            }
            return;
        }
        PatternKind::Array { entries } => {
            for e in entries {
                let name = e.borrow().name.clone();
                collect_leaves(rt, e, child(&name), leaves, bad_enums, depth + 1);
            }
            return;
        }
        PatternKind::Pointer { pointee, .. } => {
            collect_leaves(rt, pointee, format!("*{}", path), leaves, bad_enums, depth + 1);
        }
        PatternKind::Enum { entries, .. } => {
            if pb.section == 0 {
                if let Ok(v) = rt.pattern_value(p).and_then(|v| v.as_i128()) {
                    if !entries.iter().any(|(_, lo, hi)| (*lo..=*hi).contains(&v)) {
                        bad_enums.push(format!("`{}` at 0x{:X} is 0x{:X}, which is not an entry of `{}`", path, pb.offset, v, pb.type_name));
                    }
                }
            }
        }
        _ => {}
    }
    // Locals live in sections of their own.
    if pb.section == 0 && pb.size > 0 {
        leaves.push((pb.offset, pb.size, path));
    }
}
