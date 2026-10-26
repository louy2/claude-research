//! Interval parsing with root-relative jumps (`At`) and ranked termination,
//! built on `vest_lib`.
//!
//! Each region of a file (a ZIP local file, the central directory, the end of
//! central directory record) is an ordinary Vest format. [`ranked`] adds what
//! a sequential combinator library lacks for directory-based formats:
//!
//! - `At(l, r, c)`: run `c` on `root[l..r)` of the whole input;
//! - a backward scan for records anchored at the end of the file;
//! - a driver that follows offsets found in parsed values, where every jump
//!   must go to a node of strictly lower rank, so the driver terminates for
//!   any input (the rank is its `decreases` measure);
//! - a tiling check: the regions, in the format's layout order, must cover
//!   the file contiguously, each exactly once.
//!
//! The main theorem ([`ranked::theorem_non_malleable`]) says that two inputs
//! accepted with the same regions and values are the same bytes. [`zip`]
//! instantiates the driver for a strict ZIP profile.
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports)]

pub mod gen {
    pub mod zip_regions;
}

pub mod ranked;
pub mod zip;
