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
# [doc = "data type for `header`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Header<'i> {
    pub signature: &'i [u8],
    pub version: &'i [u8],
}
# [verifier::ext_equal]
pub struct HeaderSpec < T0 = Seq < u8 >, T1 = Seq < u8 > > {
    pub signature: T0,
    pub version: T1,
}
pub type HeaderInner = (Seq < u8 >, Seq < u8 >) ;
impl<'i> DeepView for Header<'i> {
    type V = HeaderSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        HeaderSpec {
            signature: self.signature.deep_view(),
            version: self.version.deep_view(),
        }
    }
}
impl<'i> Header<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().signature == self.signature.deep_view(),
    self.deep_view().version == self.version.deep_view(),
    {
        reveal(< Header as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > HeaderSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    T1)) -> Self {
        let (signature,
        version) = input ;
        Self {
            signature,
            version
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    T1) {
        let Self {
            signature,
            version
        }
        = self ;
        (signature,
        version)
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(HeaderSpec::from_structural) ;
        reveal(HeaderSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    T1)) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(HeaderSpec::from_structural) ;
        reveal(HeaderSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            signature,
            version
        }
        => (signature,
        version),
    }
   ,
    {
        reveal(HeaderSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct HeaderForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct HeaderReverse ;
impl SpecMap for HeaderForward {
    type Input = HeaderInner ;
    type Output = HeaderSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        HeaderSpec::from_structural (input)
    }
}
impl SpecMap for HeaderReverse {
    type Input = HeaderSpec ;
    type Output = HeaderInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `screen_flags`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
# [verifier::ext_equal]
pub struct ScreenFlags {
    pub globalColorTable: u8,
    pub colorResolution: u8,
    pub sortFlag: u8,
    pub globalColorTableSize: u8,
}
pub type ScreenFlagsSpec = ScreenFlags ;
pub type ScreenFlagsInner = u8 ;
impl DeepView for ScreenFlags {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl ScreenFlags {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< ScreenFlags as DeepView>::deep_view) ;
    }
}

# [doc = "data type for `logical_screen_descriptor`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct LogicalScreenDescriptor {
    pub width: u16,
    pub height: u16,
    pub flags: ScreenFlags,
    pub backgroundColorIndex: u8,
    pub pixelAspectRatio: u8,
}
# [verifier::ext_equal]
pub struct LogicalScreenDescriptorSpec < T0 = u16, T1 = u16, T2 = ScreenFlagsSpec, T3 = u8, T4 = u8 > {
    pub width: T0,
    pub height: T1,
    pub flags: T2,
    pub backgroundColorIndex: T3,
    pub pixelAspectRatio: T4,
}
pub type LogicalScreenDescriptorInner = (u16, (u16, (ScreenFlagsSpec, (u8, u8)))) ;
impl DeepView for LogicalScreenDescriptor {
    type V = LogicalScreenDescriptorSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        LogicalScreenDescriptorSpec {
            width: self.width.deep_view(),
            height: self.height.deep_view(),
            flags: self.flags.deep_view(),
            backgroundColorIndex: self.backgroundColorIndex.deep_view(),
            pixelAspectRatio: self.pixelAspectRatio.deep_view(),
        }
    }
}
impl LogicalScreenDescriptor {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().width == self.width.deep_view(),
    self.deep_view().height == self.height.deep_view(),
    self.deep_view().flags == self.flags.deep_view(),
    self.deep_view().backgroundColorIndex == self.backgroundColorIndex.deep_view(),
    self.deep_view().pixelAspectRatio == self.pixelAspectRatio.deep_view(),
    {
        reveal(< LogicalScreenDescriptor as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4 > LogicalScreenDescriptorSpec < T0, T1, T2, T3, T4 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    T4))))) -> Self {
        let (width,
        (height,
        (flags,
        (backgroundColorIndex,
        pixelAspectRatio)))) = input ;
        Self {
            width,
            height,
            flags,
            backgroundColorIndex,
            pixelAspectRatio
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    (T3,
    T4)))) {
        let Self {
            width,
            height,
            flags,
            backgroundColorIndex,
            pixelAspectRatio
        }
        = self ;
        (width,
        (height,
        (flags,
        (backgroundColorIndex,
        pixelAspectRatio))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(LogicalScreenDescriptorSpec::from_structural) ;
        reveal(LogicalScreenDescriptorSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    T4))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(LogicalScreenDescriptorSpec::from_structural) ;
        reveal(LogicalScreenDescriptorSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            width,
            height,
            flags,
            backgroundColorIndex,
            pixelAspectRatio
        }
        => (width,
        (height,
        (flags,
        (backgroundColorIndex,
        pixelAspectRatio)))),
    }
   ,
    {
        reveal(LogicalScreenDescriptorSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct LogicalScreenDescriptorForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct LogicalScreenDescriptorReverse ;
impl SpecMap for LogicalScreenDescriptorForward {
    type Input = LogicalScreenDescriptorInner ;
    type Output = LogicalScreenDescriptorSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        LogicalScreenDescriptorSpec::from_structural (input)
    }
}
impl SpecMap for LogicalScreenDescriptorReverse {
    type Input = LogicalScreenDescriptorSpec ;
    type Output = LogicalScreenDescriptorInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `gif`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Gif<'i> {
    pub header: Header<'i>,
    pub screen: LogicalScreenDescriptor,
}
# [verifier::ext_equal]
pub struct GifSpec < T0 = HeaderSpec, T1 = LogicalScreenDescriptorSpec > {
    pub header: T0,
    pub screen: T1,
}
pub type GifInner = (HeaderSpec, LogicalScreenDescriptorSpec) ;
impl<'i> DeepView for Gif<'i> {
    type V = GifSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        GifSpec {
            header: self.header.deep_view(),
            screen: self.screen.deep_view(),
        }
    }
}
impl<'i> Gif<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().header == self.header.deep_view(),
    self.deep_view().screen == self.screen.deep_view(),
    {
        reveal(< Gif as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > GifSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    T1)) -> Self {
        let (header,
        screen) = input ;
        Self {
            header,
            screen
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    T1) {
        let Self {
            header,
            screen
        }
        = self ;
        (header,
        screen)
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(GifSpec::from_structural) ;
        reveal(GifSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    T1)) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(GifSpec::from_structural) ;
        reveal(GifSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            header,
            screen
        }
        => (header,
        screen),
    }
   ,
    {
        reveal(GifSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct GifForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct GifReverse ;
impl SpecMap for GifForward {
    type Input = GifInner ;
    type Output = GifSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        GifSpec::from_structural (input)
    }
}
impl SpecMap for GifReverse {
    type Input = GifSpec ;
    type Output = GifInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `header`."]
# [derive (Clone, Copy)]
pub struct HeaderFmt ;

pub type HeaderFmtSpec = Named < Mapped < Pair < Fixed < 3 >, Fixed < 3 > >, BiMap < HeaderForward, HeaderReverse >> > ;

impl HeaderFmt {
    # [doc = "specification constructor for `header`."] pub open spec fn spec_inner() -> HeaderFmtSpec {
        Named ("header",
        Mapped {
            inner: Pair (Fixed::< 3 >,
            Fixed::< 3 >),
            mapper: BiMap (HeaderForward,
            HeaderReverse),
        }
        )
    }
}


# [doc = "named format combinator for `screen_flags`."]
# [derive (Clone, Copy)]
pub struct ScreenFlagsFmt ;

pub const SCREEN_FLAGS_GLOBALCOLORTABLE_MASK: u8 = 0b00000001u8 ;
pub const SCREEN_FLAGS_GLOBALCOLORTABLE_SHIFT: u8 = 7 ;
pub const SCREEN_FLAGS_GLOBALCOLORTABLE_MAX: u8 = 0b00000010u8 ;
pub const SCREEN_FLAGS_COLORRESOLUTION_MASK: u8 = 0b00000111u8 ;
pub const SCREEN_FLAGS_COLORRESOLUTION_SHIFT: u8 = 4 ;
pub const SCREEN_FLAGS_COLORRESOLUTION_MAX: u8 = 0b00001000u8 ;
pub const SCREEN_FLAGS_SORTFLAG_MASK: u8 = 0b00000001u8 ;
pub const SCREEN_FLAGS_SORTFLAG_SHIFT: u8 = 3 ;
pub const SCREEN_FLAGS_SORTFLAG_MAX: u8 = 0b00000010u8 ;
pub const SCREEN_FLAGS_GLOBALCOLORTABLESIZE_MASK: u8 = 0b00000111u8 ;
pub const SCREEN_FLAGS_GLOBALCOLORTABLESIZE_SHIFT: u8 = 0 ;
pub const SCREEN_FLAGS_GLOBALCOLORTABLESIZE_MAX: u8 = 0b00001000u8 ;
# [verifier::allow_in_spec]
pub fn unpack_screen_flags (raw: u8) -> (u8, u8, u8, u8) returns ((((raw >> SCREEN_FLAGS_GLOBALCOLORTABLE_SHIFT) & SCREEN_FLAGS_GLOBALCOLORTABLE_MASK) as u8), (((raw >> SCREEN_FLAGS_COLORRESOLUTION_SHIFT) & SCREEN_FLAGS_COLORRESOLUTION_MASK) as u8), (((raw >> SCREEN_FLAGS_SORTFLAG_SHIFT) & SCREEN_FLAGS_SORTFLAG_MASK) as u8), ((raw & SCREEN_FLAGS_GLOBALCOLORTABLESIZE_MASK) as u8)), {
    ((((raw >> SCREEN_FLAGS_GLOBALCOLORTABLE_SHIFT) & SCREEN_FLAGS_GLOBALCOLORTABLE_MASK) as u8),
    (((raw >> SCREEN_FLAGS_COLORRESOLUTION_SHIFT) & SCREEN_FLAGS_COLORRESOLUTION_MASK) as u8),
    (((raw >> SCREEN_FLAGS_SORTFLAG_SHIFT) & SCREEN_FLAGS_SORTFLAG_MASK) as u8),
    ((raw & SCREEN_FLAGS_GLOBALCOLORTABLESIZE_MASK) as u8))
}
# [verifier::allow_in_spec]
pub fn pack_screen_flags (globalColorTable: u8, colorResolution: u8, sortFlag: u8, globalColorTableSize: u8) -> u8 returns (((globalColorTable as u8) & SCREEN_FLAGS_GLOBALCOLORTABLE_MASK) << SCREEN_FLAGS_GLOBALCOLORTABLE_SHIFT) | (((colorResolution as u8) & SCREEN_FLAGS_COLORRESOLUTION_MASK) << SCREEN_FLAGS_COLORRESOLUTION_SHIFT) | (((sortFlag as u8) & SCREEN_FLAGS_SORTFLAG_MASK) << SCREEN_FLAGS_SORTFLAG_SHIFT) | (((globalColorTableSize as u8) & SCREEN_FLAGS_GLOBALCOLORTABLESIZE_MASK)), {
    (((globalColorTable as u8) & SCREEN_FLAGS_GLOBALCOLORTABLE_MASK) << SCREEN_FLAGS_GLOBALCOLORTABLE_SHIFT) | (((colorResolution as u8) & SCREEN_FLAGS_COLORRESOLUTION_MASK) << SCREEN_FLAGS_COLORRESOLUTION_SHIFT) | (((sortFlag as u8) & SCREEN_FLAGS_SORTFLAG_MASK) << SCREEN_FLAGS_SORTFLAG_SHIFT) | (((globalColorTableSize as u8) & SCREEN_FLAGS_GLOBALCOLORTABLESIZE_MASK))
}
# [verifier::allow_in_spec]
pub fn screen_flags_bounds (globalColorTable: u8, colorResolution: u8, sortFlag: u8, globalColorTableSize: u8) -> bool returns (globalColorTable < SCREEN_FLAGS_GLOBALCOLORTABLE_MAX) && (colorResolution < SCREEN_FLAGS_COLORRESOLUTION_MAX) && (sortFlag < SCREEN_FLAGS_SORTFLAG_MAX) && (globalColorTableSize < SCREEN_FLAGS_GLOBALCOLORTABLESIZE_MAX), {
    (globalColorTable < SCREEN_FLAGS_GLOBALCOLORTABLE_MAX) && (colorResolution < SCREEN_FLAGS_COLORRESOLUTION_MAX) && (sortFlag < SCREEN_FLAGS_SORTFLAG_MAX) && (globalColorTableSize < SCREEN_FLAGS_GLOBALCOLORTABLESIZE_MAX)
}
pub broadcast proof fn lemma_screen_flags_unpack_pack (raw: u8) by (bit_vector) ensures # [trigger]
pack_screen_flags (unpack_screen_flags (raw).0, unpack_screen_flags (raw).1, unpack_screen_flags (raw).2, unpack_screen_flags (raw).3) == raw, {
}
pub broadcast proof fn lemma_screen_flags_pack_unpack (globalColorTable: u8, colorResolution: u8, sortFlag: u8, globalColorTableSize: u8) by (bit_vector) requires # [trigger] screen_flags_bounds (globalColorTable, colorResolution, sortFlag, globalColorTableSize), ensures unpack_screen_flags (pack_screen_flags (globalColorTable, colorResolution, sortFlag, globalColorTableSize)).0 == globalColorTable, unpack_screen_flags (pack_screen_flags (globalColorTable, colorResolution, sortFlag, globalColorTableSize)).1 == colorResolution, unpack_screen_flags (pack_screen_flags (globalColorTable, colorResolution, sortFlag, globalColorTableSize)).2 == sortFlag, unpack_screen_flags (pack_screen_flags (globalColorTable, colorResolution, sortFlag, globalColorTableSize)).3 == globalColorTableSize, {
}
pub broadcast proof fn lemma_screen_flags_mapper_wf_in_out (i: u8) by (bit_vector) ensures # [trigger] screen_flags_bounds (unpack_screen_flags (i).0, unpack_screen_flags (i).1, unpack_screen_flags (i).2, unpack_screen_flags (i).3), {
}

