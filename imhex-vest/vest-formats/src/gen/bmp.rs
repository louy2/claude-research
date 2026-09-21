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
# [doc = "data type for `bitmap_file_header`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct BitmapFileHeader<'i> {
    pub signature: &'i [u8],
    pub fileSize: u32,
    pub reserved1: u16,
    pub reserved2: u16,
    pub pixelDataOffset: u32,
}
# [verifier::ext_equal]
pub struct BitmapFileHeaderSpec < T0 = Seq < u8 >, T1 = u32, T2 = u16, T3 = u16, T4 = u32 > {
    pub signature: T0,
    pub fileSize: T1,
    pub reserved1: T2,
    pub reserved2: T3,
    pub pixelDataOffset: T4,
}
pub type BitmapFileHeaderInner = (Seq < u8 >, (u32, (u16, (u16, u32)))) ;
impl<'i> DeepView for BitmapFileHeader<'i> {
    type V = BitmapFileHeaderSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        BitmapFileHeaderSpec {
            signature: self.signature.deep_view(),
            fileSize: self.fileSize.deep_view(),
            reserved1: self.reserved1.deep_view(),
            reserved2: self.reserved2.deep_view(),
            pixelDataOffset: self.pixelDataOffset.deep_view(),
        }
    }
}
impl<'i> BitmapFileHeader<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().signature == self.signature.deep_view(),
    self.deep_view().fileSize == self.fileSize.deep_view(),
    self.deep_view().reserved1 == self.reserved1.deep_view(),
    self.deep_view().reserved2 == self.reserved2.deep_view(),
    self.deep_view().pixelDataOffset == self.pixelDataOffset.deep_view(),
    {
        reveal(< BitmapFileHeader as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4 > BitmapFileHeaderSpec < T0, T1, T2, T3, T4 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    T4))))) -> Self {
        let (signature,
        (fileSize,
        (reserved1,
        (reserved2,
        pixelDataOffset)))) = input ;
        Self {
            signature,
            fileSize,
            reserved1,
            reserved2,
            pixelDataOffset
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    (T3,
    T4)))) {
        let Self {
            signature,
            fileSize,
            reserved1,
            reserved2,
            pixelDataOffset
        }
        = self ;
        (signature,
        (fileSize,
        (reserved1,
        (reserved2,
        pixelDataOffset))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(BitmapFileHeaderSpec::from_structural) ;
        reveal(BitmapFileHeaderSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    T4))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(BitmapFileHeaderSpec::from_structural) ;
        reveal(BitmapFileHeaderSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            signature,
            fileSize,
            reserved1,
            reserved2,
            pixelDataOffset
        }
        => (signature,
        (fileSize,
        (reserved1,
        (reserved2,
        pixelDataOffset)))),
    }
   ,
    {
        reveal(BitmapFileHeaderSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct BitmapFileHeaderForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct BitmapFileHeaderReverse ;
impl SpecMap for BitmapFileHeaderForward {
    type Input = BitmapFileHeaderInner ;
    type Output = BitmapFileHeaderSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        BitmapFileHeaderSpec::from_structural (input)
    }
}
impl SpecMap for BitmapFileHeaderReverse {
    type Input = BitmapFileHeaderSpec ;
    type Output = BitmapFileHeaderInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `compression`."]
# [repr (u32)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum Compression {
    RGB = 0,
    RLE8 = 1,
    RLE4 = 2,
    Bitfields = 3,
    JPEG = 4,
    PNG = 5,
    Unknown (u32),
}
pub type CompressionSpec = Compression ;
pub type CompressionInner = Sum < u32, u32 > ;
impl DeepView for Compression {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Compression {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Compression as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: CompressionInner) -> bool {
        match input {
            L (x) => x == 0 || x == 1 || x == 2 || x == 3 || x == 4 || x == 5,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: CompressionInner) -> Self {
        match input {
            L (x) => match x {
                0 => Self::RGB,
                1 => Self::RLE8,
                2 => Self::RLE4,
                3 => Self::Bitfields,
                4 => Self::JPEG,
                5 => Self::PNG,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> CompressionInner {
        match self {
            Self::RGB => L (0),
            Self::RLE8 => L (1),
            Self::RLE4 => L (2),
            Self::Bitfields => L (3),
            Self::JPEG => L (4),
            Self::PNG => L (5),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Compression::from_structural) ;
        reveal(Compression::into_structural) ;
        match self {
            Self::RGB => {
            }
           ,
            Self::RLE8 => {
            }
           ,
            Self::RLE4 => {
            }
           ,
            Self::Bitfields => {
            }
           ,
            Self::JPEG => {
            }
           ,
            Self::PNG => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: CompressionInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Compression::from_structural) ;
        reveal(Compression::into_structural) ;
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
                4 => {
                }
               ,
                5 => {
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
pub struct CompressionForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct CompressionReverse ;
impl SpecMap for CompressionForward {
    type Input = CompressionInner ;
    type Output = CompressionSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Compression::from_structural (input)
    }
}
impl SpecMap for CompressionReverse {
    type Input = CompressionSpec ;
    type Output = CompressionInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for Compression {
}

# [doc = "data type for `bitmap_info_header`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct BitmapInfoHeader {
    pub headerSize: u32,
    pub width: u32,
    pub height: u32,
    pub planes: u16,
    pub bitsPerPixel: u16,
    pub compression: Compression,
    pub imageSize: u32,
    pub xPixelsPerMeter: u32,
    pub yPixelsPerMeter: u32,
    pub colorsUsed: u32,
    pub importantColors: u32,
}
# [verifier::ext_equal]
pub struct BitmapInfoHeaderSpec < T0 = u32, T1 = u32, T2 = u32, T3 = u16, T4 = u16, T5 = CompressionSpec, T6 = u32, T7 = u32, T8 = u32, T9 = u32, T10 = u32 > {
    pub headerSize: T0,
    pub width: T1,
    pub height: T2,
    pub planes: T3,
    pub bitsPerPixel: T4,
    pub compression: T5,
    pub imageSize: T6,
    pub xPixelsPerMeter: T7,
    pub yPixelsPerMeter: T8,
    pub colorsUsed: T9,
    pub importantColors: T10,
}
pub type BitmapInfoHeaderInner = (u32, (u32, (u32, (u16, (u16, (CompressionSpec, (u32, (u32, (u32, (u32, u32)))))))))) ;
impl DeepView for BitmapInfoHeader {
    type V = BitmapInfoHeaderSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        BitmapInfoHeaderSpec {
            headerSize: self.headerSize.deep_view(),
            width: self.width.deep_view(),
            height: self.height.deep_view(),
            planes: self.planes.deep_view(),
            bitsPerPixel: self.bitsPerPixel.deep_view(),
            compression: self.compression.deep_view(),
            imageSize: self.imageSize.deep_view(),
            xPixelsPerMeter: self.xPixelsPerMeter.deep_view(),
            yPixelsPerMeter: self.yPixelsPerMeter.deep_view(),
            colorsUsed: self.colorsUsed.deep_view(),
            importantColors: self.importantColors.deep_view(),
        }
    }
}
impl BitmapInfoHeader {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().headerSize == self.headerSize.deep_view(),
    self.deep_view().width == self.width.deep_view(),
    self.deep_view().height == self.height.deep_view(),
    self.deep_view().planes == self.planes.deep_view(),
    self.deep_view().bitsPerPixel == self.bitsPerPixel.deep_view(),
    self.deep_view().compression == self.compression.deep_view(),
    self.deep_view().imageSize == self.imageSize.deep_view(),
    self.deep_view().xPixelsPerMeter == self.xPixelsPerMeter.deep_view(),
    self.deep_view().yPixelsPerMeter == self.yPixelsPerMeter.deep_view(),
    self.deep_view().colorsUsed == self.colorsUsed.deep_view(),
    self.deep_view().importantColors == self.importantColors.deep_view(),
    {
        reveal(< BitmapInfoHeader as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 > BitmapInfoHeaderSpec < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 > {
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
    T10))))))))))) -> Self {
        let (headerSize,
        (width,
        (height,
        (planes,
        (bitsPerPixel,
        (compression,
        (imageSize,
        (xPixelsPerMeter,
        (yPixelsPerMeter,
        (colorsUsed,
        importantColors)))))))))) = input ;
        Self {
            headerSize,
            width,
            height,
            planes,
            bitsPerPixel,
            compression,
            imageSize,
            xPixelsPerMeter,
            yPixelsPerMeter,
            colorsUsed,
            importantColors
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
    T10)))))))))) {
        let Self {
            headerSize,
            width,
            height,
            planes,
            bitsPerPixel,
            compression,
            imageSize,
            xPixelsPerMeter,
            yPixelsPerMeter,
            colorsUsed,
            importantColors
        }
        = self ;
        (headerSize,
        (width,
        (height,
        (planes,
        (bitsPerPixel,
        (compression,
        (imageSize,
        (xPixelsPerMeter,
        (yPixelsPerMeter,
        (colorsUsed,
        importantColors))))))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(BitmapInfoHeaderSpec::from_structural) ;
        reveal(BitmapInfoHeaderSpec::into_structural) ;
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
    T10))))))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(BitmapInfoHeaderSpec::from_structural) ;
        reveal(BitmapInfoHeaderSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            headerSize,
            width,
            height,
            planes,
            bitsPerPixel,
            compression,
            imageSize,
            xPixelsPerMeter,
            yPixelsPerMeter,
            colorsUsed,
            importantColors
        }
        => (headerSize,
        (width,
        (height,
        (planes,
        (bitsPerPixel,
        (compression,
        (imageSize,
        (xPixelsPerMeter,
        (yPixelsPerMeter,
        (colorsUsed,
        importantColors)))))))))),
    }
   ,
    {
        reveal(BitmapInfoHeaderSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct BitmapInfoHeaderForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct BitmapInfoHeaderReverse ;
impl SpecMap for BitmapInfoHeaderForward {
    type Input = BitmapInfoHeaderInner ;
    type Output = BitmapInfoHeaderSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        BitmapInfoHeaderSpec::from_structural (input)
    }
}
impl SpecMap for BitmapInfoHeaderReverse {
    type Input = BitmapInfoHeaderSpec ;
    type Output = BitmapInfoHeaderInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `bmp`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Bmp<'i> {
    pub file: BitmapFileHeader<'i>,
    pub info: BitmapInfoHeader,
}
# [verifier::ext_equal]
pub struct BmpSpec < T0 = BitmapFileHeaderSpec, T1 = BitmapInfoHeaderSpec > {
    pub file: T0,
    pub info: T1,
}
pub type BmpInner = (BitmapFileHeaderSpec, BitmapInfoHeaderSpec) ;
impl<'i> DeepView for Bmp<'i> {
    type V = BmpSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        BmpSpec {
            file: self.file.deep_view(),
            info: self.info.deep_view(),
        }
    }
}
impl<'i> Bmp<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().file == self.file.deep_view(),
    self.deep_view().info == self.info.deep_view(),
    {
        reveal(< Bmp as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > BmpSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    T1)) -> Self {
        let (file,
        info) = input ;
        Self {
            file,
            info
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    T1) {
        let Self {
            file,
            info
        }
        = self ;
        (file,
        info)
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(BmpSpec::from_structural) ;
        reveal(BmpSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    T1)) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(BmpSpec::from_structural) ;
        reveal(BmpSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            file,
            info
        }
        => (file,
        info),
    }
   ,
    {
        reveal(BmpSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct BmpForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct BmpReverse ;
impl SpecMap for BmpForward {
    type Input = BmpInner ;
    type Output = BmpSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        BmpSpec::from_structural (input)
    }
}
impl SpecMap for BmpReverse {
    type Input = BmpSpec ;
    type Output = BmpInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `bitmap_file_header`."]
# [derive (Clone, Copy)]
pub struct BitmapFileHeaderFmt ;

pub type BitmapFileHeaderFmtSpec = Named < Mapped < Pair < Fixed < 2 >, Pair < U32Le, Pair < U16Le, Pair < U16Le, U32Le > > > >, BiMap < BitmapFileHeaderForward, BitmapFileHeaderReverse >> > ;

impl BitmapFileHeaderFmt {
    # [doc = "specification constructor for `bitmap_file_header`."] pub open spec fn spec_inner() -> BitmapFileHeaderFmtSpec {
        Named ("bitmap_file_header",
        Mapped {
            inner: Pair (Fixed::< 2 >,
            Pair (U32Le,
            Pair (U16Le,
            Pair (U16Le,
            U32Le)))),
            mapper: BiMap (BitmapFileHeaderForward,
            BitmapFileHeaderReverse),
        }
        )
    }
}


# [doc = "named format combinator for `compression`."]
# [derive (Clone, Copy)]
pub struct CompressionFmt ;

pub type CompressionFmtSpec = Named < Mapped < Choice < Refined < U32Le, PredFnSpec < u32 >>, Refined < U32Le, PredFnSpec < u32 >> >, BiMap < CompressionForward, CompressionReverse >> > ;

impl CompressionFmt {
    # [doc = "specification constructor for `compression`."] pub open spec fn spec_inner() -> CompressionFmtSpec {
        Named ("compression",
        Mapped {
            inner: Choice (Refined (U32Le,
            | x: u32 | (((((x == 0) || (x == 1)) || (x == 2)) || (x == 3)) || (x == 4)) || (x == 5)),
            Refined (U32Le,
            | x: u32 | (((((x != 0) && (x != 1)) && (x != 2)) && (x != 3)) && (x != 4)) && (x != 5))),
            mapper: BiMap (CompressionForward,
            CompressionReverse),
        }
        )
    }
}


