//! The Thrift compact protocol, restricted to its canonical encoding.
//!
//! A value is parsed from an absolute position of a byte sequence. The
//! profile accepts exactly one encoding per value:
//!
//! - varints are minimal (no trailing zero groups) and fit in 64 bits;
//!   `i16`/`i32`/`i64` are zigzag varints in their range;
//! - a field header uses the short form (delta in the high nibble) exactly
//!   when the id delta is 1..=15, and field ids strictly increase;
//! - a list header uses the short form exactly when the size is below 15;
//! - booleans in a list are the bytes 1 (true) and 2 (false);
//! - sets and maps are not accepted (Parquet does not use them).
//!
//! The list element type is kept in the value (writers use 1 or 2 for
//! lists of booleans, and fastparquet 0 for empty lists), so it is not a
//! source of malleability either.
//!
//! Nesting is bounded by a depth `d`: a struct or list at depth `d` holds
//! values at depth `d - 1`, and nothing nests at depth 0.
//!
//! [`lemma_value_ps`] proves that the serializer reproduces what the parser
//! consumed; [`theorem_thrift_non_malleable`] is the corollary.
use vest_lib::core::exec::bytes_eq;
use vstd::prelude::*;
use vstd::slice::slice_subrange;

verus! {

pub open spec fn u64_limit() -> int {
    0x1_0000_0000_0000_0000
}

/// Lengths and list sizes are varint32 in Thrift.
pub open spec fn len_limit() -> int {
    0x8000_0000
}

pub enum TVal {
    Bool(bool),
    Byte(u8),
    I16(int),
    I32(int),
    I64(int),
    Double(Seq<u8>),
    Binary(Seq<u8>),
    List(u8, Seq<TVal>),
    Struct(Seq<TField>),
}

pub struct TField {
    pub id: int,
    pub val: TVal,
}

// ============================================================
// Varints and zigzag
// ============================================================

/// A minimal unsigned LEB128 varint of at most `k` bytes, below 2^64.
pub open spec fn spec_varint(s: Seq<u8>, i: int, k: nat) -> Option<(int, nat)>
    decreases k,
{
    if k == 0 || i < 0 || i >= s.len() {
        None
    } else if s[i] < 128 {
        Some((i + 1, s[i] as nat))
    } else {
        match spec_varint(s, i + 1, (k - 1) as nat) {
            Some((j, r)) => {
                let v = (s[i] - 128) + 128 * r;
                if r == 0 || v >= u64_limit() {
                    None
                } else {
                    Some((j, v as nat))
                }
            },
            None => None,
        }
    }
}

pub open spec fn ser_varint(v: nat) -> Seq<u8>
    decreases v,
{
    if v < 128 {
        seq![v as u8]
    } else {
        seq![(v % 128 + 128) as u8] + ser_varint(v / 128)
    }
}

pub open spec fn unzz(u: nat) -> int {
    if u % 2 == 0 {
        (u / 2) as int
    } else {
        -((u / 2) as int) - 1
    }
}

pub open spec fn zz(x: int) -> nat {
    if x >= 0 {
        (2 * x) as nat
    } else {
        (-2 * x - 1) as nat
    }
}

pub proof fn lemma_zz_unzz(u: nat)
    ensures
        zz(unzz(u)) == u,
{
}

pub proof fn lemma_varint_ps(s: Seq<u8>, i: int, k: nat)
    ensures
        spec_varint(s, i, k) matches Some((j, v)) ==> {
            &&& 0 <= i < j <= s.len()
            &&& v < u64_limit()
            &&& ser_varint(v) == s.subrange(i, j)
        },
    decreases k,
{
    if k > 0 && 0 <= i < s.len() && s[i] >= 128 {
        lemma_varint_ps(s, i + 1, (k - 1) as nat);
        if let Some((j, r)) = spec_varint(s, i + 1, (k - 1) as nat) {
            let a = s[i] - 128;
            let v = a + 128 * r;
            if r != 0 && v < u64_limit() {
                assert(v % 128 == a && v / 128 == r) by (nonlinear_arith)
                    requires
                        0 <= a < 128,
                        v == a + 128 * r,
                ;
                assert(ser_varint(v as nat) =~= seq![s[i]] + ser_varint(r));
                assert(s.subrange(i, j) =~= seq![s[i]] + s.subrange(i + 1, j));
            }
        }
    } else if k > 0 && 0 <= i < s.len() {
        assert(s.subrange(i, i + 1) =~= seq![s[i]]);
    }
}

// ============================================================
// Parser
// ============================================================

/// Bounds of the zigzag value of `i16`, `i32`, `i64` (types 4, 5, 6).
pub open spec fn int_limit(t: int) -> int {
    if t == 4 {
        0x1_0000
    } else if t == 5 {
        0x1_0000_0000
    } else {
        u64_limit()
    }
}

pub open spec fn int_val(t: int, x: int) -> TVal {
    if t == 4 {
        TVal::I16(x)
    } else if t == 5 {
        TVal::I32(x)
    } else {
        TVal::I64(x)
    }
}

/// Element types a list may declare: booleans (1 or 2), byte to binary,
/// list, struct; 0 only for an empty list.
pub open spec fn elem_type_ok(et: int, n: int) -> bool {
    (1 <= et <= 9) || et == 12 || (et == 0 && n == 0)
}

/// The compact type code a value is written with in a field header.
pub open spec fn tcode(v: TVal) -> int {
    match v {
        TVal::Bool(b) => if b {
            1
        } else {
            2
        },
        TVal::Byte(_) => 3,
        TVal::I16(_) => 4,
        TVal::I32(_) => 5,
        TVal::I64(_) => 6,
        TVal::Double(_) => 7,
        TVal::Binary(_) => 8,
        TVal::List(_, _) => 9,
        TVal::Struct(_) => 12,
    }
}

/// The size of a list from its header at `i`: the high nibble, or a
/// varint after it when the nibble is 15 (and then the size is at least 15).
pub open spec fn spec_list_size(s: Seq<u8>, i: int) -> Option<(int, nat)> {
    if i < 0 || i >= s.len() {
        None
    } else if s[i] / 16 == 15 {
        match spec_varint(s, i + 1, 10) {
            Some((j, n)) => if 15 <= n < len_limit() {
                Some((j, n))
            } else {
                None
            },
            None => None,
        }
    } else {
        Some((i + 1, (s[i] / 16) as nat))
    }
}

/// A value of compact type `t` at position `i`. Types 1 and 2 are list
/// elements of type bool (a byte 1 or 2); in a field, booleans live in the
/// field header and are handled by [`spec_fields`].
pub open spec fn spec_value(s: Seq<u8>, i: int, t: int, d: nat) -> Option<(int, TVal)>
    decreases d, s.len() - i, 0nat,
{
    if i < 0 || i >= s.len() {
        None
    } else if t == 1 || t == 2 {
        if s[i] == 1 || s[i] == 2 {
            Some((i + 1, TVal::Bool(s[i] == 1)))
        } else {
            None
        }
    } else if t == 3 {
        Some((i + 1, TVal::Byte(s[i])))
    } else if t == 4 || t == 5 || t == 6 {
        match spec_varint(s, i, 10) {
            Some((j, u)) => if u < int_limit(t) {
                Some((j, int_val(t, unzz(u))))
            } else {
                None
            },
            None => None,
        }
    } else if t == 7 {
        if i + 8 <= s.len() {
            Some((i + 8, TVal::Double(s.subrange(i, i + 8))))
        } else {
            None
        }
    } else if t == 8 {
        match spec_varint(s, i, 10) {
            Some((j, n)) => if n < len_limit() && j + n <= s.len() {
                Some((j + n, TVal::Binary(s.subrange(j, j + n))))
            } else {
                None
            },
            None => None,
        }
    } else if t == 9 {
        if d == 0 {
            None
        } else {
            let et = (s[i] % 16) as int;
            match spec_list_size(s, i) {
                None => None,
                Some((j, n)) => if !elem_type_ok(et, n as int) {
                    None
                } else {
                    match spec_elems(s, j, et, n, (d - 1) as nat) {
                        Some((e, xs)) => Some((e, TVal::List(et as u8, xs))),
                        None => None,
                    }
                },
            }
        }
    } else if t == 12 {
        if d == 0 {
            None
        } else {
            match spec_fields(s, i, 0, (d - 1) as nat) {
                Some((e, fs)) => Some((e, TVal::Struct(fs))),
                None => None,
            }
        }
    } else {
        None
    }
}

/// `n` list elements of type `et` at position `i`.
pub open spec fn spec_elems(s: Seq<u8>, i: int, et: int, n: nat, d: nat) -> Option<
    (int, Seq<TVal>),
>
    decreases d, s.len() - i, 1nat,
{
    if i < 0 || i > s.len() {
        None
    } else if n == 0 {
        Some((i, seq![]))
    } else if i == s.len() {
        None
    } else {
        match spec_value(s, i, et, d) {
            Some((j, x)) => if j <= i || j > s.len() {
                None
            } else {
                match spec_elems(s, j, et, (n - 1) as nat, d) {
                    Some((e, xs)) => Some((e, seq![x] + xs)),
                    None => None,
                }
            },
            None => None,
        }
    }
}

/// The field id a header at `i` announces, and where its value starts.
pub open spec fn spec_field_id(s: Seq<u8>, i: int, last: int) -> Option<(int, int)> {
    let delta = (s[i] / 16) as int;
    if delta != 0 {
        Some((i + 1, last + delta))
    } else {
        match spec_varint(s, i + 1, 10) {
            Some((p, u)) => if u < 0x1_0000 {
                Some((p, unzz(u)))
            } else {
                None
            },
            None => None,
        }
    }
}

/// The fields of a struct from position `i` to its stop byte; `last` is the
/// id of the previous field (0 at the start).
pub open spec fn spec_fields(s: Seq<u8>, i: int, last: int, d: nat) -> Option<
    (int, Seq<TField>),
>
    decreases d, s.len() - i, 1nat,
{
    if i < 0 || i >= s.len() {
        None
    } else if s[i] == 0 {
        Some((i + 1, seq![]))
    } else {
        let t = (s[i] % 16) as int;
        let delta = (s[i] / 16) as int;
        match spec_field_id(s, i, last) {
            None => None,
            Some((p, id)) => {
                if p <= i || p > s.len() || !(last < id <= 32767) || (delta == 0 && id - last
                    <= 15) {
                    None
                } else {
                    let pv = if t == 1 || t == 2 {
                        Some((p, TVal::Bool(t == 1)))
                    } else {
                        spec_value(s, p, t, d)
                    };
                    match pv {
                        None => None,
                        Some((q, v)) => if q <= i || q > s.len() {
                            None
                        } else {
                            match spec_fields(s, q, id, d) {
                                Some((e, fs)) => Some((e, seq![TField { id, val: v }] + fs)),
                                None => None,
                            }
                        },
                    }
                }
            },
        }
    }
}

/// A struct at position `i`.
pub open spec fn spec_struct(s: Seq<u8>, i: int, d: nat) -> Option<(int, TVal)> {
    spec_value(s, i, 12, d)
}

// ============================================================
// Serializer
// ============================================================

pub open spec fn list_header(et: u8, n: nat) -> Seq<u8> {
    if n < 15 {
        seq![(n * 16 + et) as u8]
    } else {
        seq![(240 + et) as u8] + ser_varint(n)
    }
}

pub open spec fn field_header(id: int, last: int, t: int) -> Seq<u8> {
    if 1 <= id - last <= 15 {
        seq![((id - last) * 16 + t) as u8]
    } else {
        seq![t as u8] + ser_varint(zz(id))
    }
}

pub open spec fn ser_value(v: TVal, d: nat) -> Seq<u8>
    decreases d, 0nat, 0nat,
{
    match v {
        TVal::Bool(b) => seq![
            if b {
                1u8
            } else {
                2u8
            },
        ],
        TVal::Byte(x) => seq![x],
        TVal::I16(x) => ser_varint(zz(x)),
        TVal::I32(x) => ser_varint(zz(x)),
        TVal::I64(x) => ser_varint(zz(x)),
        TVal::Double(b) => b,
        TVal::Binary(b) => ser_varint(b.len()) + b,
        TVal::List(et, xs) => if d == 0 {
            seq![]
        } else {
            list_header(et, xs.len()) + ser_elems(xs, (d - 1) as nat)
        },
        TVal::Struct(fs) => if d == 0 {
            seq![]
        } else {
            ser_fields(fs, 0, (d - 1) as nat)
        },
    }
}

pub open spec fn ser_elems(xs: Seq<TVal>, d: nat) -> Seq<u8>
    decreases d, 1nat, xs.len(),
{
    if xs.len() == 0 {
        seq![]
    } else {
        ser_value(xs[0], d) + ser_elems(xs.drop_first(), d)
    }
}

/// The payload of a field: nothing for a boolean (it is in the header).
pub open spec fn ser_payload(v: TVal, d: nat) -> Seq<u8>
    decreases d, 1nat, 0nat,
{
    if v is Bool {
        seq![]
    } else {
        ser_value(v, d)
    }
}

pub open spec fn ser_fields(fs: Seq<TField>, last: int, d: nat) -> Seq<u8>
    decreases d, 1nat, fs.len(),
{
    if fs.len() == 0 {
        seq![0u8]
    } else {
        field_header(fs[0].id, last, tcode(fs[0].val)) + ser_payload(fs[0].val, d) + ser_fields(
            fs.drop_first(),
            fs[0].id,
            d,
        )
    }
}

// ============================================================
// Parse, then serialize: the consumed bytes come back
// ============================================================

pub proof fn lemma_value_ps(s: Seq<u8>, i: int, t: int, d: nat)
    ensures
        spec_value(s, i, t, d) matches Some((j, v)) ==> {
            &&& 0 <= i < j <= s.len()
            &&& ser_value(v, d) == s.subrange(i, j)
            &&& (t == 1 || t == 2) ==> v is Bool
            &&& !(t == 1 || t == 2) ==> tcode(v) == t
        },
    decreases d, s.len() - i, 0nat,
{
    if i < 0 || i >= s.len() {
    } else if t == 1 || t == 2 || t == 3 {
        assert(s.subrange(i, i + 1) =~= seq![s[i]]);
    } else if t == 4 || t == 5 || t == 6 {
        lemma_varint_ps(s, i, 10);
        if let Some((j, u)) = spec_varint(s, i, 10) {
            lemma_zz_unzz(u);
        }
    } else if t == 7 {
    } else if t == 8 {
        lemma_varint_ps(s, i, 10);
        if let Some((j, n)) = spec_varint(s, i, 10) {
            if n < len_limit() && j + n <= s.len() {
                assert(s.subrange(i, j + n) =~= s.subrange(i, j) + s.subrange(j, j + n));
            }
        }
    } else if t == 9 {
        if d > 0 {
            let et = (s[i] % 16) as int;
            if let Some((j, n)) = spec_list_size(s, i) {
                if elem_type_ok(et, n as int) {
                    lemma_elems_ps(s, j, et, n, (d - 1) as nat);
                    if let Some((e, xs)) = spec_elems(s, j, et, n, (d - 1) as nat) {
                        let h = s[i];
                        assert(h == (h / 16) * 16 + h % 16);
                        if h / 16 == 15 {
                            lemma_varint_ps(s, i + 1, 10);
                            assert(s.subrange(i, j) =~= seq![h] + s.subrange(i + 1, j));
                        } else {
                            assert(s.subrange(i, j) =~= seq![h]);
                        }
                        assert(list_header(et as u8, xs.len()) == s.subrange(i, j));
                        assert(s.subrange(i, e) =~= s.subrange(i, j) + s.subrange(j, e));
                    }
                }
            }
        }
    } else if t == 12 {
        if d > 0 {
            lemma_fields_ps(s, i, 0, (d - 1) as nat);
        }
    }
}

pub proof fn lemma_elems_ps(s: Seq<u8>, i: int, et: int, n: nat, d: nat)
    ensures
        spec_elems(s, i, et, n, d) matches Some((e, xs)) ==> {
            &&& 0 <= i <= e <= s.len()
            &&& xs.len() == n
            &&& ser_elems(xs, d) == s.subrange(i, e)
        },
    decreases d, s.len() - i, 1nat,
{
    if i < 0 || i > s.len() {
    } else if n == 0 {
        assert(s.subrange(i, i) =~= Seq::<u8>::empty());
    } else if i < s.len() {
        lemma_value_ps(s, i, et, d);
        if let Some((j, x)) = spec_value(s, i, et, d) {
            if j > i && j <= s.len() {
                lemma_elems_ps(s, j, et, (n - 1) as nat, d);
                if let Some((e, rest)) = spec_elems(s, j, et, (n - 1) as nat, d) {
                    let xs = seq![x] + rest;
                    assert(xs[0] == x);
                    assert(xs.drop_first() =~= rest);
                    assert(s.subrange(i, e) =~= s.subrange(i, j) + s.subrange(j, e));
                }
            }
        }
    }
}

pub proof fn lemma_fields_ps(s: Seq<u8>, i: int, last: int, d: nat)
    ensures
        spec_fields(s, i, last, d) matches Some((e, fs)) ==> {
            &&& 0 <= i < e <= s.len()
            &&& ser_fields(fs, last, d) == s.subrange(i, e)
        },
    decreases d, s.len() - i, 1nat,
{
    if i < 0 || i >= s.len() {
    } else if s[i] == 0 {
        assert(s.subrange(i, i + 1) =~= seq![0u8]);
    } else {
        let b = s[i];
        let t = (b % 16) as int;
        let delta = (b / 16) as int;
        assert(b == delta * 16 + t);
        if let Some((p, id)) = spec_field_id(s, i, last) {
            if i < p <= s.len() && last < id <= 32767 && !(delta == 0 && id - last <= 15) {
                let pv = if t == 1 || t == 2 {
                    Some((p, TVal::Bool(t == 1)))
                } else {
                    spec_value(s, p, t, d)
                };
                if delta == 0 {
                    lemma_varint_ps(s, i + 1, 10);
                    let u = (spec_varint(s, i + 1, 10)->0).1;
                    lemma_zz_unzz(u);
                    assert(s.subrange(i, p) =~= seq![b] + s.subrange(i + 1, p));
                } else {
                    assert(s.subrange(i, p) =~= seq![b]);
                }
                if !(t == 1 || t == 2) {
                    lemma_value_ps(s, p, t, d);
                }
                if let Some((q, v)) = pv {
                    if q > i && q <= s.len() {
                        lemma_fields_ps(s, q, id, d);
                        if let Some((e, rest)) = spec_fields(s, q, id, d) {
                            let fs = seq![TField { id, val: v }] + rest;
                            assert(fs[0] == TField { id, val: v });
                            assert(fs.drop_first() =~= rest);
                            assert(tcode(v) == t);
                            assert(field_header(id, last, t) == s.subrange(i, p));
                            if t == 1 || t == 2 {
                                assert(s.subrange(p, q) =~= Seq::<u8>::empty());
                            }
                            assert(ser_payload(v, d) == s.subrange(p, q));
                            assert(s.subrange(i, e) =~= s.subrange(i, p) + s.subrange(p, q)
                                + s.subrange(q, e));
                        }
                    }
                }
            }
        }
    }
}

/// Two inputs that parse to the same struct consumed the same bytes.
pub proof fn theorem_thrift_non_malleable(s1: Seq<u8>, i1: int, s2: Seq<u8>, i2: int, d: nat)
    requires
        spec_struct(s1, i1, d) is Some,
        spec_struct(s2, i2, d) is Some,
        (spec_struct(s1, i1, d)->0).1 == (spec_struct(s2, i2, d)->0).1,
    ensures
        s1.subrange(i1, (spec_struct(s1, i1, d)->0).0) == s2.subrange(
            i2,
            (spec_struct(s2, i2, d)->0).0,
        ),
{
    lemma_value_ps(s1, i1, 12, d);
    lemma_value_ps(s2, i2, 12, d);
}

// ============================================================
// Field lookup
// ============================================================

/// The value of the field `id` (field ids are unique in a parsed struct).
pub open spec fn get(fs: Seq<TField>, id: int) -> Option<TVal>
    decreases fs.len(),
{
    if fs.len() == 0 {
        None
    } else if fs[0].id == id {
        Some(fs[0].val)
    } else {
        get(fs.drop_first(), id)
    }
}

// ============================================================
// Executable parser
// ============================================================

pub enum EVal<'a> {
    Bool(bool),
    Byte(u8),
    I16(i16),
    I32(i32),
    I64(i64),
    Double(&'a [u8]),
    Binary(&'a [u8]),
    List(u8, Vec<EVal<'a>>),
    Struct(Vec<EField<'a>>),
}

pub struct EField<'a> {
    pub id: i16,
    pub val: EVal<'a>,
}

/// The spec value of an executable value nested at most `d` deep.
pub open spec fn ev(v: EVal, d: nat) -> TVal
    decreases d, 0nat,
{
    match v {
        EVal::Bool(b) => TVal::Bool(b),
        EVal::Byte(x) => TVal::Byte(x),
        EVal::I16(x) => TVal::I16(x as int),
        EVal::I32(x) => TVal::I32(x as int),
        EVal::I64(x) => TVal::I64(x as int),
        EVal::Double(b) => TVal::Double(b@),
        EVal::Binary(b) => TVal::Binary(b@),
        EVal::List(et, xs) => if d == 0 {
            TVal::List(et, seq![])
        } else {
            TVal::List(et, evs(xs@, (d - 1) as nat))
        },
        EVal::Struct(fs) => if d == 0 {
            TVal::Struct(seq![])
        } else {
            TVal::Struct(efs(fs@, (d - 1) as nat))
        },
    }
}

pub open spec fn evs(xs: Seq<EVal>, d: nat) -> Seq<TVal>
    decreases d, 1nat,
{
    Seq::new(xs.len(), |k: int| ev(xs[k], d))
}

pub open spec fn efs(fs: Seq<EField>, d: nat) -> Seq<TField>
    decreases d, 1nat,
{
    Seq::new(fs.len(), |k: int| TField { id: fs[k].id as int, val: ev(fs[k].val, d) })
}

pub open spec fn opt_prepend<T>(acc: Seq<T>, r: Option<(int, Seq<T>)>) -> Option<(int, Seq<T>)> {
    match r {
        Some((e, xs)) => Some((e, acc + xs)),
        None => None,
    }
}

/// Viewing `v` with more depth than `d` changes nothing: `v` nests less
/// than `d` deep. Parsed values are stable at the depth they were parsed at.
pub open spec fn stable(v: EVal, d: nat) -> bool {
    forall|d2: nat| d2 >= d ==> #[trigger] ev(v, d2) == ev(v, d)
}

pub proof fn lemma_stable_field(fs: Vec<EField>, d: nat, k: int)
    requires
        stable(EVal::Struct(fs), d + 1),
        0 <= k < fs@.len(),
    ensures
        stable(fs@[k].val, d),
{
    assert forall|d2: nat| d2 >= d implies #[trigger] ev(fs@[k].val, d2) == ev(fs@[k].val, d) by {
        assert(ev(EVal::Struct(fs), d2 + 1) == ev(EVal::Struct(fs), d + 1));
        assert(efs(fs@, d2)[k] == efs(fs@, d)[k]);
    }
}

pub proof fn lemma_stable_elem(et: u8, xs: Vec<EVal>, d: nat, k: int)
    requires
        stable(EVal::List(et, xs), d + 1),
        0 <= k < xs@.len(),
    ensures
        stable(xs@[k], d),
{
    assert forall|d2: nat| d2 >= d implies #[trigger] ev(xs@[k], d2) == ev(xs@[k], d) by {
        assert(ev(EVal::List(et, xs), d2 + 1) == ev(EVal::List(et, xs), d + 1));
        assert(evs(xs@, d2)[k] == evs(xs@, d)[k]);
    }
}

pub fn varint(s: &[u8], i: usize, k: u8) -> (r: Option<(usize, u64)>)
    ensures
        match r {
            Some((j, v)) => spec_varint(s@, i as int, k as nat) == Some((j as int, v as nat)),
            None => spec_varint(s@, i as int, k as nat) is None,
        },
        r matches Some((j, _)) ==> i < j <= s@.len(),
    decreases k,
{
    proof {
        lemma_varint_ps(s@, i as int, k as nat);
    }
    if k == 0 || i >= s.len() {
        return None;
    }
    let b = s[i];
    if b < 128 {
        return Some((i + 1, b as u64));
    }
    match varint(s, i + 1, k - 1) {
        None => None,
        Some((j, r)) => {
            if r == 0 {
                return None;
            }
            let v: u128 = (b - 128) as u128 + 128 * (r as u128);
            if v >= 0x1_0000_0000_0000_0000u128 {
                None
            } else {
                Some((j, v as u64))
            }
        },
    }
}

fn unzz_exec(u: u64) -> (z: i64)
    ensures
        z as int == unzz(u as nat),
{
    if u % 2 == 0 {
        (u / 2) as i64
    } else {
        -((u / 2) as i64) - 1
    }
}

fn list_size(s: &[u8], i: usize) -> (r: Option<(usize, u64)>)
    ensures
        match r {
            Some((j, n)) => spec_list_size(s@, i as int) == Some((j as int, n as nat)),
            None => spec_list_size(s@, i as int) is None,
        },
        r matches Some((j, _)) ==> i < j <= s@.len(),
{
    if i >= s.len() {
        return None;
    }
    if s[i] / 16 == 15 {
        match varint(s, i + 1, 10) {
            Some((j, n)) => if 15 <= n && n < 0x8000_0000 {
                Some((j, n))
            } else {
                None
            },
            None => None,
        }
    } else {
        Some((i + 1, (s[i] / 16) as u64))
    }
}

pub proof fn lemma_leaf_stable(v: EVal, d: nat)
    requires
        !(v is List || v is Struct),
    ensures
        stable(v, d),
{
}

/// Parses a value of compact type `t` at `i`; see [`spec_value`].
pub fn parse_value<'a>(s: &'a [u8], i: usize, t: u8, d: u64) -> (r: Option<(usize, EVal<'a>)>)
    ensures
        match r {
            Some((j, v)) => spec_value(s@, i as int, t as int, d as nat) == Some(
                (j as int, ev(v, d as nat)),
            ),
            None => spec_value(s@, i as int, t as int, d as nat) is None,
        },
        r matches Some((_, v)) ==> stable(v, d as nat),
        r matches Some((j, _)) ==> i < j <= s@.len(),
    decreases d,
{
    proof {
        lemma_value_ps(s@, i as int, t as int, d as nat);
    }
    if i >= s.len() {
        return None;
    }
    let b = s[i];
    if t == 1 || t == 2 {
        if b == 1 || b == 2 {
            proof {
                lemma_leaf_stable(EVal::Bool(b == 1), d as nat);
            }
            return Some((i + 1, EVal::Bool(b == 1)));
        }
        return None;
    }
    if t == 3 {
        proof {
            lemma_leaf_stable(EVal::Byte(b), d as nat);
        }
        return Some((i + 1, EVal::Byte(b)));
    }
    if t == 4 || t == 5 || t == 6 {
        let (j, u) = match varint(s, i, 10) {
            Some(x) => x,
            None => {
                return None;
            },
        };
        if (t == 4 && u >= 0x1_0000) || (t == 5 && u >= 0x1_0000_0000) {
            return None;
        }
        let z = unzz_exec(u);
        let v = if t == 4 {
            EVal::I16(z as i16)
        } else if t == 5 {
            EVal::I32(z as i32)
        } else {
            EVal::I64(z)
        };
        proof {
            lemma_leaf_stable(v, d as nat);
        }
        return Some((j, v));
    }
    if t == 7 {
        if s.len() - i >= 8 {
            let v = EVal::Double(slice_subrange(s, i, i + 8));
            proof {
                lemma_leaf_stable(v, d as nat);
            }
            return Some((i + 8, v));
        }
        return None;
    }
    if t == 8 {
        let (j, n) = match varint(s, i, 10) {
            Some(x) => x,
            None => {
                return None;
            },
        };
        if n >= 0x8000_0000 || n as usize > s.len() - j {
            return None;
        }
        let n = n as usize;
        let v = EVal::Binary(slice_subrange(s, j, j + n));
        proof {
            lemma_leaf_stable(v, d as nat);
        }
        return Some((j + n, v));
    }
    if t == 9 {
        if d == 0 {
            return None;
        }
        let et = b % 16;
        let (j, n) = match list_size(s, i) {
            Some(x) => x,
            None => {
                return None;
            },
        };
        if !((1 <= et && et <= 9) || et == 12 || (et == 0 && n == 0)) {
            return None;
        }
        let ghost d1 = (d - 1) as nat;
        let mut xs: Vec<EVal<'a>> = Vec::new();
        let mut p = j;
        let mut k: u64 = 0;
        proof {
            assert(evs(xs@, d1) =~= Seq::<TVal>::empty());
            assert(opt_prepend(Seq::<TVal>::empty(), spec_elems(s@, j as int, et as int, n as nat, d1))
                =~= spec_elems(s@, j as int, et as int, n as nat, d1)) by {
                if let Some((e, ys)) = spec_elems(s@, j as int, et as int, n as nat, d1) {
                    assert(Seq::<TVal>::empty() + ys =~= ys);
                }
            }
        }
        while k < n
            invariant
                d >= 1,
                d1 == (d - 1) as nat,
                k <= n,
                p <= s@.len(),
                spec_value(s@, i as int, t as int, d as nat) == match spec_elems(
                    s@,
                    j as int,
                    et as int,
                    n as nat,
                    d1,
                ) {
                    Some((e, ys)) => Some((e, TVal::List(et, ys))),
                    None => None,
                },
                spec_elems(s@, j as int, et as int, n as nat, d1) == opt_prepend(
                    evs(xs@, d1),
                    spec_elems(s@, p as int, et as int, (n - k) as nat, d1),
                ),
                forall|q: int| 0 <= q < xs@.len() ==> stable(#[trigger] xs@[q], d1),
            decreases n - k,
        {
            if p >= s.len() {
                return None;
            }
            let (q, x) = match parse_value(s, p, et, d - 1) {
                Some(r) => r,
                None => {
                    return None;
                },
            };
            if q <= p || q > s.len() {
                return None;
            }
            let ghost old_xs = xs@;
            proof {
                if let Some((e, rest)) = spec_elems(s@, q as int, et as int, (n - k - 1) as nat, d1) {
                    assert(evs(old_xs, d1) + (seq![ev(x, d1)] + rest) =~= evs(old_xs.push(x), d1)
                        + rest);
                }
            }
            xs.push(x);
            p = q;
            k = k + 1;
        }
        let ghost xsv = xs@;
        proof {
            assert(evs(xs@, d1) + Seq::<TVal>::empty() =~= evs(xs@, d1));
            assert forall|d2: nat| d2 >= d as nat implies #[trigger] ev(EVal::List(et, xs), d2)
                == ev(EVal::List(et, xs), d as nat) by {
                let d3 = (d2 - 1) as nat;
                assert forall|q: int| 0 <= q < xsv.len() implies evs(xsv, d3)[q] == evs(xsv, d1)[q] by {
                    assert(stable(xsv[q], d1));
                    assert(ev(xsv[q], d3) == ev(xsv[q], d1));
                }
                assert(evs(xsv, d3) =~= evs(xsv, d1));
            }
        }
        return Some((p, EVal::List(et, xs)));
    }
    if t == 12 {
        if d == 0 {
            return None;
        }
        let ghost d1 = (d - 1) as nat;
        let mut fs: Vec<EField<'a>> = Vec::new();
        let mut p = i;
        let mut last: i64 = 0;
        proof {
            assert(efs(fs@, d1) =~= Seq::<TField>::empty());
            if let Some((e, ys)) = spec_fields(s@, i as int, 0, d1) {
                assert(Seq::<TField>::empty() + ys =~= ys);
            }
        }
        loop
            invariant
                d >= 1,
                d1 == (d - 1) as nat,
                i <= p <= s@.len(),
                0 <= last <= 32767,
                spec_value(s@, i as int, t as int, d as nat) == match spec_fields(
                    s@,
                    i as int,
                    0,
                    d1,
                ) {
                    Some((e, ys)) => Some((e, TVal::Struct(ys))),
                    None => None,
                },
                spec_fields(s@, i as int, 0, d1) == opt_prepend(
                    efs(fs@, d1),
                    spec_fields(s@, p as int, last as int, d1),
                ),
                forall|q: int| 0 <= q < fs@.len() ==> stable(#[trigger] fs@[q].val, d1),
            decreases s@.len() - p,
        {
            if p >= s.len() {
                return None;
            }
            let h = s[p];
            if h == 0 {
                let ghost fsv = fs@;
                proof {
                    assert(efs(fs@, d1) + Seq::<TField>::empty() =~= efs(fs@, d1));
                    assert forall|d2: nat| d2 >= d as nat implies #[trigger] ev(
                        EVal::Struct(fs),
                        d2,
                    ) == ev(EVal::Struct(fs), d as nat) by {
                        let d3 = (d2 - 1) as nat;
                        assert forall|q: int| 0 <= q < fsv.len() implies efs(fsv, d3)[q] == efs(
                            fsv,
                            d1,
                        )[q] by {
                            assert(stable(fsv[q].val, d1));
                            assert(ev(fsv[q].val, d3) == ev(fsv[q].val, d1));
                        }
                        assert(efs(fsv, d3) =~= efs(fsv, d1));
                    }
                }
                return Some((p + 1, EVal::Struct(fs)));
            }
            let ft = h % 16;
            let delta = h / 16;
            let (q0, id): (usize, i64) = if delta != 0 {
                (p + 1, last + delta as i64)
            } else {
                match varint(s, p + 1, 10) {
                    Some((q, u)) => if u < 0x1_0000 {
                        (q, unzz_exec(u))
                    } else {
                        return None;
                    },
                    None => {
                        return None;
                    },
                }
            };
            assert(spec_field_id(s@, p as int, last as int) == Some((q0 as int, id as int)));
            if !(q0 > p && q0 <= s.len() && last < id && id <= 32767 && !(delta == 0 && id - last
                <= 15)) {
                return None;
            }
            let (q, v) = if ft == 1 || ft == 2 {
                (q0, EVal::Bool(ft == 1))
            } else {
                match parse_value(s, q0, ft, d - 1) {
                    Some(r) => r,
                    None => {
                        return None;
                    },
                }
            };
            if q <= p || q > s.len() {
                return None;
            }
            let ghost old_fs = fs@;
            let f = EField { id: id as i16, val: v };
            proof {
                if let Some((e, rest)) = spec_fields(s@, q as int, id as int, d1) {
                    assert(efs(old_fs, d1) + (seq![TField { id: id as int, val: ev(v, d1) }]
                        + rest) =~= efs(old_fs.push(f), d1) + rest);
                }
            }
            fs.push(f);
            p = q;
            last = id;
        }
    }
    None
}

/// Looks up field `id`; see [`get`].
pub fn get_field<'b, 'a>(fs: &'b Vec<EField<'a>>, id: i16, Ghost(d): Ghost<nat>) -> (r: Option<
    &'b EVal<'a>,
>)
    ensures
        match r {
            Some(v) => get(efs(fs@, d), id as int) == Some(ev(*v, d)),
            None => get(efs(fs@, d), id as int) is None,
        },
        r matches Some(v) ==> exists|k: int| 0 <= k < fs@.len() && *v == fs@[k].val,
{
    let mut k: usize = 0;
    assert(efs(fs@, d).skip(0) =~= efs(fs@, d));
    while k < fs.len()
        invariant
            k <= fs@.len(),
            get(efs(fs@, d), id as int) == get(efs(fs@, d).skip(k as int), id as int),
        decreases fs@.len() - k,
    {
        let ghost sk = efs(fs@, d).skip(k as int);
        assert(sk[0] == TField { id: fs@[k as int].id as int, val: ev(fs@[k as int].val, d) });
        assert(sk.drop_first() =~= efs(fs@, d).skip(k + 1));
        if fs[k].id == id {
            return Some(&fs[k].val);
        }
        k += 1;
    }
    assert(efs(fs@, d).skip(k as int) =~= Seq::<TField>::empty());
    None
}

/// Equality of the spec values of two executable values.
#[verifier::loop_isolation(false)]
pub fn tval_eq(a: &EVal, b: &EVal, d: u64) -> (r: bool)
    ensures
        r == (ev(*a, d as nat) == ev(*b, d as nat)),
    decreases d,
{
    match (a, b) {
        (EVal::Bool(x), EVal::Bool(y)) => *x == *y,
        (EVal::Byte(x), EVal::Byte(y)) => *x == *y,
        (EVal::I16(x), EVal::I16(y)) => *x == *y,
        (EVal::I32(x), EVal::I32(y)) => *x == *y,
        (EVal::I64(x), EVal::I64(y)) => *x == *y,
        (EVal::Double(x), EVal::Double(y)) => bytes_eq(x, y),
        (EVal::Binary(x), EVal::Binary(y)) => bytes_eq(x, y),
        (EVal::List(e1, x1), EVal::List(e2, x2)) => {
            if *e1 != *e2 {
                return false;
            }
            if d == 0 {
                return true;
            }
            let ghost d1 = (d - 1) as nat;
            if x1.len() != x2.len() {
                assert(evs(x1@, d1).len() != evs(x2@, d1).len());
                return false;
            }
            let mut k: usize = 0;
            while k < x1.len()
                invariant
                    d >= 1,
                    d1 == (d - 1) as nat,
                    x1@.len() == x2@.len(),
                    k <= x1@.len(),
                    forall|q: int| 0 <= q < k ==> ev(x1@[q], d1) == ev(x2@[q], d1),
                decreases x1@.len() - k,
            {
                if !tval_eq(&x1[k], &x2[k], d - 1) {
                    assert(evs(x1@, d1)[k as int] != evs(x2@, d1)[k as int]);
                    assert(ev(*a, d as nat) == TVal::List(*e1, evs(x1@, d1)));
                    assert(ev(*b, d as nat) == TVal::List(*e2, evs(x2@, d1)));
                    return false;
                }
                k += 1;
            }
            assert(evs(x1@, d1) =~= evs(x2@, d1));
            true
        },
        (EVal::Struct(f1), EVal::Struct(f2)) => {
            if d == 0 {
                return true;
            }
            let ghost d1 = (d - 1) as nat;
            if f1.len() != f2.len() {
                assert(efs(f1@, d1).len() != efs(f2@, d1).len());
                return false;
            }
            let mut k: usize = 0;
            while k < f1.len()
                invariant
                    d >= 1,
                    d1 == (d - 1) as nat,
                    f1@.len() == f2@.len(),
                    k <= f1@.len(),
                    forall|q: int|
                        0 <= q < k ==> efs(f1@, d1)[q] == efs(f2@, d1)[q],
                decreases f1@.len() - k,
            {
                if f1[k].id != f2[k].id || !tval_eq(&f1[k].val, &f2[k].val, d - 1) {
                    assert(efs(f1@, d1)[k as int] != efs(f2@, d1)[k as int]);
                    assert(ev(*a, d as nat) == TVal::Struct(efs(f1@, d1)));
                    assert(ev(*b, d as nat) == TVal::Struct(efs(f2@, d1)));
                    return false;
                }
                k += 1;
            }
            assert(efs(f1@, d1) =~= efs(f2@, d1));
            true
        },
        _ => false,
    }
}

} // verus!