pub type ScreenFlagsFmtSpec = Named < Bits < U8, (u8, u8, u8, u8), ScreenFlagsSpec > > ;

impl ScreenFlagsFmt {
    # [doc = "specification constructor for `screen_flags`."] pub open spec fn spec_inner() -> ScreenFlagsFmtSpec {
        Named ("screen_flags",
        Bits {
            repr: U8,
            unpack: | packed: u8 | unpack_screen_flags (packed),
            pack: | unpacked: (u8,
            u8,
            u8,
            u8) | {
                let (globalColorTable,
                colorResolution,
                sortFlag,
                globalColorTableSize) = unpacked ;
                pack_screen_flags (globalColorTable,
                colorResolution,
                sortFlag,
                globalColorTableSize)
            }
           ,
            refinement: | unpacked: (u8,
            u8,
            u8,
            u8) | {
                let (globalColorTable,
                colorResolution,
                sortFlag,
                globalColorTableSize) = unpacked ;
                true
            }
           ,
            ctor: | unpacked: (u8,
            u8,
            u8,
            u8) | {
                let (globalColorTable,
                colorResolution,
                sortFlag,
                globalColorTableSize) = unpacked ;
                ScreenFlagsSpec {
                    globalColorTable: globalColorTable,
                    colorResolution: colorResolution,
                    sortFlag: sortFlag,
                    globalColorTableSize: globalColorTableSize
                }
            }
           ,
            dtor: | value: ScreenFlagsSpec | {
                let ScreenFlagsSpec {
                    globalColorTable,
                    colorResolution,
                    sortFlag,
                    globalColorTableSize
                }
                = value ;
                (globalColorTable,
                colorResolution,
                sortFlag,
                globalColorTableSize)
            }
           ,
            consistent: | value: ScreenFlagsSpec | {
                let ScreenFlagsSpec {
                    globalColorTable,
                    colorResolution,
                    sortFlag,
                    globalColorTableSize
                }
                = value ;
                screen_flags_bounds (globalColorTable,
                colorResolution,
                sortFlag,
                globalColorTableSize)
            }
           ,
        }
        )
    }
}


