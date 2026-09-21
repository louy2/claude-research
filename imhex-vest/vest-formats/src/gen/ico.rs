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
# [doc = "data type for `image_type`."]
# [repr (u16)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum ImageType {
    Icon = 1,
    Cursor = 2,
    Unknown (u16),
}
pub type ImageTypeSpec = ImageType ;
pub type ImageTypeInner = Sum < u16, u16 > ;
impl DeepView for ImageType {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl ImageType {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< ImageType as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: ImageTypeInner) -> bool {
        match input {
            L (x) => x == 1 || x == 2,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: ImageTypeInner) -> Self {
        match input {
            L (x) => match x {
                1 => Self::Icon,
                2 => Self::Cursor,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> ImageTypeInner {
        match self {
            Self::Icon => L (1),
            Self::Cursor => L (2),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(ImageType::from_structural) ;
        reveal(ImageType::into_structural) ;
        match self {
            Self::Icon => {
            }
           ,
            Self::Cursor => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: ImageTypeInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(ImageType::from_structural) ;
        reveal(ImageType::into_structural) ;
        match input {
            L (x) => match x {
                1 => {
                }
               ,
                2 => {
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
pub struct ImageTypeForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ImageTypeReverse ;
impl SpecMap for ImageTypeForward {
    type Input = ImageTypeInner ;
    type Output = ImageTypeSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        ImageType::from_structural (input)
    }
}
impl SpecMap for ImageTypeReverse {
    type Input = ImageTypeSpec ;
    type Output = ImageTypeInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for ImageType {
}

# [doc = "data type for `header`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Header {
    pub reserved: u16,
    pub type_: ImageType,
    pub count: u16,
}
# [verifier::ext_equal]
pub struct HeaderSpec < T0 = u16, T1 = ImageTypeSpec, T2 = u16 > {
    pub reserved: T0,
    pub type_: T1,
    pub count: T2,
}
pub type HeaderInner = (u16, (ImageTypeSpec, u16)) ;
impl DeepView for Header {
    type V = HeaderSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        HeaderSpec {
            reserved: self.reserved.deep_view(),
            type_: self.type_.deep_view(),
            count: self.count.deep_view(),
        }
    }
}
impl Header {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().reserved == self.reserved.deep_view(),
    self.deep_view().type_ == self.type_.deep_view(),
    self.deep_view().count == self.count.deep_view(),
    {
        reveal(< Header as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2 > HeaderSpec < T0, T1, T2 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    T2))) -> Self {
        let (reserved,
        (type_,
        count)) = input ;
        Self {
            reserved,
            type_,
            count
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    T2)) {
        let Self {
            reserved,
            type_,
            count
        }
        = self ;
        (reserved,
        (type_,
        count))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(HeaderSpec::from_structural) ;
        reveal(HeaderSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    T2))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(HeaderSpec::from_structural) ;
        reveal(HeaderSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            reserved,
            type_,
            count
        }
        => (reserved,
        (type_,
        count)),
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

# [doc = "data type for `dir_entry`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct DirEntry {
    pub width: u8,
    pub height: u8,
    pub colorCount: u8,
    pub reserved: u8,
    pub planes: u16,
    pub bitCount: u16,
    pub size: u32,
    pub offset: u32,
}
# [verifier::ext_equal]
pub struct DirEntrySpec < T0 = u8, T1 = u8, T2 = u8, T3 = u8, T4 = u16, T5 = u16, T6 = u32, T7 = u32 > {
    pub width: T0,
    pub height: T1,
    pub colorCount: T2,
    pub reserved: T3,
    pub planes: T4,
    pub bitCount: T5,
    pub size: T6,
    pub offset: T7,
}
pub type DirEntryInner = (u8, (u8, (u8, (u8, (u16, (u16, (u32, u32))))))) ;
impl DeepView for DirEntry {
    type V = DirEntrySpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        DirEntrySpec {
            width: self.width.deep_view(),
            height: self.height.deep_view(),
            colorCount: self.colorCount.deep_view(),
            reserved: self.reserved.deep_view(),
            planes: self.planes.deep_view(),
            bitCount: self.bitCount.deep_view(),
            size: self.size.deep_view(),
            offset: self.offset.deep_view(),
        }
    }
}
impl DirEntry {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().width == self.width.deep_view(),
    self.deep_view().height == self.height.deep_view(),
    self.deep_view().colorCount == self.colorCount.deep_view(),
    self.deep_view().reserved == self.reserved.deep_view(),
    self.deep_view().planes == self.planes.deep_view(),
    self.deep_view().bitCount == self.bitCount.deep_view(),
    self.deep_view().size == self.size.deep_view(),
    self.deep_view().offset == self.offset.deep_view(),
    {
        reveal(< DirEntry as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7 > DirEntrySpec < T0, T1, T2, T3, T4, T5, T6, T7 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    T7)))))))) -> Self {
        let (width,
        (height,
        (colorCount,
        (reserved,
        (planes,
        (bitCount,
        (size,
        offset))))))) = input ;
        Self {
            width,
            height,
            colorCount,
            reserved,
            planes,
            bitCount,
            size,
            offset
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
            width,
            height,
            colorCount,
            reserved,
            planes,
            bitCount,
            size,
            offset
        }
        = self ;
        (width,
        (height,
        (colorCount,
        (reserved,
        (planes,
        (bitCount,
        (size,
        offset)))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(DirEntrySpec::from_structural) ;
        reveal(DirEntrySpec::into_structural) ;
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
        reveal(DirEntrySpec::from_structural) ;
        reveal(DirEntrySpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            width,
            height,
            colorCount,
            reserved,
            planes,
            bitCount,
            size,
            offset
        }
        => (width,
        (height,
        (colorCount,
        (reserved,
        (planes,
        (bitCount,
        (size,
        offset))))))),
    }
   ,
    {
        reveal(DirEntrySpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct DirEntryForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct DirEntryReverse ;
impl SpecMap for DirEntryForward {
    type Input = DirEntryInner ;
    type Output = DirEntrySpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        DirEntrySpec::from_structural (input)
    }
}
impl SpecMap for DirEntryReverse {
    type Input = DirEntrySpec ;
    type Output = DirEntryInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `ico`."]
# [derive (Debug, PartialEq, Eq, Clone)]
pub struct Ico {
    pub header: Header,
    pub entries: Vec < DirEntry >,
}
# [verifier::ext_equal]
pub struct IcoSpec < T0 = HeaderSpec, T1 = Seq < DirEntrySpec > > {
    pub header: T0,
    pub entries: T1,
}
pub type IcoInner = (HeaderSpec, Seq < DirEntrySpec >) ;
impl DeepView for Ico {
    type V = IcoSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        IcoSpec {
            header: self.header.deep_view(),
            entries: self.entries.deep_view(),
        }
    }
}
impl Ico {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().header == self.header.deep_view(),
    self.deep_view().entries == self.entries.deep_view(),
    {
        reveal(< Ico as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > IcoSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    T1)) -> Self {
        let (header,
        entries) = input ;
        Self {
            header,
            entries
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    T1) {
        let Self {
            header,
            entries
        }
        = self ;
        (header,
        entries)
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(IcoSpec::from_structural) ;
        reveal(IcoSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    T1)) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(IcoSpec::from_structural) ;
        reveal(IcoSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            header,
            entries
        }
        => (header,
        entries),
    }
   ,
    {
        reveal(IcoSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct IcoForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct IcoReverse ;
impl SpecMap for IcoForward {
    type Input = IcoInner ;
    type Output = IcoSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        IcoSpec::from_structural (input)
    }
}
impl SpecMap for IcoReverse {
    type Input = IcoSpec ;
    type Output = IcoInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `image_type`."]
# [derive (Clone, Copy)]
pub struct ImageTypeFmt ;

pub type ImageTypeFmtSpec = Named < Mapped < Choice < Refined < U16Le, PredFnSpec < u16 >>, Refined < U16Le, PredFnSpec < u16 >> >, BiMap < ImageTypeForward, ImageTypeReverse >> > ;

impl ImageTypeFmt {
    # [doc = "specification constructor for `image_type`."] pub open spec fn spec_inner() -> ImageTypeFmtSpec {
        Named ("image_type",
        Mapped {
            inner: Choice (Refined (U16Le,
            | x: u16 | (x == 1) || (x == 2)),
            Refined (U16Le,
            | x: u16 | (x != 1) && (x != 2))),
            mapper: BiMap (ImageTypeForward,
            ImageTypeReverse),
        }
        )
    }
}


# [doc = "named format combinator for `header`."]
# [derive (Clone, Copy)]
pub struct HeaderFmt ;

pub type HeaderFmtSpec = Named < Mapped < Pair < U16Le, Pair < ImageTypeFmt, U16Le > >, BiMap < HeaderForward, HeaderReverse >> > ;

impl HeaderFmt {
    # [doc = "specification constructor for `header`."] pub open spec fn spec_inner() -> HeaderFmtSpec {
        Named ("header",
        Mapped {
            inner: Pair (U16Le,
            Pair (ImageTypeFmt,
            U16Le)),
            mapper: BiMap (HeaderForward,
            HeaderReverse),
        }
        )
    }
}


# [doc = "named format combinator for `dir_entry`."]
# [derive (Clone, Copy)]
pub struct DirEntryFmt ;

pub type DirEntryFmtSpec = Named < Mapped < Pair < U8, Pair < U8, Pair < U8, Pair < U8, Pair < U16Le, Pair < U16Le, Pair < U32Le, U32Le > > > > > > >, BiMap < DirEntryForward, DirEntryReverse >> > ;

impl DirEntryFmt {
    # [doc = "specification constructor for `dir_entry`."] pub open spec fn spec_inner() -> DirEntryFmtSpec {
        Named ("dir_entry",
        Mapped {
            inner: Pair (U8,
            Pair (U8,
            Pair (U8,
            Pair (U8,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U32Le,
            U32Le))))))),
            mapper: BiMap (DirEntryForward,
            DirEntryReverse),
        }
        )
    }
}


# [doc = "named format combinator for `ico`."]
# [derive (Clone, Copy)]
pub struct IcoFmt ;

pub type IcoFmtSpec = Named < Mapped < Bind < HeaderFmt, spec_fn (HeaderSpec) -> RepeatN < DirEntryFmt, u16 > >, BiMap < IcoForward, IcoReverse >> > ;

impl IcoFmt {
    # [doc = "specification constructor for `ico`."] pub open spec fn spec_inner() -> IcoFmtSpec {
        Named ("ico",
        Mapped {
            inner: Bind (HeaderFmt,
            | header: HeaderSpec | RepeatN (header.count,
            DirEntryFmt)),
            mapper: BiMap (IcoForward,
            IcoReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for ImageTypeFmt {
        type PVal = ImageTypeSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ImageTypeFmt {
        type Val = ImageTypeSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ImageTypeFmt {
        type SValue = ImageTypeSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ImageTypeFmt {
        type SVal = ImageTypeSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ImageTypeFmt {
        type T = ImageTypeSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

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

    impl SpecParser for DirEntryFmt {
        type PVal = DirEntrySpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for DirEntryFmt {
        type Val = DirEntrySpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for DirEntryFmt {
        type SValue = DirEntrySpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for DirEntryFmt {
        type SVal = DirEntrySpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for DirEntryFmt {
        type T = DirEntrySpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for IcoFmt {
        type PVal = IcoSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for IcoFmt {
        type Val = IcoSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for IcoFmt {
        type SValue = IcoSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for IcoFmt {
        type SVal = IcoSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for IcoFmt {
        type T = IcoSpec ;
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
        ImageType::lemma_from_into,
        ImageType::lemma_into_from,
        HeaderSpec::lemma_from_into,
        HeaderSpec::lemma_into_from,
        DirEntrySpec::lemma_from_into,
        DirEntrySpec::lemma_into_from,
        IcoSpec::lemma_from_into,
        IcoSpec::lemma_into_from,
    };

    impl SafeParser for ImageTypeFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ImageTypeFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ImageTypeFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ImageTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ImageTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ImageType::structural_valid (input)) ;
                ImageType::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ImageTypeFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ImageTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ImageType::structural_valid (input)) ;
                ImageType::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ImageTypeFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ImageTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ImageTypeFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ImageTypeFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ImageTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ImageTypeFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ImageTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ImageTypeFmt as Consistency>::consistent) ;
            reveal(< ImageTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ImageTypeSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                ImageType::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ImageTypeFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ImageTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ImageType::structural_valid (input)) ;
                ImageType::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ImageTypeFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ImageTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ImageTypeFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ImageTypeFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ImageTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ImageTypeFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

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

    impl SafeParser for DirEntryFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for DirEntryFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for DirEntryFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecParser>::spec_parse) ;
            reveal(< DirEntryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: DirEntryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                DirEntrySpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecParser>::spec_parse) ;
            reveal(< DirEntryFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: DirEntryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                DirEntrySpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for DirEntryFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< DirEntryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for DirEntryFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< DirEntryFmt as SpecSerializer>::spec_serialize) ;
            reveal(< DirEntryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for DirEntryFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecParser>::spec_parse) ;
            reveal(< DirEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< DirEntryFmt as Consistency>::consistent) ;
            reveal(< DirEntryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: DirEntrySpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                DirEntrySpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for DirEntryFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: DirEntryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                DirEntrySpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for DirEntryFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< DirEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< DirEntryFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for DirEntryFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< DirEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< DirEntryFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for IcoFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< IcoFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for IcoFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< IcoFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for IcoFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< IcoFmt as SpecParser>::spec_parse) ;
            reveal(< IcoFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IcoInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IcoSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< IcoFmt as SpecParser>::spec_parse) ;
            reveal(< IcoFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IcoInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IcoSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for IcoFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< IcoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< IcoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IcoFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for IcoFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< IcoFmt as SpecSerializer>::spec_serialize) ;
            reveal(< IcoFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for IcoFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< IcoFmt as SpecParser>::spec_parse) ;
            reveal(< IcoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IcoFmt as Consistency>::consistent) ;
            reveal(< IcoFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: IcoSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                IcoSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for IcoFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< IcoFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IcoInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IcoSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for IcoFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< IcoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IcoFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for IcoFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< IcoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IcoFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for ImageTypeFmt {
        type PT = ImageType;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ImageTypeFmt as SpecParser>::spec_parse);
            reveal(<ImageType as DeepView>::deep_view);
            reveal(ImageType::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U16Le.parse(&rest)?;
            let enum_val = match v {
                1 => ImageType::Icon,
                2 => ImageType::Cursor,
                x => ImageType::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, ImageType> for ImageTypeFmt {
        fn serialize_into(&self, v: &ImageType, obuf: &mut Output) {
            reveal(<ImageTypeFmt as SpecSerializer>::spec_serialize);
            reveal(<ImageTypeFmt as SpecByteLen>::byte_len);
            reveal(<ImageType as DeepView>::deep_view);
            reveal(ImageType::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                ImageType::Icon => 1,
                ImageType::Cursor => 2,
                ImageType::Unknown (x) => x,
            };
            U16Le.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<ImageType> for ImageTypeFmt {
        fn prepare(&self, v: &ImageType) -> Result<usize, PreSerializeError> {
            reveal(<ImageTypeFmt as SpecByteLen>::byte_len);
            reveal(<ImageType as DeepView>::deep_view);
            reveal(ImageType::into_structural);
            let tag = match *v {
                ImageType::Icon => 1,
                ImageType::Cursor => 2,
                ImageType::Unknown (x) if x != 1 && x != 2 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U16Le.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for HeaderFmt {
        type PT = Header;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<HeaderFmt as SpecParser>::spec_parse);
            reveal(<Header as DeepView>::deep_view);
            reveal(HeaderSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, reserved) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, type_) = (Named ("image_type", ImageTypeFmt)).parse (& rest) ?;
            proof {
                type_.lemma_deep_view();
            }
            let rest = rest.skip(n2);
            let (n3, count) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n3);
            let total_n = n1 + n2 + n3;
            let final_v = Header {
                reserved,
                type_,
                count,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Header> for HeaderFmt {
        fn serialize_into(&self, v: &Header, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<HeaderFmt as SpecSerializer>::spec_serialize);
            reveal(<HeaderFmt as SpecByteLen>::byte_len);
            reveal(<Header as DeepView>::deep_view);
            reveal(HeaderSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Header {
                reserved,
                type_,
                count,
            } = v;
            proof {
                type_.lemma_deep_view();
            }

            U16Le.serialize_into(reserved, obuf);
            ImageTypeFmt.serialize_into(type_, obuf);
            U16Le.serialize_into(count, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Header> for HeaderFmt {
        fn prepare(&self, v: &Header) -> Result<usize, PreSerializeError> {
            reveal(<HeaderFmt as SpecByteLen>::byte_len);
            reveal(<Header as DeepView>::deep_view);
            reveal(HeaderSpec::into_structural);
            let Header {
                reserved,
                type_,
                count,
            } = v;
            proof {
                type_.lemma_deep_view();
            }

            let l1 = (U16Le).prepare (reserved) ?;
            let l2 = (Named ("image_type", ImageTypeFmt)).prepare (type_) ?;
            let l3 = (U16Le).prepare (count) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for DirEntryFmt {
        type PT = DirEntry;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<DirEntryFmt as SpecParser>::spec_parse);
            reveal(<DirEntry as DeepView>::deep_view);
            reveal(DirEntrySpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, width) = (U8).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, height) = (U8).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, colorCount) = (U8).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, reserved) = (U8).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, planes) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, bitCount) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, size) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, offset) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n8);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8;
            let final_v = DirEntry {
                width,
                height,
                colorCount,
                reserved,
                planes,
                bitCount,
                size,
                offset,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, DirEntry> for DirEntryFmt {
        fn serialize_into(&self, v: &DirEntry, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<DirEntryFmt as SpecSerializer>::spec_serialize);
            reveal(<DirEntryFmt as SpecByteLen>::byte_len);
            reveal(<DirEntry as DeepView>::deep_view);
            reveal(DirEntrySpec::into_structural);
            let ghost old_obuf = obuf@;

            let DirEntry {
                width,
                height,
                colorCount,
                reserved,
                planes,
                bitCount,
                size,
                offset,
            } = v;
            U8.serialize_into(width, obuf);
            U8.serialize_into(height, obuf);
            U8.serialize_into(colorCount, obuf);
            U8.serialize_into(reserved, obuf);
            U16Le.serialize_into(planes, obuf);
            U16Le.serialize_into(bitCount, obuf);
            U32Le.serialize_into(size, obuf);
            U32Le.serialize_into(offset, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<DirEntry> for DirEntryFmt {
        fn prepare(&self, v: &DirEntry) -> Result<usize, PreSerializeError> {
            reveal(<DirEntryFmt as SpecByteLen>::byte_len);
            reveal(<DirEntry as DeepView>::deep_view);
            reveal(DirEntrySpec::into_structural);
            let DirEntry {
                width,
                height,
                colorCount,
                reserved,
                planes,
                bitCount,
                size,
                offset,
            } = v;
            let l1 = (U8).prepare (width) ?;
            let l2 = (U8).prepare (height) ?;
            let l3 = (U8).prepare (colorCount) ?;
            let l4 = (U8).prepare (reserved) ?;
            let l5 = (U16Le).prepare (planes) ?;
            let l6 = (U16Le).prepare (bitCount) ?;
            let l7 = (U32Le).prepare (size) ?;
            let l8 = (U32Le).prepare (offset) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for IcoFmt {
        type PT = Ico;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<IcoFmt as SpecParser>::spec_parse);
            reveal(<Ico as DeepView>::deep_view);
            reveal(IcoSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, header) = (Named ("header", HeaderFmt)).parse (& rest) ?;
            let rest = rest.skip(n1);
            proof {
                header.lemma_deep_view_fields();
                header.deep_view().lemma_into_structural_fields();
            }

            let (n2, entries) = (RepeatN (header.count, DirEntryFmt)).parse (& rest) ?;
            let rest = rest.skip(n2);
            let total_n = n1 + n2;
            let final_v = Ico {
                header,
                entries,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Ico> for IcoFmt {
        fn serialize_into(&self, v: &Ico, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<IcoFmt as SpecSerializer>::spec_serialize);
            reveal(<IcoFmt as SpecByteLen>::byte_len);
            reveal(<Ico as DeepView>::deep_view);
            reveal(IcoSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Ico {
                header,
                entries,
            } = v;
            proof {
                header.lemma_deep_view_fields();
                header.deep_view().lemma_into_structural_fields();
            }

            HeaderFmt.serialize_into(header, obuf);
            RepeatN (header.count, DirEntryFmt).serialize_into(entries, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Ico> for IcoFmt {
        fn prepare(&self, v: &Ico) -> Result<usize, PreSerializeError> {
            reveal(<IcoFmt as SpecByteLen>::byte_len);
            reveal(<Ico as DeepView>::deep_view);
            reveal(IcoSpec::into_structural);
            let Ico {
                header,
                entries,
            } = v;
            proof {
                header.lemma_deep_view_fields();
                header.deep_view().lemma_into_structural_fields();
            }

            let l1 = (Named ("header", HeaderFmt)).prepare (header) ?;
            let l2 = (RepeatN (header.count, DirEntryFmt)).prepare (entries) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
