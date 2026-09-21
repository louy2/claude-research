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
# [doc = "data type for `posix_header`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct PosixHeader<'i> {
    pub name: &'i [u8],
    pub mode: &'i [u8],
    pub uid: &'i [u8],
    pub gid: &'i [u8],
    pub size: &'i [u8],
    pub mtime: &'i [u8],
    pub checksum: &'i [u8],
    pub typeflag: u8,
    pub linkname: &'i [u8],
    pub magic: &'i [u8],
    pub version: &'i [u8],
    pub uname: &'i [u8],
    pub gname: &'i [u8],
    pub devmajor: &'i [u8],
    pub devminor: &'i [u8],
    pub prefix: &'i [u8],
    pub _pad1: &'i [u8],
}
# [verifier::ext_equal]
pub struct PosixHeaderSpec < T0 = Seq < u8 >, T1 = Seq < u8 >, T2 = Seq < u8 >, T3 = Seq < u8 >, T4 = Seq < u8 >, T5 = Seq < u8 >, T6 = Seq < u8 >, T7 = u8, T8 = Seq < u8 >, T9 = Seq < u8 >, T10 = Seq < u8 >, T11 = Seq < u8 >, T12 = Seq < u8 >, T13 = Seq < u8 >, T14 = Seq < u8 >, T15 = Seq < u8 >, T16 = Seq < u8 > > {
    pub name: T0,
    pub mode: T1,
    pub uid: T2,
    pub gid: T3,
    pub size: T4,
    pub mtime: T5,
    pub checksum: T6,
    pub typeflag: T7,
    pub linkname: T8,
    pub magic: T9,
    pub version: T10,
    pub uname: T11,
    pub gname: T12,
    pub devmajor: T13,
    pub devminor: T14,
    pub prefix: T15,
    pub _pad1: T16,
}
pub type PosixHeaderInner = (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (u8, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, (Seq < u8 >, Seq < u8 >)))))))))))))))) ;
impl<'i> DeepView for PosixHeader<'i> {
    type V = PosixHeaderSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        PosixHeaderSpec {
            name: self.name.deep_view(),
            mode: self.mode.deep_view(),
            uid: self.uid.deep_view(),
            gid: self.gid.deep_view(),
            size: self.size.deep_view(),
            mtime: self.mtime.deep_view(),
            checksum: self.checksum.deep_view(),
            typeflag: self.typeflag.deep_view(),
            linkname: self.linkname.deep_view(),
            magic: self.magic.deep_view(),
            version: self.version.deep_view(),
            uname: self.uname.deep_view(),
            gname: self.gname.deep_view(),
            devmajor: self.devmajor.deep_view(),
            devminor: self.devminor.deep_view(),
            prefix: self.prefix.deep_view(),
            _pad1: self._pad1.deep_view(),
        }
    }
}
impl<'i> PosixHeader<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().name == self.name.deep_view(),
    self.deep_view().mode == self.mode.deep_view(),
    self.deep_view().uid == self.uid.deep_view(),
    self.deep_view().gid == self.gid.deep_view(),
    self.deep_view().size == self.size.deep_view(),
    self.deep_view().mtime == self.mtime.deep_view(),
    self.deep_view().checksum == self.checksum.deep_view(),
    self.deep_view().typeflag == self.typeflag.deep_view(),
    self.deep_view().linkname == self.linkname.deep_view(),
    self.deep_view().magic == self.magic.deep_view(),
    self.deep_view().version == self.version.deep_view(),
    self.deep_view().uname == self.uname.deep_view(),
    self.deep_view().gname == self.gname.deep_view(),
    self.deep_view().devmajor == self.devmajor.deep_view(),
    self.deep_view().devminor == self.devminor.deep_view(),
    self.deep_view().prefix == self.prefix.deep_view(),
    self.deep_view()._pad1 == self._pad1.deep_view(),
    {
        reveal(< PosixHeader as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16 > PosixHeaderSpec < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16 > {
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
    (T13,
    (T14,
    (T15,
    T16))))))))))))))))) -> Self {
        let (name,
        (mode,
        (uid,
        (gid,
        (size,
        (mtime,
        (checksum,
        (typeflag,
        (linkname,
        (magic,
        (version,
        (uname,
        (gname,
        (devmajor,
        (devminor,
        (prefix,
        _pad1)))))))))))))))) = input ;
        Self {
            name,
            mode,
            uid,
            gid,
            size,
            mtime,
            checksum,
            typeflag,
            linkname,
            magic,
            version,
            uname,
            gname,
            devmajor,
            devminor,
            prefix,
            _pad1
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
    (T13,
    (T14,
    (T15,
    T16)))))))))))))))) {
        let Self {
            name,
            mode,
            uid,
            gid,
            size,
            mtime,
            checksum,
            typeflag,
            linkname,
            magic,
            version,
            uname,
            gname,
            devmajor,
            devminor,
            prefix,
            _pad1
        }
        = self ;
        (name,
        (mode,
        (uid,
        (gid,
        (size,
        (mtime,
        (checksum,
        (typeflag,
        (linkname,
        (magic,
        (version,
        (uname,
        (gname,
        (devmajor,
        (devminor,
        (prefix,
        _pad1))))))))))))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(PosixHeaderSpec::from_structural) ;
        reveal(PosixHeaderSpec::into_structural) ;
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
    (T13,
    (T14,
    (T15,
    T16))))))))))))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(PosixHeaderSpec::from_structural) ;
        reveal(PosixHeaderSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            name,
            mode,
            uid,
            gid,
            size,
            mtime,
            checksum,
            typeflag,
            linkname,
            magic,
            version,
            uname,
            gname,
            devmajor,
            devminor,
            prefix,
            _pad1
        }
        => (name,
        (mode,
        (uid,
        (gid,
        (size,
        (mtime,
        (checksum,
        (typeflag,
        (linkname,
        (magic,
        (version,
        (uname,
        (gname,
        (devmajor,
        (devminor,
        (prefix,
        _pad1)))))))))))))))),
    }
   ,
    {
        reveal(PosixHeaderSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct PosixHeaderForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct PosixHeaderReverse ;
impl SpecMap for PosixHeaderForward {
    type Input = PosixHeaderInner ;
    type Output = PosixHeaderSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        PosixHeaderSpec::from_structural (input)
    }
}
impl SpecMap for PosixHeaderReverse {
    type Input = PosixHeaderSpec ;
    type Output = PosixHeaderInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `posix_header`."]
# [derive (Clone, Copy)]
pub struct PosixHeaderFmt ;

pub type PosixHeaderFmtSpec = Named < Mapped < Pair < Fixed < 100 >, Pair < Fixed < 8 >, Pair < Fixed < 8 >, Pair < Fixed < 8 >, Pair < Fixed < 12 >, Pair < Fixed < 12 >, Pair < Fixed < 8 >, Pair < U8, Pair < Fixed < 100 >, Pair < Fixed < 6 >, Pair < Fixed < 2 >, Pair < Fixed < 32 >, Pair < Fixed < 32 >, Pair < Fixed < 8 >, Pair < Fixed < 8 >, Pair < Fixed < 155 >, Fixed < 12 > > > > > > > > > > > > > > > > >, BiMap < PosixHeaderForward, PosixHeaderReverse >> > ;

impl PosixHeaderFmt {
    # [doc = "specification constructor for `posix_header`."] pub open spec fn spec_inner() -> PosixHeaderFmtSpec {
        Named ("posix_header",
        Mapped {
            inner: Pair (Fixed::< 100 >,
            Pair (Fixed::< 8 >,
            Pair (Fixed::< 8 >,
            Pair (Fixed::< 8 >,
            Pair (Fixed::< 12 >,
            Pair (Fixed::< 12 >,
            Pair (Fixed::< 8 >,
            Pair (U8,
            Pair (Fixed::< 100 >,
            Pair (Fixed::< 6 >,
            Pair (Fixed::< 2 >,
            Pair (Fixed::< 32 >,
            Pair (Fixed::< 32 >,
            Pair (Fixed::< 8 >,
            Pair (Fixed::< 8 >,
            Pair (Fixed::< 155 >,
            Fixed::< 12 >)))))))))))))))),
            mapper: BiMap (PosixHeaderForward,
            PosixHeaderReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for PosixHeaderFmt {
        type PVal = PosixHeaderSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for PosixHeaderFmt {
        type Val = PosixHeaderSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for PosixHeaderFmt {
        type SValue = PosixHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for PosixHeaderFmt {
        type SVal = PosixHeaderSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for PosixHeaderFmt {
        type T = PosixHeaderSpec ;
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
        PosixHeaderSpec::lemma_from_into,
        PosixHeaderSpec::lemma_into_from,
    };

    impl SafeParser for PosixHeaderFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for PosixHeaderFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for PosixHeaderFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< PosixHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: PosixHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                PosixHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< PosixHeaderFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: PosixHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                PosixHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for PosixHeaderFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< PosixHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for PosixHeaderFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< PosixHeaderFmt as SpecSerializer>::spec_serialize) ;
            reveal(< PosixHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for PosixHeaderFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecParser>::spec_parse) ;
            reveal(< PosixHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< PosixHeaderFmt as Consistency>::consistent) ;
            reveal(< PosixHeaderFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: PosixHeaderSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                PosixHeaderSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for PosixHeaderFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: PosixHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                PosixHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for PosixHeaderFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< PosixHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< PosixHeaderFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for PosixHeaderFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< PosixHeaderFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< PosixHeaderFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for PosixHeaderFmt {
        type PT = PosixHeader<'i>;

        #[verifier::spinoff_prover]
        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<PosixHeaderFmt as SpecParser>::spec_parse);
            reveal(<PosixHeader as DeepView>::deep_view);
            reveal(PosixHeaderSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, name) = (Fixed::< 100 >).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, mode) = (Fixed::< 8 >).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, uid) = (Fixed::< 8 >).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, gid) = (Fixed::< 8 >).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, size) = (Fixed::< 12 >).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, mtime) = (Fixed::< 12 >).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, checksum) = (Fixed::< 8 >).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, typeflag) = (U8).parse (& rest) ?;
            let rest = rest.skip(n8);
            let (n9, linkname) = (Fixed::< 100 >).parse (& rest) ?;
            let rest = rest.skip(n9);
            let (n10, magic) = (Fixed::< 6 >).parse (& rest) ?;
            let rest = rest.skip(n10);
            let (n11, version) = (Fixed::< 2 >).parse (& rest) ?;
            let rest = rest.skip(n11);
            let (n12, uname) = (Fixed::< 32 >).parse (& rest) ?;
            let rest = rest.skip(n12);
            let (n13, gname) = (Fixed::< 32 >).parse (& rest) ?;
            let rest = rest.skip(n13);
            let (n14, devmajor) = (Fixed::< 8 >).parse (& rest) ?;
            let rest = rest.skip(n14);
            let (n15, devminor) = (Fixed::< 8 >).parse (& rest) ?;
            let rest = rest.skip(n15);
            let (n16, prefix) = (Fixed::< 155 >).parse (& rest) ?;
            let rest = rest.skip(n16);
            let (n17, _pad1) = (Fixed::< 12 >).parse (& rest) ?;
            let rest = rest.skip(n17);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9 + n10 + n11 + n12 + n13 + n14 + n15 + n16 + n17;
            let final_v = PosixHeader {
                name,
                mode,
                uid,
                gid,
                size,
                mtime,
                checksum,
                typeflag,
                linkname,
                magic,
                version,
                uname,
                gname,
                devmajor,
                devminor,
                prefix,
                _pad1,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, PosixHeader<'i>> for PosixHeaderFmt {
        #[verifier::spinoff_prover]
        fn serialize_into(&self, v: &PosixHeader<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<PosixHeaderFmt as SpecSerializer>::spec_serialize);
            reveal(<PosixHeaderFmt as SpecByteLen>::byte_len);
            reveal(<PosixHeader as DeepView>::deep_view);
            reveal(PosixHeaderSpec::into_structural);
            let ghost old_obuf = obuf@;

            let PosixHeader {
                name,
                mode,
                uid,
                gid,
                size,
                mtime,
                checksum,
                typeflag,
                linkname,
                magic,
                version,
                uname,
                gname,
                devmajor,
                devminor,
                prefix,
                _pad1,
            } = v;
            Fixed::< 100 >.serialize_into(* name, obuf);
            Fixed::< 8 >.serialize_into(* mode, obuf);
            Fixed::< 8 >.serialize_into(* uid, obuf);
            Fixed::< 8 >.serialize_into(* gid, obuf);
            Fixed::< 12 >.serialize_into(* size, obuf);
            Fixed::< 12 >.serialize_into(* mtime, obuf);
            Fixed::< 8 >.serialize_into(* checksum, obuf);
            U8.serialize_into(typeflag, obuf);
            Fixed::< 100 >.serialize_into(* linkname, obuf);
            Fixed::< 6 >.serialize_into(* magic, obuf);
            Fixed::< 2 >.serialize_into(*version, obuf);
            Fixed::< 32 >.serialize_into(* uname, obuf);
            Fixed::< 32 >.serialize_into(* gname, obuf);
            Fixed::< 8 >.serialize_into(* devmajor, obuf);
            Fixed::< 8 >.serialize_into(* devminor, obuf);
            Fixed::< 155 >.serialize_into(* prefix, obuf);
            Fixed::< 12 >.serialize_into(* _pad1, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<PosixHeader<'i>> for PosixHeaderFmt {
        #[verifier::spinoff_prover]
        fn prepare(&self, v: &PosixHeader<'i>) -> Result<usize, PreSerializeError> {
            reveal(<PosixHeaderFmt as SpecByteLen>::byte_len);
            reveal(<PosixHeader as DeepView>::deep_view);
            reveal(PosixHeaderSpec::into_structural);
            let PosixHeader {
                name,
                mode,
                uid,
                gid,
                size,
                mtime,
                checksum,
                typeflag,
                linkname,
                magic,
                version,
                uname,
                gname,
                devmajor,
                devminor,
                prefix,
                _pad1,
            } = v;
            let l1 = (Fixed::< 100 >).prepare (name) ?;
            let l2 = (Fixed::< 8 >).prepare (mode) ?;
            let l3 = (Fixed::< 8 >).prepare (uid) ?;
            let l4 = (Fixed::< 8 >).prepare (gid) ?;
            let l5 = (Fixed::< 12 >).prepare (size) ?;
            let l6 = (Fixed::< 12 >).prepare (mtime) ?;
            let l7 = (Fixed::< 8 >).prepare (checksum) ?;
            let l8 = (U8).prepare (typeflag) ?;
            let l9 = (Fixed::< 100 >).prepare (linkname) ?;
            let l10 = (Fixed::< 6 >).prepare (magic) ?;
            let l11 = (Fixed::< 2 >).prepare (version) ?;
            let l12 = (Fixed::< 32 >).prepare (uname) ?;
            let l13 = (Fixed::< 32 >).prepare (gname) ?;
            let l14 = (Fixed::< 8 >).prepare (devmajor) ?;
            let l15 = (Fixed::< 8 >).prepare (devminor) ?;
            let l16 = (Fixed::< 155 >).prepare (prefix) ?;
            let l17 = (Fixed::< 12 >).prepare (_pad1) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l9).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l10).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l11).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l12).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l13).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l14).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l15).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l16).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l17).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