# [doc = "named format combinator for `bitmap_info_header`."]
# [derive (Clone, Copy)]
pub struct BitmapInfoHeaderFmt ;

pub type BitmapInfoHeaderFmtSpec = Named < Mapped < Pair < U32Le, Pair < U32Le, Pair < U32Le, Pair < U16Le, Pair < U16Le, Pair < CompressionFmt, Pair < U32Le, Pair < U32Le, Pair < U32Le, Pair < U32Le, U32Le > > > > > > > > > >, BiMap < BitmapInfoHeaderForward, BitmapInfoHeaderReverse >> > ;

impl BitmapInfoHeaderFmt {
    # [doc = "specification constructor for `bitmap_info_header`."] pub open spec fn spec_inner() -> BitmapInfoHeaderFmtSpec {
        Named ("bitmap_info_header",
        Mapped {
            inner: Pair (U32Le,
            Pair (U32Le,
            Pair (U32Le,
            Pair (U16Le,
            Pair (U16Le,
            Pair (CompressionFmt,
            Pair (U32Le,
            Pair (U32Le,
            Pair (U32Le,
            Pair (U32Le,
            U32Le)))))))))),
            mapper: BiMap (BitmapInfoHeaderForward,
            BitmapInfoHeaderReverse),
        }
        )
    }
}


