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
# [doc = "data type for `channels`."]
# [repr (u8)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum Channels {
    RGB = 3,
    RGBA = 4,
    Unknown (u8),
}
pub type ChannelsSpec = Channels ;
pub type ChannelsInner = Sum < u8, u8 > ;
impl DeepView for Channels {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Channels {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Channels as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: ChannelsInner) -> bool {
        match input {
            L (x) => x == 3 || x == 4,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: ChannelsInner) -> Self {
        match input {
            L (x) => match x {
                3 => Self::RGB,
                4 => Self::RGBA,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> ChannelsInner {
        match self {
            Self::RGB => L (3),
            Self::RGBA => L (4),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Channels::from_structural) ;
        reveal(Channels::into_structural) ;
        match self {
            Self::RGB => {
            }
           ,
            Self::RGBA => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: ChannelsInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Channels::from_structural) ;
        reveal(Channels::into_structural) ;
        match input {
            L (x) => match x {
                3 => {
                }
               ,
                4 => {
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
pub struct ChannelsForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ChannelsReverse ;
impl SpecMap for ChannelsForward {
    type Input = ChannelsInner ;
    type Output = ChannelsSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Channels::from_structural (input)
    }
}
impl SpecMap for ChannelsReverse {
    type Input = ChannelsSpec ;
    type Output = ChannelsInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for Channels {
}

# [doc = "data type for `colorspace`."]
# [repr (u8)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum Colorspace {
    SRGB = 0,
    Linear = 1,
    Unknown (u8),
}
pub type ColorspaceSpec = Colorspace ;
pub type ColorspaceInner = Sum < u8, u8 > ;
impl DeepView for Colorspace {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Colorspace {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Colorspace as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: ColorspaceInner) -> bool {
        match input {
            L (x) => x == 0 || x == 1,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: ColorspaceInner) -> Self {
        match input {
            L (x) => match x {
                0 => Self::SRGB,
                1 => Self::Linear,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> ColorspaceInner {
        match self {
            Self::SRGB => L (0),
            Self::Linear => L (1),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Colorspace::from_structural) ;
        reveal(Colorspace::into_structural) ;
        match self {
            Self::SRGB => {
            }
           ,
            Self::Linear => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: ColorspaceInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Colorspace::from_structural) ;
        reveal(Colorspace::into_structural) ;
        match input {
            L (x) => match x {
                0 => {
                }
               ,
                1 => {
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
pub struct ColorspaceForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ColorspaceReverse ;
impl SpecMap for ColorspaceForward {
    type Input = ColorspaceInner ;
    type Output = ColorspaceSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Colorspace::from_structural (input)
    }
}
impl SpecMap for ColorspaceReverse {
    type Input = ColorspaceSpec ;
    type Output = ColorspaceInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for Colorspace {
}

# [doc = "data type for `qoi`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Qoi {
    pub magic: [u8 ;
    4],
    pub width: u32,
    pub height: u32,
    pub channels: Channels,
    pub colorspace: Colorspace,
}
# [verifier::ext_equal]
pub struct QoiSpec < T0 = Seq < u8 >, T1 = u32, T2 = u32, T3 = ChannelsSpec, T4 = ColorspaceSpec > {
    pub magic: T0,
    pub width: T1,
    pub height: T2,
    pub channels: T3,
    pub colorspace: T4,
}
pub type QoiInner = (Seq < u8 >, (u32, (u32, (ChannelsSpec, ColorspaceSpec)))) ;
impl DeepView for Qoi {
    type V = QoiSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        QoiSpec {
            magic: self.magic.deep_view(),
            width: self.width.deep_view(),
            height: self.height.deep_view(),
            channels: self.channels.deep_view(),
            colorspace: self.colorspace.deep_view(),
        }
    }
}
impl Qoi {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().magic == self.magic.deep_view(),
    self.deep_view().width == self.width.deep_view(),
    self.deep_view().height == self.height.deep_view(),
    self.deep_view().channels == self.channels.deep_view(),
    self.deep_view().colorspace == self.colorspace.deep_view(),
    {
        reveal(< Qoi as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4 > QoiSpec < T0, T1, T2, T3, T4 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    T4))))) -> Self {
        let (magic,
        (width,
        (height,
        (channels,
        colorspace)))) = input ;
        Self {
            magic,
            width,
            height,
            channels,
            colorspace
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    (T3,
    T4)))) {
        let Self {
            magic,
            width,
            height,
            channels,
            colorspace
        }
        = self ;
        (magic,
        (width,
        (height,
        (channels,
        colorspace))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(QoiSpec::from_structural) ;
        reveal(QoiSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    T4))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(QoiSpec::from_structural) ;
        reveal(QoiSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            magic,
            width,
            height,
            channels,
            colorspace
        }
        => (magic,
        (width,
        (height,
        (channels,
        colorspace)))),
    }
   ,
    {
        reveal(QoiSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct QoiForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct QoiReverse ;
impl SpecMap for QoiForward {
    type Input = QoiInner ;
    type Output = QoiSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        QoiSpec::from_structural (input)
    }
}
impl SpecMap for QoiReverse {
    type Input = QoiSpec ;
    type Output = QoiInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `channels`."]
# [derive (Clone, Copy)]
pub struct ChannelsFmt ;

pub type ChannelsFmtSpec = Named < Mapped < Choice < Refined < U8, PredFnSpec < u8 >>, Refined < U8, PredFnSpec < u8 >> >, BiMap < ChannelsForward, ChannelsReverse >> > ;

impl ChannelsFmt {
    # [doc = "specification constructor for `channels`."] pub open spec fn spec_inner() -> ChannelsFmtSpec {
        Named ("channels",
        Mapped {
            inner: Choice (Refined (U8,
            | x: u8 | (x == 3) || (x == 4)),
            Refined (U8,
            | x: u8 | (x != 3) && (x != 4))),
            mapper: BiMap (ChannelsForward,
            ChannelsReverse),
        }
        )
    }
}


# [doc = "named format combinator for `colorspace`."]
# [derive (Clone, Copy)]
pub struct ColorspaceFmt ;

pub type ColorspaceFmtSpec = Named < Mapped < Choice < Refined < U8, PredFnSpec < u8 >>, Refined < U8, PredFnSpec < u8 >> >, BiMap < ColorspaceForward, ColorspaceReverse >> > ;

impl ColorspaceFmt {
    # [doc = "specification constructor for `colorspace`."] pub open spec fn spec_inner() -> ColorspaceFmtSpec {
        Named ("colorspace",
        Mapped {
            inner: Choice (Refined (U8,
            | x: u8 | (x == 0) || (x == 1)),
            Refined (U8,
            | x: u8 | (x != 0) && (x != 1))),
            mapper: BiMap (ColorspaceForward,
            ColorspaceReverse),
        }
        )
    }
}


# [doc = "named format combinator for `qoi`."]
# [derive (Clone, Copy)]
pub struct QoiFmt ;

pub type QoiFmtSpec = Named < Mapped < Pair < Const < Fixed < 4 >, [u8 ;
4] >, Pair < U32Be, Pair < U32Be, Pair < ChannelsFmt, ColorspaceFmt > > > >, BiMap < QoiForward, QoiReverse >> > ;

impl QoiFmt {
    # [doc = "specification constructor for `qoi`."] pub open spec fn spec_inner() -> QoiFmtSpec {
        Named ("qoi",
        Mapped {
            inner: Pair (Const (Fixed::< 4 >,
            [0x71u8, 0x6fu8, 0x69u8, 0x66u8]),
            Pair (U32Be,
            Pair (U32Be,
            Pair (ChannelsFmt,
            ColorspaceFmt)))),
            mapper: BiMap (QoiForward,
            QoiReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for ChannelsFmt {
        type PVal = ChannelsSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ChannelsFmt {
        type Val = ChannelsSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ChannelsFmt {
        type SValue = ChannelsSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ChannelsFmt {
        type SVal = ChannelsSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ChannelsFmt {
        type T = ChannelsSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for ColorspaceFmt {
        type PVal = ColorspaceSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ColorspaceFmt {
        type Val = ColorspaceSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ColorspaceFmt {
        type SValue = ColorspaceSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ColorspaceFmt {
        type SVal = ColorspaceSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ColorspaceFmt {
        type T = ColorspaceSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for QoiFmt {
        type PVal = QoiSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for QoiFmt {
        type Val = QoiSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for QoiFmt {
        type SValue = QoiSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for QoiFmt {
        type SVal = QoiSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for QoiFmt {
        type T = QoiSpec ;
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
        Channels::lemma_from_into,
        Channels::lemma_into_from,
        Colorspace::lemma_from_into,
        Colorspace::lemma_into_from,
        QoiSpec::lemma_from_into,
        QoiSpec::lemma_into_from,
    };

    impl SafeParser for ChannelsFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ChannelsFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ChannelsFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecParser>::spec_parse) ;
            reveal(< ChannelsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ChannelsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Channels::structural_valid (input)) ;
                Channels::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecParser>::spec_parse) ;
            reveal(< ChannelsFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ChannelsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Channels::structural_valid (input)) ;
                Channels::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ChannelsFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ChannelsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ChannelsFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ChannelsFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ChannelsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ChannelsFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecParser>::spec_parse) ;
            reveal(< ChannelsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ChannelsFmt as Consistency>::consistent) ;
            reveal(< ChannelsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ChannelsSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Channels::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ChannelsFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ChannelsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Channels::structural_valid (input)) ;
                Channels::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ChannelsFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ChannelsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ChannelsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ChannelsFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ChannelsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ChannelsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for ColorspaceFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ColorspaceFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ColorspaceFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecParser>::spec_parse) ;
            reveal(< ColorspaceFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ColorspaceInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Colorspace::structural_valid (input)) ;
                Colorspace::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecParser>::spec_parse) ;
            reveal(< ColorspaceFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ColorspaceInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Colorspace::structural_valid (input)) ;
                Colorspace::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ColorspaceFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ColorspaceFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ColorspaceFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ColorspaceFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ColorspaceFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ColorspaceFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecParser>::spec_parse) ;
            reveal(< ColorspaceFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ColorspaceFmt as Consistency>::consistent) ;
            reveal(< ColorspaceFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ColorspaceSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Colorspace::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ColorspaceFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ColorspaceInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Colorspace::structural_valid (input)) ;
                Colorspace::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ColorspaceFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ColorspaceFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ColorspaceFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ColorspaceFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ColorspaceFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ColorspaceFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for QoiFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< QoiFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for QoiFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< QoiFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for QoiFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< QoiFmt as SpecParser>::spec_parse) ;
            reveal(< QoiFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: QoiInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                QoiSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< QoiFmt as SpecParser>::spec_parse) ;
            reveal(< QoiFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: QoiInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                QoiSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for QoiFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< QoiFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< QoiFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< QoiFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for QoiFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< QoiFmt as SpecSerializer>::spec_serialize) ;
            reveal(< QoiFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for QoiFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< QoiFmt as SpecParser>::spec_parse) ;
            reveal(< QoiFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< QoiFmt as Consistency>::consistent) ;
            reveal(< QoiFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: QoiSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                QoiSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for QoiFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< QoiFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: QoiInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                QoiSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for QoiFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< QoiFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< QoiFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for QoiFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< QoiFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< QoiFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for ChannelsFmt {
        type PT = Channels;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ChannelsFmt as SpecParser>::spec_parse);
            reveal(<Channels as DeepView>::deep_view);
            reveal(Channels::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U8.parse(&rest)?;
            let enum_val = match v {
                3 => Channels::RGB,
                4 => Channels::RGBA,
                x => Channels::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Channels> for ChannelsFmt {
        fn serialize_into(&self, v: &Channels, obuf: &mut Output) {
            reveal(<ChannelsFmt as SpecSerializer>::spec_serialize);
            reveal(<ChannelsFmt as SpecByteLen>::byte_len);
            reveal(<Channels as DeepView>::deep_view);
            reveal(Channels::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                Channels::RGB => 3,
                Channels::RGBA => 4,
                Channels::Unknown (x) => x,
            };
            U8.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Channels> for ChannelsFmt {
        fn prepare(&self, v: &Channels) -> Result<usize, PreSerializeError> {
            reveal(<ChannelsFmt as SpecByteLen>::byte_len);
            reveal(<Channels as DeepView>::deep_view);
            reveal(Channels::into_structural);
            let tag = match *v {
                Channels::RGB => 3,
                Channels::RGBA => 4,
                Channels::Unknown (x) if x != 3 && x != 4 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U8.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for ColorspaceFmt {
        type PT = Colorspace;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ColorspaceFmt as SpecParser>::spec_parse);
            reveal(<Colorspace as DeepView>::deep_view);
            reveal(Colorspace::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U8.parse(&rest)?;
            let enum_val = match v {
                0 => Colorspace::SRGB,
                1 => Colorspace::Linear,
                x => Colorspace::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Colorspace> for ColorspaceFmt {
        fn serialize_into(&self, v: &Colorspace, obuf: &mut Output) {
            reveal(<ColorspaceFmt as SpecSerializer>::spec_serialize);
            reveal(<ColorspaceFmt as SpecByteLen>::byte_len);
            reveal(<Colorspace as DeepView>::deep_view);
            reveal(Colorspace::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                Colorspace::SRGB => 0,
                Colorspace::Linear => 1,
                Colorspace::Unknown (x) => x,
            };
            U8.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Colorspace> for ColorspaceFmt {
        fn prepare(&self, v: &Colorspace) -> Result<usize, PreSerializeError> {
            reveal(<ColorspaceFmt as SpecByteLen>::byte_len);
            reveal(<Colorspace as DeepView>::deep_view);
            reveal(Colorspace::into_structural);
            let tag = match *v {
                Colorspace::SRGB => 0,
                Colorspace::Linear => 1,
                Colorspace::Unknown (x) if x != 0 && x != 1 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U8.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for QoiFmt {
        type PT = Qoi;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<QoiFmt as SpecParser>::spec_parse);
            reveal(<Qoi as DeepView>::deep_view);
            reveal(QoiSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, magic) = Const (Fixed::< 4 >, [0x71, 0x6f, 0x69, 0x66]).parse(&rest)?;
            let rest = rest.skip(n1);
            let (n2, width) = (U32Be).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, height) = (U32Be).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, channels) = (Named ("channels", ChannelsFmt)).parse (& rest) ?;
            proof {
                channels.lemma_deep_view();
            }
            let rest = rest.skip(n4);
            let (n5, colorspace) = (Named ("colorspace", ColorspaceFmt)).parse (& rest) ?;
            proof {
                colorspace.lemma_deep_view();
            }
            let rest = rest.skip(n5);
            let total_n = n1 + n2 + n3 + n4 + n5;
            let final_v = Qoi {
                magic,
                width,
                height,
                channels,
                colorspace,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Qoi> for QoiFmt {
        fn serialize_into(&self, v: &Qoi, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<QoiFmt as SpecSerializer>::spec_serialize);
            reveal(<QoiFmt as SpecByteLen>::byte_len);
            reveal(<Qoi as DeepView>::deep_view);
            reveal(QoiSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Qoi {
                magic,
                width,
                height,
                channels,
                colorspace,
            } = v;
            proof {
                channels.lemma_deep_view();
                colorspace.lemma_deep_view();
            }

            Fixed::< 4 >.serialize_into(magic, obuf);
            U32Be.serialize_into(width, obuf);
            U32Be.serialize_into(height, obuf);
            ChannelsFmt.serialize_into(channels, obuf);
            ColorspaceFmt.serialize_into(colorspace, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Qoi> for QoiFmt {
        fn prepare(&self, v: &Qoi) -> Result<usize, PreSerializeError> {
            reveal(<QoiFmt as SpecByteLen>::byte_len);
            reveal(<Qoi as DeepView>::deep_view);
            reveal(QoiSpec::into_structural);
            let Qoi {
                magic,
                width,
                height,
                channels,
                colorspace,
            } = v;
            proof {
                channels.lemma_deep_view();
                colorspace.lemma_deep_view();
            }

            let l1 = (Const (Fixed::< 4 >, [0x71, 0x6f, 0x69, 0x66])).prepare (magic) ?;
            let l2 = (U32Be).prepare (width) ?;
            let l3 = (U32Be).prepare (height) ?;
            let l4 = (Named ("channels", ChannelsFmt)).prepare (channels) ?;
            let l5 = (Named ("colorspace", ColorspaceFmt)).prepare (colorspace) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