# [doc = "named format combinator for `logical_screen_descriptor`."]
# [derive (Clone, Copy)]
pub struct LogicalScreenDescriptorFmt ;

pub type LogicalScreenDescriptorFmtSpec = Named < Mapped < Pair < U16Le, Pair < U16Le, Pair < ScreenFlagsFmt, Pair < U8, U8 > > > >, BiMap < LogicalScreenDescriptorForward, LogicalScreenDescriptorReverse >> > ;

impl LogicalScreenDescriptorFmt {
    # [doc = "specification constructor for `logical_screen_descriptor`."] pub open spec fn spec_inner() -> LogicalScreenDescriptorFmtSpec {
        Named ("logical_screen_descriptor",
        Mapped {
            inner: Pair (U16Le,
            Pair (U16Le,
            Pair (ScreenFlagsFmt,
            Pair (U8,
            U8)))),
            mapper: BiMap (LogicalScreenDescriptorForward,
            LogicalScreenDescriptorReverse),
        }
        )
    }
}


# [doc = "named format combinator for `gif`."]
# [derive (Clone, Copy)]
pub struct GifFmt ;

pub type GifFmtSpec = Named < Mapped < Pair < HeaderFmt, LogicalScreenDescriptorFmt >, BiMap < GifForward, GifReverse >> > ;

