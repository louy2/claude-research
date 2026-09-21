//! Native implementations of the `builtin::std::*` functions that the ImHex
//! standard library wraps, registered under both the `builtin::std::...`
//! names (used by the real `std/*.pat` includes) and the plain `std::...`
//! names (so patterns work without an include directory).

use super::{BitfieldOrder, PatternKind, Runtime, Value};
use crate::ast::Endian;
use crate::error::{Error, Result};

fn arg<'a>(args: &'a [Value], i: usize, fname: &str) -> Result<&'a Value> {
    args.get(i).ok_or_else(|| Error::new(format!("{}: missing argument {}", fname, i + 1)))
}

fn scalar(rt: &mut Runtime, v: &Value) -> Result<Value> {
    match v {
        Value::Pattern(p) => rt.value_of(p),
        other => Ok(other.clone()),
    }
}

fn endian_arg(v: &Value, default: Endian) -> Result<Endian> {
    Ok(match v.as_u64()? {
        1 => Endian::Big,
        2 => Endian::Little,
        _ => default,
    })
}

macro_rules! reg {
    ($rt:expr, $name:expr, $f:expr) => {{
        let f: super::BuiltinFn = $f;
        $rt.builtins.insert(format!("builtin::{}", $name), f);
        $rt.builtins.insert($name.to_string(), f);
    }};
}

