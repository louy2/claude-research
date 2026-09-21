//! Expression evaluation and value arithmetic.

use super::{Runtime, Value};
use crate::ast::*;
use crate::error::{Error, Result};
use crate::interp::{PatternKind, PatternRef};

impl Runtime {
    /// Evaluates an expression to a value. Scalar patterns are read; struct,
    /// array and bitfield patterns are returned as [`Value::Pattern`].
    pub fn eval(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(l) => Ok(match l {
                Literal::Unsigned(v) => Value::Unsigned(*v),
                Literal::Signed(v) => Value::Signed(*v),
                Literal::Float(f) => Value::Float(*f),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Char(c) => Value::Char(*c),
                Literal::String(s) => Value::Str(s.clone()),
            }),
            Expr::Dollar => Ok(Value::Unsigned(self.cursor as u128)),
            Expr::Null => Ok(Value::Null),
            Expr::Ident(name) => self.eval_ident(name),
            Expr::Scoped(path) => self.eval_scoped(path),
            Expr::Parent | Expr::This | Expr::Member { .. } => {
                let p = self.eval_pattern(expr)?;
                self.value_of(&p)
            }
            Expr::Index { object, index } => {
                if matches!(object.as_ref(), Expr::Dollar) {
                    // `$[addr]` reads the byte at `addr`.
                    let addr = self.eval(index)?.as_u64()?;
                    let reader = crate::reader::Reader::new(self.section_data(self.current_section));
                    return Ok(Value::Unsigned(reader.unsigned(addr, 1, Endian::Little)?));
                }
                if let Ok(p) = self.eval_pattern(expr) {
                    return self.value_of(&p);
                }
                // Indexing a value: strings yield chars, packs yield items.
                let obj = self.eval(object)?;
                let i = self.eval(index)?.as_u64()? as usize;
                match obj {
                    Value::Str(s) => s
                        .as_bytes()
                        .get(i)
                        .map(|b| Value::Char(*b))
                        .ok_or_else(|| Error::new(format!("string index {} out of range", i))),
                    Value::Pack(items) => items.get(i).cloned().ok_or_else(|| Error::new("pack index out of range")),
                    Value::List(items) => items.borrow().get(i).cloned().ok_or_else(|| Error::new("array index out of range")),
                    other => Err(Error::new(format!("cannot index a value of type {}", other.type_name()))),
                }
            }
            Expr::Unary { op, operand } => {
                let v = self.eval(operand)?;
                unary(*op, v)
            }
            Expr::Binary { op, left, right } => {
                // Short-circuit boolean operators.
                match op {
                    BinaryOp::And => {
                        let l = self.eval(left)?.as_bool()?;
                        if !l {
                            return Ok(Value::Bool(false));
                        }
                        return Ok(Value::Bool(self.eval(right)?.as_bool()?));
                    }
                    BinaryOp::Or => {
                        let l = self.eval(left)?.as_bool()?;
                        if l {
                            return Ok(Value::Bool(true));
                        }
                        return Ok(Value::Bool(self.eval(right)?.as_bool()?));
                    }
                    _ => {}
                }
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                binary(*op, l, r)
            }
            Expr::Ternary { cond, then, otherwise } => {
                if self.eval(cond)?.as_bool()? {
                    self.eval(then)
                } else {
                    self.eval(otherwise)
                }
            }
            Expr::Cast { ty, value } => {
                let v = self.eval(value)?;
                let v = match v {
                    Value::Pattern(p) => self.value_of(&p)?,
                    other => other,
                };
                match &ty.base {
                    TypeBase::Builtin(b) => {
                        let v = cast_builtin(*b, v)?;
                        // `be u16(x)` reinterprets the bytes of `x` in the
                        // other byte order.
                        if ty.endian == Some(Endian::Big) {
                            if let Some(width) = b.size() {
                                if b.is_unsigned() || b.is_signed() {
                                    let raw = v.as_i128()? as u128;
                                    let mut swapped: u128 = 0;
                                    for i in 0..width {
                                        swapped |= ((raw >> (8 * i)) & 0xFF) << (8 * (width - 1 - i));
                                    }
                                    return cast_builtin(*b, Value::Unsigned(swapped));
                                }
                            }
                        }
                        Ok(v)
                    }
                    _ => Err(Error::new("casts only support builtin types")),
                }
            }
            Expr::Reinterpret { value, ty } => {
                let v = self.eval(value)?;
                let inst = self.instantiate(ty)?;
                match inst.as_builtin() {
                    Some(b) => cast_builtin(b, v),
                    None => Ok(v),
                }
            }
            Expr::Call { name, args, .. } => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval_arg(a)?);
                }
                self.call_function(name, vals)
            }
            Expr::SizeOf(arg) => self.eval_sizeof(arg),
            Expr::AddressOf(arg) => match arg {
                TypeOrExpr::Expr(e) => match e.as_ref() {
                    Expr::Dollar => Ok(Value::Unsigned(self.cursor as u128)),
                    e => {
                        let p = self.eval_pattern(e)?;
                        let off = p.borrow().offset;
                        Ok(Value::Unsigned(off as u128))
                    }
                },
                TypeOrExpr::Type(t) => {
                    // A bare identifier parsed as a type: it names a pattern.
                    let e = type_as_expr(t).ok_or_else(|| Error::new("addressof needs a pattern"))?;
                    let p = self.eval_pattern(&e)?;
                    let off = p.borrow().offset;
                    Ok(Value::Unsigned(off as u128))
                }
            },
            Expr::TypeNameOf(arg) => match arg {
                TypeOrExpr::Type(t) => {
                    if let Some(e) = type_as_expr(t) {
                        if let Ok(p) = self.eval_pattern(&e) {
                            let name = p.borrow().type_name.clone();
                            return Ok(Value::Str(name));
                        }
                    }
                    let inst = self.instantiate(t)?;
                    Ok(Value::Str(inst.display))
                }
                TypeOrExpr::Expr(e) => {
                    let p = self.eval_pattern(e)?;
                    let name = p.borrow().type_name.clone();
                    Ok(Value::Str(name))
                }
            },
        }
    }

    /// Evaluates a call argument: patterns are passed by reference so that
    /// `ref` parameters and formatters can see the pattern itself.
    fn eval_arg(&mut self, e: &Expr) -> Result<Value> {
        match e {
            Expr::Ident(_) | Expr::Member { .. } | Expr::Index { .. } | Expr::This | Expr::Parent => {
                if let Ok(p) = self.eval_pattern(e) {
                    return Ok(Value::Pattern(p));
                }
                self.eval(e)
            }
            _ => self.eval(e),
        }
    }

    fn eval_sizeof(&mut self, arg: &TypeOrExpr) -> Result<Value> {
        match arg {
            TypeOrExpr::Expr(e) => match e.as_ref() {
                // `sizeof($)` is the size of the whole data.
                Expr::Dollar => Ok(Value::Unsigned(self.data_size() as u128)),
                e => {
                    if let Ok(p) = self.eval_pattern(e) {
                        let size = p.borrow().size;
                        return Ok(Value::Unsigned(size as u128));
                    }
                    let v = self.eval(e)?;
                    Ok(Value::Unsigned(value_size(&v) as u128))
                }
            },
            TypeOrExpr::Type(t) => {
                // Prefer a pattern/variable of that name; otherwise a type.
                if let Some(e) = type_as_expr(t) {
                    if let Ok(p) = self.eval_pattern(&e) {
                        let size = p.borrow().size;
                        return Ok(Value::Unsigned(size as u128));
                    }
                    if let Expr::Ident(name) = &e {
                        if let Some(slot) = self.lookup_slot(name) {
                            let v = slot.value.clone();
                            return Ok(Value::Unsigned(value_size(&v) as u128));
                        }
                    }
                }
                let inst = self.instantiate(t)?;
                Ok(Value::Unsigned(self.sizeof_type(&inst)? as u128))
            }
        }
    }

    fn eval_ident(&mut self, name: &str) -> Result<Value> {
        if let Some(slot) = self.lookup_slot(name) {
            let v = slot.value.clone();
            return Ok(match v {
                Value::Pattern(p) => self.value_of(&p)?,
                other => other,
            });
        }
        if let Some(m) = self.lookup_member(name) {
            return self.value_of(&m);
        }
        // Top-level placed patterns are visible by name everywhere.
        if let Some(p) = self.find_global_pattern(name) {
            return self.value_of(&p);
        }
        match name {
            "nan" => return Ok(Value::Float(f64::NAN)),
            "inf" => return Ok(Value::Float(f64::INFINITY)),
            _ => {}
        }
        Err(Error::new(format!("unknown identifier `{}`", name)))
    }

    fn find_global_pattern(&self, name: &str) -> Option<PatternRef> {
        self.patterns.iter().rev().find(|p| p.borrow().name == name).cloned()
    }

    fn eval_scoped(&mut self, path: &[String]) -> Result<Value> {
        // Enum value `Type::Variant`.
        if path.len() >= 2 {
            let (type_path, variant) = path.split_at(path.len() - 1);
            let decl = match self.find_type(type_path) {
                Some(d) if matches!(d.kind, super::TypeKind::Alias(..)) => {
                    let tr = TypeRef { reference: false, endian: None, base: TypeBase::Custom { name: type_path.to_vec(), args: Vec::new() } };
                    self.instantiate(&tr).ok().and_then(|i| i.decl().cloned())
                }
                other => other,
            };
            if let Some(decl) = decl {
                if let super::TypeKind::Enum(def) = &decl.kind {
                    let def = def.clone();
                    let entries = self.enum_entries(&decl, &def)?;
                    if let Some((_, v, _)) = entries.iter().find(|(n, _, _)| n == &variant[0]) {
                        let underlying = self.instantiate(&def.ty)?;
                        let signed = underlying.as_builtin().map(|b| b.is_signed()).unwrap_or(false);
                        return Ok(if signed { Value::Signed(*v) } else { Value::Unsigned(*v as u128) });
                    }
                    return Err(Error::new(format!("enum `{}` has no entry `{}`", decl.full_name, variant[0])));
                }
            }
        }
        // Namespaced global variable.
        for cand in self.candidate_names(path) {
            if let Some(slot) = self.scopes_global_slot(&cand) {
                return Ok(slot);
            }
        }
        Err(Error::new(format!("unknown name `{}`", path.join("::"))))
    }

    /// Resolves an expression that denotes a pattern (`a.b[2].c`, `this`,
    /// `parent`, a member or a global).
    pub fn eval_pattern(&mut self, expr: &Expr) -> Result<PatternRef> {
        match expr {
            Expr::This => self.this_pattern(),
            Expr::Parent => self.parent_pattern(),
            Expr::Ident(name) => {
                if let Some(slot) = self.lookup_slot(name) {
                    if let Value::Pattern(p) = &slot.value {
                        return Ok(p.clone());
                    }
                    return Err(Error::new(format!("`{}` is not a pattern", name)));
                }
                if let Some(t) = self.lookup_template_type(name) {
                    let _ = t;
                    return Err(Error::new(format!("`{}` is a type, not a pattern", name)));
                }
                if let Some(m) = self.lookup_member(name) {
                    return Ok(m);
                }
                if let Some(p) = self.find_global_pattern(name) {
                    return Ok(p);
                }
                Err(Error::new(format!("unknown pattern `{}`", name)))
            }
            Expr::Scoped(path) => {
                for cand in self.candidate_names(path) {
                    let parts: Vec<&str> = cand.split("::").collect();
                    if let Some(p) = self.find_global_pattern(parts[parts.len() - 1]) {
                        return Ok(p);
                    }
                }
                Err(Error::new(format!("unknown pattern `{}`", path.join("::"))))
            }
            Expr::Member { object, member } => {
                if member == "parent" {
                    // `x.parent`: find the scope in which `x` is `this` and
                    // return that scope's parent.
                    let obj = self.eval_pattern(object)?;
                    for s in self.scopes.iter().rev() {
                        if let Some(t) = &s.this {
                            if std::rc::Rc::ptr_eq(t, &obj) {
                                return s.parent.clone().ok_or_else(|| Error::new("pattern has no parent"));
                            }
                        }
                    }
                    return Err(Error::new(format!("cannot determine the parent of `{}`", obj.borrow().name)));
                }
                let obj = self.eval_pattern(object)?;
                let obj = deref_pointer(obj);
                let m = obj.borrow().member(member);
                match m {
                    Some(m) => Ok(m),
                    None => Err(Error::new(format!("pattern `{}` has no member `{}`", obj.borrow().name, member))),
                }
            }
            Expr::Index { object, index } => {
                let obj = self.eval_pattern(object)?;
                let obj = deref_pointer(obj);
                let i = self.eval(index)?.as_u64()?;
                self.array_element(&obj, i)
            }
            Expr::Call { name, args, .. } => {
                let mut vals = Vec::new();
                for a in args {
                    vals.push(self.eval_arg(a)?);
                }
                match self.call_function(name, vals)? {
                    Value::Pattern(p) => Ok(p),
                    _ => Err(Error::new("function did not return a pattern")),
                }
            }
            _ => Err(Error::new("expression does not denote a pattern")),
        }
    }

    fn this_pattern(&self) -> Result<PatternRef> {
        for s in self.scopes.iter().rev() {
            if let Some(t) = &s.this {
                return Ok(t.clone());
            }
            if s.kind == super::ScopeKind::Function {
                break;
            }
        }
        Err(Error::new("`this` used outside of a type"))
    }

    fn parent_pattern(&self) -> Result<PatternRef> {
        for s in self.scopes.iter().rev() {
            if s.this.is_some() || s.kind == super::ScopeKind::Function {
                return s.parent.clone().ok_or_else(|| Error::new("pattern has no parent"));
            }
        }
        Err(Error::new("`parent` used outside of a type"))
    }

    fn scopes_global_slot(&self, full: &str) -> Option<Value> {
        // Namespaced globals are registered under their plain name in the
        // global scope; strip the namespace for lookup.
        let plain = full.rsplit("::").next().unwrap_or(full);
        self.scopes[0].vars.get(plain).map(|s| s.value.clone())
    }
}

