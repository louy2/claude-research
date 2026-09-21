//! Verified parsers for popular binary formats.
//!
//! Each module is generated: `formats/<name>.hexpat` (an ImHex pattern) is
//! translated by `hexpat emit-vest` into `vest/<name>.vest`, which the Vest
//! compiler turns into `src/gen/<name>.rs`. The generated code is ordinary
//! Rust built on `vest_lib` combinators and carries the Verus proofs of
//! memory safety, panic freedom and parse/serialize round-tripping (checked
//! with `cargo verus verify` when Verus is installed; not required to build).
//!
//! Regenerate with `scripts/regen-formats.sh`.
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports)]

pub mod gen {
    pub mod bmp;
    pub mod elf;
    pub mod gif;
    pub mod gzip;
    pub mod ico;
    pub mod png;
    pub mod qoi;
    pub mod tar;
    pub mod wav;
    pub mod zip;
}

pub use gen::*;