# [doc = "named format combinator for `bmp`."]
# [derive (Clone, Copy)]
pub struct BmpFmt ;

pub type BmpFmtSpec = Named < Mapped < Pair < BitmapFileHeaderFmt, BitmapInfoHeaderFmt >, BiMap < BmpForward, BmpReverse >> > ;

impl BmpFmt {
    # [doc = "specification constructor for `bmp`."] pub open spec fn spec_inner() -> BmpFmtSpec {
        Named ("bmp",
        Mapped {
            inner: Pair (BitmapFileHeaderFmt,
            BitmapInfoHeaderFmt),
            mapper: BiMap (BmpForward,
            BmpReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for BitmapFileHeaderFmt {
        type PVal = BitmapFileHeaderSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for BitmapFileHeaderFmt {
        type Val = BitmapFileHeaderSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for BitmapFileHeaderFmt {
        type SValue = BitmapFileHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for BitmapFileHeaderFmt {
        type SVal = BitmapFileHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for BitmapFileHeaderFmt {
        type T = BitmapFileHeaderSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for CompressionFmt {
        type PVal = CompressionSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for CompressionFmt {
        type Val = CompressionSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for CompressionFmt {
        type SValue = CompressionSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for CompressionFmt {
        type SVal = CompressionSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for CompressionFmt {
        type T = CompressionSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for BitmapInfoHeaderFmt {
        type PVal = BitmapInfoHeaderSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for BitmapInfoHeaderFmt {
        type Val = BitmapInfoHeaderSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for BitmapInfoHeaderFmt {
        type SValue = BitmapInfoHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for BitmapInfoHeaderFmt {
        type SVal = BitmapInfoHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for BitmapInfoHeaderFmt {
        type T = BitmapInfoHeaderSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for BmpFmt {
        type PVal = BmpSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for BmpFmt {
        type Val = BmpSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for BmpFmt {
        type SValue = BmpSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for BmpFmt {
        type SVal = BmpSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for BmpFmt {
        type T = BmpSpec ;
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
        BitmapFileHeaderSpec::lemma_from_into,
        BitmapFileHeaderSpec::lemma_into_from,
        Compression::lemma_from_into,
        Compression::lemma_into_from,
        BitmapInfoHeaderSpec::lemma_from_into,
        BitmapInfoHeaderSpec::lemma_into_from,
        BmpSpec::lemma_from_into,
        BmpSpec::lemma_into_from,
    };

    impl SafeParser for BitmapFileHeaderFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for BitmapFileHeaderFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for BitmapFileHeaderFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< BitmapFileHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BitmapFileHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BitmapFileHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< BitmapFileHeaderFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BitmapFileHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BitmapFileHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for BitmapFileHeaderFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BitmapFileHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for BitmapFileHeaderFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< BitmapFileHeaderFmt as SpecSerializer>::spec_serialize) ;
            reveal(< BitmapFileHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for BitmapFileHeaderFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< BitmapFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BitmapFileHeaderFmt as Consistency>::consistent) ;
            reveal(< BitmapFileHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: BitmapFileHeaderSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                BitmapFileHeaderSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for BitmapFileHeaderFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BitmapFileHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BitmapFileHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for BitmapFileHeaderFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< BitmapFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BitmapFileHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for BitmapFileHeaderFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< BitmapFileHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BitmapFileHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for CompressionFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< CompressionFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for CompressionFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< CompressionFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for CompressionFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< CompressionFmt as SpecParser>::spec_parse) ;
            reveal(< CompressionFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CompressionInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Compression::structural_valid (input)) ;
                Compression::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< CompressionFmt as SpecParser>::spec_parse) ;
            reveal(< CompressionFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CompressionInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Compression::structural_valid (input)) ;
                Compression::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for CompressionFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< CompressionFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< CompressionFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CompressionFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for CompressionFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< CompressionFmt as SpecSerializer>::spec_serialize) ;
            reveal(< CompressionFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for CompressionFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< CompressionFmt as SpecParser>::spec_parse) ;
            reveal(< CompressionFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CompressionFmt as Consistency>::consistent) ;
            reveal(< CompressionFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: CompressionSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Compression::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for CompressionFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< CompressionFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CompressionInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Compression::structural_valid (input)) ;
                Compression::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for CompressionFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< CompressionFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CompressionFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for CompressionFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< CompressionFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CompressionFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for BitmapInfoHeaderFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for BitmapInfoHeaderFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for BitmapInfoHeaderFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< BitmapInfoHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BitmapInfoHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BitmapInfoHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< BitmapInfoHeaderFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BitmapInfoHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BitmapInfoHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for BitmapInfoHeaderFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BitmapInfoHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for BitmapInfoHeaderFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< BitmapInfoHeaderFmt as SpecSerializer>::spec_serialize) ;
            reveal(< BitmapInfoHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for BitmapInfoHeaderFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< BitmapInfoHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BitmapInfoHeaderFmt as Consistency>::consistent) ;
            reveal(< BitmapInfoHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: BitmapInfoHeaderSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                BitmapInfoHeaderSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for BitmapInfoHeaderFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BitmapInfoHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BitmapInfoHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for BitmapInfoHeaderFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< BitmapInfoHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BitmapInfoHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for BitmapInfoHeaderFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< BitmapInfoHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BitmapInfoHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for BmpFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< BmpFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for BmpFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< BmpFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for BmpFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< BmpFmt as SpecParser>::spec_parse) ;
            reveal(< BmpFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BmpInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BmpSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< BmpFmt as SpecParser>::spec_parse) ;
            reveal(< BmpFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BmpInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BmpSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for BmpFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< BmpFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< BmpFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BmpFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for BmpFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< BmpFmt as SpecSerializer>::spec_serialize) ;
            reveal(< BmpFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for BmpFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< BmpFmt as SpecParser>::spec_parse) ;
            reveal(< BmpFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BmpFmt as Consistency>::consistent) ;
            reveal(< BmpFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: BmpSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                BmpSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for BmpFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< BmpFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: BmpInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                BmpSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for BmpFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< BmpFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BmpFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for BmpFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< BmpFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< BmpFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for BitmapFileHeaderFmt {
        type PT = BitmapFileHeader<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<BitmapFileHeaderFmt as SpecParser>::spec_parse);
            reveal(<BitmapFileHeader as DeepView>::deep_view);
            reveal(BitmapFileHeaderSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, signature) = (Fixed::< 2 >).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, fileSize) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, reserved1) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, reserved2) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, pixelDataOffset) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let total_n = n1 + n2 + n3 + n4 + n5;
            let final_v = BitmapFileHeader {
                signature,
                fileSize,
                reserved1,
                reserved2,
                pixelDataOffset,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, BitmapFileHeader<'i>> for BitmapFileHeaderFmt {
        fn serialize_into(&self, v: &BitmapFileHeader<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<BitmapFileHeaderFmt as SpecSerializer>::spec_serialize);
            reveal(<BitmapFileHeaderFmt as SpecByteLen>::byte_len);
            reveal(<BitmapFileHeader as DeepView>::deep_view);
            reveal(BitmapFileHeaderSpec::into_structural);
            let ghost old_obuf = obuf@;

            let BitmapFileHeader {
                signature,
                fileSize,
                reserved1,
                reserved2,
                pixelDataOffset,
            } = v;
            Fixed::< 2 >.serialize_into(* signature, obuf);
            U32Le.serialize_into(fileSize, obuf);
            U16Le.serialize_into(reserved1, obuf);
            U16Le.serialize_into(reserved2, obuf);
            U32Le.serialize_into(pixelDataOffset, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<BitmapFileHeader<'i>> for BitmapFileHeaderFmt {
        fn prepare(&self, v: &BitmapFileHeader<'i>) -> Result<usize, PreSerializeError> {
            reveal(<BitmapFileHeaderFmt as SpecByteLen>::byte_len);
            reveal(<BitmapFileHeader as DeepView>::deep_view);
            reveal(BitmapFileHeaderSpec::into_structural);
            let BitmapFileHeader {
                signature,
                fileSize,
                reserved1,
                reserved2,
                pixelDataOffset,
            } = v;
            let l1 = (Fixed::< 2 >).prepare (signature) ?;
            let l2 = (U32Le).prepare (fileSize) ?;
            let l3 = (U16Le).prepare (reserved1) ?;
            let l4 = (U16Le).prepare (reserved2) ?;
            let l5 = (U32Le).prepare (pixelDataOffset) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for CompressionFmt {
        type PT = Compression;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<CompressionFmt as SpecParser>::spec_parse);
            reveal(<Compression as DeepView>::deep_view);
            reveal(Compression::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U32Le.parse(&rest)?;
            let enum_val = match v {
                0 => Compression::RGB,
                1 => Compression::RLE8,
                2 => Compression::RLE4,
                3 => Compression::Bitfields,
                4 => Compression::JPEG,
                5 => Compression::PNG,
                x => Compression::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Compression> for CompressionFmt {
        fn serialize_into(&self, v: &Compression, obuf: &mut Output) {
            reveal(<CompressionFmt as SpecSerializer>::spec_serialize);
            reveal(<CompressionFmt as SpecByteLen>::byte_len);
            reveal(<Compression as DeepView>::deep_view);
            reveal(Compression::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                Compression::RGB => 0,
                Compression::RLE8 => 1,
                Compression::RLE4 => 2,
                Compression::Bitfields => 3,
                Compression::JPEG => 4,
                Compression::PNG => 5,
                Compression::Unknown (x) => x,
            };
            U32Le.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Compression> for CompressionFmt {
        fn prepare(&self, v: &Compression) -> Result<usize, PreSerializeError> {
            reveal(<CompressionFmt as SpecByteLen>::byte_len);
            reveal(<Compression as DeepView>::deep_view);
            reveal(Compression::into_structural);
            let tag = match *v {
                Compression::RGB => 0,
                Compression::RLE8 => 1,
                Compression::RLE4 => 2,
                Compression::Bitfields => 3,
                Compression::JPEG => 4,
                Compression::PNG => 5,
                Compression::Unknown (x) if x != 0 && x != 1 && x != 2 && x != 3 && x != 4 && x != 5 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U32Le.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for BitmapInfoHeaderFmt {
        type PT = BitmapInfoHeader;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<BitmapInfoHeaderFmt as SpecParser>::spec_parse);
            reveal(<BitmapInfoHeader as DeepView>::deep_view);
            reveal(BitmapInfoHeaderSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, headerSize) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, width) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, height) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, planes) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, bitsPerPixel) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, compression) = (Named ("compression", CompressionFmt)).parse (& rest) ?;
            proof {
                compression.lemma_deep_view();
            }
            let rest = rest.skip(n6);
            let (n7, imageSize) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, xPixelsPerMeter) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n8);
            let (n9, yPixelsPerMeter) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n9);
            let (n10, colorsUsed) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n10);
            let (n11, importantColors) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n11);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9 + n10 + n11;
            let final_v = BitmapInfoHeader {
                headerSize,
                width,
                height,
                planes,
                bitsPerPixel,
                compression,
                imageSize,
                xPixelsPerMeter,
                yPixelsPerMeter,
                colorsUsed,
                importantColors,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, BitmapInfoHeader> for BitmapInfoHeaderFmt {
        fn serialize_into(&self, v: &BitmapInfoHeader, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<BitmapInfoHeaderFmt as SpecSerializer>::spec_serialize);
            reveal(<BitmapInfoHeaderFmt as SpecByteLen>::byte_len);
            reveal(<BitmapInfoHeader as DeepView>::deep_view);
            reveal(BitmapInfoHeaderSpec::into_structural);
            let ghost old_obuf = obuf@;

            let BitmapInfoHeader {
                headerSize,
                width,
                height,
                planes,
                bitsPerPixel,
                compression,
                imageSize,
                xPixelsPerMeter,
                yPixelsPerMeter,
                colorsUsed,
                importantColors,
            } = v;
            proof {
                compression.lemma_deep_view();
            }

            U32Le.serialize_into(headerSize, obuf);
            U32Le.serialize_into(width, obuf);
            U32Le.serialize_into(height, obuf);
            U16Le.serialize_into(planes, obuf);
            U16Le.serialize_into(bitsPerPixel, obuf);
            CompressionFmt.serialize_into(compression, obuf);
            U32Le.serialize_into(imageSize, obuf);
            U32Le.serialize_into(xPixelsPerMeter, obuf);
            U32Le.serialize_into(yPixelsPerMeter, obuf);
            U32Le.serialize_into(colorsUsed, obuf);
            U32Le.serialize_into(importantColors, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<BitmapInfoHeader> for BitmapInfoHeaderFmt {
        fn prepare(&self, v: &BitmapInfoHeader) -> Result<usize, PreSerializeError> {
            reveal(<BitmapInfoHeaderFmt as SpecByteLen>::byte_len);
            reveal(<BitmapInfoHeader as DeepView>::deep_view);
            reveal(BitmapInfoHeaderSpec::into_structural);
            let BitmapInfoHeader {
                headerSize,
                width,
                height,
                planes,
                bitsPerPixel,
                compression,
                imageSize,
                xPixelsPerMeter,
                yPixelsPerMeter,
                colorsUsed,
                importantColors,
            } = v;
            proof {
                compression.lemma_deep_view();
            }

            let l1 = (U32Le).prepare (headerSize) ?;
            let l2 = (U32Le).prepare (width) ?;
            let l3 = (U32Le).prepare (height) ?;
            let l4 = (U16Le).prepare (planes) ?;
            let l5 = (U16Le).prepare (bitsPerPixel) ?;
            let l6 = (Named ("compression", CompressionFmt)).prepare (compression) ?;
            let l7 = (U32Le).prepare (imageSize) ?;
            let l8 = (U32Le).prepare (xPixelsPerMeter) ?;
            let l9 = (U32Le).prepare (yPixelsPerMeter) ?;
            let l10 = (U32Le).prepare (colorsUsed) ?;
            let l11 = (U32Le).prepare (importantColors) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l9).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l10).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l11).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for BmpFmt {
        type PT = Bmp<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<BmpFmt as SpecParser>::spec_parse);
            reveal(<Bmp as DeepView>::deep_view);
            reveal(BmpSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, file) = (Named ("bitmap_file_header", BitmapFileHeaderFmt)).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, info) = (Named ("bitmap_info_header", BitmapInfoHeaderFmt)).parse (& rest) ?;
            let rest = rest.skip(n2);
            let total_n = n1 + n2;
            let final_v = Bmp {
                file,
                info,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Bmp<'i>> for BmpFmt {
        fn serialize_into(&self, v: &Bmp<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<BmpFmt as SpecSerializer>::spec_serialize);
            reveal(<BmpFmt as SpecByteLen>::byte_len);
            reveal(<Bmp as DeepView>::deep_view);
            reveal(BmpSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Bmp {
                file,
                info,
            } = v;
            BitmapFileHeaderFmt.serialize_into(file, obuf);
            BitmapInfoHeaderFmt.serialize_into(info, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Bmp<'i>> for BmpFmt {
        fn prepare(&self, v: &Bmp<'i>) -> Result<usize, PreSerializeError> {
            reveal(<BmpFmt as SpecByteLen>::byte_len);
            reveal(<Bmp as DeepView>::deep_view);
            reveal(BmpSpec::into_structural);
            let Bmp {
                file,
                info,
            } = v;
            let l1 = (Named ("bitmap_file_header", BitmapFileHeaderFmt)).prepare (file) ?;
            let l2 = (Named ("bitmap_info_header", BitmapInfoHeaderFmt)).prepare (info) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