/// Follows a pointer pattern to its pointee.
fn deref_pointer(p: PatternRef) -> PatternRef {
    let target = match &p.borrow().kind {
        PatternKind::Pointer { pointee, .. } => Some(pointee.clone()),
        _ => None,
    };
    target.unwrap_or(p)
}

/// Turns `T` (a bare name parsed as a type) back into an expression.
fn type_as_expr(t: &TypeRef) -> Option<Expr> {
    match &t.base {
        TypeBase::Custom { name, args } if args.is_empty() && !t.reference && t.endian.is_none() => {
            if name.len() == 1 {
                Some(Expr::Ident(name[0].clone()))
            } else {
                Some(Expr::Scoped(name.clone()))
            }
        }
        _ => None,
    }
}

/// Size in bytes of a value used with `sizeof`.
fn value_size(v: &Value) -> u64 {
    match v {
        Value::Unsigned(_) | Value::Signed(_) => 16,
        Value::Float(_) => 8,
        Value::Bool(_) | Value::Char(_) => 1,
        Value::Char16(_) => 2,
        Value::Str(s) => s.len() as u64,
        Value::Pattern(p) => p.borrow().size,
        Value::Pack(items) => items.len() as u64,
        Value::List(items) => items.borrow().len() as u64,
        Value::Null => 0,
    }
}

