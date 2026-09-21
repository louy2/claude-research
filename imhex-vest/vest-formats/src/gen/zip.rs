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
# [doc = "data type for `method`."]
# [repr (u16)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum Method {
    Stored = 0,
    Deflated = 8,
    Bzip2 = 12,
    Lzma = 14,
    Zstd = 93,
    Unknown (u16),
}
pub type MethodSpec = Method ;
pub type MethodInner = Sum < u16, u16 > ;
impl DeepView for Method {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Method {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Method as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: MethodInner) -> bool {
        match input {
            L (x) => x == 0 || x == 8 || x == 12 || x == 14 || x == 93,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: MethodInner) -> Self {
        match input {
            L (x) => match x {
                0 => Self::Stored,
                8 => Self::Deflated,
                12 => Self::Bzip2,
                14 => Self::Lzma,
                93 => Self::Zstd,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> MethodInner {
        match self {
            Self::Stored => L (0),
            Self::Deflated => L (8),
            Self::Bzip2 => L (12),
            Self::Lzma => L (14),
            Self::Zstd => L (93),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Method::from_structural) ;
        reveal(Method::into_structural) ;
        match self {
            Self::Stored => {
            }
           ,
            Self::Deflated => {
            }
           ,
            Self::Bzip2 => {
            }
           ,
            Self::Lzma => {
            }
           ,
            Self::Zstd => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: MethodInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Method::from_structural) ;
        reveal(Method::into_structural) ;
        match input {
            L (x) => match x {
                0 => {
                }
               ,
                8 => {
                }
               ,
                12 => {
                }
               ,
                14 => {
                }
               ,
                93 => {
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
pub struct MethodForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct MethodReverse ;
impl SpecMap for MethodForward {
    type Input = MethodInner ;
    type Output = MethodSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Method::from_structural (input)
    }
}
impl SpecMap for MethodReverse {
    type Input = MethodSpec ;
    type Output = MethodInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for Method {
}

# [doc = "data type for `local_file_header`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct LocalFileHeader<'i> {
    pub signature: &'i [u8],
    pub version: u16,
    pub flags: u16,
    pub method: Method,
    pub modTime: u16,
    pub modDate: u16,
    pub crc32: u32,
    pub compressedSize: u32,
    pub uncompressedSize: u32,
    pub nameLength: u16,
    pub extraLength: u16,
    pub name: &'i [u8],
    pub extra: &'i [u8],
    pub data: &'i [u8],
}
# [verifier::ext_equal]
pub struct LocalFileHeaderSpec < T0 = Seq < u8 >, T1 = u16, T2 = u16, T3 = MethodSpec, T4 = u16, T5 = u16, T6 = u32, T7 = u32, T8 = u32, T9 = u16, T10 = u16, T11 = Seq < u8 >, T12 = Seq < u8 >, T13 = Seq < u8 > > {
    pub signature: T0,
    pub version: T1,
    pub flags: T2,
    pub method: T3,
    pub modTime: T4,
    pub modDate: T5,
    pub crc32: T6,
    pub compressedSize: T7,
    pub uncompressedSize: T8,
    pub nameLength: T9,
    pub extraLength: T10,
    pub name: T11,
    pub extra: T12,
    pub data: T13,
}
pub type LocalFileHeaderInner = (Seq < u8 >, (u16, (u16, (MethodSpec, (u16, (u16, (u32, (u32, (u32, (u16, (u16, (Seq < u8 >, (Seq < u8 >, Seq < u8 >))))))))))))) ;
impl<'i> DeepView for LocalFileHeader<'i> {
    type V = LocalFileHeaderSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        LocalFileHeaderSpec {
            signature: self.signature.deep_view(),
            version: self.version.deep_view(),
            flags: self.flags.deep_view(),
            method: self.method.deep_view(),
            modTime: self.modTime.deep_view(),
            modDate: self.modDate.deep_view(),
            crc32: self.crc32.deep_view(),
            compressedSize: self.compressedSize.deep_view(),
            uncompressedSize: self.uncompressedSize.deep_view(),
            nameLength: self.nameLength.deep_view(),
            extraLength: self.extraLength.deep_view(),
            name: self.name.deep_view(),
            extra: self.extra.deep_view(),
            data: self.data.deep_view(),
        }
    }
}
impl<'i> LocalFileHeader<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().signature == self.signature.deep_view(),
    self.deep_view().version == self.version.deep_view(),
    self.deep_view().flags == self.flags.deep_view(),
    self.deep_view().method == self.method.deep_view(),
    self.deep_view().modTime == self.modTime.deep_view(),
    self.deep_view().modDate == self.modDate.deep_view(),
    self.deep_view().crc32 == self.crc32.deep_view(),
    self.deep_view().compressedSize == self.compressedSize.deep_view(),
    self.deep_view().uncompressedSize == self.uncompressedSize.deep_view(),
    self.deep_view().nameLength == self.nameLength.deep_view(),
    self.deep_view().extraLength == self.extraLength.deep_view(),
    self.deep_view().name == self.name.deep_view(),
    self.deep_view().extra == self.extra.deep_view(),
    self.deep_view().data == self.data.deep_view(),
    {
        reveal(< LocalFileHeader as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13 > LocalFileHeaderSpec < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    (T7,
    (T8,
    (T9,
    (T10,
    (T11,
    (T12,
    T13)))))))))))))) -> Self {
        let (signature,
        (version,
        (flags,
        (method,
        (modTime,
        (modDate,
        (crc32,
        (compressedSize,
        (uncompressedSize,
        (nameLength,
        (extraLength,
        (name,
        (extra,
        data))))))))))))) = input ;
        Self {
            signature,
            version,
            flags,
            method,
            modTime,
            modDate,
            crc32,
            compressedSize,
            uncompressedSize,
            nameLength,
            extraLength,
            name,
            extra,
            data
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    (T7,
    (T8,
    (T9,
    (T10,
    (T11,
    (T12,
    T13))))))))))))) {
        let Self {
            signature,
            version,
            flags,
            method,
            modTime,
            modDate,
            crc32,
            compressedSize,
            uncompressedSize,
            nameLength,
            extraLength,
            name,
            extra,
            data
        }
        = self ;
        (signature,
        (version,
        (flags,
        (method,
        (modTime,
        (modDate,
        (crc32,
        (compressedSize,
        (uncompressedSize,
        (nameLength,
        (extraLength,
        (name,
        (extra,
        data)))))))))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(LocalFileHeaderSpec::from_structural) ;
        reveal(LocalFileHeaderSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    (T7,
    (T8,
    (T9,
    (T10,
    (T11,
    (T12,
    T13)))))))))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(LocalFileHeaderSpec::from_structural) ;
        reveal(LocalFileHeaderSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            signature,
            version,
            flags,
            method,
            modTime,
            modDate,
            crc32,
            compressedSize,
            uncompressedSize,
            nameLength,
            extraLength,
            name,
            extra,
            data
        }
        => (signature,
        (version,
        (flags,
        (method,
        (modTime,
        (modDate,
        (crc32,
        (compressedSize,
        (uncompressedSize,
        (nameLength,
        (extraLength,
        (name,
        (extra,
        data))))))))))))),
    }
   ,
    {
        reveal(LocalFileHeaderSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct LocalFileHeaderForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct LocalFileHeaderReverse ;
impl SpecMap for LocalFileHeaderForward {
    type Input = LocalFileHeaderInner ;
    type Output = LocalFileHeaderSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        LocalFileHeaderSpec::from_structural (input)
    }
}
impl SpecMap for LocalFileHeaderReverse {
    type Input = LocalFileHeaderSpec ;
    type Output = LocalFileHeaderInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `zip`."]
# [derive (Debug, PartialEq, Eq, Clone)]
pub struct Zip<'i> {
    pub entries: Vec < LocalFileHeader<'i> >,
}
# [verifier::ext_equal]
pub struct ZipSpec < T0 = Seq < LocalFileHeaderSpec > > {
    pub entries: T0,
}
pub type ZipInner = Seq < LocalFileHeaderSpec > ;
impl<'i> DeepView for Zip<'i> {
    type V = ZipSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        ZipSpec {
            entries: self.entries.deep_view(),
        }
    }
}
impl<'i> Zip<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().entries == self.entries.deep_view(),
    {
        reveal(< Zip as DeepView>::deep_view) ;
    }
}
impl < T0 > ZipSpec < T0 > {
    # [verifier::opaque] pub open spec fn from_structural (input: T0) -> Self {
        let entries = input ;
        Self {
            entries
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> T0 {
        let Self {
            entries
        }
        = self ;
        entries
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(ZipSpec::from_structural) ;
        reveal(ZipSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: T0) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(ZipSpec::from_structural) ;
        reveal(ZipSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            entries
        }
        => entries,
    }
   ,
    {
        reveal(ZipSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ZipForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ZipReverse ;
impl SpecMap for ZipForward {
    type Input = ZipInner ;
    type Output = ZipSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        ZipSpec::from_structural (input)
    }
}
impl SpecMap for ZipReverse {
    type Input = ZipSpec ;
    type Output = ZipInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `method`."]
# [derive (Clone, Copy)]
pub struct MethodFmt ;

pub type MethodFmtSpec = Named < Mapped < Choice < Refined < U16Le, PredFnSpec < u16 >>, Refined < U16Le, PredFnSpec < u16 >> >, BiMap < MethodForward, MethodReverse >> > ;

impl MethodFmt {
    # [doc = "specification constructor for `method`."] pub open spec fn spec_inner() -> MethodFmtSpec {
        Named ("method",
        Mapped {
            inner: Choice (Refined (U16Le,
            | x: u16 | ((((x == 0) || (x == 8)) || (x == 12)) || (x == 14)) || (x == 93)),
            Refined (U16Le,
            | x: u16 | ((((x != 0) && (x != 8)) && (x != 12)) && (x != 14)) && (x != 93))),
            mapper: BiMap (MethodForward,
            MethodReverse),
        }
        )
    }
}


# [doc = "named format combinator for `local_file_header`."]
# [derive (Clone, Copy)]
pub struct LocalFileHeaderFmt ;

pub type LocalFileHeaderFmtSpec = Named < Mapped < Pair < Fixed < 4 >, Pair < U16Le, Pair < U16Le, Pair < MethodFmt, Pair < U16Le, Pair < U16Le, Pair < U32Le, Bind < U32Le, spec_fn (u32) -> Pair < U32Le, Bind < U16Le, spec_fn (u16) -> Bind < U16Le, spec_fn (u16) -> Pair < Varied < u16 >, Pair < Varied < u16 >, Varied < u32 > > > > > > > > > > > > > >, BiMap < LocalFileHeaderForward, LocalFileHeaderReverse >> > ;

impl LocalFileHeaderFmt {
    # [doc = "specification constructor for `local_file_header`."] pub open spec fn spec_inner() -> LocalFileHeaderFmtSpec {
        Named ("local_file_header",
        Mapped {
            inner: Pair (Fixed::< 4 >,
            Pair (U16Le,
            Pair (U16Le,
            Pair (MethodFmt,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U32Le,
            Bind (U32Le,
            | compressedSize: u32 | Pair (U32Le,
            Bind (U16Le,
            | nameLength: u16 | Bind (U16Le,
            | extraLength: u16 | Pair (Varied (nameLength),
            Pair (Varied (extraLength),
            Varied (compressedSize)))))))))))))),
            mapper: BiMap (LocalFileHeaderForward,
            LocalFileHeaderReverse),
        }
        )
    }
}


