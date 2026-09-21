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
# [doc = "data type for `riff_header`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct RiffHeader<'i> {
    pub riff: &'i [u8],
    pub size: u32,
    pub wave: &'i [u8],
}
# [verifier::ext_equal]
pub struct RiffHeaderSpec < T0 = Seq < u8 >, T1 = u32, T2 = Seq < u8 > > {
    pub riff: T0,
    pub size: T1,
    pub wave: T2,
}
pub type RiffHeaderInner = (Seq < u8 >, (u32, Seq < u8 >)) ;
impl<'i> DeepView for RiffHeader<'i> {
    type V = RiffHeaderSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        RiffHeaderSpec {
            riff: self.riff.deep_view(),
            size: self.size.deep_view(),
            wave: self.wave.deep_view(),
        }
    }
}
impl<'i> RiffHeader<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().riff == self.riff.deep_view(),
    self.deep_view().size == self.size.deep_view(),
    self.deep_view().wave == self.wave.deep_view(),
    {
        reveal(< RiffHeader as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2 > RiffHeaderSpec < T0, T1, T2 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    T2))) -> Self {
        let (riff,
        (size,
        wave)) = input ;
        Self {
            riff,
            size,
            wave
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    T2)) {
        let Self {
            riff,
            size,
            wave
        }
        = self ;
        (riff,
        (size,
        wave))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(RiffHeaderSpec::from_structural) ;
        reveal(RiffHeaderSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    T2))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(RiffHeaderSpec::from_structural) ;
        reveal(RiffHeaderSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            riff,
            size,
            wave
        }
        => (riff,
        (size,
        wave)),
    }
   ,
    {
        reveal(RiffHeaderSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct RiffHeaderForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct RiffHeaderReverse ;
impl SpecMap for RiffHeaderForward {
    type Input = RiffHeaderInner ;
    type Output = RiffHeaderSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        RiffHeaderSpec::from_structural (input)
    }
}
impl SpecMap for RiffHeaderReverse {
    type Input = RiffHeaderSpec ;
    type Output = RiffHeaderInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `format`."]
# [repr (u16)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum Format {
    PCM = 1,
    IEEEFloat = 3,
    ALaw = 6,
    MuLaw = 7,
    Extensible = 65534,
    Unknown (u16),
}
pub type FormatSpec = Format ;
pub type FormatInner = Sum < u16, u16 > ;
impl DeepView for Format {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Format {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Format as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: FormatInner) -> bool {
        match input {
            L (x) => x == 1 || x == 3 || x == 6 || x == 7 || x == 65534,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: FormatInner) -> Self {
        match input {
            L (x) => match x {
                1 => Self::PCM,
                3 => Self::IEEEFloat,
                6 => Self::ALaw,
                7 => Self::MuLaw,
                65534 => Self::Extensible,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> FormatInner {
        match self {
            Self::PCM => L (1),
            Self::IEEEFloat => L (3),
            Self::ALaw => L (6),
            Self::MuLaw => L (7),
            Self::Extensible => L (65534),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Format::from_structural) ;
        reveal(Format::into_structural) ;
        match self {
            Self::PCM => {
            }
           ,
            Self::IEEEFloat => {
            }
           ,
            Self::ALaw => {
            }
           ,
            Self::MuLaw => {
            }
           ,
            Self::Extensible => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: FormatInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Format::from_structural) ;
        reveal(Format::into_structural) ;
        match input {
            L (x) => match x {
                1 => {
                }
               ,
                3 => {
                }
               ,
                6 => {
                }
               ,
                7 => {
                }
               ,
                65534 => {
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
pub struct FormatForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct FormatReverse ;
impl SpecMap for FormatForward {
    type Input = FormatInner ;
    type Output = FormatSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Format::from_structural (input)
    }
}
impl SpecMap for FormatReverse {
    type Input = FormatSpec ;
    type Output = FormatInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for Format {
}

# [doc = "data type for `fmt_chunk`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct FmtChunk<'i> {
    pub id: &'i [u8],
    pub size: u32,
    pub format: Format,
    pub channels: u16,
    pub sampleRate: u32,
    pub byteRate: u32,
    pub blockAlign: u16,
    pub bitsPerSample: u16,
}
# [verifier::ext_equal]
pub struct FmtChunkSpec < T0 = Seq < u8 >, T1 = u32, T2 = FormatSpec, T3 = u16, T4 = u32, T5 = u32, T6 = u16, T7 = u16 > {
    pub id: T0,
    pub size: T1,
    pub format: T2,
    pub channels: T3,
    pub sampleRate: T4,
    pub byteRate: T5,
    pub blockAlign: T6,
    pub bitsPerSample: T7,
}
pub type FmtChunkInner = (Seq < u8 >, (u32, (FormatSpec, (u16, (u32, (u32, (u16, u16))))))) ;
impl<'i> DeepView for FmtChunk<'i> {
    type V = FmtChunkSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        FmtChunkSpec {
            id: self.id.deep_view(),
            size: self.size.deep_view(),
            format: self.format.deep_view(),
            channels: self.channels.deep_view(),
            sampleRate: self.sampleRate.deep_view(),
            byteRate: self.byteRate.deep_view(),
            blockAlign: self.blockAlign.deep_view(),
            bitsPerSample: self.bitsPerSample.deep_view(),
        }
    }
}
impl<'i> FmtChunk<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().id == self.id.deep_view(),
    self.deep_view().size == self.size.deep_view(),
    self.deep_view().format == self.format.deep_view(),
    self.deep_view().channels == self.channels.deep_view(),
    self.deep_view().sampleRate == self.sampleRate.deep_view(),
    self.deep_view().byteRate == self.byteRate.deep_view(),
    self.deep_view().blockAlign == self.blockAlign.deep_view(),
    self.deep_view().bitsPerSample == self.bitsPerSample.deep_view(),
    {
        reveal(< FmtChunk as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7 > FmtChunkSpec < T0, T1, T2, T3, T4, T5, T6, T7 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    T7)))))))) -> Self {
        let (id,
        (size,
        (format,
        (channels,
        (sampleRate,
        (byteRate,
        (blockAlign,
        bitsPerSample))))))) = input ;
        Self {
            id,
            size,
            format,
            channels,
            sampleRate,
            byteRate,
            blockAlign,
            bitsPerSample
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    T7))))))) {
        let Self {
            id,
            size,
            format,
            channels,
            sampleRate,
            byteRate,
            blockAlign,
            bitsPerSample
        }
        = self ;
        (id,
        (size,
        (format,
        (channels,
        (sampleRate,
        (byteRate,
        (blockAlign,
        bitsPerSample)))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(FmtChunkSpec::from_structural) ;
        reveal(FmtChunkSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    T7)))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(FmtChunkSpec::from_structural) ;
        reveal(FmtChunkSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            id,
            size,
            format,
            channels,
            sampleRate,
            byteRate,
            blockAlign,
            bitsPerSample
        }
        => (id,
        (size,
        (format,
        (channels,
        (sampleRate,
        (byteRate,
        (blockAlign,
        bitsPerSample))))))),
    }
   ,
    {
        reveal(FmtChunkSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct FmtChunkForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct FmtChunkReverse ;
impl SpecMap for FmtChunkForward {
    type Input = FmtChunkInner ;
    type Output = FmtChunkSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        FmtChunkSpec::from_structural (input)
    }
}
impl SpecMap for FmtChunkReverse {
    type Input = FmtChunkSpec ;
    type Output = FmtChunkInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `wav`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Wav<'i> {
    pub riff: RiffHeader<'i>,
    pub fmt: FmtChunk<'i>,
}
# [verifier::ext_equal]
pub struct WavSpec < T0 = RiffHeaderSpec, T1 = FmtChunkSpec > {
    pub riff: T0,
    pub fmt: T1,
}
pub type WavInner = (RiffHeaderSpec, FmtChunkSpec) ;
impl<'i> DeepView for Wav<'i> {
    type V = WavSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        WavSpec {
            riff: self.riff.deep_view(),
            fmt: self.fmt.deep_view(),
        }
    }
}
impl<'i> Wav<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().riff == self.riff.deep_view(),
    self.deep_view().fmt == self.fmt.deep_view(),
    {
        reveal(< Wav as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > WavSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    T1)) -> Self {
        let (riff,
        fmt) = input ;
        Self {
            riff,
            fmt
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    T1) {
        let Self {
            riff,
            fmt
        }
        = self ;
        (riff,
        fmt)
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(WavSpec::from_structural) ;
        reveal(WavSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    T1)) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(WavSpec::from_structural) ;
        reveal(WavSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            riff,
            fmt
        }
        => (riff,
        fmt),
    }
   ,
    {
        reveal(WavSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct WavForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct WavReverse ;
impl SpecMap for WavForward {
    type Input = WavInner ;
    type Output = WavSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        WavSpec::from_structural (input)
    }
}
impl SpecMap for WavReverse {
    type Input = WavSpec ;
    type Output = WavInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `riff_header`."]
# [derive (Clone, Copy)]
pub struct RiffHeaderFmt ;

pub type RiffHeaderFmtSpec = Named < Mapped < Pair < Fixed < 4 >, Pair < U32Le, Fixed < 4 > > >, BiMap < RiffHeaderForward, RiffHeaderReverse >> > ;

impl RiffHeaderFmt {
    # [doc = "specification constructor for `riff_header`."] pub open spec fn spec_inner() -> RiffHeaderFmtSpec {
        Named ("riff_header",
        Mapped {
            inner: Pair (Fixed::< 4 >,
            Pair (U32Le,
            Fixed::< 4 >)),
            mapper: BiMap (RiffHeaderForward,
            RiffHeaderReverse),
        }
        )
    }
}


# [doc = "named format combinator for `format`."]
# [derive (Clone, Copy)]
pub struct FormatFmt ;

pub type FormatFmtSpec = Named < Mapped < Choice < Refined < U16Le, PredFnSpec < u16 >>, Refined < U16Le, PredFnSpec < u16 >> >, BiMap < FormatForward, FormatReverse >> > ;

impl FormatFmt {
    # [doc = "specification constructor for `format`."] pub open spec fn spec_inner() -> FormatFmtSpec {
        Named ("format",
        Mapped {
            inner: Choice (Refined (U16Le,
            | x: u16 | ((((x == 1) || (x == 3)) || (x == 6)) || (x == 7)) || (x == 65534)),
            Refined (U16Le,
            | x: u16 | ((((x != 1) && (x != 3)) && (x != 6)) && (x != 7)) && (x != 65534))),
            mapper: BiMap (FormatForward,
            FormatReverse),
        }
        )
    }
}


# [doc = "named format combinator for `fmt_chunk`."]
# [derive (Clone, Copy)]
pub struct FmtChunkFmt ;

pub type FmtChunkFmtSpec = Named < Mapped < Pair < Fixed < 4 >, Pair < U32Le, Pair < FormatFmt, Pair < U16Le, Pair < U32Le, Pair < U32Le, Pair < U16Le, U16Le > > > > > > >, BiMap < FmtChunkForward, FmtChunkReverse >> > ;

impl FmtChunkFmt {
    # [doc = "specification constructor for `fmt_chunk`."] pub open spec fn spec_inner() -> FmtChunkFmtSpec {
        Named ("fmt_chunk",
        Mapped {
            inner: Pair (Fixed::< 4 >,
            Pair (U32Le,
            Pair (FormatFmt,
            Pair (U16Le,
            Pair (U32Le,
            Pair (U32Le,
            Pair (U16Le,
            U16Le))))))),
            mapper: BiMap (FmtChunkForward,
            FmtChunkReverse),
        }
        )
    }
}


# [doc = "named format combinator for `wav`."]
# [derive (Clone, Copy)]
pub struct WavFmt ;

pub type WavFmtSpec = Named < Mapped < Pair < RiffHeaderFmt, FmtChunkFmt >, BiMap < WavForward, WavReverse >> > ;

impl WavFmt {
    # [doc = "specification constructor for `wav`."] pub open spec fn spec_inner() -> WavFmtSpec {
        Named ("wav",
        Mapped {
            inner: Pair (RiffHeaderFmt,
            FmtChunkFmt),
            mapper: BiMap (WavForward,
            WavReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for RiffHeaderFmt {
        type PVal = RiffHeaderSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for RiffHeaderFmt {
        type Val = RiffHeaderSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for RiffHeaderFmt {
        type SValue = RiffHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for RiffHeaderFmt {
        type SVal = RiffHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for RiffHeaderFmt {
        type T = RiffHeaderSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for FormatFmt {
        type PVal = FormatSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for FormatFmt {
        type Val = FormatSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for FormatFmt {
        type SValue = FormatSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for FormatFmt {
        type SVal = FormatSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for FormatFmt {
        type T = FormatSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for FmtChunkFmt {
        type PVal = FmtChunkSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for FmtChunkFmt {
        type Val = FmtChunkSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for FmtChunkFmt {
        type SValue = FmtChunkSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for FmtChunkFmt {
        type SVal = FmtChunkSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for FmtChunkFmt {
        type T = FmtChunkSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for WavFmt {
        type PVal = WavSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for WavFmt {
        type Val = WavSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for WavFmt {
        type SValue = WavSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for WavFmt {
        type SVal = WavSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for WavFmt {
        type T = WavSpec ;
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
        RiffHeaderSpec::lemma_from_into,
        RiffHeaderSpec::lemma_into_from,
        Format::lemma_from_into,
        Format::lemma_into_from,
        FmtChunkSpec::lemma_from_into,
        FmtChunkSpec::lemma_into_from,
        WavSpec::lemma_from_into,
        WavSpec::lemma_into_from,
    };

    impl SafeParser for RiffHeaderFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for RiffHeaderFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for RiffHeaderFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< RiffHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: RiffHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                RiffHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< RiffHeaderFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: RiffHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                RiffHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for RiffHeaderFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< RiffHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for RiffHeaderFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< RiffHeaderFmt as SpecSerializer>::spec_serialize) ;
            reveal(< RiffHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for RiffHeaderFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< RiffHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< RiffHeaderFmt as Consistency>::consistent) ;
            reveal(< RiffHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: RiffHeaderSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                RiffHeaderSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for RiffHeaderFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: RiffHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                RiffHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for RiffHeaderFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< RiffHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< RiffHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for RiffHeaderFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< RiffHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< RiffHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for FormatFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< FormatFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for FormatFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< FormatFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for FormatFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< FormatFmt as SpecParser>::spec_parse) ;
            reveal(< FormatFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FormatInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Format::structural_valid (input)) ;
                Format::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< FormatFmt as SpecParser>::spec_parse) ;
            reveal(< FormatFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FormatInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Format::structural_valid (input)) ;
                Format::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for FormatFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< FormatFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< FormatFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FormatFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for FormatFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< FormatFmt as SpecSerializer>::spec_serialize) ;
            reveal(< FormatFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for FormatFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< FormatFmt as SpecParser>::spec_parse) ;
            reveal(< FormatFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FormatFmt as Consistency>::consistent) ;
            reveal(< FormatFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: FormatSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Format::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for FormatFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< FormatFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FormatInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Format::structural_valid (input)) ;
                Format::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for FormatFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< FormatFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FormatFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for FormatFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< FormatFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FormatFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for FmtChunkFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for FmtChunkFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for FmtChunkFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecParser>::spec_parse) ;
            reveal(< FmtChunkFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FmtChunkInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                FmtChunkSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecParser>::spec_parse) ;
            reveal(< FmtChunkFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FmtChunkInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                FmtChunkSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for FmtChunkFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FmtChunkFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for FmtChunkFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< FmtChunkFmt as SpecSerializer>::spec_serialize) ;
            reveal(< FmtChunkFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for FmtChunkFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecParser>::spec_parse) ;
            reveal(< FmtChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FmtChunkFmt as Consistency>::consistent) ;
            reveal(< FmtChunkFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: FmtChunkSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                FmtChunkSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for FmtChunkFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FmtChunkInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                FmtChunkSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for FmtChunkFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< FmtChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FmtChunkFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for FmtChunkFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< FmtChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FmtChunkFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for WavFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< WavFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for WavFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< WavFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for WavFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< WavFmt as SpecParser>::spec_parse) ;
            reveal(< WavFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: WavInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                WavSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< WavFmt as SpecParser>::spec_parse) ;
            reveal(< WavFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: WavInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                WavSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for WavFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< WavFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< WavFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< WavFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for WavFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< WavFmt as SpecSerializer>::spec_serialize) ;
            reveal(< WavFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for WavFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< WavFmt as SpecParser>::spec_parse) ;
            reveal(< WavFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< WavFmt as Consistency>::consistent) ;
            reveal(< WavFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: WavSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                WavSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for WavFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< WavFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: WavInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                WavSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for WavFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< WavFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< WavFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for WavFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< WavFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< WavFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for RiffHeaderFmt {
        type PT = RiffHeader<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<RiffHeaderFmt as SpecParser>::spec_parse);
            reveal(<RiffHeader as DeepView>::deep_view);
            reveal(RiffHeaderSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, riff) = (Fixed::< 4 >).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, size) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, wave) = (Fixed::< 4 >).parse (& rest) ?;
            let rest = rest.skip(n3);
            let total_n = n1 + n2 + n3;
            let final_v = RiffHeader {
                riff,
                size,
                wave,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, RiffHeader<'i>> for RiffHeaderFmt {
        fn serialize_into(&self, v: &RiffHeader<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<RiffHeaderFmt as SpecSerializer>::spec_serialize);
            reveal(<RiffHeaderFmt as SpecByteLen>::byte_len);
            reveal(<RiffHeader as DeepView>::deep_view);
            reveal(RiffHeaderSpec::into_structural);
            let ghost old_obuf = obuf@;

            let RiffHeader {
                riff,
                size,
                wave,
            } = v;
            Fixed::< 4 >.serialize_into(* riff, obuf);
            U32Le.serialize_into(size, obuf);
            Fixed::< 4 >.serialize_into(* wave, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<RiffHeader<'i>> for RiffHeaderFmt {
        fn prepare(&self, v: &RiffHeader<'i>) -> Result<usize, PreSerializeError> {
            reveal(<RiffHeaderFmt as SpecByteLen>::byte_len);
            reveal(<RiffHeader as DeepView>::deep_view);
            reveal(RiffHeaderSpec::into_structural);
            let RiffHeader {
                riff,
                size,
                wave,
            } = v;
            let l1 = (Fixed::< 4 >).prepare (riff) ?;
            let l2 = (U32Le).prepare (size) ?;
            let l3 = (Fixed::< 4 >).prepare (wave) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for FormatFmt {
        type PT = Format;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<FormatFmt as SpecParser>::spec_parse);
            reveal(<Format as DeepView>::deep_view);
            reveal(Format::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U16Le.parse(&rest)?;
            let enum_val = match v {
                1 => Format::PCM,
                3 => Format::IEEEFloat,
                6 => Format::ALaw,
                7 => Format::MuLaw,
                65534 => Format::Extensible,
                x => Format::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Format> for FormatFmt {
        fn serialize_into(&self, v: &Format, obuf: &mut Output) {
            reveal(<FormatFmt as SpecSerializer>::spec_serialize);
            reveal(<FormatFmt as SpecByteLen>::byte_len);
            reveal(<Format as DeepView>::deep_view);
            reveal(Format::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                Format::PCM => 1,
                Format::IEEEFloat => 3,
                Format::ALaw => 6,
                Format::MuLaw => 7,
                Format::Extensible => 65534,
                Format::Unknown (x) => x,
            };
            U16Le.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Format> for FormatFmt {
        fn prepare(&self, v: &Format) -> Result<usize, PreSerializeError> {
            reveal(<FormatFmt as SpecByteLen>::byte_len);
            reveal(<Format as DeepView>::deep_view);
            reveal(Format::into_structural);
            let tag = match *v {
                Format::PCM => 1,
                Format::IEEEFloat => 3,
                Format::ALaw => 6,
                Format::MuLaw => 7,
                Format::Extensible => 65534,
                Format::Unknown (x) if x != 1 && x != 3 && x != 6 && x != 7 && x != 65534 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U16Le.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for FmtChunkFmt {
        type PT = FmtChunk<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<FmtChunkFmt as SpecParser>::spec_parse);
            reveal(<FmtChunk as DeepView>::deep_view);
            reveal(FmtChunkSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, id) = (Fixed::< 4 >).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, size) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, format) = (Named ("format", FormatFmt)).parse (& rest) ?;
            proof {
                format.lemma_deep_view();
            }
            let rest = rest.skip(n3);
            let (n4, channels) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, sampleRate) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, byteRate) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, blockAlign) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, bitsPerSample) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n8);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8;
            let final_v = FmtChunk {
                id,
                size,
                format,
                channels,
                sampleRate,
                byteRate,
                blockAlign,
                bitsPerSample,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, FmtChunk<'i>> for FmtChunkFmt {
        fn serialize_into(&self, v: &FmtChunk<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<FmtChunkFmt as SpecSerializer>::spec_serialize);
            reveal(<FmtChunkFmt as SpecByteLen>::byte_len);
            reveal(<FmtChunk as DeepView>::deep_view);
            reveal(FmtChunkSpec::into_structural);
            let ghost old_obuf = obuf@;

            let FmtChunk {
                id,
                size,
                format,
                channels,
                sampleRate,
                byteRate,
                blockAlign,
                bitsPerSample,
            } = v;
            proof {
                format.lemma_deep_view();
            }

            Fixed::< 4 >.serialize_into(* id, obuf);
            U32Le.serialize_into(size, obuf);
            FormatFmt.serialize_into(format, obuf);
            U16Le.serialize_into(channels, obuf);
            U32Le.serialize_into(sampleRate, obuf);
            U32Le.serialize_into(byteRate, obuf);
            U16Le.serialize_into(blockAlign, obuf);
            U16Le.serialize_into(bitsPerSample, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<FmtChunk<'i>> for FmtChunkFmt {
        fn prepare(&self, v: &FmtChunk<'i>) -> Result<usize, PreSerializeError> {
            reveal(<FmtChunkFmt as SpecByteLen>::byte_len);
            reveal(<FmtChunk as DeepView>::deep_view);
            reveal(FmtChunkSpec::into_structural);
            let FmtChunk {
                id,
                size,
                format,
                channels,
                sampleRate,
                byteRate,
                blockAlign,
                bitsPerSample,
            } = v;
            proof {
                format.lemma_deep_view();
            }

            let l1 = (Fixed::< 4 >).prepare (id) ?;
            let l2 = (U32Le).prepare (size) ?;
            let l3 = (Named ("format", FormatFmt)).prepare (format) ?;
            let l4 = (U16Le).prepare (channels) ?;
            let l5 = (U32Le).prepare (sampleRate) ?;
            let l6 = (U32Le).prepare (byteRate) ?;
            let l7 = (U16Le).prepare (blockAlign) ?;
            let l8 = (U16Le).prepare (bitsPerSample) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for WavFmt {
        type PT = Wav<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<WavFmt as SpecParser>::spec_parse);
            reveal(<Wav as DeepView>::deep_view);
            reveal(WavSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, riff) = (Named ("riff_header", RiffHeaderFmt)).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, fmt) = (Named ("fmt_chunk", FmtChunkFmt)).parse (& rest) ?;
            let rest = rest.skip(n2);
            let total_n = n1 + n2;
            let final_v = Wav {
                riff,
                fmt,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Wav<'i>> for WavFmt {
        fn serialize_into(&self, v: &Wav<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<WavFmt as SpecSerializer>::spec_serialize);
            reveal(<WavFmt as SpecByteLen>::byte_len);
            reveal(<Wav as DeepView>::deep_view);
            reveal(WavSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Wav {
                riff,
                fmt,
            } = v;
            RiffHeaderFmt.serialize_into(riff, obuf);
            FmtChunkFmt.serialize_into(fmt, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Wav<'i>> for WavFmt {
        fn prepare(&self, v: &Wav<'i>) -> Result<usize, PreSerializeError> {
            reveal(<WavFmt as SpecByteLen>::byte_len);
            reveal(<Wav as DeepView>::deep_view);
            reveal(WavSpec::into_structural);
            let Wav {
                riff,
                fmt,
            } = v;
            let l1 = (Named ("riff_header", RiffHeaderFmt)).prepare (riff) ?;
            let l2 = (Named ("fmt_chunk", FmtChunkFmt)).prepare (fmt) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
