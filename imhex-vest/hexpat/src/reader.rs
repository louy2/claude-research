//! Byte-level decoding built on `vest_lib`'s verified primitive combinators.
//!
//! Every fixed-width integer that the interpreter reads goes through a
//! `vest_lib` format (`U8`, `U16Le`, `U32Be`, ... and `Varied`/`Fixed` for
//! byte strings). Widths that `vest_lib` does not provide as a single
//! combinator (48, 96 and 128 bits) are composed with `Pair` from the ones
//! it does, so the decoding of every integer is still delegated to verified
//! code; only the arithmetic that glues the halves together lives here.

use vest_lib::combinators::{
    Fixed, Pair, RepeatN, Varied, I16Be, I16Le, I32Be, I32Le, I64Be, I64Le, I8, U16Be, U16Le, U24Be, U24Le, U32Be,
    U32Le, U64Be, U64Le, U8,
};
use vest_lib::core::exec::parser::Parser;

use crate::ast::Endian;
use crate::error::{Error, Result};

/// A read-only view over the bytes of one section.
#[derive(Clone, Copy)]
pub struct Reader<'a> {
    pub data: &'a [u8],
}

fn short(offset: u64, size: u64, len: usize) -> Error {
    Error::new(format!(
        "read of {} byte(s) at 0x{:X} is out of bounds (data size 0x{:X})",
        size, offset, len
    ))
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data }
    }

    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn slice(&self, offset: u64, size: u64) -> Result<&'a [u8]> {
        let end = offset.checked_add(size).ok_or_else(|| short(offset, size, self.data.len()))?;
        if end > self.data.len() as u64 {
            return Err(short(offset, size, self.data.len()));
        }
        Ok(&self.data[offset as usize..end as usize])
    }

    /// Reads `size` raw bytes with `vest_lib::combinators::Varied`.
    pub fn bytes(&self, offset: u64, size: u64) -> Result<&'a [u8]> {
        let buf = self.slice(offset, size)?;
        let (n, out) = Varied(size as usize)
            .parse(&buf)
            .map_err(|e| Error::new(format!("vest Varied parse failed: {:?}", e)))?;
        debug_assert_eq!(n as u64, size);
        Ok(out)
    }

    /// Reads exactly four bytes with `vest_lib::combinators::Fixed<4>`
    /// (used for magic numbers).
    pub fn fixed4(&self, offset: u64) -> Result<[u8; 4]> {
        let buf = self.slice(offset, 4)?;
        let (_, out) = Fixed::<4>.parse(&buf).map_err(|e| Error::new(format!("vest Fixed parse failed: {:?}", e)))?;
        Ok([out[0], out[1], out[2], out[3]])
    }

    /// Reads an unsigned integer of `size` bytes (1..=16).
    pub fn unsigned(&self, offset: u64, size: u64, endian: Endian) -> Result<u128> {
        let buf = self.slice(offset, size)?;
        macro_rules! run {
            ($fmt:expr) => {{
                let (_, v) = $fmt.parse(&buf).map_err(|e| Error::new(format!("vest parse failed: {:?}", e)))?;
                v
            }};
        }
        Ok(match (size, endian) {
            (1, _) => run!(U8) as u128,
            (2, Endian::Little) => run!(U16Le) as u128,
            (2, Endian::Big) => run!(U16Be) as u128,
            (3, Endian::Little) => run!(U24Le) as u128,
            (3, Endian::Big) => run!(U24Be) as u128,
            (4, Endian::Little) => run!(U32Le) as u128,
            (4, Endian::Big) => run!(U32Be) as u128,
            (6, Endian::Little) => {
                let (lo, hi) = run!(Pair(U32Le, U16Le));
                (hi as u128) << 32 | lo as u128
            }
            (6, Endian::Big) => {
                let (hi, lo) = run!(Pair(U16Be, U32Be));
                (hi as u128) << 32 | lo as u128
            }
            (8, Endian::Little) => run!(U64Le) as u128,
            (8, Endian::Big) => run!(U64Be) as u128,
            (12, Endian::Little) => {
                let (lo, (mid, hi)) = run!(Pair(U32Le, Pair(U32Le, U32Le)));
                (hi as u128) << 64 | (mid as u128) << 32 | lo as u128
            }
            (12, Endian::Big) => {
                let (hi, (mid, lo)) = run!(Pair(U32Be, Pair(U32Be, U32Be)));
                (hi as u128) << 64 | (mid as u128) << 32 | lo as u128
            }
            (16, Endian::Little) => {
                let (lo, hi) = run!(Pair(U64Le, U64Le));
                (hi as u128) << 64 | lo as u128
            }
            (16, Endian::Big) => {
                let (hi, lo) = run!(Pair(U64Be, U64Be));
                (hi as u128) << 64 | lo as u128
            }
            _ => {
                // Any other width: fold bytes manually (not reachable from the
                // built-in type set, kept for `read_unsigned` with odd sizes).
                let mut v: u128 = 0;
                match endian {
                    Endian::Little => {
                        for b in buf.iter().rev() {
                            v = (v << 8) | *b as u128;
                        }
                    }
                    Endian::Big => {
                        for b in buf {
                            v = (v << 8) | *b as u128;
                        }
                    }
                }
                v
            }
        })
    }

    /// Reads a signed integer of `size` bytes (1..=16), sign-extended.
    pub fn signed(&self, offset: u64, size: u64, endian: Endian) -> Result<i128> {
        let buf = self.slice(offset, size)?;
        macro_rules! run {
            ($fmt:expr) => {{
                let (_, v) = $fmt.parse(&buf).map_err(|e| Error::new(format!("vest parse failed: {:?}", e)))?;
                v
            }};
        }
        Ok(match (size, endian) {
            (1, _) => run!(I8) as i128,
            (2, Endian::Little) => run!(I16Le) as i128,
            (2, Endian::Big) => run!(I16Be) as i128,
            (4, Endian::Little) => run!(I32Le) as i128,
            (4, Endian::Big) => run!(I32Be) as i128,
            (8, Endian::Little) => run!(I64Le) as i128,
            (8, Endian::Big) => run!(I64Be) as i128,
            _ => {
                let raw = self.unsigned(offset, size, endian)?;
                sign_extend(raw, size * 8)
            }
        })
    }

    /// Reads `count` little- or big-endian `u8`/`u16`/`u32`/`u64` elements in
    /// one `RepeatN` parse. Used for homogeneous scalar arrays.
    pub fn unsigned_array(&self, offset: u64, elem: u64, count: u64, endian: Endian) -> Result<Vec<u128>> {
        let total = elem.checked_mul(count).ok_or_else(|| Error::new("array size overflow"))?;
        let buf = self.slice(offset, total)?;
        let n = count as usize;
        macro_rules! run {
            ($fmt:expr) => {{
                let (_, v) = RepeatN(n, $fmt)
                    .parse(&buf)
                    .map_err(|e| Error::new(format!("vest RepeatN parse failed: {:?}", e)))?;
                v.into_iter().map(|x| x as u128).collect()
            }};
        }
        Ok(match (elem, endian) {
            (1, _) => run!(U8),
            (2, Endian::Little) => run!(U16Le),
            (2, Endian::Big) => run!(U16Be),
            (4, Endian::Little) => run!(U32Le),
            (4, Endian::Big) => run!(U32Be),
            (8, Endian::Little) => run!(U64Le),
            (8, Endian::Big) => run!(U64Be),
            _ => (0..count).map(|i| self.unsigned(offset + i * elem, elem, endian)).collect::<Result<Vec<_>>>()?,
        })
    }

    pub fn f32(&self, offset: u64, endian: Endian) -> Result<f64> {
        Ok(f32::from_bits(self.unsigned(offset, 4, endian)? as u32) as f64)
    }

    pub fn f64(&self, offset: u64, endian: Endian) -> Result<f64> {
        Ok(f64::from_bits(self.unsigned(offset, 8, endian)? as u64))
    }

    /// Position of the first zero byte at or after `offset`, if any.
    pub fn find_zero(&self, offset: u64, width: u64) -> Option<u64> {
        let mut pos = offset;
        while pos + width <= self.len() {
            if self.data[pos as usize..(pos + width) as usize].iter().all(|b| *b == 0) {
                return Some(pos);
            }
            pos += width;
        }
        None
    }
}