pub fn register(rt: &mut Runtime) {
    // ---- std ---------------------------------------------------------
    reg!(rt, "std::print", |rt, args| {
        let s = format_args(rt, &args, "std::print")?;
        if !rt.is_dry_run() {
            rt.console.push(s);
        }
        Ok(Value::Null)
    });
    reg!(rt, "std::format", |rt, args| Ok(Value::Str(format_args(rt, &args, "std::format")?)));
    reg!(rt, "std::error", |rt, args| {
        let msg = scalar(rt, arg(&args, 0, "std::error")?)?.as_str()?;
        Err(Error::new(msg))
    });
    reg!(rt, "std::warning", |rt, args| {
        let msg = scalar(rt, arg(&args, 0, "std::warning")?)?.as_str()?;
        if !rt.is_dry_run() {
            rt.warnings.push(msg);
        }
        Ok(Value::Null)
    });
    reg!(rt, "std::assert", |rt, args| {
        let cond = scalar(rt, arg(&args, 0, "std::assert")?)?.as_bool()?;
        if !cond {
            let msg = scalar(rt, arg(&args, 1, "std::assert")?)?.as_str()?;
            return Err(Error::new(format!("assertion failed '{}'", msg)));
        }
        Ok(Value::Null)
    });
    reg!(rt, "std::assert_warn", |rt, args| {
        let cond = scalar(rt, arg(&args, 0, "std::assert_warn")?)?.as_bool()?;
        if !cond {
            let msg = scalar(rt, arg(&args, 1, "std::assert_warn")?)?.as_str()?;
            if !rt.is_dry_run() {
                rt.warnings.push(format!("assertion failed '{}'", msg));
            }
        }
        Ok(Value::Null)
    });
    reg!(rt, "std::env", |rt, args| {
        let name = scalar(rt, arg(&args, 0, "std::env")?)?.as_str()?;
        Ok(rt.env_var(&name).unwrap_or(Value::Null))
    });
    reg!(rt, "std::sizeof_pack", |_rt, args| Ok(Value::Unsigned(args.len() as u128)));
    reg!(rt, "std::unimplemented", |_rt, _args| Err(Error::new("Unimplemented code path reached!")));

    // ---- std::mem ----------------------------------------------------
    reg!(rt, "std::mem::base_address", |_rt, _args| Ok(Value::Unsigned(0)));
    reg!(rt, "std::mem::size", |rt, _args| Ok(Value::Unsigned(rt.data_size() as u128)));
    reg!(rt, "std::mem::eof", |rt, _args| Ok(Value::Bool(rt.cursor >= rt.data_size())));
    reg!(rt, "std::mem::reached", |rt, args| {
        let a = scalar(rt, arg(&args, 0, "std::mem::reached")?)?.as_u64()?;
        Ok(Value::Bool(rt.cursor >= a))
    });
    reg!(rt, "std::mem::align_to", |rt, args| {
        let alignment = scalar(rt, arg(&args, 0, "std::mem::align_to")?)?.as_u128()?;
        let value = scalar(rt, arg(&args, 1, "std::mem::align_to")?)?.as_u128()?;
        if alignment == 0 {
            return Ok(Value::Unsigned(value));
        }
        let rem = value % alignment;
        Ok(Value::Unsigned(if rem != 0 { value + (alignment - rem) } else { value }))
    });
    reg!(rt, "std::mem::read_unsigned", |rt, args| {
        let addr = scalar(rt, arg(&args, 0, "std::mem::read_unsigned")?)?.as_u64()?;
        let size = scalar(rt, arg(&args, 1, "std::mem::read_unsigned")?)?.as_u64()?;
        let endian = match args.get(2) {
            Some(e) => endian_arg(&scalar(rt, e)?, rt.default_endian)?,
            None => rt.default_endian,
        };
        let reader = crate::reader::Reader::new(rt.section_data(rt.current_section));
        // Out-of-bounds reads yield zero, as in the reference runtime.
        Ok(Value::Unsigned(reader.unsigned(addr, size, endian).unwrap_or(0)))
    });
    reg!(rt, "std::mem::read_signed", |rt, args| {
        let addr = scalar(rt, arg(&args, 0, "std::mem::read_signed")?)?.as_u64()?;
        let size = scalar(rt, arg(&args, 1, "std::mem::read_signed")?)?.as_u64()?;
        let endian = match args.get(2) {
            Some(e) => endian_arg(&scalar(rt, e)?, rt.default_endian)?,
            None => rt.default_endian,
        };
        let reader = crate::reader::Reader::new(rt.section_data(rt.current_section));
        Ok(Value::Signed(reader.signed(addr, size, endian).unwrap_or(0)))
    });
    reg!(rt, "std::mem::read_string", |rt, args| {
        let addr = scalar(rt, arg(&args, 0, "std::mem::read_string")?)?.as_u64()?;
        let size = scalar(rt, arg(&args, 1, "std::mem::read_string")?)?.as_u64()?;
        let reader = crate::reader::Reader::new(rt.section_data(rt.current_section));
        let available = reader.len().saturating_sub(addr).min(size);
        let bytes = if available == 0 { &[][..] } else { reader.bytes(addr, available)? };
        Ok(Value::Str(crate::lower::bytes_to_string(bytes)))
    });
    reg!(rt, "std::mem::read_bits", |rt, args| {
        let byte_offset = scalar(rt, arg(&args, 0, "std::mem::read_bits")?)?.as_u64()?;
        let bit_offset = scalar(rt, arg(&args, 1, "std::mem::read_bits")?)?.as_u64()?;
        let bit_size = scalar(rt, arg(&args, 2, "std::mem::read_bits")?)?.as_u64()?;
        let reader = crate::reader::Reader::new(rt.section_data(rt.current_section));
        let nbytes = (bit_offset + bit_size).div_ceil(8).max(1);
        let bytes = reader.bytes(byte_offset, nbytes)?;
        let mut v: u128 = 0;
        for b in bytes.iter().rev() {
            v = (v << 8) | *b as u128;
        }
        let mask = if bit_size >= 128 { u128::MAX } else { (1u128 << bit_size) - 1 };
        Ok(Value::Unsigned((v >> bit_offset) & mask))
    });
    reg!(rt, "std::mem::find_sequence_in_range", |rt, args| {
        let occurrence = scalar(rt, arg(&args, 0, "find_sequence_in_range")?)?.as_u64()?;
        let from = scalar(rt, arg(&args, 1, "find_sequence_in_range")?)?.as_u64()?;
        let to = scalar(rt, arg(&args, 2, "find_sequence_in_range")?)?.as_u64()?;
        let mut needle = Vec::new();
        for a in &args[3..] {
            needle.push(scalar(rt, a)?.as_u128()? as u8);
        }
        Ok(find_in_range(rt, occurrence, from, to, &needle))
    });
    reg!(rt, "std::mem::find_string_in_range", |rt, args| {
        let occurrence = scalar(rt, arg(&args, 0, "find_string_in_range")?)?.as_u64()?;
        let from = scalar(rt, arg(&args, 1, "find_string_in_range")?)?.as_u64()?;
        let to = scalar(rt, arg(&args, 2, "find_string_in_range")?)?.as_u64()?;
        let needle = scalar(rt, arg(&args, 3, "find_string_in_range")?)?.as_str()?;
        Ok(find_in_range(rt, occurrence, from, to, needle.as_bytes()))
    });
    reg!(rt, "std::mem::find_sequence", |rt, args| {
        let occurrence = scalar(rt, arg(&args, 0, "find_sequence")?)?.as_u64()?;
        let mut needle = Vec::new();
        for a in &args[1..] {
            needle.push(scalar(rt, a)?.as_u128()? as u8);
        }
        let size = rt.data_size();
        Ok(find_in_range(rt, occurrence, 0, size, &needle))
    });
    reg!(rt, "std::mem::find_string", |rt, args| {
        let occurrence = scalar(rt, arg(&args, 0, "find_string")?)?.as_u64()?;
        let needle = scalar(rt, arg(&args, 1, "find_string")?)?.as_str()?;
        let size = rt.data_size();
        Ok(find_in_range(rt, occurrence, 0, size, needle.as_bytes()))
    });
    reg!(rt, "std::mem::create_section", |rt, _args| Ok(Value::Unsigned(rt.create_section_buffer(0) as u128)));
    reg!(rt, "std::mem::delete_section", |_rt, _args| Ok(Value::Null));
    reg!(rt, "std::mem::get_section_size", |rt, args| {
        let id = scalar(rt, arg(&args, 0, "get_section_size")?)?.as_u64()? as usize;
        Ok(Value::Unsigned(rt.section_data(id).len() as u128))
    });
    reg!(rt, "std::mem::set_section_size", |rt, args| {
        let id = scalar(rt, arg(&args, 0, "set_section_size")?)?.as_u64()? as usize;
        let size = scalar(rt, arg(&args, 1, "set_section_size")?)?.as_u64()?;
        rt.resize_section(id, size)?;
        Ok(Value::Null)
    });
    reg!(rt, "std::mem::copy_section_to_section", |rt, args| {
        let from = scalar(rt, arg(&args, 0, "copy_section_to_section")?)?.as_u64()? as usize;
        let from_addr = scalar(rt, arg(&args, 1, "copy_section_to_section")?)?.as_u64()? as usize;
        let to = scalar(rt, arg(&args, 2, "copy_section_to_section")?)?.as_u64()? as usize;
        let to_addr = scalar(rt, arg(&args, 3, "copy_section_to_section")?)?.as_u64()? as usize;
        let size = scalar(rt, arg(&args, 4, "copy_section_to_section")?)?.as_u64()? as usize;
        let src = rt.section_data(from).get(from_addr..from_addr + size).ok_or_else(|| Error::new("copy source out of bounds"))?.to_vec();
        let dst = rt.section_mut(to)?;
        if dst.len() < to_addr + size {
            dst.resize(to_addr + size, 0);
        }
        dst[to_addr..to_addr + size].copy_from_slice(&src);
        Ok(Value::Null)
    });
    reg!(rt, "std::mem::copy_value_to_section", |rt, args| {
        let value = arg(&args, 0, "copy_value_to_section")?.clone();
        let to = scalar(rt, arg(&args, 1, "copy_value_to_section")?)?.as_u64()? as usize;
        let to_addr = scalar(rt, arg(&args, 2, "copy_value_to_section")?)?.as_u64()? as usize;
        let bytes: Vec<u8> = match &value {
            Value::Pattern(p) => {
                let pb = p.borrow();
                rt.section_data(pb.section).get(pb.offset as usize..(pb.offset + pb.size) as usize).ok_or_else(|| Error::new("pattern out of bounds"))?.to_vec()
            }
            Value::Str(s) => s.as_bytes().to_vec(),
            other => other.as_u128()?.to_le_bytes().to_vec(),
        };
        let dst = rt.section_mut(to)?;
        if dst.len() < to_addr + bytes.len() {
            dst.resize(to_addr + bytes.len(), 0);
        }
        dst[to_addr..to_addr + bytes.len()].copy_from_slice(&bytes);
        Ok(Value::Null)
    });
    reg!(rt, "std::mem::current_bit_offset", |rt, _args| Ok(Value::Unsigned(rt.bit_cursor() as u128)));

    // ---- std::core ---------------------------------------------------
    reg!(rt, "std::core::array_index", |rt, _args| Ok(Value::Unsigned(rt.array_index().unwrap_or(0) as u128)));
    reg!(rt, "std::core::member_count", |_rt, args| {
        let p = arg(&args, 0, "member_count")?.as_pattern()?;
        let n = p.borrow().entry_count().unwrap_or_else(|| p.borrow().members().len() as u64);
        Ok(Value::Unsigned(n as u128))
    });
    reg!(rt, "std::core::has_member", |rt, args| {
        let p = arg(&args, 0, "has_member")?.as_pattern()?;
        let name = scalar(rt, arg(&args, 1, "has_member")?)?.as_str()?;
        let found = p.borrow().member(&name).is_some();
        Ok(Value::Bool(found))
    });
    reg!(rt, "std::core::formatted_value", |rt, args| {
        let p = arg(&args, 0, "formatted_value")?.as_pattern()?;
        Ok(Value::Str(rt.formatted_value(&p)?))
    });
    reg!(rt, "std::core::is_valid_enum", |rt, args| {
        let p = arg(&args, 0, "is_valid_enum")?.as_pattern()?;
        let entries = match &p.borrow().kind {
            PatternKind::Enum { entries, .. } => entries.clone(),
            _ => return Ok(Value::Bool(false)),
        };
        let v = rt.pattern_value(&p)?.as_i128()?;
        Ok(Value::Bool(entries.iter().any(|(_, a, b)| v >= *a && v <= *b)))
    });
    reg!(rt, "std::core::has_attribute", |rt, args| {
        let p = arg(&args, 0, "has_attribute")?.as_pattern()?;
        let name = scalar(rt, arg(&args, 1, "has_attribute")?)?.as_str()?;
        let found = p.borrow().has_attribute(&name);
        Ok(Value::Bool(found))
    });
    reg!(rt, "std::core::get_attribute_argument", |rt, args| {
        let p = arg(&args, 0, "get_attribute_argument")?.as_pattern()?;
        let name = scalar(rt, arg(&args, 1, "get_attribute_argument")?)?.as_str()?;
        let idx = match args.get(2) {
            Some(v) => scalar(rt, v)?.as_u64()? as usize,
            None => 0,
        };
        let pb = p.borrow();
        Ok(pb.attributes.iter().find(|(n, _)| *n == name).and_then(|(_, a)| a.get(idx).cloned()).unwrap_or(Value::Null))
    });
    reg!(rt, "std::core::get_attribute_value", |rt, args| {
        let p = arg(&args, 0, "get_attribute_value")?.as_pattern()?;
        let name = scalar(rt, arg(&args, 1, "get_attribute_value")?)?.as_str()?;
        let pb = p.borrow();
        Ok(pb.attributes.iter().find(|(n, _)| *n == name).and_then(|(_, a)| a.first().cloned()).unwrap_or(Value::Null))
    });
    reg!(rt, "std::core::set_pattern_color", |rt, args| {
        let p = arg(&args, 0, "set_pattern_color")?.as_pattern()?;
        let c = scalar(rt, arg(&args, 1, "set_pattern_color")?)?.as_u64()? as u32;
        p.borrow_mut().color = Some(c);
        Ok(Value::Null)
    });
    reg!(rt, "std::core::set_display_name", |rt, args| {
        let p = arg(&args, 0, "set_display_name")?.as_pattern()?;
        let n = scalar(rt, arg(&args, 1, "set_display_name")?)?.as_str()?;
        p.borrow_mut().display_name = Some(n);
        Ok(Value::Null)
    });
    reg!(rt, "std::core::set_pattern_comment", |rt, args| {
        let p = arg(&args, 0, "set_pattern_comment")?.as_pattern()?;
        let n = scalar(rt, arg(&args, 1, "set_pattern_comment")?)?.as_str()?;
        p.borrow_mut().comment = Some(n);
        Ok(Value::Null)
    });
    reg!(rt, "std::core::set_endian", |rt, args| {
        let e = endian_arg(&scalar(rt, arg(&args, 0, "set_endian")?)?, rt.default_endian)?;
        rt.default_endian = e;
        Ok(Value::Null)
    });
    reg!(rt, "std::core::get_endian", |rt, _args| Ok(Value::Unsigned(match rt.default_endian {
        Endian::Big => 1,
        Endian::Little => 2,
    })));
    reg!(rt, "std::core::set_bitfield_order", |rt, args| {
        let o = scalar(rt, arg(&args, 0, "set_bitfield_order")?)?.as_u64()?;
        rt.set_bitfield_order(if o == 1 { BitfieldOrder::MostToLeast } else { BitfieldOrder::LeastToMost });
        Ok(Value::Null)
    });
    reg!(rt, "std::core::get_bitfield_order", |rt, _args| Ok(Value::Unsigned(match rt.bitfield_order() {
        BitfieldOrder::MostToLeast => 1,
        BitfieldOrder::LeastToMost => 2,
    })));
    reg!(rt, "std::core::execute_function", |rt, args| {
        let name = scalar(rt, arg(&args, 0, "execute_function")?)?.as_str()?;
        let path: Vec<String> = name.split("::").map(|s| s.to_string()).collect();
        rt.call_function(&path, args[1..].to_vec())
    });
    reg!(rt, "std::core::set_pattern_palette_colors", |_rt, _args| Ok(Value::Null));
    reg!(rt, "std::core::reset_pattern_palette", |_rt, _args| Ok(Value::Null));

    // ---- std::math ---------------------------------------------------
    macro_rules! math1 {
        ($name:expr, $f:expr) => {
            reg!(rt, $name, |rt, args| {
                let x = scalar(rt, arg(&args, 0, $name)?)?.as_f64()?;
                let f: fn(f64) -> f64 = $f;
                Ok(Value::Float(f(x)))
            });
        };
    }
    math1!("std::math::floor", f64::floor);
    math1!("std::math::ceil", f64::ceil);
    math1!("std::math::round", f64::round);
    math1!("std::math::trunc", f64::trunc);
    math1!("std::math::log10", f64::log10);
    math1!("std::math::log2", f64::log2);
    math1!("std::math::ln", f64::ln);
    math1!("std::math::exp", f64::exp);
    math1!("std::math::sqrt", f64::sqrt);
    math1!("std::math::cbrt", f64::cbrt);
    math1!("std::math::sin", f64::sin);
    math1!("std::math::cos", f64::cos);
    math1!("std::math::tan", f64::tan);
    math1!("std::math::asin", f64::asin);
    math1!("std::math::acos", f64::acos);
    math1!("std::math::atan", f64::atan);
    math1!("std::math::sinh", f64::sinh);
    math1!("std::math::cosh", f64::cosh);
    math1!("std::math::tanh", f64::tanh);
    math1!("std::math::asinh", f64::asinh);
    math1!("std::math::acosh", f64::acosh);
    math1!("std::math::atanh", f64::atanh);
    reg!(rt, "std::math::pow", |rt, args| {
        let b = scalar(rt, arg(&args, 0, "pow")?)?;
        let e = scalar(rt, arg(&args, 1, "pow")?)?;
        if !b.is_float() && !e.is_float() {
            let base = b.as_i128()?;
            let exp = e.as_i128()?;
            if exp >= 0 {
                let r = (base as i128).checked_pow(exp as u32).ok_or_else(|| Error::new("pow overflow"))?;
                return Ok(if b.is_signed() || e.is_signed() { Value::Signed(r) } else { Value::Unsigned(r as u128) });
            }
        }
        Ok(Value::Float(b.as_f64()?.powf(e.as_f64()?)))
    });
    reg!(rt, "std::math::fmod", |rt, args| {
        let a = scalar(rt, arg(&args, 0, "fmod")?)?.as_f64()?;
        let b = scalar(rt, arg(&args, 1, "fmod")?)?.as_f64()?;
        Ok(Value::Float(a % b))
    });
    reg!(rt, "std::math::atan2", |rt, args| {
        let a = scalar(rt, arg(&args, 0, "atan2")?)?.as_f64()?;
        let b = scalar(rt, arg(&args, 1, "atan2")?)?.as_f64()?;
        Ok(Value::Float(a.atan2(b)))
    });
    reg!(rt, "std::math::min", |rt, args| {
        let a = scalar(rt, arg(&args, 0, "min")?)?;
        let b = scalar(rt, arg(&args, 1, "min")?)?;
        Ok(if super::expr::compare(&a, &b, crate::ast::BinaryOp::Le)? { a } else { b })
    });
    reg!(rt, "std::math::max", |rt, args| {
        let a = scalar(rt, arg(&args, 0, "max")?)?;
        let b = scalar(rt, arg(&args, 1, "max")?)?;
        Ok(if super::expr::compare(&a, &b, crate::ast::BinaryOp::Ge)? { a } else { b })
    });
    reg!(rt, "std::math::clamp", |rt, args| {
        let x = scalar(rt, arg(&args, 0, "clamp")?)?;
        let lo = scalar(rt, arg(&args, 1, "clamp")?)?;
        let hi = scalar(rt, arg(&args, 2, "clamp")?)?;
        if super::expr::compare(&x, &lo, crate::ast::BinaryOp::Lt)? {
            return Ok(lo);
        }
        if super::expr::compare(&x, &hi, crate::ast::BinaryOp::Gt)? {
            return Ok(hi);
        }
        Ok(x)
    });
    reg!(rt, "std::math::abs", |rt, args| {
        let x = scalar(rt, arg(&args, 0, "abs")?)?;
        Ok(match x {
            Value::Float(f) => Value::Float(f.abs()),
            Value::Signed(i) => Value::Signed(i.abs()),
            other => other,
        })
    });
    reg!(rt, "std::math::sign", |rt, args| {
        let x = scalar(rt, arg(&args, 0, "sign")?)?;
        Ok(Value::Signed(match x {
            Value::Float(f) => f.signum() as i128,
            other => other.as_i128()?.signum(),
        }))
    });
    reg!(rt, "std::math::factorial", |rt, args| {
        let n = scalar(rt, arg(&args, 0, "factorial")?)?.as_u64()?;
        let mut r: u128 = 1;
        for i in 2..=n {
            r = r.checked_mul(i as u128).ok_or_else(|| Error::new("factorial overflow"))?;
        }
        Ok(Value::Unsigned(r))
    });

    // ---- std::string -------------------------------------------------
    reg!(rt, "std::string::length", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "length")?)?.as_str()?;
        Ok(Value::Unsigned(s.chars().count() as u128))
    });
    reg!(rt, "std::string::at", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "at")?)?.as_str()?;
        let i = scalar(rt, arg(&args, 1, "at")?)?.as_u64()? as usize;
        let b = s.as_bytes().get(i).copied().ok_or_else(|| Error::new("std::string::at: index out of range"))?;
        Ok(Value::Char(b))
    });
    reg!(rt, "std::string::substr", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "substr")?)?.as_str()?;
        let pos = scalar(rt, arg(&args, 1, "substr")?)?.as_u64()? as usize;
        let count = scalar(rt, arg(&args, 2, "substr")?)?.as_u64()? as usize;
        let chars: Vec<char> = s.chars().collect();
        if pos > chars.len() {
            return Err(Error::new("std::string::substr: position out of range"));
        }
        Ok(Value::Str(chars[pos..(pos + count).min(chars.len())].iter().collect()))
    });
    reg!(rt, "std::string::parse_int", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "parse_int")?)?.as_str()?;
        let base = scalar(rt, arg(&args, 1, "parse_int")?)?.as_u64()? as u32;
        let t = s.trim_matches(|c: char| c == '\0' || c.is_whitespace());
        let t = t.trim_start_matches('0');
        if t.is_empty() {
            return Ok(Value::Signed(0));
        }
        let (neg, digits) = match t.strip_prefix('-') {
            Some(d) => (true, d),
            None => (false, t),
        };
        let end = digits.find(|c: char| !c.is_digit(base)).unwrap_or(digits.len());
        let v = i128::from_str_radix(&digits[..end], base).unwrap_or(0);
        Ok(Value::Signed(if neg { -v } else { v }))
    });
    reg!(rt, "std::string::parse_float", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "parse_float")?)?.as_str()?;
        Ok(Value::Float(s.trim().parse::<f64>().unwrap_or(0.0)))
    });
    reg!(rt, "std::string::to_string", |rt, args| {
        let v = scalar(rt, arg(&args, 0, "to_string")?)?;
        Ok(Value::Str(v.as_str()?))
    });
    reg!(rt, "std::string::starts_with", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "starts_with")?)?.as_str()?;
        let p = scalar(rt, arg(&args, 1, "starts_with")?)?.as_str()?;
        Ok(Value::Bool(s.starts_with(&p)))
    });
    reg!(rt, "std::string::ends_with", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "ends_with")?)?.as_str()?;
        let p = scalar(rt, arg(&args, 1, "ends_with")?)?.as_str()?;
        Ok(Value::Bool(s.ends_with(&p)))
    });
    reg!(rt, "std::string::contains", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "contains")?)?.as_str()?;
        let p = scalar(rt, arg(&args, 1, "contains")?)?.as_str()?;
        Ok(Value::Bool(s.contains(&p)))
    });
    reg!(rt, "std::string::reverse", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "reverse")?)?.as_str()?;
        Ok(Value::Str(s.chars().rev().collect()))
    });
    reg!(rt, "std::string::to_upper", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "to_upper")?)?.as_str()?;
        Ok(Value::Str(s.to_uppercase()))
    });
    reg!(rt, "std::string::to_lower", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "to_lower")?)?.as_str()?;
        Ok(Value::Str(s.to_lowercase()))
    });
    reg!(rt, "std::string::replace", |rt, args| {
        let s = scalar(rt, arg(&args, 0, "replace")?)?.as_str()?;
        let p = scalar(rt, arg(&args, 1, "replace")?)?.as_str()?;
        let r = scalar(rt, arg(&args, 2, "replace")?)?.as_str()?;
        Ok(Value::Str(s.replace(&p, &r)))
    });

    // ---- std::ctype --------------------------------------------------
    macro_rules! ctype {
        ($name:expr, $f:expr) => {
            reg!(rt, $name, |rt, args| {
                let c = scalar(rt, arg(&args, 0, $name)?)?;
                let b = match c {
                    Value::Char(c) => c,
                    Value::Str(s) => s.as_bytes().first().copied().unwrap_or(0),
                    other => other.as_u128()? as u8,
                };
                let f: fn(u8) -> bool = $f;
                Ok(Value::Bool(f(b)))
            });
        };
    }
    ctype!("std::ctype::isprint", |b| (0x20..0x7F).contains(&b));
    ctype!("std::ctype::isdigit", |b| b.is_ascii_digit());
    ctype!("std::ctype::isalpha", |b| b.is_ascii_alphabetic());
    ctype!("std::ctype::isalnum", |b| b.is_ascii_alphanumeric());
    ctype!("std::ctype::isspace", |b| b.is_ascii_whitespace());
    ctype!("std::ctype::isupper", |b| b.is_ascii_uppercase());
    ctype!("std::ctype::islower", |b| b.is_ascii_lowercase());
    ctype!("std::ctype::isxdigit", |b| b.is_ascii_hexdigit());
    ctype!("std::ctype::ispunct", |b| b.is_ascii_punctuation());
    ctype!("std::ctype::iscntrl", |b| b.is_ascii_control());
    ctype!("std::ctype::isgraph", |b| b.is_ascii_graphic());
    ctype!("std::ctype::isblank", |b| b == b' ' || b == b'\t');

    // ---- std::time ---------------------------------------------------
    reg!(rt, "std::time::epoch", |_rt, _args| {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        Ok(Value::Unsigned(now as u128))
    });
    reg!(rt, "std::time::to_utc", |rt, args| {
        let t = scalar(rt, arg(&args, 0, "to_utc")?)?.as_i128()?;
        Ok(Value::Unsigned(pack_time(civil_from_epoch(t as i64))))
    });
    reg!(rt, "std::time::to_local", |rt, args| {
        let t = scalar(rt, arg(&args, 0, "to_local")?)?.as_i128()?;
        Ok(Value::Unsigned(pack_time(civil_from_epoch(t as i64))))
    });
    reg!(rt, "std::time::format", |rt, args| {
        let fmt = scalar(rt, arg(&args, 0, "time::format")?)?.as_str()?;
        let packed = scalar(rt, arg(&args, 1, "time::format")?)?.as_u128()?;
        Ok(Value::Str(strftime(&fmt, &unpack_time(packed))))
    });
    reg!(rt, "std::time::to_epoch", |rt, args| {
        let packed = scalar(rt, arg(&args, 0, "to_epoch")?)?.as_u128()?;
        Ok(Value::Signed(epoch_from_civil(&unpack_time(packed)) as i128))
    });

    // ---- std::hash ---------------------------------------------------
    reg!(rt, "std::hash::crc32", |rt, args| {
        let p = arg(&args, 0, "crc32")?.as_pattern()?;
        let init = scalar(rt, arg(&args, 1, "crc32")?)?.as_u128()? as u32;
        let poly = scalar(rt, arg(&args, 2, "crc32")?)?.as_u128()? as u32;
        let xorout = scalar(rt, arg(&args, 3, "crc32")?)?.as_u128()? as u32;
        let refl_in = scalar(rt, arg(&args, 4, "crc32")?)?.as_bool()?;
        let refl_out = scalar(rt, arg(&args, 5, "crc32")?)?.as_bool()?;
        let pb = p.borrow();
        let data = rt.section_data(pb.section).get(pb.offset as usize..(pb.offset + pb.size) as usize).unwrap_or(&[]).to_vec();
        Ok(Value::Unsigned(crc32(&data, init, poly, xorout, refl_in, refl_out) as u128))
    });
    reg!(rt, "std::random::generate", |_rt, _args| Ok(Value::Unsigned(4)));
    reg!(rt, "std::random::set_seed", |_rt, _args| Ok(Value::Null));
}

