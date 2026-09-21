# ! [allow (warnings)] use vest_lib::combinators::mapped::spec::* ;
use vest_lib::combinators::* ;
use vest_lib::combinators::recursive::* ;
use Sum::Inl as L ;
use Sum::Inr as R ;
use vest_lib::Never ;
use vest_lib::core::exec::input::{
    InputBuf,
    InputSlice
}
;
use vest_lib::core::exec::output::OutputBuf ;
use vest_lib::core::exec::parser::* ;
use vest_lib::core::exec::serializer::* ;
use vest_lib::core::exec::ParseError ;
use vest_lib::core::exec::bytes_eq ;
use vest_lib::core::{
    proof::*,
    spec::*
}
;
use vest_lib::primitives::btcvarint::VarInt ;
use vest_lib::primitives::leb128::ULeb128 ;
use vstd::prelude::* ;
verus! {
// ============================================================
// Data Types
// ============================================================
# [doc = "data type for `flags`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
# [verifier::ext_equal]
pub struct Flags {
    pub _pad1: u8,
    pub comment: u8,
    pub name: u8,
    pub extra: u8,
    pub hcrc: u8,
    pub text: u8,
}
pub type FlagsSpec = Flags ;
pub type FlagsInner = u8 ;
impl DeepView for Flags {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Flags {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Flags as DeepView>::deep_view) ;
    }
}