/// Sign-extends the low `bits` bits of `raw`.
pub fn sign_extend(raw: u128, bits: u64) -> i128 {
    if bits == 0 || bits >= 128 {
        return raw as i128;
    }
    let shift = 128 - bits as u32;
    ((raw << shift) as i128) >> shift
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_round_trip_through_vest() {
        let data = [0x78, 0x56, 0x34, 0x12, 0xFF, 0xFE, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A];
        let r = Reader::new(&data);
        assert_eq!(r.unsigned(0, 4, Endian::Little).unwrap(), 0x12345678);
        assert_eq!(r.unsigned(0, 4, Endian::Big).unwrap(), 0x78563412);
        assert_eq!(r.unsigned(0, 3, Endian::Little).unwrap(), 0x345678);
        assert_eq!(r.signed(4, 1, Endian::Little).unwrap(), -1);
        assert_eq!(r.signed(4, 2, Endian::Little).unwrap(), -257);
        assert_eq!(r.unsigned(4, 6, Endian::Big).unwrap(), 0xFFFE01020304);
        assert_eq!(r.unsigned(0, 16, Endian::Little).unwrap(), 0x0A090807060504030201FEFF12345678);
        assert_eq!(r.unsigned_array(0, 2, 2, Endian::Little).unwrap(), vec![0x5678, 0x1234]);
        assert!(r.unsigned(14, 4, Endian::Little).is_err());
    }
}