fn format_args(rt: &mut Runtime, args: &[Value], fname: &str) -> Result<String> {
    let fmt = arg(args, 0, fname)?;
    let fmt = scalar(rt, fmt)?;
    let mut rest = Vec::with_capacity(args.len().saturating_sub(1));
    for a in &args[1..] {
        rest.push(match a {
            Value::Pattern(p) => {
                let pb = p.borrow();
                let is_simple = pb.kind.is_scalar()
                    || matches!(pb.kind, PatternKind::String | PatternKind::WideString | PatternKind::BitfieldField { .. } | PatternKind::Pointer { .. });
                drop(pb);
                if is_simple {
                    rt.pattern_value(p)?
                } else {
                    Value::Str(rt.formatted_value(p)?)
                }
            }
            other => other.clone(),
        });
    }
    match fmt {
        Value::Str(s) => super::format::format(&s, &rest),
        other => Ok(other.to_display_string()),
    }
}

fn find_in_range(rt: &Runtime, occurrence: u64, from: u64, to: u64, needle: &[u8]) -> Value {
    let data = rt.section_data(rt.current_section);
    let to = to.min(data.len() as u64) as usize;
    let from = from.min(to as u64) as usize;
    if needle.is_empty() || to - from < needle.len() {
        return Value::Signed(-1);
    }
    let mut seen = 0u64;
    let hay = &data[from..to];
    let mut i = 0;
    while i + needle.len() <= hay.len() {
        if &hay[i..i + needle.len()] == needle {
            if seen == occurrence {
                return Value::Signed((from + i) as i128);
            }
            seen += 1;
        }
        i += 1;
    }
    Value::Signed(-1)
}