/// Applies a builtin-type cast to a value.
pub fn cast_builtin(b: BuiltinType, v: Value) -> Result<Value> {
    use BuiltinType::*;
    let v = match v {
        Value::Pack(_) => return Err(Error::new("cannot cast a parameter pack")),
        other => other,
    };
    Ok(match b {
        Str => Value::Str(match &v {
            Value::Str(s) => s.clone(),
            Value::Char(c) => (*c as char).to_string(),
            other => other.to_display_string(),
        }),
        Bool => Value::Bool(v.as_bool()?),
        Char => Value::Char(match &v {
            Value::Str(s) => s.as_bytes().first().copied().unwrap_or(0),
            other => other.as_i128()? as u8,
        }),
        Char16 => Value::Char16(v.as_i128()? as u16),
        Float => Value::Float(v.as_f64()? as f32 as f64),
        Double => Value::Float(v.as_f64()?),
        Auto => v,
        _ => {
            let bits = b.size().unwrap() * 8;
            let raw = match &v {
                Value::Float(f) => *f as i128 as u128,
                Value::Str(s) => s.parse::<i128>().map_err(|_| Error::new(format!("cannot convert \"{}\" to an integer", s)))? as u128,
                other => other.as_i128()? as u128,
            };
            let mask = if bits >= 128 { u128::MAX } else { (1u128 << bits) - 1 };
            let truncated = raw & mask;
            if b.is_signed() {
                Value::Signed(crate::reader::sign_extend(truncated, bits))
            } else {
                Value::Unsigned(truncated)
            }
        }
    })
}