# [doc = "data type for `os`."]
# [repr (u8)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum Os {
    Fat = 0,
    Amiga = 1,
    Vms = 2,
    Unix = 3,
    Macintosh = 7,
    Ntfs = 11,
    Unknown (u8),
}
pub type OsSpec = Os ;
pub type OsInner = Sum < u8, u8 > ;
impl DeepView for Os {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Os {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Os as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: OsInner) -> bool {
        match input {
            L (x) => x == 0 || x == 1 || x == 2 || x == 3 || x == 7 || x == 11,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: OsInner) -> Self {
        match input {
            L (x) => match x {
                0 => Self::Fat,
                1 => Self::Amiga,
                2 => Self::Vms,
                3 => Self::Unix,
                7 => Self::Macintosh,
                11 => Self::Ntfs,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> OsInner {
        match self {
            Self::Fat => L (0),
            Self::Amiga => L (1),
            Self::Vms => L (2),
            Self::Unix => L (3),
            Self::Macintosh => L (7),
            Self::Ntfs => L (11),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Os::from_structural) ;
        reveal(Os::into_structural) ;
        match self {
            Self::Fat => {
            }
           ,
            Self::Amiga => {
            }
           ,
            Self::Vms => {
            }
           ,
            Self::Unix => {
            }
           ,
            Self::Macintosh => {
            }
           ,
            Self::Ntfs => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: OsInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Os::from_structural) ;
        reveal(Os::into_structural) ;
        match input {
            L (x) => match x {
                0 => {
                }
               ,
                1 => {
                }
               ,
                2 => {
                }
               ,
                3 => {
                }
               ,
                7 => {
                }
               ,
                11 => {
                }
               ,
                _ => {
                    assert (false) ;
                }
            }
           ,
            R (_) => {
            }
           ,
        }
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct OsForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct OsReverse ;
impl SpecMap for OsForward {
    type Input = OsInner ;
    type Output = OsSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Os::from_structural (input)
    }
}
impl SpecMap for OsReverse {
    type Input = OsSpec ;
    type Output = OsInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for Os {
}

# [doc = "data type for `gzip`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Gzip {
    pub id1: u8,
    pub id2: u8,
    pub compressionMethod: u8,
    pub flags: Flags,
    pub mtime: u32,
    pub extraFlags: u8,
    pub os: Os,
}
# [verifier::ext_equal]
pub struct GzipSpec < T0 = u8, T1 = u8, T2 = u8, T3 = FlagsSpec, T4 = u32, T5 = u8, T6 = OsSpec > {
    pub id1: T0,
    pub id2: T1,
    pub compressionMethod: T2,
    pub flags: T3,
    pub mtime: T4,
    pub extraFlags: T5,
    pub os: T6,
}
pub type GzipInner = (u8, (u8, (u8, (FlagsSpec, (u32, (u8, OsSpec)))))) ;
impl DeepView for Gzip {
    type V = GzipSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        GzipSpec {
            id1: self.id1.deep_view(),
            id2: self.id2.deep_view(),
            compressionMethod: self.compressionMethod.deep_view(),
            flags: self.flags.deep_view(),
            mtime: self.mtime.deep_view(),
            extraFlags: self.extraFlags.deep_view(),
            os: self.os.deep_view(),
        }
    }
}
impl Gzip {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().id1 == self.id1.deep_view(),
    self.deep_view().id2 == self.id2.deep_view(),
    self.deep_view().compressionMethod == self.compressionMethod.deep_view(),
    self.deep_view().flags == self.flags.deep_view(),
    self.deep_view().mtime == self.mtime.deep_view(),
    self.deep_view().extraFlags == self.extraFlags.deep_view(),
    self.deep_view().os == self.os.deep_view(),
    {
        reveal(< Gzip as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6 > GzipSpec < T0, T1, T2, T3, T4, T5, T6 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6))))))) -> Self {
        let (id1,
        (id2,
        (compressionMethod,
        (flags,
        (mtime,
        (extraFlags,
        os)))))) = input ;
        Self {
            id1,
            id2,
            compressionMethod,
            flags,
            mtime,
            extraFlags,
            os
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6)))))) {
        let Self {
            id1,
            id2,
            compressionMethod,
            flags,
            mtime,
            extraFlags,
            os
        }
        = self ;
        (id1,
        (id2,
        (compressionMethod,
        (flags,
        (mtime,
        (extraFlags,
        os))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(GzipSpec::from_structural) ;
        reveal(GzipSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(GzipSpec::from_structural) ;
        reveal(GzipSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            id1,
            id2,
            compressionMethod,
            flags,
            mtime,
            extraFlags,
            os
        }
        => (id1,
        (id2,
        (compressionMethod,
        (flags,
        (mtime,
        (extraFlags,
        os)))))),
    }
   ,
    {
        reveal(GzipSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct GzipForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct GzipReverse ;
impl SpecMap for GzipForward {
    type Input = GzipInner ;
    type Output = GzipSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        GzipSpec::from_structural (input)
    }
}
impl SpecMap for GzipReverse {
    type Input = GzipSpec ;
    type Output = GzipInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `flags`."]
# [derive (Clone, Copy)]
pub struct FlagsFmt ;

pub const FLAGS__PAD1_MASK: u8 = 0b00000111u8 ;
pub const FLAGS__PAD1_SHIFT: u8 = 5 ;
pub const FLAGS__PAD1_MAX: u8 = 0b00001000u8 ;
pub const FLAGS_COMMENT_MASK: u8 = 0b00000001u8 ;
pub const FLAGS_COMMENT_SHIFT: u8 = 4 ;
pub const FLAGS_COMMENT_MAX: u8 = 0b00000010u8 ;
pub const FLAGS_NAME_MASK: u8 = 0b00000001u8 ;
pub const FLAGS_NAME_SHIFT: u8 = 3 ;
pub const FLAGS_NAME_MAX: u8 = 0b00000010u8 ;
pub const FLAGS_EXTRA_MASK: u8 = 0b00000001u8 ;
pub const FLAGS_EXTRA_SHIFT: u8 = 2 ;
pub const FLAGS_EXTRA_MAX: u8 = 0b00000010u8 ;
pub const FLAGS_HCRC_MASK: u8 = 0b00000001u8 ;
pub const FLAGS_HCRC_SHIFT: u8 = 1 ;
pub const FLAGS_HCRC_MAX: u8 = 0b00000010u8 ;
pub const FLAGS_TEXT_MASK: u8 = 0b00000001u8 ;
pub const FLAGS_TEXT_SHIFT: u8 = 0 ;
pub const FLAGS_TEXT_MAX: u8 = 0b00000010u8 ;
# [verifier::allow_in_spec]
pub fn unpack_flags (raw: u8) -> (u8, u8, u8, u8, u8, u8) returns ((((raw >> FLAGS__PAD1_SHIFT) & FLAGS__PAD1_MASK) as u8), (((raw >> FLAGS_COMMENT_SHIFT) & FLAGS_COMMENT_MASK) as u8), (((raw >> FLAGS_NAME_SHIFT) & FLAGS_NAME_MASK) as u8), (((raw >> FLAGS_EXTRA_SHIFT) & FLAGS_EXTRA_MASK) as u8), (((raw >> FLAGS_HCRC_SHIFT) & FLAGS_HCRC_MASK) as u8), ((raw & FLAGS_TEXT_MASK) as u8)), {
    ((((raw >> FLAGS__PAD1_SHIFT) & FLAGS__PAD1_MASK) as u8),
    (((raw >> FLAGS_COMMENT_SHIFT) & FLAGS_COMMENT_MASK) as u8),
    (((raw >> FLAGS_NAME_SHIFT) & FLAGS_NAME_MASK) as u8),
    (((raw >> FLAGS_EXTRA_SHIFT) & FLAGS_EXTRA_MASK) as u8),
    (((raw >> FLAGS_HCRC_SHIFT) & FLAGS_HCRC_MASK) as u8),
    ((raw & FLAGS_TEXT_MASK) as u8))
}
# [verifier::allow_in_spec]
pub fn pack_flags (_pad1: u8, comment: u8, name: u8, extra: u8, hcrc: u8, text: u8) -> u8 returns (((_pad1 as u8) & FLAGS__PAD1_MASK) << FLAGS__PAD1_SHIFT) | (((comment as u8) & FLAGS_COMMENT_MASK) << FLAGS_COMMENT_SHIFT) | (((name as u8) & FLAGS_NAME_MASK) << FLAGS_NAME_SHIFT) | (((extra as u8) & FLAGS_EXTRA_MASK) << FLAGS_EXTRA_SHIFT) | (((hcrc as u8) & FLAGS_HCRC_MASK) << FLAGS_HCRC_SHIFT) | (((text as u8) & FLAGS_TEXT_MASK)), {
    (((_pad1 as u8) & FLAGS__PAD1_MASK) << FLAGS__PAD1_SHIFT) | (((comment as u8) & FLAGS_COMMENT_MASK) << FLAGS_COMMENT_SHIFT) | (((name as u8) & FLAGS_NAME_MASK) << FLAGS_NAME_SHIFT) | (((extra as u8) & FLAGS_EXTRA_MASK) << FLAGS_EXTRA_SHIFT) | (((hcrc as u8) & FLAGS_HCRC_MASK) << FLAGS_HCRC_SHIFT) | (((text as u8) & FLAGS_TEXT_MASK))
}
# [verifier::allow_in_spec]
pub fn flags_bounds (_pad1: u8, comment: u8, name: u8, extra: u8, hcrc: u8, text: u8) -> bool returns (_pad1 < FLAGS__PAD1_MAX) && (comment < FLAGS_COMMENT_MAX) && (name < FLAGS_NAME_MAX) && (extra < FLAGS_EXTRA_MAX) && (hcrc < FLAGS_HCRC_MAX) && (text < FLAGS_TEXT_MAX), {
    (_pad1 < FLAGS__PAD1_MAX) && (comment < FLAGS_COMMENT_MAX) && (name < FLAGS_NAME_MAX) && (extra < FLAGS_EXTRA_MAX) && (hcrc < FLAGS_HCRC_MAX) && (text < FLAGS_TEXT_MAX)
}
pub broadcast proof fn lemma_flags_unpack_pack (raw: u8) by (bit_vector) ensures # [trigger]
pack_flags (unpack_flags (raw).0, unpack_flags (raw).1, unpack_flags (raw).2, unpack_flags (raw).3, unpack_flags (raw).4, unpack_flags (raw).5) == raw, {
}
pub broadcast proof fn lemma_flags_pack_unpack (_pad1: u8, comment: u8, name: u8, extra: u8, hcrc: u8, text: u8) by (bit_vector) requires # [trigger] flags_bounds (_pad1, comment, name, extra, hcrc, text), ensures unpack_flags (pack_flags (_pad1, comment, name, extra, hcrc, text)).0 == _pad1, unpack_flags (pack_flags (_pad1, comment, name, extra, hcrc, text)).1 == comment, unpack_flags (pack_flags (_pad1, comment, name, extra, hcrc, text)).2 == name, unpack_flags (pack_flags (_pad1, comment, name, extra, hcrc, text)).3 == extra, unpack_flags (pack_flags (_pad1, comment, name, extra, hcrc, text)).4 == hcrc, unpack_flags (pack_flags (_pad1, comment, name, extra, hcrc, text)).5 == text, {
}
pub broadcast proof fn lemma_flags_mapper_wf_in_out (i: u8) by (bit_vector) ensures # [trigger] flags_bounds (unpack_flags (i).0, unpack_flags (i).1, unpack_flags (i).2, unpack_flags (i).3, unpack_flags (i).4, unpack_flags (i).5), {
}

pub type FlagsFmtSpec = Named < Bits < U8, (u8, u8, u8, u8, u8, u8), FlagsSpec > > ;

impl FlagsFmt {
    # [doc = "specification constructor for `flags`."] pub open spec fn spec_inner() -> FlagsFmtSpec {
        Named ("flags",
        Bits {
            repr: U8,
            unpack: | packed: u8 | unpack_flags (packed),
            pack: | unpacked: (u8,
            u8,
            u8,
            u8,
            u8,
            u8) | {
                let (_pad1,
                comment,
                name,
                extra,
                hcrc,
                text) = unpacked ;
                pack_flags (_pad1,
                comment,
                name,
                extra,
                hcrc,
                text)
            }
           ,
            refinement: | unpacked: (u8,
            u8,
            u8,
            u8,
            u8,
            u8) | {
                let (_pad1,
                comment,
                name,
                extra,
                hcrc,
                text) = unpacked ;
                true
            }
           ,
            ctor: | unpacked: (u8,
            u8,
            u8,
            u8,
            u8,
            u8) | {
                let (_pad1,
                comment,
                name,
                extra,
                hcrc,
                text) = unpacked ;
                FlagsSpec {
                    _pad1: _pad1,
                    comment: comment,
                    name: name,
                    extra: extra,
                    hcrc: hcrc,
                    text: text
                }
            }
           ,
            dtor: | value: FlagsSpec | {
                let FlagsSpec {
                    _pad1,
                    comment,
                    name,
                    extra,
                    hcrc,
                    text
                }
                = value ;
                (_pad1,
                comment,
                name,
                extra,
                hcrc,
                text)
            }
           ,
            consistent: | value: FlagsSpec | {
                let FlagsSpec {
                    _pad1,
                    comment,
                    name,
                    extra,
                    hcrc,
                    text
                }
                = value ;
                flags_bounds (_pad1,
                comment,
                name,
                extra,
                hcrc,
                text)
            }
           ,
        }
        )
    }
}


# [doc = "named format combinator for `os`."]
# [derive (Clone, Copy)]
pub struct OsFmt ;

pub type OsFmtSpec = Named < Mapped < Choice < Refined < U8, PredFnSpec < u8 >>, Refined < U8, PredFnSpec < u8 >> >, BiMap < OsForward, OsReverse >> > ;

impl OsFmt {
    # [doc = "specification constructor for `os`."] pub open spec fn spec_inner() -> OsFmtSpec {
        Named ("os",
        Mapped {
            inner: Choice (Refined (U8,
            | x: u8 | (((((x == 0) || (x == 1)) || (x == 2)) || (x == 3)) || (x == 7)) || (x == 11)),
            Refined (U8,
            | x: u8 | (((((x != 0) && (x != 1)) && (x != 2)) && (x != 3)) && (x != 7)) && (x != 11))),
            mapper: BiMap (OsForward,
            OsReverse),
        }
        )
    }
}


# [doc = "named format combinator for `gzip`."]
# [derive (Clone, Copy)]
pub struct GzipFmt ;

pub type GzipFmtSpec = Named < Mapped < Pair < U8, Pair < U8, Pair < U8, Pair < FlagsFmt, Pair < U32Le, Pair < U8, OsFmt > > > > > >, BiMap < GzipForward, GzipReverse >> > ;

impl GzipFmt {
    # [doc = "specification constructor for `gzip`."] pub open spec fn spec_inner() -> GzipFmtSpec {
        Named ("gzip",
        Mapped {
            inner: Pair (U8,
            Pair (U8,
            Pair (U8,
            Pair (FlagsFmt,
            Pair (U32Le,
            Pair (U8,
            OsFmt)))))),
            mapper: BiMap (GzipForward,
            GzipReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for FlagsFmt {
        type PVal = FlagsSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for FlagsFmt {
        type Val = FlagsSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for FlagsFmt {
        type SValue = FlagsSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for FlagsFmt {
        type SVal = FlagsSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for FlagsFmt {
        type T = FlagsSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for OsFmt {
        type PVal = OsSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for OsFmt {
        type Val = OsSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for OsFmt {
        type SValue = OsSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for OsFmt {
        type SVal = OsSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for OsFmt {
        type T = OsSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for GzipFmt {
        type PVal = GzipSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for GzipFmt {
        type Val = GzipSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for GzipFmt {
        type SValue = GzipSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for GzipFmt {
        type SVal = GzipSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for GzipFmt {
        type T = GzipSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }
}

// ============================================================
// Proven Format Properties
// ============================================================
mod derived_proofs {
    use super::*;
    broadcast use {
        vest_lib::combinators::disjoint::disjointness_lemmas,
        Os::lemma_from_into,
        Os::lemma_into_from,
        GzipSpec::lemma_from_into,
        GzipSpec::lemma_into_from,
    };

    impl SafeParser for FlagsFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for FlagsFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for FlagsFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            reveal(< FlagsFmt as SpecByteLen>::byte_len) ;
            let fmt = FlagsFmt::spec_inner() ;
            broadcast use lemma_flags_unpack_pack,
            lemma_flags_mapper_wf_in_out ;
            assert (fmt.1.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            reveal(< FlagsFmt as Consistency>::consistent) ;
            broadcast use lemma_flags_unpack_pack,
            lemma_flags_mapper_wf_in_out ;
            let fmt = FlagsFmt::spec_inner() ;
            assert (fmt.1.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for FlagsFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< FlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< FlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FlagsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for FlagsFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< FlagsFmt as SpecSerializer>::spec_serialize) ;
            reveal(< FlagsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for FlagsFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< FlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FlagsFmt as SpecByteLen>::byte_len) ;
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            broadcast use lemma_flags_pack_unpack ;
            let fmt = FlagsFmt::spec_inner() ;
            assert (fmt.1.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for FlagsFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            broadcast use lemma_flags_unpack_pack,
            lemma_flags_mapper_wf_in_out ;
            let fmt = FlagsFmt::spec_inner() ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for FlagsFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< FlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FlagsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for FlagsFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< FlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FlagsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for OsFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< OsFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for OsFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< OsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for OsFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< OsFmt as SpecParser>::spec_parse) ;
            reveal(< OsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: OsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Os::structural_valid (input)) ;
                Os::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< OsFmt as SpecParser>::spec_parse) ;
            reveal(< OsFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: OsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Os::structural_valid (input)) ;
                Os::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for OsFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< OsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< OsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< OsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for OsFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< OsFmt as SpecSerializer>::spec_serialize) ;
            reveal(< OsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for OsFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< OsFmt as SpecParser>::spec_parse) ;
            reveal(< OsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< OsFmt as Consistency>::consistent) ;
            reveal(< OsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: OsSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Os::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for OsFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< OsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: OsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Os::structural_valid (input)) ;
                Os::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for OsFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< OsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< OsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for OsFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< OsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< OsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for GzipFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< GzipFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for GzipFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< GzipFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for GzipFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< GzipFmt as SpecParser>::spec_parse) ;
            reveal(< GzipFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: GzipInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                GzipSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< GzipFmt as SpecParser>::spec_parse) ;
            reveal(< GzipFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: GzipInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                GzipSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for GzipFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< GzipFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< GzipFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< GzipFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for GzipFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< GzipFmt as SpecSerializer>::spec_serialize) ;
            reveal(< GzipFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for GzipFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< GzipFmt as SpecParser>::spec_parse) ;
            reveal(< GzipFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< GzipFmt as Consistency>::consistent) ;
            reveal(< GzipFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: GzipSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                GzipSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for GzipFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< GzipFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: GzipInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                GzipSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for GzipFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< GzipFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< GzipFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for GzipFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< GzipFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< GzipFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }
}

// ============================================================
// Executable Implementations
// ============================================================
mod exec_impls {
    use super::*;

    impl<'i> Parser<&'i [u8]> for FlagsFmt {
        type PT = Flags;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<FlagsFmt as SpecParser>::spec_parse);
            reveal(<Flags as DeepView>::deep_view);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, raw) = U8.parse(ibuf)?;
            let (_pad1, comment, name, extra, hcrc, text) = unpack_flags(raw);
            let final_v = Flags {
                _pad1: _pad1,
                comment: comment,
                name: name,
                extra: extra,
                hcrc: hcrc,
                text: text,
            };
            assert(self.spec_parse(ibuf@) == Some((n as int, final_v.deep_view())));
            Ok((n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Flags> for FlagsFmt {
        fn serialize_into(&self, v: &Flags, obuf: &mut Output) {
            reveal(<FlagsFmt as SpecSerializer>::spec_serialize);
            reveal(<FlagsFmt as SpecByteLen>::byte_len);
            reveal(<Flags as DeepView>::deep_view);
            let ghost old_obuf = obuf@;

            let Flags {
                _pad1,
                comment,
                name,
                extra,
                hcrc,
                text
            }
            = *v ;
            let packed = pack_flags(_pad1, comment, name, extra, hcrc, text);
            U8.serialize_into(&packed, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Flags> for FlagsFmt {
        fn prepare(&self, v: &Flags) -> Result<usize, PreSerializeError> {
            reveal(<FlagsFmt as SpecByteLen>::byte_len);
            reveal(<Flags as DeepView>::deep_view);
            let Flags {
                _pad1,
                comment,
                name,
                extra,
                hcrc,
                text
            }
            = *v ;
            if !(flags_bounds (_pad1, comment, name, extra, hcrc, text)) {
                return Err(PreSerializeError::not_compliant(ComplianceErrorKind::PredicateFailed));
            }
            let packed = pack_flags(_pad1, comment, name, extra, hcrc, text);
            U8.prepare(&packed)
        }
    }



    impl<'i> Parser<&'i [u8]> for OsFmt {
        type PT = Os;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<OsFmt as SpecParser>::spec_parse);
            reveal(<Os as DeepView>::deep_view);
            reveal(Os::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U8.parse(&rest)?;
            let enum_val = match v {
                0 => Os::Fat,
                1 => Os::Amiga,
                2 => Os::Vms,
                3 => Os::Unix,
                7 => Os::Macintosh,
                11 => Os::Ntfs,
                x => Os::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Os> for OsFmt {
        fn serialize_into(&self, v: &Os, obuf: &mut Output) {
            reveal(<OsFmt as SpecSerializer>::spec_serialize);
            reveal(<OsFmt as SpecByteLen>::byte_len);
            reveal(<Os as DeepView>::deep_view);
            reveal(Os::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                Os::Fat => 0,
                Os::Amiga => 1,
                Os::Vms => 2,
                Os::Unix => 3,
                Os::Macintosh => 7,
                Os::Ntfs => 11,
                Os::Unknown (x) => x,
            };
            U8.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Os> for OsFmt {
        fn prepare(&self, v: &Os) -> Result<usize, PreSerializeError> {
            reveal(<OsFmt as SpecByteLen>::byte_len);
            reveal(<Os as DeepView>::deep_view);
            reveal(Os::into_structural);
            let tag = match *v {
                Os::Fat => 0,
                Os::Amiga => 1,
                Os::Vms => 2,
                Os::Unix => 3,
                Os::Macintosh => 7,
                Os::Ntfs => 11,
                Os::Unknown (x) if x != 0 && x != 1 && x != 2 && x != 3 && x != 7 && x != 11 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U8.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for GzipFmt {
        type PT = Gzip;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<GzipFmt as SpecParser>::spec_parse);
            reveal(<Gzip as DeepView>::deep_view);
            reveal(GzipSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, id1) = (U8).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, id2) = (U8).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, compressionMethod) = (U8).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, flags) = (Named ("flags", FlagsFmt)).parse (& rest) ?;
            proof {
                flags.lemma_deep_view();
            }
            let rest = rest.skip(n4);
            let (n5, mtime) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, extraFlags) = (U8).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, os) = (Named ("os", OsFmt)).parse (& rest) ?;
            proof {
                os.lemma_deep_view();
            }
            let rest = rest.skip(n7);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7;
            let final_v = Gzip {
                id1,
                id2,
                compressionMethod,
                flags,
                mtime,
                extraFlags,
                os,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Gzip> for GzipFmt {
        fn serialize_into(&self, v: &Gzip, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<GzipFmt as SpecSerializer>::spec_serialize);
            reveal(<GzipFmt as SpecByteLen>::byte_len);
            reveal(<Gzip as DeepView>::deep_view);
            reveal(GzipSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Gzip {
                id1,
                id2,
                compressionMethod,
                flags,
                mtime,
                extraFlags,
                os,
            } = v;
            proof {
                flags.lemma_deep_view();
                os.lemma_deep_view();
            }

            U8.serialize_into(id1, obuf);
            U8.serialize_into(id2, obuf);
            U8.serialize_into(compressionMethod, obuf);
            FlagsFmt.serialize_into(flags, obuf);
            U32Le.serialize_into(mtime, obuf);
            U8.serialize_into(extraFlags, obuf);
            OsFmt.serialize_into(os, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Gzip> for GzipFmt {
        fn prepare(&self, v: &Gzip) -> Result<usize, PreSerializeError> {
            reveal(<GzipFmt as SpecByteLen>::byte_len);
            reveal(<Gzip as DeepView>::deep_view);
            reveal(GzipSpec::into_structural);
            let Gzip {
                id1,
                id2,
                compressionMethod,
                flags,
                mtime,
                extraFlags,
                os,
            } = v;
            proof {
                flags.lemma_deep_view();
                os.lemma_deep_view();
            }

            let l1 = (U8).prepare (id1) ?;
            let l2 = (U8).prepare (id2) ?;
            let l3 = (U8).prepare (compressionMethod) ?;
            let l4 = (Named ("flags", FlagsFmt)).prepare (flags) ?;
            let l5 = (U32Le).prepare (mtime) ?;
            let l6 = (U8).prepare (extraFlags) ?;
            let l7 = (Named ("os", OsFmt)).prepare (os) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