/// Broken-down time matching `std::time::Time` (packed as the `TimeConverter`
/// union layout: sec, min, hour, mday, mon, year(s16), wday, yday(u16), isdst).
#[derive(Debug, Clone, Copy, Default)]
pub struct Civil {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub mday: u8,
    pub mon: u8,
    pub year: i16,
    pub wday: u8,
    pub yday: u16,
}

pub fn civil_from_epoch(t: i64) -> Civil {
    let days = t.div_euclid(86400);
    let rem = t.rem_euclid(86400);
    // Howard Hinnant's civil_from_days.
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let wday = (days + 4).rem_euclid(7);
    let jan1 = days_from_civil(y, 1, 1);
    Civil {
        sec: (rem % 60) as u8,
        min: ((rem / 60) % 60) as u8,
        hour: (rem / 3600) as u8,
        mday: d as u8,
        mon: (m - 1) as u8,
        year: (y - 1900) as i16,
        wday: wday as u8,
        yday: (days - jan1) as u16,
    }
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn epoch_from_civil(c: &Civil) -> i64 {
    days_from_civil(c.year as i64 + 1900, c.mon as i64 + 1, c.mday as i64) * 86400 + c.hour as i64 * 3600 + c.min as i64 * 60 + c.sec as i64
}

pub fn pack_time(c: Civil) -> u128 {
    let mut b = [0u8; 16];
    b[0] = c.sec;
    b[1] = c.min;
    b[2] = c.hour;
    b[3] = c.mday;
    b[4] = c.mon;
    b[5..7].copy_from_slice(&c.year.to_le_bytes());
    b[7] = c.wday;
    b[8..10].copy_from_slice(&c.yday.to_le_bytes());
    u128::from_le_bytes(b)
}

pub fn unpack_time(v: u128) -> Civil {
    let b = v.to_le_bytes();
    Civil {
        sec: b[0],
        min: b[1],
        hour: b[2],
        mday: b[3],
        mon: b[4],
        year: i16::from_le_bytes([b[5], b[6]]),
        wday: b[7],
        yday: u16::from_le_bytes([b[8], b[9]]),
    }
}

const WDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

pub fn strftime(fmt: &str, c: &Civil) -> String {
    let mut out = String::new();
    let mut chars = fmt.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        let Some(spec) = chars.next() else { break };
        let year = c.year as i64 + 1900;
        match spec {
            'Y' => out.push_str(&year.to_string()),
            'y' => out.push_str(&format!("{:02}", year.rem_euclid(100))),
            'm' => out.push_str(&format!("{:02}", c.mon + 1)),
            'd' => out.push_str(&format!("{:02}", c.mday)),
            'e' => out.push_str(&format!("{:2}", c.mday)),
            'H' => out.push_str(&format!("{:02}", c.hour)),
            'M' => out.push_str(&format!("{:02}", c.min)),
            'S' => out.push_str(&format!("{:02}", c.sec)),
            'j' => out.push_str(&format!("{:03}", c.yday + 1)),
            'a' => out.push_str(WDAYS[(c.wday % 7) as usize]),
            'b' | 'h' => out.push_str(MONTHS[(c.mon % 12) as usize]),
            'F' => out.push_str(&format!("{}-{:02}-{:02}", year, c.mon + 1, c.mday)),
            'T' => out.push_str(&format!("{:02}:{:02}:{:02}", c.hour, c.min, c.sec)),
            'D' => out.push_str(&format!("{:02}/{:02}/{:02}", c.mon + 1, c.mday, year.rem_euclid(100))),
            'c' => out.push_str(&format!(
                "{} {} {:2} {:02}:{:02}:{:02} {}",
                WDAYS[(c.wday % 7) as usize],
                MONTHS[(c.mon % 12) as usize],
                c.mday,
                c.hour,
                c.min,
                c.sec,
                year
            )),
            'Z' => out.push_str("UTC"),
            'z' => out.push_str("+0000"),
            '%' => out.push('%'),
            other => {
                out.push('%');
                out.push(other);
            }
        }
    }
    out
}