# [doc = "named format combinator for `zip`."]
# [derive (Clone, Copy)]
pub struct ZipFmt ;

pub type ZipFmtSpec = Named < Mapped < RepeatTillEnd < LocalFileHeaderFmt >, BiMap < ZipForward, ZipReverse >> > ;

impl ZipFmt {
    # [doc = "specification constructor for `zip`."] pub open spec fn spec_inner() -> ZipFmtSpec {
        Named ("zip",
        Mapped {
            inner: RepeatTillEnd (LocalFileHeaderFmt),
            mapper: BiMap (ZipForward,
            ZipReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for MethodFmt {
        type PVal = MethodSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for MethodFmt {
        type Val = MethodSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for MethodFmt {
        type SValue = MethodSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for MethodFmt {
        type SVal = MethodSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for MethodFmt {
        type T = MethodSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for LocalFileHeaderFmt {
        type PVal = LocalFileHeaderSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for LocalFileHeaderFmt {
        type Val = LocalFileHeaderSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for LocalFileHeaderFmt {
        type SValue = LocalFileHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for LocalFileHeaderFmt {
        type SVal = LocalFileHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for LocalFileHeaderFmt {
        type T = LocalFileHeaderSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for ZipFmt {
        type PVal = ZipSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ZipFmt {
        type Val = ZipSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ZipFmt {
        type SValue = ZipSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ZipFmt {
        type SVal = ZipSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ZipFmt {
        type T = ZipSpec ;
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
        Method::lemma_from_into,
        Method::lemma_into_from,
        LocalFileHeaderSpec::lemma_from_into,
        LocalFileHeaderSpec::lemma_into_from,
        ZipSpec::lemma_from_into,
        ZipSpec::lemma_into_from,
    };

    impl SafeParser for MethodFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< MethodFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for MethodFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< MethodFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for MethodFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< MethodFmt as SpecParser>::spec_parse) ;
            reveal(< MethodFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: MethodInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Method::structural_valid (input)) ;
                Method::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< MethodFmt as SpecParser>::spec_parse) ;
            reveal(< MethodFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: MethodInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Method::structural_valid (input)) ;
                Method::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for MethodFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< MethodFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< MethodFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< MethodFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for MethodFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< MethodFmt as SpecSerializer>::spec_serialize) ;
            reveal(< MethodFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for MethodFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< MethodFmt as SpecParser>::spec_parse) ;
            reveal(< MethodFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< MethodFmt as Consistency>::consistent) ;
            reveal(< MethodFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: MethodSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Method::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for MethodFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< MethodFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: MethodInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Method::structural_valid (input)) ;
                Method::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for MethodFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< MethodFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< MethodFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for MethodFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< MethodFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< MethodFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for LocalFileHeaderFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for LocalFileHeaderFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for LocalFileHeaderFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< LocalFileHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LocalFileHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LocalFileHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< LocalFileHeaderFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LocalFileHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LocalFileHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for LocalFileHeaderFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LocalFileHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for LocalFileHeaderFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< LocalFileHeaderFmt as SpecSerializer>::spec_serialize) ;
            reveal(< LocalFileHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for LocalFileHeaderFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< LocalFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LocalFileHeaderFmt as Consistency>::consistent) ;
            reveal(< LocalFileHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: LocalFileHeaderSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                LocalFileHeaderSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for LocalFileHeaderFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LocalFileHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LocalFileHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for LocalFileHeaderFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< LocalFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LocalFileHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for LocalFileHeaderFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< LocalFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LocalFileHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for ZipFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ZipFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ZipFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ZipFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ZipFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ZipFmt as SpecParser>::spec_parse) ;
            reveal(< ZipFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ZipInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ZipSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ZipFmt as SpecParser>::spec_parse) ;
            reveal(< ZipFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ZipInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ZipSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl GoodSerializer for ZipFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ZipFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ZipFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ZipFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ZipFmt as SpecParser>::spec_parse) ;
            reveal(< ZipFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ZipFmt as Consistency>::consistent) ;
            reveal(< ZipFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ZipSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                ZipSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ZipFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ZipFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ZipInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ZipSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializers for ZipFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ZipFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ZipFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for MethodFmt {
        type PT = Method;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<MethodFmt as SpecParser>::spec_parse);
            reveal(<Method as DeepView>::deep_view);
            reveal(Method::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U16Le.parse(&rest)?;
            let enum_val = match v {
                0 => Method::Stored,
                8 => Method::Deflated,
                12 => Method::Bzip2,
                14 => Method::Lzma,
                93 => Method::Zstd,
                x => Method::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Method> for MethodFmt {
        fn serialize_into(&self, v: &Method, obuf: &mut Output) {
            reveal(<MethodFmt as SpecSerializer>::spec_serialize);
            reveal(<MethodFmt as SpecByteLen>::byte_len);
            reveal(<Method as DeepView>::deep_view);
            reveal(Method::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                Method::Stored => 0,
                Method::Deflated => 8,
                Method::Bzip2 => 12,
                Method::Lzma => 14,
                Method::Zstd => 93,
                Method::Unknown (x) => x,
            };
            U16Le.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Method> for MethodFmt {
        fn prepare(&self, v: &Method) -> Result<usize, PreSerializeError> {
            reveal(<MethodFmt as SpecByteLen>::byte_len);
            reveal(<Method as DeepView>::deep_view);
            reveal(Method::into_structural);
            let tag = match *v {
                Method::Stored => 0,
                Method::Deflated => 8,
                Method::Bzip2 => 12,
                Method::Lzma => 14,
                Method::Zstd => 93,
                Method::Unknown (x) if x != 0 && x != 8 && x != 12 && x != 14 && x != 93 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U16Le.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for LocalFileHeaderFmt {
        type PT = LocalFileHeader<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<LocalFileHeaderFmt as SpecParser>::spec_parse);
            reveal(<LocalFileHeader as DeepView>::deep_view);
            reveal(LocalFileHeaderSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, signature) = (Fixed::< 4 >).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, version) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, flags) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, method) = (Named ("method", MethodFmt)).parse (& rest) ?;
            proof {
                method.lemma_deep_view();
            }
            let rest = rest.skip(n4);
            let (n5, modTime) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, modDate) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, crc32) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, compressedSize) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n8);
            let (n9, uncompressedSize) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n9);
            let (n10, nameLength) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n10);
            let (n11, extraLength) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n11);
            let (n12, name) = (Varied (nameLength)).parse (& rest) ?;
            let rest = rest.skip(n12);
            let (n13, extra) = (Varied (extraLength)).parse (& rest) ?;
            let rest = rest.skip(n13);
            let (n14, data) = (Varied (compressedSize)).parse (& rest) ?;
            let rest = rest.skip(n14);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9 + n10 + n11 + n12 + n13 + n14;
            let final_v = LocalFileHeader {
                signature,
                version,
                flags,
                method,
                modTime,
                modDate,
                crc32,
                compressedSize,
                uncompressedSize,
                nameLength,
                extraLength,
                name,
                extra,
                data,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, LocalFileHeader<'i>> for LocalFileHeaderFmt {
        fn serialize_into(&self, v: &LocalFileHeader<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<LocalFileHeaderFmt as SpecSerializer>::spec_serialize);
            reveal(<LocalFileHeaderFmt as SpecByteLen>::byte_len);
            reveal(<LocalFileHeader as DeepView>::deep_view);
            reveal(LocalFileHeaderSpec::into_structural);
            let ghost old_obuf = obuf@;

            let LocalFileHeader {
                signature,
                version,
                flags,
                method,
                modTime,
                modDate,
                crc32,
                compressedSize,
                uncompressedSize,
                nameLength,
                extraLength,
                name,
                extra,
                data,
            } = v;
            proof {
                method.lemma_deep_view();
            }

            Fixed::< 4 >.serialize_into(* signature, obuf);
            U16Le.serialize_into(version, obuf);
            U16Le.serialize_into(flags, obuf);
            MethodFmt.serialize_into(method, obuf);
            U16Le.serialize_into(modTime, obuf);
            U16Le.serialize_into(modDate, obuf);
            U32Le.serialize_into(crc32, obuf);
            U32Le.serialize_into(compressedSize, obuf);
            U32Le.serialize_into(uncompressedSize, obuf);
            U16Le.serialize_into(nameLength, obuf);
            U16Le.serialize_into(extraLength, obuf);
            Varied (* nameLength).serialize_into(* name, obuf);
            Varied (* extraLength).serialize_into(* extra, obuf);
            Varied (* compressedSize).serialize_into(* data, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<LocalFileHeader<'i>> for LocalFileHeaderFmt {
        fn prepare(&self, v: &LocalFileHeader<'i>) -> Result<usize, PreSerializeError> {
            reveal(<LocalFileHeaderFmt as SpecByteLen>::byte_len);
            reveal(<LocalFileHeader as DeepView>::deep_view);
            reveal(LocalFileHeaderSpec::into_structural);
            let LocalFileHeader {
                signature,
                version,
                flags,
                method,
                modTime,
                modDate,
                crc32,
                compressedSize,
                uncompressedSize,
                nameLength,
                extraLength,
                name,
                extra,
                data,
            } = v;
            proof {
                method.lemma_deep_view();
            }

            let l1 = (Fixed::< 4 >).prepare (signature) ?;
            let l2 = (U16Le).prepare (version) ?;
            let l3 = (U16Le).prepare (flags) ?;
            let l4 = (Named ("method", MethodFmt)).prepare (method) ?;
            let l5 = (U16Le).prepare (modTime) ?;
            let l6 = (U16Le).prepare (modDate) ?;
            let l7 = (U32Le).prepare (crc32) ?;
            let l8 = (U32Le).prepare (compressedSize) ?;
            let l9 = (U32Le).prepare (uncompressedSize) ?;
            let l10 = (U16Le).prepare (nameLength) ?;
            let l11 = (U16Le).prepare (extraLength) ?;
            let l12 = (Varied (* nameLength)).prepare (name) ?;
            let l13 = (Varied (* extraLength)).prepare (extra) ?;
            let l14 = (Varied (* compressedSize)).prepare (data) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l9).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l10).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l11).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l12).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l13).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l14).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for ZipFmt {
        type PT = Zip<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<ZipFmt as SpecParser>::spec_parse);
            reveal(<Zip as DeepView>::deep_view);
            reveal(ZipSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, entries) = (RepeatTillEnd (LocalFileHeaderFmt)).parse (& rest) ?;
            let rest = rest.skip(n1);
            let total_n = n1;
            let final_v = Zip {
                entries,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Zip<'i>> for ZipFmt {
        fn serialize_into(&self, v: &Zip<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<ZipFmt as SpecSerializer>::spec_serialize);
            reveal(<ZipFmt as SpecByteLen>::byte_len);
            reveal(<Zip as DeepView>::deep_view);
            reveal(ZipSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Zip {
                entries,
            } = v;
            Star (LocalFileHeaderFmt).serialize_into(entries, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Zip<'i>> for ZipFmt {
        fn prepare(&self, v: &Zip<'i>) -> Result<usize, PreSerializeError> {
            reveal(<ZipFmt as SpecByteLen>::byte_len);
            reveal(<Zip as DeepView>::deep_view);
            reveal(ZipSpec::into_structural);
            let Zip {
                entries,
            } = v;
            let l1 = (Star (LocalFileHeaderFmt)).prepare (entries) ?;
            let total_len = l1;
            Ok(total_len)
        }
    }

}
}
