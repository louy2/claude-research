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
# [doc = "data type for `head`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Head {
    pub magic: [u8 ;
    4],
}
# [verifier::ext_equal]
pub struct HeadSpec < T0 = Seq < u8 > > {
    pub magic: T0,
}
pub type HeadInner = Seq < u8 > ;
impl DeepView for Head {
    type V = HeadSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        HeadSpec {
            magic: self.magic.deep_view(),
        }
    }
}
impl Head {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().magic == self.magic.deep_view(),
    {
        reveal(< Head as DeepView>::deep_view) ;
    }
}
impl < T0 > HeadSpec < T0 > {
    # [verifier::opaque] pub open spec fn from_structural (input: T0) -> Self {
        let magic = input ;
        Self {
            magic
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> T0 {
        let Self {
            magic
        }
        = self ;
        magic
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(HeadSpec::from_structural) ;
        reveal(HeadSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: T0) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(HeadSpec::from_structural) ;
        reveal(HeadSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            magic
        }
        => magic,
    }
   ,
    {
        reveal(HeadSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct HeadForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct HeadReverse ;
impl SpecMap for HeadForward {
    type Input = HeadInner ;
    type Output = HeadSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        HeadSpec::from_structural (input)
    }
}
impl SpecMap for HeadReverse {
    type Input = HeadSpec ;
    type Output = HeadInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `tail`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Tail {
    pub metadata_len: u32,
    pub magic: [u8 ;
    4],
}
# [verifier::ext_equal]
pub struct TailSpec < T0 = u32, T1 = Seq < u8 > > {
    pub metadata_len: T0,
    pub magic: T1,
}
pub type TailInner = (u32, Seq < u8 >) ;
impl DeepView for Tail {
    type V = TailSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        TailSpec {
            metadata_len: self.metadata_len.deep_view(),
            magic: self.magic.deep_view(),
        }
    }
}
impl Tail {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().metadata_len == self.metadata_len.deep_view(),
    self.deep_view().magic == self.magic.deep_view(),
    {
        reveal(< Tail as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > TailSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    T1)) -> Self {
        let (metadata_len,
        magic) = input ;
        Self {
            metadata_len,
            magic
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    T1) {
        let Self {
            metadata_len,
            magic
        }
        = self ;
        (metadata_len,
        magic)
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(TailSpec::from_structural) ;
        reveal(TailSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    T1)) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(TailSpec::from_structural) ;
        reveal(TailSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            metadata_len,
            magic
        }
        => (metadata_len,
        magic),
    }
   ,
    {
        reveal(TailSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct TailForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct TailReverse ;
impl SpecMap for TailForward {
    type Input = TailInner ;
    type Output = TailSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        TailSpec::from_structural (input)
    }
}
impl SpecMap for TailReverse {
    type Input = TailSpec ;
    type Output = TailInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `head`."]
# [derive (Clone, Copy)]
pub struct HeadFmt ;

pub type HeadFmtSpec = Named < Mapped < Const < Fixed < 4 >, [u8 ;
4] >, BiMap < HeadForward, HeadReverse >> > ;

impl HeadFmt {
    # [doc = "specification constructor for `head`."] pub open spec fn spec_inner() -> HeadFmtSpec {
        Named ("head",
        Mapped {
            inner: Const (Fixed::< 4 >,
            [0x50u8, 0x41u8, 0x52u8, 0x31u8]),
            mapper: BiMap (HeadForward,
            HeadReverse),
        }
        )
    }
}


# [doc = "named format combinator for `tail`."]
# [derive (Clone, Copy)]
pub struct TailFmt ;

pub type TailFmtSpec = Named < Mapped < Pair < U32Le, Const < Fixed < 4 >, [u8 ;
4] > >, BiMap < TailForward, TailReverse >> > ;

impl TailFmt {
    # [doc = "specification constructor for `tail`."] pub open spec fn spec_inner() -> TailFmtSpec {
        Named ("tail",
        Mapped {
            inner: Pair (U32Le,
            Const (Fixed::< 4 >,
            [0x50u8, 0x41u8, 0x52u8, 0x31u8])),
            mapper: BiMap (TailForward,
            TailReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for HeadFmt {
        type PVal = HeadSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for HeadFmt {
        type Val = HeadSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for HeadFmt {
        type SValue = HeadSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for HeadFmt {
        type SVal = HeadSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for HeadFmt {
        type T = HeadSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for TailFmt {
        type PVal = TailSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for TailFmt {
        type Val = TailSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for TailFmt {
        type SValue = TailSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for TailFmt {
        type SVal = TailSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for TailFmt {
        type T = TailSpec ;
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
        HeadSpec::lemma_from_into,
        HeadSpec::lemma_into_from,
        TailSpec::lemma_from_into,
        TailSpec::lemma_into_from,
    };

    impl SafeParser for HeadFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< HeadFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for HeadFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< HeadFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for HeadFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< HeadFmt as SpecParser>::spec_parse) ;
            reveal(< HeadFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: HeadInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                HeadSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< HeadFmt as SpecParser>::spec_parse) ;
            reveal(< HeadFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: HeadInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                HeadSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for HeadFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< HeadFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< HeadFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< HeadFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for HeadFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< HeadFmt as SpecSerializer>::spec_serialize) ;
            reveal(< HeadFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for HeadFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< HeadFmt as SpecParser>::spec_parse) ;
            reveal(< HeadFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< HeadFmt as Consistency>::consistent) ;
            reveal(< HeadFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: HeadSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                HeadSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for HeadFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< HeadFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: HeadInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                HeadSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for HeadFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< HeadFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< HeadFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for HeadFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< HeadFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< HeadFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for TailFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< TailFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for TailFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< TailFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for TailFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< TailFmt as SpecParser>::spec_parse) ;
            reveal(< TailFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: TailInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                TailSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< TailFmt as SpecParser>::spec_parse) ;
            reveal(< TailFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: TailInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                TailSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for TailFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< TailFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< TailFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< TailFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for TailFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< TailFmt as SpecSerializer>::spec_serialize) ;
            reveal(< TailFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for TailFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< TailFmt as SpecParser>::spec_parse) ;
            reveal(< TailFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< TailFmt as Consistency>::consistent) ;
            reveal(< TailFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: TailSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                TailSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for TailFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< TailFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: TailInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                TailSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for TailFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< TailFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< TailFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for TailFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< TailFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< TailFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for HeadFmt {
        type PT = Head;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<HeadFmt as SpecParser>::spec_parse);
            reveal(<Head as DeepView>::deep_view);
            reveal(HeadSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, magic) = Const (Fixed::< 4 >, [0x50, 0x41, 0x52, 0x31]).parse(&rest)?;
            let rest = rest.skip(n1);
            let total_n = n1;
            let final_v = Head {
                magic,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Head> for HeadFmt {
        fn serialize_into(&self, v: &Head, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<HeadFmt as SpecSerializer>::spec_serialize);
            reveal(<HeadFmt as SpecByteLen>::byte_len);
            reveal(<Head as DeepView>::deep_view);
            reveal(HeadSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Head {
                magic,
            } = v;
            Fixed::< 4 >.serialize_into(magic, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Head> for HeadFmt {
        fn prepare(&self, v: &Head) -> Result<usize, PreSerializeError> {
            reveal(<HeadFmt as SpecByteLen>::byte_len);
            reveal(<Head as DeepView>::deep_view);
            reveal(HeadSpec::into_structural);
            let Head {
                magic,
            } = v;
            let l1 = (Const (Fixed::< 4 >, [0x50, 0x41, 0x52, 0x31])).prepare (magic) ?;
            let total_len = l1;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for TailFmt {
        type PT = Tail;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<TailFmt as SpecParser>::spec_parse);
            reveal(<Tail as DeepView>::deep_view);
            reveal(TailSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, metadata_len) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, magic) = Const (Fixed::< 4 >, [0x50, 0x41, 0x52, 0x31]).parse(&rest)?;
            let rest = rest.skip(n2);
            let total_n = n1 + n2;
            let final_v = Tail {
                metadata_len,
                magic,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Tail> for TailFmt {
        fn serialize_into(&self, v: &Tail, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<TailFmt as SpecSerializer>::spec_serialize);
            reveal(<TailFmt as SpecByteLen>::byte_len);
            reveal(<Tail as DeepView>::deep_view);
            reveal(TailSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Tail {
                metadata_len,
                magic,
            } = v;
            U32Le.serialize_into(metadata_len, obuf);
            Fixed::< 4 >.serialize_into(magic, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Tail> for TailFmt {
        fn prepare(&self, v: &Tail) -> Result<usize, PreSerializeError> {
            reveal(<TailFmt as SpecByteLen>::byte_len);
            reveal(<Tail as DeepView>::deep_view);
            reveal(TailSpec::into_structural);
            let Tail {
                metadata_len,
                magic,
            } = v;
            let l1 = (U32Le).prepare (metadata_len) ?;
            let l2 = (Const (Fixed::< 4 >, [0x50, 0x41, 0x52, 0x31])).prepare (magic) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