fn reflect(mut v: u32, bits: u32) -> u32 {
    let mut r = 0u32;
    for _ in 0..bits {
        r = (r << 1) | (v & 1);
        v >>= 1;
    }
    r
}

pub fn crc32(data: &[u8], init: u32, poly: u32, xorout: u32, refl_in: bool, refl_out: bool) -> u32 {
    let mut crc = init;
    for &b in data {
        let byte = if refl_in { reflect(b as u32, 8) as u8 } else { b };
        crc ^= (byte as u32) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 { (crc << 1) ^ poly } else { crc << 1 };
        }
    }
    if refl_out {
        crc = reflect(crc, 32);
    }
    crc ^ xorout
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_ieee() {
        assert_eq!(crc32(b"123456789", 0xFFFF_FFFF, 0x04C1_1DB7, 0xFFFF_FFFF, true, true), 0xCBF4_3926);
    }

    #[test]
    fn civil_round_trip() {
        let c = civil_from_epoch(1_700_000_000);
        assert_eq!((c.year as i64 + 1900, c.mon + 1, c.mday, c.hour, c.min, c.sec), (2023, 11, 14, 22, 13, 20));
        assert_eq!(epoch_from_civil(&c), 1_700_000_000);
        assert_eq!(unpack_time(pack_time(c)).yday, c.yday);
    }
}