impl GifFmt {
    # [doc = "specification constructor for `gif`."] pub open spec fn spec_inner() -> GifFmtSpec {
        Named ("gif",
        Mapped {
            inner: Pair (HeaderFmt,
            LogicalScreenDescriptorFmt),
            mapper: BiMap (GifForward,
            GifReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for HeaderFmt {
        type PVal = HeaderSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for HeaderFmt {
        type Val = HeaderSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for HeaderFmt {
        type SValue = HeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for HeaderFmt {
        type SVal = HeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for HeaderFmt {
        type T = HeaderSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for ScreenFlagsFmt {
        type PVal = ScreenFlagsSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ScreenFlagsFmt {
        type Val = ScreenFlagsSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ScreenFlagsFmt {
        type SValue = ScreenFlagsSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ScreenFlagsFmt {
        type SVal = ScreenFlagsSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ScreenFlagsFmt {
        type T = ScreenFlagsSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for LogicalScreenDescriptorFmt {
        type PVal = LogicalScreenDescriptorSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for LogicalScreenDescriptorFmt {
        type Val = LogicalScreenDescriptorSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for LogicalScreenDescriptorFmt {
        type SValue = LogicalScreenDescriptorSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for LogicalScreenDescriptorFmt {
        type SVal = LogicalScreenDescriptorSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for LogicalScreenDescriptorFmt {
        type T = LogicalScreenDescriptorSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for GifFmt {
        type PVal = GifSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for GifFmt {
        type Val = GifSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for GifFmt {
        type SValue = GifSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for GifFmt {
        type SVal = GifSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for GifFmt {
        type T = GifSpec ;
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
        HeaderSpec::lemma_from_into,
        HeaderSpec::lemma_into_from,
        LogicalScreenDescriptorSpec::lemma_from_into,
        LogicalScreenDescriptorSpec::lemma_into_from,
        GifSpec::lemma_from_into,
        GifSpec::lemma_into_from,
    };

    impl SafeParser for HeaderFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< HeaderFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for HeaderFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< HeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for HeaderFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< HeaderFmt as SpecParser>::spec_parse) ;
            reveal(< HeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: HeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                HeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< HeaderFmt as SpecParser>::spec_parse) ;
            reveal(< HeaderFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: HeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                HeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for HeaderFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< HeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< HeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< HeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for HeaderFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< HeaderFmt as SpecSerializer>::spec_serialize) ;
            reveal(< HeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for HeaderFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< HeaderFmt as SpecParser>::spec_parse) ;
            reveal(< HeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< HeaderFmt as Consistency>::consistent) ;
            reveal(< HeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: HeaderSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                HeaderSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for HeaderFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< HeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: HeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                HeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for HeaderFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< HeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< HeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for HeaderFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< HeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< HeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for ScreenFlagsFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ScreenFlagsFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ScreenFlagsFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecParser>::spec_parse) ;
            reveal(< ScreenFlagsFmt as SpecByteLen>::byte_len) ;
            let fmt = ScreenFlagsFmt::spec_inner() ;
            broadcast use lemma_screen_flags_unpack_pack,
            lemma_screen_flags_mapper_wf_in_out ;
            assert (fmt.1.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecParser>::spec_parse) ;
            reveal(< ScreenFlagsFmt as Consistency>::consistent) ;
            broadcast use lemma_screen_flags_unpack_pack,
            lemma_screen_flags_mapper_wf_in_out ;
            let fmt = ScreenFlagsFmt::spec_inner() ;
            assert (fmt.1.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ScreenFlagsFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ScreenFlagsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ScreenFlagsFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ScreenFlagsFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ScreenFlagsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ScreenFlagsFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ScreenFlagsFmt as SpecByteLen>::byte_len) ;
            reveal(< ScreenFlagsFmt as SpecParser>::spec_parse) ;
            broadcast use lemma_screen_flags_pack_unpack ;
            let fmt = ScreenFlagsFmt::spec_inner() ;
            assert (fmt.1.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ScreenFlagsFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecParser>::spec_parse) ;
            broadcast use lemma_screen_flags_unpack_pack,
            lemma_screen_flags_mapper_wf_in_out ;
            let fmt = ScreenFlagsFmt::spec_inner() ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ScreenFlagsFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ScreenFlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ScreenFlagsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ScreenFlagsFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ScreenFlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ScreenFlagsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for LogicalScreenDescriptorFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for LogicalScreenDescriptorFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for LogicalScreenDescriptorFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecParser>::spec_parse) ;
            reveal(< LogicalScreenDescriptorFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LogicalScreenDescriptorInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LogicalScreenDescriptorSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecParser>::spec_parse) ;
            reveal(< LogicalScreenDescriptorFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LogicalScreenDescriptorInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LogicalScreenDescriptorSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for LogicalScreenDescriptorFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LogicalScreenDescriptorFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for LogicalScreenDescriptorFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< LogicalScreenDescriptorFmt as SpecSerializer>::spec_serialize) ;
            reveal(< LogicalScreenDescriptorFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for LogicalScreenDescriptorFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecParser>::spec_parse) ;
            reveal(< LogicalScreenDescriptorFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LogicalScreenDescriptorFmt as Consistency>::consistent) ;
            reveal(< LogicalScreenDescriptorFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: LogicalScreenDescriptorSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                LogicalScreenDescriptorSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for LogicalScreenDescriptorFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LogicalScreenDescriptorInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LogicalScreenDescriptorSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for LogicalScreenDescriptorFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< LogicalScreenDescriptorFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LogicalScreenDescriptorFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for LogicalScreenDescriptorFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< LogicalScreenDescriptorFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LogicalScreenDescriptorFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for GifFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< GifFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for GifFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< GifFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for GifFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< GifFmt as SpecParser>::spec_parse) ;
            reveal(< GifFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: GifInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                GifSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< GifFmt as SpecParser>::spec_parse) ;
            reveal(< GifFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: GifInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                GifSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for GifFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< GifFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< GifFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< GifFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for GifFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< GifFmt as SpecSerializer>::spec_serialize) ;
            reveal(< GifFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for GifFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< GifFmt as SpecParser>::spec_parse) ;
            reveal(< GifFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< GifFmt as Consistency>::consistent) ;
            reveal(< GifFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: GifSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                GifSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for GifFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< GifFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: GifInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                GifSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for GifFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< GifFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< GifFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for GifFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< GifFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< GifFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for HeaderFmt {
        type PT = Header<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<HeaderFmt as SpecParser>::spec_parse);
            reveal(<Header as DeepView>::deep_view);
            reveal(HeaderSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, signature) = (Fixed::< 3 >).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, version) = (Fixed::< 3 >).parse (& rest) ?;
            let rest = rest.skip(n2);
            let total_n = n1 + n2;
            let final_v = Header {
                signature,
                version,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Header<'i>> for HeaderFmt {
        fn serialize_into(&self, v: &Header<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<HeaderFmt as SpecSerializer>::spec_serialize);
            reveal(<HeaderFmt as SpecByteLen>::byte_len);
            reveal(<Header as DeepView>::deep_view);
            reveal(HeaderSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Header {
                signature,
                version,
            } = v;
            Fixed::< 3 >.serialize_into(* signature, obuf);
            Fixed::< 3 >.serialize_into(*version, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Header<'i>> for HeaderFmt {
        fn prepare(&self, v: &Header<'i>) -> Result<usize, PreSerializeError> {
            reveal(<HeaderFmt as SpecByteLen>::byte_len);
            reveal(<Header as DeepView>::deep_view);
            reveal(HeaderSpec::into_structural);
            let Header {
                signature,
                version,
            } = v;
            let l1 = (Fixed::< 3 >).prepare (signature) ?;
            let l2 = (Fixed::< 3 >).prepare (version) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for ScreenFlagsFmt {
        type PT = ScreenFlags;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ScreenFlagsFmt as SpecParser>::spec_parse);
            reveal(<ScreenFlags as DeepView>::deep_view);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, raw) = U8.parse(ibuf)?;
            let (globalColorTable, colorResolution, sortFlag, globalColorTableSize) = unpack_screen_flags(raw);
            let final_v = ScreenFlags {
                globalColorTable: globalColorTable,
                colorResolution: colorResolution,
                sortFlag: sortFlag,
                globalColorTableSize: globalColorTableSize,
            };
            assert(self.spec_parse(ibuf@) == Some((n as int, final_v.deep_view())));
            Ok((n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, ScreenFlags> for ScreenFlagsFmt {
        fn serialize_into(&self, v: &ScreenFlags, obuf: &mut Output) {
            reveal(<ScreenFlagsFmt as SpecSerializer>::spec_serialize);
            reveal(<ScreenFlagsFmt as SpecByteLen>::byte_len);
            reveal(<ScreenFlags as DeepView>::deep_view);
            let ghost old_obuf = obuf@;

            let ScreenFlags {
                globalColorTable,
                colorResolution,
                sortFlag,
                globalColorTableSize
            }
            = *v ;
            let packed = pack_screen_flags(globalColorTable, colorResolution, sortFlag, globalColorTableSize);
            U8.serialize_into(&packed, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<ScreenFlags> for ScreenFlagsFmt {
        fn prepare(&self, v: &ScreenFlags) -> Result<usize, PreSerializeError> {
            reveal(<ScreenFlagsFmt as SpecByteLen>::byte_len);
            reveal(<ScreenFlags as DeepView>::deep_view);
            let ScreenFlags {
                globalColorTable,
                colorResolution,
                sortFlag,
                globalColorTableSize
            }
            = *v ;
            if !(screen_flags_bounds (globalColorTable, colorResolution, sortFlag, globalColorTableSize)) {
                return Err(PreSerializeError::not_compliant(ComplianceErrorKind::PredicateFailed));
            }
            let packed = pack_screen_flags(globalColorTable, colorResolution, sortFlag, globalColorTableSize);
            U8.prepare(&packed)
        }
    }



    impl<'i> Parser<&'i [u8]> for LogicalScreenDescriptorFmt {
        type PT = LogicalScreenDescriptor;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<LogicalScreenDescriptorFmt as SpecParser>::spec_parse);
            reveal(<LogicalScreenDescriptor as DeepView>::deep_view);
            reveal(LogicalScreenDescriptorSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, width) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, height) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, flags) = (Named ("screen_flags", ScreenFlagsFmt)).parse (& rest) ?;
            proof {
                flags.lemma_deep_view();
            }
            let rest = rest.skip(n3);
            let (n4, backgroundColorIndex) = (U8).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, pixelAspectRatio) = (U8).parse (& rest) ?;
            let rest = rest.skip(n5);
            let total_n = n1 + n2 + n3 + n4 + n5;
            let final_v = LogicalScreenDescriptor {
                width,
                height,
                flags,
                backgroundColorIndex,
                pixelAspectRatio,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, LogicalScreenDescriptor> for LogicalScreenDescriptorFmt {
        fn serialize_into(&self, v: &LogicalScreenDescriptor, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<LogicalScreenDescriptorFmt as SpecSerializer>::spec_serialize);
            reveal(<LogicalScreenDescriptorFmt as SpecByteLen>::byte_len);
            reveal(<LogicalScreenDescriptor as DeepView>::deep_view);
            reveal(LogicalScreenDescriptorSpec::into_structural);
            let ghost old_obuf = obuf@;

            let LogicalScreenDescriptor {
                width,
                height,
                flags,
                backgroundColorIndex,
                pixelAspectRatio,
            } = v;
            proof {
                flags.lemma_deep_view();
            }

            U16Le.serialize_into(width, obuf);
            U16Le.serialize_into(height, obuf);
            ScreenFlagsFmt.serialize_into(flags, obuf);
            U8.serialize_into(backgroundColorIndex, obuf);
            U8.serialize_into(pixelAspectRatio, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<LogicalScreenDescriptor> for LogicalScreenDescriptorFmt {
        fn prepare(&self, v: &LogicalScreenDescriptor) -> Result<usize, PreSerializeError> {
            reveal(<LogicalScreenDescriptorFmt as SpecByteLen>::byte_len);
            reveal(<LogicalScreenDescriptor as DeepView>::deep_view);
            reveal(LogicalScreenDescriptorSpec::into_structural);
            let LogicalScreenDescriptor {
                width,
                height,
                flags,
                backgroundColorIndex,
                pixelAspectRatio,
            } = v;
            proof {
                flags.lemma_deep_view();
            }

            let l1 = (U16Le).prepare (width) ?;
            let l2 = (U16Le).prepare (height) ?;
            let l3 = (Named ("screen_flags", ScreenFlagsFmt)).prepare (flags) ?;
            let l4 = (U8).prepare (backgroundColorIndex) ?;
            let l5 = (U8).prepare (pixelAspectRatio) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for GifFmt {
        type PT = Gif<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<GifFmt as SpecParser>::spec_parse);
            reveal(<Gif as DeepView>::deep_view);
            reveal(GifSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, header) = (Named ("header", HeaderFmt)).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, screen) = (Named ("logical_screen_descriptor", LogicalScreenDescriptorFmt)).parse (& rest) ?;
            let rest = rest.skip(n2);
            let total_n = n1 + n2;
            let final_v = Gif {
                header,
                screen,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Gif<'i>> for GifFmt {
        fn serialize_into(&self, v: &Gif<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<GifFmt as SpecSerializer>::spec_serialize);
            reveal(<GifFmt as SpecByteLen>::byte_len);
            reveal(<Gif as DeepView>::deep_view);
            reveal(GifSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Gif {
                header,
                screen,
            } = v;
            HeaderFmt.serialize_into(header, obuf);
            LogicalScreenDescriptorFmt.serialize_into(screen, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Gif<'i>> for GifFmt {
        fn prepare(&self, v: &Gif<'i>) -> Result<usize, PreSerializeError> {
            reveal(<GifFmt as SpecByteLen>::byte_len);
            reveal(<Gif as DeepView>::deep_view);
            reveal(GifSpec::into_structural);
            let Gif {
                header,
                screen,
            } = v;
            let l1 = (Named ("header", HeaderFmt)).prepare (header) ?;
            let l2 = (Named ("logical_screen_descriptor", LogicalScreenDescriptorFmt)).prepare (screen) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
