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
# [doc = "data type for `signature`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Signature<'i> {
    pub highBit: u8,
    pub png: &'i [u8],
    pub dosLineEnding: &'i [u8],
    pub dosEof: u8,
    pub unixLineEnding: u8,
}
# [verifier::ext_equal]
pub struct SignatureSpec < T0 = u8, T1 = Seq < u8 >, T2 = Seq < u8 >, T3 = u8, T4 = u8 > {
    pub highBit: T0,
    pub png: T1,
    pub dosLineEnding: T2,
    pub dosEof: T3,
    pub unixLineEnding: T4,
}
pub type SignatureInner = (u8, (Seq < u8 >, (Seq < u8 >, (u8, u8)))) ;
impl<'i> DeepView for Signature<'i> {
    type V = SignatureSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        SignatureSpec {
            highBit: self.highBit.deep_view(),
            png: self.png.deep_view(),
            dosLineEnding: self.dosLineEnding.deep_view(),
            dosEof: self.dosEof.deep_view(),
            unixLineEnding: self.unixLineEnding.deep_view(),
        }
    }
}
impl<'i> Signature<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().highBit == self.highBit.deep_view(),
    self.deep_view().png == self.png.deep_view(),
    self.deep_view().dosLineEnding == self.dosLineEnding.deep_view(),
    self.deep_view().dosEof == self.dosEof.deep_view(),
    self.deep_view().unixLineEnding == self.unixLineEnding.deep_view(),
    {
        reveal(< Signature as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4 > SignatureSpec < T0, T1, T2, T3, T4 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    T4))))) -> Self {
        let (highBit,
        (png,
        (dosLineEnding,
        (dosEof,
        unixLineEnding)))) = input ;
        Self {
            highBit,
            png,
            dosLineEnding,
            dosEof,
            unixLineEnding
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    (T3,
    T4)))) {
        let Self {
            highBit,
            png,
            dosLineEnding,
            dosEof,
            unixLineEnding
        }
        = self ;
        (highBit,
        (png,
        (dosLineEnding,
        (dosEof,
        unixLineEnding))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(SignatureSpec::from_structural) ;
        reveal(SignatureSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    T4))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(SignatureSpec::from_structural) ;
        reveal(SignatureSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            highBit,
            png,
            dosLineEnding,
            dosEof,
            unixLineEnding
        }
        => (highBit,
        (png,
        (dosLineEnding,
        (dosEof,
        unixLineEnding)))),
    }
   ,
    {
        reveal(SignatureSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct SignatureForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct SignatureReverse ;
impl SpecMap for SignatureForward {
    type Input = SignatureInner ;
    type Output = SignatureSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        SignatureSpec::from_structural (input)
    }
}
impl SpecMap for SignatureReverse {
    type Input = SignatureSpec ;
    type Output = SignatureInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `color_type`."]
# [repr (u8)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2,
    Palette = 3,
    GrayscaleAlpha = 4,
    RGBA = 6,
    Unknown (u8),
}
pub type ColorTypeSpec = ColorType ;
pub type ColorTypeInner = Sum < u8, u8 > ;
impl DeepView for ColorType {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl ColorType {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< ColorType as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: ColorTypeInner) -> bool {
        match input {
            L (x) => x == 0 || x == 2 || x == 3 || x == 4 || x == 6,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: ColorTypeInner) -> Self {
        match input {
            L (x) => match x {
                0 => Self::Grayscale,
                2 => Self::RGB,
                3 => Self::Palette,
                4 => Self::GrayscaleAlpha,
                6 => Self::RGBA,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> ColorTypeInner {
        match self {
            Self::Grayscale => L (0),
            Self::RGB => L (2),
            Self::Palette => L (3),
            Self::GrayscaleAlpha => L (4),
            Self::RGBA => L (6),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(ColorType::from_structural) ;
        reveal(ColorType::into_structural) ;
        match self {
            Self::Grayscale => {
            }
           ,
            Self::RGB => {
            }
           ,
            Self::Palette => {
            }
           ,
            Self::GrayscaleAlpha => {
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
    pub broadcast proof fn lemma_into_from (input: ColorTypeInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(ColorType::from_structural) ;
        reveal(ColorType::into_structural) ;
        match input {
            L (x) => match x {
                0 => {
                }
               ,
                2 => {
                }
               ,
                3 => {
                }
               ,
                4 => {
                }
               ,
                6 => {
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
pub struct ColorTypeForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ColorTypeReverse ;
impl SpecMap for ColorTypeForward {
    type Input = ColorTypeInner ;
    type Output = ColorTypeSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        ColorType::from_structural (input)
    }
}
impl SpecMap for ColorTypeReverse {
    type Input = ColorTypeSpec ;
    type Output = ColorTypeInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for ColorType {
}

# [doc = "data type for `ihdr`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Ihdr {
    pub width: u32,
    pub height: u32,
    pub bitDepth: u8,
    pub colorType: ColorType,
    pub compressionMethod: u8,
    pub filterMethod: u8,
    pub interlaceMethod: u8,
}
# [verifier::ext_equal]
pub struct IhdrSpec < T0 = u32, T1 = u32, T2 = u8, T3 = ColorTypeSpec, T4 = u8, T5 = u8, T6 = u8 > {
    pub width: T0,
    pub height: T1,
    pub bitDepth: T2,
    pub colorType: T3,
    pub compressionMethod: T4,
    pub filterMethod: T5,
    pub interlaceMethod: T6,
}
pub type IhdrInner = (u32, (u32, (u8, (ColorTypeSpec, (u8, (u8, u8)))))) ;
impl DeepView for Ihdr {
    type V = IhdrSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        IhdrSpec {
            width: self.width.deep_view(),
            height: self.height.deep_view(),
            bitDepth: self.bitDepth.deep_view(),
            colorType: self.colorType.deep_view(),
            compressionMethod: self.compressionMethod.deep_view(),
            filterMethod: self.filterMethod.deep_view(),
            interlaceMethod: self.interlaceMethod.deep_view(),
        }
    }
}
impl Ihdr {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().width == self.width.deep_view(),
    self.deep_view().height == self.height.deep_view(),
    self.deep_view().bitDepth == self.bitDepth.deep_view(),
    self.deep_view().colorType == self.colorType.deep_view(),
    self.deep_view().compressionMethod == self.compressionMethod.deep_view(),
    self.deep_view().filterMethod == self.filterMethod.deep_view(),
    self.deep_view().interlaceMethod == self.interlaceMethod.deep_view(),
    {
        reveal(< Ihdr as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6 > IhdrSpec < T0, T1, T2, T3, T4, T5, T6 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6))))))) -> Self {
        let (width,
        (height,
        (bitDepth,
        (colorType,
        (compressionMethod,
        (filterMethod,
        interlaceMethod)))))) = input ;
        Self {
            width,
            height,
            bitDepth,
            colorType,
            compressionMethod,
            filterMethod,
            interlaceMethod
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
            width,
            height,
            bitDepth,
            colorType,
            compressionMethod,
            filterMethod,
            interlaceMethod
        }
        = self ;
        (width,
        (height,
        (bitDepth,
        (colorType,
        (compressionMethod,
        (filterMethod,
        interlaceMethod))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(IhdrSpec::from_structural) ;
        reveal(IhdrSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(IhdrSpec::from_structural) ;
        reveal(IhdrSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            width,
            height,
            bitDepth,
            colorType,
            compressionMethod,
            filterMethod,
            interlaceMethod
        }
        => (width,
        (height,
        (bitDepth,
        (colorType,
        (compressionMethod,
        (filterMethod,
        interlaceMethod)))))),
    }
   ,
    {
        reveal(IhdrSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct IhdrForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct IhdrReverse ;
impl SpecMap for IhdrForward {
    type Input = IhdrInner ;
    type Output = IhdrSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        IhdrSpec::from_structural (input)
    }
}
impl SpecMap for IhdrReverse {
    type Input = IhdrSpec ;
    type Output = IhdrInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `chunk`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Chunk<'i> {
    pub length: u32,
    pub type_: &'i [u8],
    pub ihdr: ChunkIhdr<'i>,
    pub crc: u32,
}
# [verifier::ext_equal]
pub struct ChunkSpec < T0 = u32, T1 = Seq < u8 >, T2 = ChunkIhdrSpec, T3 = u32 > {
    pub length: T0,
    pub type_: T1,
    pub ihdr: T2,
    pub crc: T3,
}
pub type ChunkInner = (u32, (Seq < u8 >, (ChunkIhdrSpec, u32))) ;
impl<'i> DeepView for Chunk<'i> {
    type V = ChunkSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        ChunkSpec {
            length: self.length.deep_view(),
            type_: self.type_.deep_view(),
            ihdr: self.ihdr.deep_view(),
            crc: self.crc.deep_view(),
        }
    }
}
impl<'i> Chunk<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().length == self.length.deep_view(),
    self.deep_view().type_ == self.type_.deep_view(),
    self.deep_view().ihdr == self.ihdr.deep_view(),
    self.deep_view().crc == self.crc.deep_view(),
    {
        reveal(< Chunk as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3 > ChunkSpec < T0, T1, T2, T3 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    T3)))) -> Self {
        let (length,
        (type_,
        (ihdr,
        crc))) = input ;
        Self {
            length,
            type_,
            ihdr,
            crc
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    T3))) {
        let Self {
            length,
            type_,
            ihdr,
            crc
        }
        = self ;
        (length,
        (type_,
        (ihdr,
        crc)))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(ChunkSpec::from_structural) ;
        reveal(ChunkSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    T3)))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(ChunkSpec::from_structural) ;
        reveal(ChunkSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            length,
            type_,
            ihdr,
            crc
        }
        => (length,
        (type_,
        (ihdr,
        crc))),
    }
   ,
    {
        reveal(ChunkSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ChunkForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ChunkReverse ;
impl SpecMap for ChunkForward {
    type Input = ChunkInner ;
    type Output = ChunkSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        ChunkSpec::from_structural (input)
    }
}
impl SpecMap for ChunkReverse {
    type Input = ChunkSpec ;
    type Output = ChunkInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `png`."]
# [derive (Debug, PartialEq, Eq, Clone)]
pub struct Png<'i> {
    pub signature: Signature<'i>,
    pub chunks: Vec < Chunk<'i> >,
}
# [verifier::ext_equal]
pub struct PngSpec < T0 = SignatureSpec, T1 = Seq < ChunkSpec > > {
    pub signature: T0,
    pub chunks: T1,
}
pub type PngInner = (SignatureSpec, Seq < ChunkSpec >) ;
impl<'i> DeepView for Png<'i> {
    type V = PngSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        PngSpec {
            signature: self.signature.deep_view(),
            chunks: self.chunks.deep_view(),
        }
    }
}
impl<'i> Png<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().signature == self.signature.deep_view(),
    self.deep_view().chunks == self.chunks.deep_view(),
    {
        reveal(< Png as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > PngSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    T1)) -> Self {
        let (signature,
        chunks) = input ;
        Self {
            signature,
            chunks
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    T1) {
        let Self {
            signature,
            chunks
        }
        = self ;
        (signature,
        chunks)
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(PngSpec::from_structural) ;
        reveal(PngSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    T1)) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(PngSpec::from_structural) ;
        reveal(PngSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            signature,
            chunks
        }
        => (signature,
        chunks),
    }
   ,
    {
        reveal(PngSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct PngForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct PngReverse ;
impl SpecMap for PngForward {
    type Input = PngInner ;
    type Output = PngSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        PngSpec::from_structural (input)
    }
}
impl SpecMap for PngReverse {
    type Input = PngSpec ;
    type Output = PngInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `chunk_ihdr`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChunkIhdr<'i> {
    Variant1 (Ihdr),
    Default (&'i [u8]),
}
# [verifier::ext_equal]
pub enum ChunkIhdrSpec < T0 = IhdrSpec, T1 = Seq < u8 > > {
    Variant1 (T0),
    Default (T1),
}
pub type ChunkIhdrInner = Sum < IhdrSpec, Seq < u8 > > ;
impl<'i> DeepView for ChunkIhdr<'i> {
    type V = ChunkIhdrSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        match self {
            ChunkIhdr::Variant1 (v) => ChunkIhdrSpec::Variant1 (v.deep_view()),
            ChunkIhdr::Default (v) => ChunkIhdrSpec::Default (v.deep_view()),
        }
    }
}
impl<'i> ChunkIhdr<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view() == match self {
        ChunkIhdr::Variant1 (v) => ChunkIhdrSpec::Variant1 (v.deep_view()),
        ChunkIhdr::Default (v) => ChunkIhdrSpec::Default (v.deep_view()),
    }
   ,
    {
        reveal(< ChunkIhdr as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > ChunkIhdrSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: Sum < T0,
    T1 >) -> Self {
        match input {
            L (value) => Self::Variant1 (value),
            R (value) => Self::Default (value),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> Sum < T0,
    T1 > {
        match self {
            Self::Variant1 (value) => L (value),
            Self::Default (value) => R (value),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(ChunkIhdrSpec::from_structural) ;
        reveal(ChunkIhdrSpec::into_structural) ;
        match self {
            Self::Variant1 (_) => {
            }
           ,
            Self::Default (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: Sum < T0,
    T1 >) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(ChunkIhdrSpec::from_structural) ;
        reveal(ChunkIhdrSpec::into_structural) ;
        match input {
            L (_) => {
            }
           ,
            R (_) => {
            }
           ,
        }
    }
    pub proof fn lemma_into_structural_variant (self) ensures Self::into_structural (self) == match self {
        Self::Variant1 (value) => L (value),
        Self::Default (value) => R (value),
    }
   ,
    {
        reveal(ChunkIhdrSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ChunkIhdrForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ChunkIhdrReverse ;
impl SpecMap for ChunkIhdrForward {
    type Input = ChunkIhdrInner ;
    type Output = ChunkIhdrSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        ChunkIhdrSpec::from_structural (input)
    }
}
impl SpecMap for ChunkIhdrReverse {
    type Input = ChunkIhdrSpec ;
    type Output = ChunkIhdrInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `signature`."]
# [derive (Clone, Copy)]
pub struct SignatureFmt ;

pub type SignatureFmtSpec = Named < Mapped < Pair < U8, Pair < Fixed < 3 >, Pair < Fixed < 2 >, Pair < U8, U8 > > > >, BiMap < SignatureForward, SignatureReverse >> > ;

impl SignatureFmt {
    # [doc = "specification constructor for `signature`."] pub open spec fn spec_inner() -> SignatureFmtSpec {
        Named ("signature",
        Mapped {
            inner: Pair (U8,
            Pair (Fixed::< 3 >,
            Pair (Fixed::< 2 >,
            Pair (U8,
            U8)))),
            mapper: BiMap (SignatureForward,
            SignatureReverse),
        }
        )
    }
}


# [doc = "named format combinator for `color_type`."]
# [derive (Clone, Copy)]
pub struct ColorTypeFmt ;

pub type ColorTypeFmtSpec = Named < Mapped < Choice < Refined < U8, PredFnSpec < u8 >>, Refined < U8, PredFnSpec < u8 >> >, BiMap < ColorTypeForward, ColorTypeReverse >> > ;

impl ColorTypeFmt {
    # [doc = "specification constructor for `color_type`."] pub open spec fn spec_inner() -> ColorTypeFmtSpec {
        Named ("color_type",
        Mapped {
            inner: Choice (Refined (U8,
            | x: u8 | ((((x == 0) || (x == 2)) || (x == 3)) || (x == 4)) || (x == 6)),
            Refined (U8,
            | x: u8 | ((((x != 0) && (x != 2)) && (x != 3)) && (x != 4)) && (x != 6))),
            mapper: BiMap (ColorTypeForward,
            ColorTypeReverse),
        }
        )
    }
}


# [doc = "named format combinator for `ihdr`."]
# [derive (Clone, Copy)]
pub struct IhdrFmt ;

pub type IhdrFmtSpec = Named < Mapped < Pair < U32Be, Pair < U32Be, Pair < U8, Pair < ColorTypeFmt, Pair < U8, Pair < U8, U8 > > > > > >, BiMap < IhdrForward, IhdrReverse >> > ;

impl IhdrFmt {
    # [doc = "specification constructor for `ihdr`."] pub open spec fn spec_inner() -> IhdrFmtSpec {
        Named ("ihdr",
        Mapped {
            inner: Pair (U32Be,
            Pair (U32Be,
            Pair (U8,
            Pair (ColorTypeFmt,
            Pair (U8,
            Pair (U8,
            U8)))))),
            mapper: BiMap (IhdrForward,
            IhdrReverse),
        }
        )
    }
}


# [doc = "named format combinator for `chunk`."]
# [derive (Clone, Copy)]
pub struct ChunkFmt ;

pub type ChunkFmtSpec = Named < Mapped < Bind < U32Be, spec_fn (u32) -> Bind < Fixed < 4 >, spec_fn (Seq < u8 >) -> Pair < ChunkIhdrFmtSpec, U32Be > > >, BiMap < ChunkForward, ChunkReverse >> > ;

impl ChunkFmt {
    # [doc = "specification constructor for `chunk`."] pub open spec fn spec_inner() -> ChunkFmtSpec {
        Named ("chunk",
        Mapped {
            inner: Bind (U32Be,
            | length: u32 | Bind (Fixed::< 4 >,
            | type_: Seq < u8 > | Pair (ChunkIhdrFmt::spec_inner (length,
            type_),
            U32Be))),
            mapper: BiMap (ChunkForward,
            ChunkReverse),
        }
        )
    }
}


# [doc = "named format combinator for `png`."]
# [derive (Clone, Copy)]
pub struct PngFmt ;

pub type PngFmtSpec = Named < Mapped < Pair < SignatureFmt, RepeatTillEnd < ChunkFmt > >, BiMap < PngForward, PngReverse >> > ;

impl PngFmt {
    # [doc = "specification constructor for `png`."] pub open spec fn spec_inner() -> PngFmtSpec {
        Named ("png",
        Mapped {
            inner: Pair (SignatureFmt,
            RepeatTillEnd (ChunkFmt)),
            mapper: BiMap (PngForward,
            PngReverse),
        }
        )
    }
}


# [doc = "named format combinator for `chunk_ihdr`."]
# [derive (Clone, Copy)]
pub struct ChunkIhdrFmt<'i> {
    length: u32,
    type_: &'i [u8],
}
impl<'i> ChunkIhdrFmt<'i> {
    # [verifier::type_invariant] spec fn wf (& self) -> bool {
        true
    }
    pub closed spec fn length_spec (& self) -> u32 {
        self.length.deep_view()
    }
    pub closed spec fn type__spec (& self) -> Seq < u8 > {
        self.type_.deep_view()
    }
    pub closed spec fn spec (length: u32,
    type_: &'i [u8]) -> Self {
        ChunkIhdrFmt {
            length,
            type_
        }
    }
}

pub type ChunkIhdrFmtSpec = Named < Mapped < Sum < IhdrFmt, Varied < u32 > >, BiMap < ChunkIhdrForward, ChunkIhdrReverse >> > ;

impl<'i> ChunkIhdrFmt<'i> {
    # [doc = "specification constructor for `chunk_ihdr`."] pub open spec fn spec_inner (length: u32,
    type_: Seq < u8 >) -> ChunkIhdrFmtSpec {
        Named ("chunk_ihdr",
        Mapped {
            inner: match type_ {
                x if x == [0x49u8, 0x48u8, 0x44u8, 0x52u8].deep_view() => L (IhdrFmt),
                _ => R (Varied (length)),
            }
           ,
            mapper: BiMap (ChunkIhdrForward,
            ChunkIhdrReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for SignatureFmt {
        type PVal = SignatureSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for SignatureFmt {
        type Val = SignatureSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for SignatureFmt {
        type SValue = SignatureSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for SignatureFmt {
        type SVal = SignatureSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for SignatureFmt {
        type T = SignatureSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for ColorTypeFmt {
        type PVal = ColorTypeSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ColorTypeFmt {
        type Val = ColorTypeSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ColorTypeFmt {
        type SValue = ColorTypeSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ColorTypeFmt {
        type SVal = ColorTypeSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ColorTypeFmt {
        type T = ColorTypeSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for IhdrFmt {
        type PVal = IhdrSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for IhdrFmt {
        type Val = IhdrSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for IhdrFmt {
        type SValue = IhdrSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for IhdrFmt {
        type SVal = IhdrSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for IhdrFmt {
        type T = IhdrSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for ChunkFmt {
        type PVal = ChunkSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ChunkFmt {
        type Val = ChunkSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ChunkFmt {
        type SValue = ChunkSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ChunkFmt {
        type SVal = ChunkSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ChunkFmt {
        type T = ChunkSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for PngFmt {
        type PVal = PngSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for PngFmt {
        type Val = PngSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for PngFmt {
        type SValue = PngSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for PngFmt {
        type SVal = PngSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for PngFmt {
        type T = PngSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl<'i> SpecParser for ChunkIhdrFmt<'i> {
        type PVal = ChunkIhdrSpec ;
        open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner (self.length_spec(),
            self.type__spec()).spec_parse (ibuf)
        }
    }
    impl<'i> Consistency for ChunkIhdrFmt<'i> {
        type Val = ChunkIhdrSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner (self.length_spec(),
            self.type__spec()).consistent (v)
        }
    }
    impl<'i> SpecSerializerDps for ChunkIhdrFmt<'i> {
        type SValue = ChunkIhdrSpec ;
        open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner (self.length_spec(),
            self.type__spec()).spec_serialize_dps (v,
            obuf)
        }
    }
    impl<'i> SpecSerializer for ChunkIhdrFmt<'i> {
        type SVal = ChunkIhdrSpec ;
        open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner (self.length_spec(),
            self.type__spec()).spec_serialize (v)
        }
    }
    impl<'i> SpecByteLen for ChunkIhdrFmt<'i> {
        type T = ChunkIhdrSpec ;
        open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner (self.length_spec(),
            self.type__spec()).byte_len (v)
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
        SignatureSpec::lemma_from_into,
        SignatureSpec::lemma_into_from,
        ColorType::lemma_from_into,
        ColorType::lemma_into_from,
        IhdrSpec::lemma_from_into,
        IhdrSpec::lemma_into_from,
        ChunkSpec::lemma_from_into,
        ChunkSpec::lemma_into_from,
        PngSpec::lemma_from_into,
        PngSpec::lemma_into_from,
        ChunkIhdrSpec::lemma_from_into,
        ChunkIhdrSpec::lemma_into_from,
    };

    impl SafeParser for SignatureFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< SignatureFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for SignatureFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< SignatureFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for SignatureFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< SignatureFmt as SpecParser>::spec_parse) ;
            reveal(< SignatureFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: SignatureInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                SignatureSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< SignatureFmt as SpecParser>::spec_parse) ;
            reveal(< SignatureFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: SignatureInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                SignatureSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for SignatureFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< SignatureFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< SignatureFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< SignatureFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for SignatureFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< SignatureFmt as SpecSerializer>::spec_serialize) ;
            reveal(< SignatureFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for SignatureFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< SignatureFmt as SpecParser>::spec_parse) ;
            reveal(< SignatureFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< SignatureFmt as Consistency>::consistent) ;
            reveal(< SignatureFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: SignatureSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                SignatureSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for SignatureFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< SignatureFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: SignatureInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                SignatureSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for SignatureFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< SignatureFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< SignatureFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for SignatureFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< SignatureFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< SignatureFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for ColorTypeFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ColorTypeFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ColorTypeFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ColorTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ColorTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ColorType::structural_valid (input)) ;
                ColorType::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ColorTypeFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ColorTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ColorType::structural_valid (input)) ;
                ColorType::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ColorTypeFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ColorTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ColorTypeFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ColorTypeFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ColorTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ColorTypeFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ColorTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ColorTypeFmt as Consistency>::consistent) ;
            reveal(< ColorTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ColorTypeSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                ColorType::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ColorTypeFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ColorTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ColorType::structural_valid (input)) ;
                ColorType::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ColorTypeFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ColorTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ColorTypeFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ColorTypeFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ColorTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ColorTypeFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for IhdrFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< IhdrFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for IhdrFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< IhdrFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for IhdrFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< IhdrFmt as SpecParser>::spec_parse) ;
            reveal(< IhdrFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IhdrInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IhdrSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< IhdrFmt as SpecParser>::spec_parse) ;
            reveal(< IhdrFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IhdrInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IhdrSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for IhdrFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< IhdrFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< IhdrFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IhdrFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for IhdrFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< IhdrFmt as SpecSerializer>::spec_serialize) ;
            reveal(< IhdrFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for IhdrFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< IhdrFmt as SpecParser>::spec_parse) ;
            reveal(< IhdrFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IhdrFmt as Consistency>::consistent) ;
            reveal(< IhdrFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: IhdrSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                IhdrSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for IhdrFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< IhdrFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IhdrInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IhdrSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for IhdrFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< IhdrFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IhdrFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for IhdrFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< IhdrFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IhdrFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for ChunkFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ChunkFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ChunkFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ChunkFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ChunkFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ChunkFmt as SpecParser>::spec_parse) ;
            reveal(< ChunkFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ChunkInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ChunkSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ChunkFmt as SpecParser>::spec_parse) ;
            reveal(< ChunkFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ChunkInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ChunkSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ChunkFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ChunkFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ChunkFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ChunkFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ChunkFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ChunkFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ChunkFmt as SpecParser>::spec_parse) ;
            reveal(< ChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ChunkFmt as Consistency>::consistent) ;
            reveal(< ChunkFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ChunkSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                ChunkSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ChunkFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ChunkFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ChunkInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ChunkSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ChunkFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ChunkFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ChunkFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ChunkFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ChunkFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for PngFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< PngFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for PngFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< PngFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for PngFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< PngFmt as SpecParser>::spec_parse) ;
            reveal(< PngFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: PngInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                PngSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< PngFmt as SpecParser>::spec_parse) ;
            reveal(< PngFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: PngInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                PngSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl GoodSerializer for PngFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< PngFmt as SpecSerializer>::spec_serialize) ;
            reveal(< PngFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for PngFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< PngFmt as SpecParser>::spec_parse) ;
            reveal(< PngFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< PngFmt as Consistency>::consistent) ;
            reveal(< PngFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: PngSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                PngSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for PngFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< PngFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: PngInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                PngSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializers for PngFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< PngFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< PngFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl<'i> SafeParser for ChunkIhdrFmt<'i> {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            Self::spec_inner (self.length_spec(),
            self.type__spec()).lemma_parse_safe (ibuf) ;
        }
    }
    impl<'i> Productive for ChunkIhdrFmt<'i> {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner (self.length_spec(),
            self.type__spec()).productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl<'i> SoundParser for ChunkIhdrFmt<'i> {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert forall | input: ChunkIhdrInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ChunkIhdrSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert forall | input: ChunkIhdrInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ChunkIhdrSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl<'i> NonTailFmt for ChunkIhdrFmt<'i> {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl<'i> GoodSerializer for ChunkIhdrFmt<'i> {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl<'i> SPRoundTripDps for ChunkIhdrFmt<'i> {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert forall | output: ChunkIhdrSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                ChunkIhdrSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl<'i> NonMalleable for ChunkIhdrFmt<'i> {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert forall | input: ChunkIhdrInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ChunkIhdrSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl<'i> EquivSerializersGeneral for ChunkIhdrFmt<'i> {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl<'i> EquivSerializers for ChunkIhdrFmt<'i> {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            let fmt = Self::spec_inner (self.length_spec(),
            self.type__spec()) ;
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

    impl<'i> Parser<&'i [u8]> for SignatureFmt {
        type PT = Signature<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<SignatureFmt as SpecParser>::spec_parse);
            reveal(<Signature as DeepView>::deep_view);
            reveal(SignatureSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, highBit) = (U8).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, png) = (Fixed::< 3 >).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, dosLineEnding) = (Fixed::< 2 >).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, dosEof) = (U8).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, unixLineEnding) = (U8).parse (& rest) ?;
            let rest = rest.skip(n5);
            let total_n = n1 + n2 + n3 + n4 + n5;
            let final_v = Signature {
                highBit,
                png,
                dosLineEnding,
                dosEof,
                unixLineEnding,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Signature<'i>> for SignatureFmt {
        fn serialize_into(&self, v: &Signature<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<SignatureFmt as SpecSerializer>::spec_serialize);
            reveal(<SignatureFmt as SpecByteLen>::byte_len);
            reveal(<Signature as DeepView>::deep_view);
            reveal(SignatureSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Signature {
                highBit,
                png,
                dosLineEnding,
                dosEof,
                unixLineEnding,
            } = v;
            U8.serialize_into(highBit, obuf);
            Fixed::< 3 >.serialize_into(* png, obuf);
            Fixed::< 2 >.serialize_into(* dosLineEnding, obuf);
            U8.serialize_into(dosEof, obuf);
            U8.serialize_into(unixLineEnding, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Signature<'i>> for SignatureFmt {
        fn prepare(&self, v: &Signature<'i>) -> Result<usize, PreSerializeError> {
            reveal(<SignatureFmt as SpecByteLen>::byte_len);
            reveal(<Signature as DeepView>::deep_view);
            reveal(SignatureSpec::into_structural);
            let Signature {
                highBit,
                png,
                dosLineEnding,
                dosEof,
                unixLineEnding,
            } = v;
            let l1 = (U8).prepare (highBit) ?;
            let l2 = (Fixed::< 3 >).prepare (png) ?;
            let l3 = (Fixed::< 2 >).prepare (dosLineEnding) ?;
            let l4 = (U8).prepare (dosEof) ?;
            let l5 = (U8).prepare (unixLineEnding) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for ColorTypeFmt {
        type PT = ColorType;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ColorTypeFmt as SpecParser>::spec_parse);
            reveal(<ColorType as DeepView>::deep_view);
            reveal(ColorType::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U8.parse(&rest)?;
            let enum_val = match v {
                0 => ColorType::Grayscale,
                2 => ColorType::RGB,
                3 => ColorType::Palette,
                4 => ColorType::GrayscaleAlpha,
                6 => ColorType::RGBA,
                x => ColorType::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, ColorType> for ColorTypeFmt {
        fn serialize_into(&self, v: &ColorType, obuf: &mut Output) {
            reveal(<ColorTypeFmt as SpecSerializer>::spec_serialize);
            reveal(<ColorTypeFmt as SpecByteLen>::byte_len);
            reveal(<ColorType as DeepView>::deep_view);
            reveal(ColorType::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                ColorType::Grayscale => 0,
                ColorType::RGB => 2,
                ColorType::Palette => 3,
                ColorType::GrayscaleAlpha => 4,
                ColorType::RGBA => 6,
                ColorType::Unknown (x) => x,
            };
            U8.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<ColorType> for ColorTypeFmt {
        fn prepare(&self, v: &ColorType) -> Result<usize, PreSerializeError> {
            reveal(<ColorTypeFmt as SpecByteLen>::byte_len);
            reveal(<ColorType as DeepView>::deep_view);
            reveal(ColorType::into_structural);
            let tag = match *v {
                ColorType::Grayscale => 0,
                ColorType::RGB => 2,
                ColorType::Palette => 3,
                ColorType::GrayscaleAlpha => 4,
                ColorType::RGBA => 6,
                ColorType::Unknown (x) if x != 0 && x != 2 && x != 3 && x != 4 && x != 6 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U8.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for IhdrFmt {
        type PT = Ihdr;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<IhdrFmt as SpecParser>::spec_parse);
            reveal(<Ihdr as DeepView>::deep_view);
            reveal(IhdrSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, width) = (U32Be).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, height) = (U32Be).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, bitDepth) = (U8).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, colorType) = (Named ("color_type", ColorTypeFmt)).parse (& rest) ?;
            proof {
                colorType.lemma_deep_view();
            }
            let rest = rest.skip(n4);
            let (n5, compressionMethod) = (U8).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, filterMethod) = (U8).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, interlaceMethod) = (U8).parse (& rest) ?;
            let rest = rest.skip(n7);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7;
            let final_v = Ihdr {
                width,
                height,
                bitDepth,
                colorType,
                compressionMethod,
                filterMethod,
                interlaceMethod,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Ihdr> for IhdrFmt {
        fn serialize_into(&self, v: &Ihdr, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<IhdrFmt as SpecSerializer>::spec_serialize);
            reveal(<IhdrFmt as SpecByteLen>::byte_len);
            reveal(<Ihdr as DeepView>::deep_view);
            reveal(IhdrSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Ihdr {
                width,
                height,
                bitDepth,
                colorType,
                compressionMethod,
                filterMethod,
                interlaceMethod,
            } = v;
            proof {
                colorType.lemma_deep_view();
            }

            U32Be.serialize_into(width, obuf);
            U32Be.serialize_into(height, obuf);
            U8.serialize_into(bitDepth, obuf);
            ColorTypeFmt.serialize_into(colorType, obuf);
            U8.serialize_into(compressionMethod, obuf);
            U8.serialize_into(filterMethod, obuf);
            U8.serialize_into(interlaceMethod, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Ihdr> for IhdrFmt {
        fn prepare(&self, v: &Ihdr) -> Result<usize, PreSerializeError> {
            reveal(<IhdrFmt as SpecByteLen>::byte_len);
            reveal(<Ihdr as DeepView>::deep_view);
            reveal(IhdrSpec::into_structural);
            let Ihdr {
                width,
                height,
                bitDepth,
                colorType,
                compressionMethod,
                filterMethod,
                interlaceMethod,
            } = v;
            proof {
                colorType.lemma_deep_view();
            }

            let l1 = (U32Be).prepare (width) ?;
            let l2 = (U32Be).prepare (height) ?;
            let l3 = (U8).prepare (bitDepth) ?;
            let l4 = (Named ("color_type", ColorTypeFmt)).prepare (colorType) ?;
            let l5 = (U8).prepare (compressionMethod) ?;
            let l6 = (U8).prepare (filterMethod) ?;
            let l7 = (U8).prepare (interlaceMethod) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for ChunkFmt {
        type PT = Chunk<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<ChunkFmt as SpecParser>::spec_parse);
            reveal(<Chunk as DeepView>::deep_view);
            reveal(ChunkSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, length) = (U32Be).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, type_) = (Fixed::< 4 >).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, ihdr) = (Named ("chunk_ihdr", ChunkIhdrFmt {
                length: length,
                type_: type_
            }
            )).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, crc) = (U32Be).parse (& rest) ?;
            let rest = rest.skip(n4);
            let total_n = n1 + n2 + n3 + n4;
            let final_v = Chunk {
                length,
                type_,
                ihdr,
                crc,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Chunk<'i>> for ChunkFmt {
        fn serialize_into(&self, v: &Chunk<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<ChunkFmt as SpecSerializer>::spec_serialize);
            reveal(<ChunkFmt as SpecByteLen>::byte_len);
            reveal(<Chunk as DeepView>::deep_view);
            reveal(ChunkSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Chunk {
                length,
                type_,
                ihdr,
                crc,
            } = v;
            U32Be.serialize_into(length, obuf);
            Fixed::< 4 >.serialize_into(* type_, obuf);
            ChunkIhdrFmt {
                length: *length,
                type_: * type_
            }
            .serialize_into(ihdr, obuf);
            U32Be.serialize_into(crc, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Chunk<'i>> for ChunkFmt {
        fn prepare(&self, v: &Chunk<'i>) -> Result<usize, PreSerializeError> {
            reveal(<ChunkFmt as SpecByteLen>::byte_len);
            reveal(<Chunk as DeepView>::deep_view);
            reveal(ChunkSpec::into_structural);
            let Chunk {
                length,
                type_,
                ihdr,
                crc,
            } = v;
            let l1 = (U32Be).prepare (length) ?;
            let l2 = (Fixed::< 4 >).prepare (type_) ?;
            let l3 = (Named ("chunk_ihdr", ChunkIhdrFmt {
                length: *length,
                type_: * type_
            }
            )).prepare (ihdr) ?;
            let l4 = (U32Be).prepare (crc) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for PngFmt {
        type PT = Png<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<PngFmt as SpecParser>::spec_parse);
            reveal(<Png as DeepView>::deep_view);
            reveal(PngSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, signature) = (Named ("signature", SignatureFmt)).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, chunks) = (RepeatTillEnd (ChunkFmt)).parse (& rest) ?;
            let rest = rest.skip(n2);
            let total_n = n1 + n2;
            let final_v = Png {
                signature,
                chunks,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Png<'i>> for PngFmt {
        fn serialize_into(&self, v: &Png<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<PngFmt as SpecSerializer>::spec_serialize);
            reveal(<PngFmt as SpecByteLen>::byte_len);
            reveal(<Png as DeepView>::deep_view);
            reveal(PngSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Png {
                signature,
                chunks,
            } = v;
            SignatureFmt.serialize_into(signature, obuf);
            Star (ChunkFmt).serialize_into(chunks, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Png<'i>> for PngFmt {
        fn prepare(&self, v: &Png<'i>) -> Result<usize, PreSerializeError> {
            reveal(<PngFmt as SpecByteLen>::byte_len);
            reveal(<Png as DeepView>::deep_view);
            reveal(PngSpec::into_structural);
            let Png {
                signature,
                chunks,
            } = v;
            let l1 = (Named ("signature", SignatureFmt)).prepare (signature) ?;
            let l2 = (Star (ChunkFmt)).prepare (chunks) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for ChunkIhdrFmt<'i> {
        type PT = ChunkIhdr<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ChunkIhdrFmt as SpecParser>::spec_parse);
            reveal(<ChunkIhdr as DeepView>::deep_view);
            reveal(ChunkIhdrSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            proof {
                use_type_invariant(self);
            }

            let (n, v) = match self.type_ {
                x if bytes_eq (x, &[0x49, 0x48, 0x44, 0x52]) => {
                    let (n,
                    v) = (Named ("ihdr",
                    IhdrFmt)).parse (& rest) ?;
                    (n,
                    ChunkIhdr::Variant1 (v))
                }
                ,
                _ => {
                    let (n,
                    v) = (Varied (self.length)).parse (& rest) ?;
                    (n,
                    ChunkIhdr::Default (v))
                }
                ,
            };
            assert(self.spec_parse(ibuf@) == Some((n as int, v.deep_view())));
            Ok((n, v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, ChunkIhdr<'i>> for ChunkIhdrFmt<'i> {
        fn serialize_into(&self, v: &ChunkIhdr<'i>, obuf: &mut Output) {
            reveal(<ChunkIhdrFmt as SpecSerializer>::spec_serialize);
            reveal(<ChunkIhdrFmt as SpecByteLen>::byte_len);
            reveal(<ChunkIhdr as DeepView>::deep_view);
            reveal(ChunkIhdrSpec::into_structural);
            proof {
                use_type_invariant(self);
            }

            let ghost old_obuf = obuf@;

            match (self.type_, v) {
                (x, ChunkIhdr::Variant1 (v)) if bytes_eq (x, &[0x49, 0x48, 0x44, 0x52]) => {
                    (IhdrFmt).serialize_into (v,
                    obuf) ;
                }
                ,
                (_, ChunkIhdr::Default (v)) => {
                    (Varied (self.length)).serialize_into (*v,
                    obuf) ;
                }
                ,
                _ => {},
            }

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<ChunkIhdr<'i>> for ChunkIhdrFmt<'i> {
        fn prepare(&self, v: &ChunkIhdr<'i>) -> Result<usize, PreSerializeError> {
            reveal(<ChunkIhdrFmt as SpecByteLen>::byte_len);
            reveal(<ChunkIhdr as DeepView>::deep_view);
            reveal(ChunkIhdrSpec::into_structural);
            proof {
                use_type_invariant(self);
            }

            match (self.type_, v) {
                (x, ChunkIhdr::Variant1 (v)) if bytes_eq (x, &[0x49, 0x48, 0x44, 0x52]) => (Named ("ihdr", IhdrFmt)).prepare (v),
                (x, ChunkIhdr::Default (v)) if ! bytes_eq (x, &[0x49, 0x48, 0x44, 0x52]) => (Varied (self.length)).prepare (v),
                 _ => Err(PreSerializeError::not_compliant(ComplianceErrorKind::InvalidTag)),
            }
        }
    }

}
}