fn unary(op: UnaryOp, v: Value) -> Result<Value> {
    Ok(match op {
        UnaryOp::Plus => v,
        UnaryOp::Neg => match v {
            Value::Float(f) => Value::Float(-f),
            other => Value::Signed(other.as_i128()?.wrapping_neg()),
        },
        UnaryOp::Not => Value::Bool(!v.as_bool()?),
        UnaryOp::BitNot => match v {
            Value::Signed(i) => Value::Signed(!i),
            other => Value::Unsigned(!other.as_u128()?),
        },
    })
}

/// Numeric promotion: float if either is float, signed if either is signed.
enum Num {
    U(u128, u128),
    I(i128, i128),
    F(f64, f64),
}

fn promote(l: &Value, r: &Value) -> Result<Num> {
    if l.is_float() || r.is_float() {
        return Ok(Num::F(l.as_f64()?, r.as_f64()?));
    }
    if l.is_signed() || r.is_signed() {
        return Ok(Num::I(l.as_i128()?, r.as_i128()?));
    }
    Ok(Num::U(l.as_u128()?, r.as_u128()?))
}

/// Compares two values with a relational operator.
pub fn compare(l: &Value, r: &Value, op: BinaryOp) -> Result<bool> {
    let ord = match (l, r) {
        (Value::Str(a), Value::Str(b)) => a.cmp(b),
        (Value::Str(a), Value::Char(c)) => a.as_bytes().cmp(&[*c][..]),
        (Value::Char(c), Value::Str(b)) => [*c][..].cmp(b.as_bytes()),
        (Value::Str(_), _) | (_, Value::Str(_)) => {
            // Strings never equal numbers.
            return Ok(matches!(op, BinaryOp::Ne));
        }
        (Value::Pattern(a), Value::Pattern(b)) => {
            let eq = std::rc::Rc::ptr_eq(a, b);
            return Ok(match op {
                BinaryOp::Eq => eq,
                BinaryOp::Ne => !eq,
                _ => false,
            });
        }
        (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
        _ => match promote(l, r)? {
            Num::U(a, b) => a.cmp(&b),
            Num::I(a, b) => a.cmp(&b),
            Num::F(a, b) => match a.partial_cmp(&b) {
                Some(o) => o,
                None => return Ok(matches!(op, BinaryOp::Ne)),
            },
        },
    };
    use std::cmp::Ordering::*;
    Ok(match op {
        BinaryOp::Eq => ord == Equal,
        BinaryOp::Ne => ord != Equal,
        BinaryOp::Lt => ord == Less,
        BinaryOp::Gt => ord == Greater,
        BinaryOp::Le => ord != Greater,
        BinaryOp::Ge => ord != Less,
        _ => unreachable!(),
    })
}

fn binary(op: BinaryOp, l: Value, r: Value) -> Result<Value> {
    use BinaryOp::*;
    match op {
        Eq | Ne | Lt | Gt | Le | Ge => return Ok(Value::Bool(compare(&l, &r, op)?)),
        And => return Ok(Value::Bool(l.as_bool()? && r.as_bool()?)),
        Or => return Ok(Value::Bool(l.as_bool()? || r.as_bool()?)),
        Xor => return Ok(Value::Bool(l.as_bool()? ^ r.as_bool()?)),
        _ => {}
    }
    // String operations.
    match (&l, &r) {
        (Value::Str(a), _) if op == Add => {
            let b = r.as_str()?;
            return Ok(Value::Str(format!("{}{}", a, b)));
        }
        (_, Value::Str(b)) if op == Add => {
            let a = l.as_str()?;
            return Ok(Value::Str(format!("{}{}", a, b)));
        }
        (Value::Str(a), _) if op == Mul => {
            let n = r.as_u64()?;
            return Ok(Value::Str(a.repeat(n as usize)));
        }
        (Value::Str(_), _) | (_, Value::Str(_)) => {
            return Err(Error::new(format!("operator {} is not defined for strings", op.symbol())));
        }
        _ => {}
    }
    match promote(&l, &r)? {
        Num::F(a, b) => Ok(Value::Float(match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => {
                if b == 0.0 {
                    return Err(Error::new("division by zero"));
                }
                a / b
            }
            Rem => {
                if b == 0.0 {
                    return Err(Error::new("division by zero"));
                }
                a % b
            }
            Shl | Shr | BitAnd | BitOr | BitXor => {
                let a = a as i128;
                let b = b as i128;
                return binary(op, Value::Signed(a), Value::Signed(b));
            }
            _ => unreachable!(),
        })),
        Num::I(a, b) => Ok(Value::Signed(match op {
            Add => a.wrapping_add(b),
            Sub => a.wrapping_sub(b),
            Mul => a.wrapping_mul(b),
            Div => {
                if b == 0 {
                    return Err(Error::new("division by zero"));
                }
                a.wrapping_div(b)
            }
            Rem => {
                if b == 0 {
                    return Err(Error::new("division by zero"));
                }
                a.wrapping_rem(b)
            }
            Shl => {
                if b < 0 || b >= 128 {
                    0
                } else {
                    a.wrapping_shl(b as u32)
                }
            }
            Shr => {
                if b < 0 || b >= 128 {
                    if a < 0 {
                        -1
                    } else {
                        0
                    }
                } else {
                    a >> b
                }
            }
            BitAnd => a & b,
            BitOr => a | b,
            BitXor => a ^ b,
            _ => unreachable!(),
        })),
        Num::U(a, b) => Ok(match op {
            Add => Value::Unsigned(a.wrapping_add(b)),
            Sub => {
                if b > a {
                    Value::Signed((a as i128).wrapping_sub(b as i128))
                } else {
                    Value::Unsigned(a - b)
                }
            }
            Mul => Value::Unsigned(a.wrapping_mul(b)),
            Div => {
                if b == 0 {
                    return Err(Error::new("division by zero"));
                }
                Value::Unsigned(a / b)
            }
            Rem => {
                if b == 0 {
                    return Err(Error::new("division by zero"));
                }
                Value::Unsigned(a % b)
            }
            Shl => Value::Unsigned(if b >= 128 { 0 } else { a.wrapping_shl(b as u32) }),
            Shr => Value::Unsigned(if b >= 128 { 0 } else { a >> b }),
            BitAnd => Value::Unsigned(a & b),
            BitOr => Value::Unsigned(a | b),
            BitXor => Value::Unsigned(a ^ b),
            _ => unreachable!(),
        }),
    }
}
