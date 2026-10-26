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
# [doc = "data type for `eocd`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Eocd<'i> {
    pub signature: [u8 ;
    4],
    pub disk: u16,
    pub cd_disk: u16,
    pub entries_disk: u16,
    pub entries_total: u16,
    pub cd_size: u32,
    pub cd_offset: u32,
    pub comment_len: u16,
    pub comment: &'i [u8],
}
# [verifier::ext_equal]
pub struct EocdSpec < T0 = Seq < u8 >, T1 = u16, T2 = u16, T3 = u16, T4 = u16, T5 = u32, T6 = u32, T7 = u16, T8 = Seq < u8 > > {
    pub signature: T0,
    pub disk: T1,
    pub cd_disk: T2,
    pub entries_disk: T3,
    pub entries_total: T4,
    pub cd_size: T5,
    pub cd_offset: T6,
    pub comment_len: T7,
    pub comment: T8,
}
pub type EocdInner = (Seq < u8 >, (u16, (u16, (u16, (u16, (u32, (u32, (u16, Seq < u8 >)))))))) ;
impl<'i> DeepView for Eocd<'i> {
    type V = EocdSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        EocdSpec {
            signature: self.signature.deep_view(),
            disk: self.disk.deep_view(),
            cd_disk: self.cd_disk.deep_view(),
            entries_disk: self.entries_disk.deep_view(),
            entries_total: self.entries_total.deep_view(),
            cd_size: self.cd_size.deep_view(),
            cd_offset: self.cd_offset.deep_view(),
            comment_len: self.comment_len.deep_view(),
            comment: self.comment.deep_view(),
        }
    }
}
impl<'i> Eocd<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().signature == self.signature.deep_view(),
    self.deep_view().disk == self.disk.deep_view(),
    self.deep_view().cd_disk == self.cd_disk.deep_view(),
    self.deep_view().entries_disk == self.entries_disk.deep_view(),
    self.deep_view().entries_total == self.entries_total.deep_view(),
    self.deep_view().cd_size == self.cd_size.deep_view(),
    self.deep_view().cd_offset == self.cd_offset.deep_view(),
    self.deep_view().comment_len == self.comment_len.deep_view(),
    self.deep_view().comment == self.comment.deep_view(),
    {
        reveal(< Eocd as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7, T8 > EocdSpec < T0, T1, T2, T3, T4, T5, T6, T7, T8 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    (T7,
    T8))))))))) -> Self {
        let (signature,
        (disk,
        (cd_disk,
        (entries_disk,
        (entries_total,
        (cd_size,
        (cd_offset,
        (comment_len,
        comment)))))))) = input ;
        Self {
            signature,
            disk,
            cd_disk,
            entries_disk,
            entries_total,
            cd_size,
            cd_offset,
            comment_len,
            comment
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
    T8)))))))) {
        let Self {
            signature,
            disk,
            cd_disk,
            entries_disk,
            entries_total,
            cd_size,
            cd_offset,
            comment_len,
            comment
        }
        = self ;
        (signature,
        (disk,
        (cd_disk,
        (entries_disk,
        (entries_total,
        (cd_size,
        (cd_offset,
        (comment_len,
        comment))))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(EocdSpec::from_structural) ;
        reveal(EocdSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    (T7,
    T8))))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(EocdSpec::from_structural) ;
        reveal(EocdSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            signature,
            disk,
            cd_disk,
            entries_disk,
            entries_total,
            cd_size,
            cd_offset,
            comment_len,
            comment
        }
        => (signature,
        (disk,
        (cd_disk,
        (entries_disk,
        (entries_total,
        (cd_size,
        (cd_offset,
        (comment_len,
        comment)))))))),
    }
   ,
    {
        reveal(EocdSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct EocdForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct EocdReverse ;
impl SpecMap for EocdForward {
    type Input = EocdInner ;
    type Output = EocdSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        EocdSpec::from_structural (input)
    }
}
impl SpecMap for EocdReverse {
    type Input = EocdSpec ;
    type Output = EocdInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `flags`."]
pub type Flags = u16 ;
pub type FlagsSpec = u16 ;


# [doc = "data type for `method`."]
pub type Method = u16 ;
pub type MethodSpec = u16 ;


# [doc = "data type for `file_info`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct FileInfo {
    pub version_needed: u16,
    pub flags: Flags,
    pub method: Method,
    pub mod_time: u16,
    pub mod_date: u16,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}
# [verifier::ext_equal]
pub struct FileInfoSpec < T0 = u16, T1 = FlagsSpec, T2 = MethodSpec, T3 = u16, T4 = u16, T5 = u32, T6 = u32, T7 = u32 > {
    pub version_needed: T0,
    pub flags: T1,
    pub method: T2,
    pub mod_time: T3,
    pub mod_date: T4,
    pub crc32: T5,
    pub compressed_size: T6,
    pub uncompressed_size: T7,
}
pub type FileInfoInner = (u16, (FlagsSpec, (MethodSpec, (u16, (u16, (u32, (u32, u32))))))) ;
impl DeepView for FileInfo {
    type V = FileInfoSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        FileInfoSpec {
            version_needed: self.version_needed.deep_view(),
            flags: self.flags.deep_view(),
            method: self.method.deep_view(),
            mod_time: self.mod_time.deep_view(),
            mod_date: self.mod_date.deep_view(),
            crc32: self.crc32.deep_view(),
            compressed_size: self.compressed_size.deep_view(),
            uncompressed_size: self.uncompressed_size.deep_view(),
        }
    }
}
impl FileInfo {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().version_needed == self.version_needed.deep_view(),
    self.deep_view().flags == self.flags.deep_view(),
    self.deep_view().method == self.method.deep_view(),
    self.deep_view().mod_time == self.mod_time.deep_view(),
    self.deep_view().mod_date == self.mod_date.deep_view(),
    self.deep_view().crc32 == self.crc32.deep_view(),
    self.deep_view().compressed_size == self.compressed_size.deep_view(),
    self.deep_view().uncompressed_size == self.uncompressed_size.deep_view(),
    {
        reveal(< FileInfo as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7 > FileInfoSpec < T0, T1, T2, T3, T4, T5, T6, T7 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    T7)))))))) -> Self {
        let (version_needed,
        (flags,
        (method,
        (mod_time,
        (mod_date,
        (crc32,
        (compressed_size,
        uncompressed_size))))))) = input ;
        Self {
            version_needed,
            flags,
            method,
            mod_time,
            mod_date,
            crc32,
            compressed_size,
            uncompressed_size
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
            version_needed,
            flags,
            method,
            mod_time,
            mod_date,
            crc32,
            compressed_size,
            uncompressed_size
        }
        = self ;
        (version_needed,
        (flags,
        (method,
        (mod_time,
        (mod_date,
        (crc32,
        (compressed_size,
        uncompressed_size)))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(FileInfoSpec::from_structural) ;
        reveal(FileInfoSpec::into_structural) ;
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
        reveal(FileInfoSpec::from_structural) ;
        reveal(FileInfoSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            version_needed,
            flags,
            method,
            mod_time,
            mod_date,
            crc32,
            compressed_size,
            uncompressed_size
        }
        => (version_needed,
        (flags,
        (method,
        (mod_time,
        (mod_date,
        (crc32,
        (compressed_size,
        uncompressed_size))))))),
    }
   ,
    {
        reveal(FileInfoSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct FileInfoForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct FileInfoReverse ;
impl SpecMap for FileInfoForward {
    type Input = FileInfoInner ;
    type Output = FileInfoSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        FileInfoSpec::from_structural (input)
    }
}
impl SpecMap for FileInfoReverse {
    type Input = FileInfoSpec ;
    type Output = FileInfoInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `cd_attrs`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct CdAttrs {
    pub disk_start: u16,
    pub internal_attr: u16,
    pub external_attr: u32,
    pub local_header_offset: u32,
}
# [verifier::ext_equal]
pub struct CdAttrsSpec < T0 = u16, T1 = u16, T2 = u32, T3 = u32 > {
    pub disk_start: T0,
    pub internal_attr: T1,
    pub external_attr: T2,
    pub local_header_offset: T3,
}
pub type CdAttrsInner = (u16, (u16, (u32, u32))) ;
impl DeepView for CdAttrs {
    type V = CdAttrsSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        CdAttrsSpec {
            disk_start: self.disk_start.deep_view(),
            internal_attr: self.internal_attr.deep_view(),
            external_attr: self.external_attr.deep_view(),
            local_header_offset: self.local_header_offset.deep_view(),
        }
    }
}
impl CdAttrs {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().disk_start == self.disk_start.deep_view(),
    self.deep_view().internal_attr == self.internal_attr.deep_view(),
    self.deep_view().external_attr == self.external_attr.deep_view(),
    self.deep_view().local_header_offset == self.local_header_offset.deep_view(),
    {
        reveal(< CdAttrs as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3 > CdAttrsSpec < T0, T1, T2, T3 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    T3)))) -> Self {
        let (disk_start,
        (internal_attr,
        (external_attr,
        local_header_offset))) = input ;
        Self {
            disk_start,
            internal_attr,
            external_attr,
            local_header_offset
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    (T1,
    (T2,
    T3))) {
        let Self {
            disk_start,
            internal_attr,
            external_attr,
            local_header_offset
        }
        = self ;
        (disk_start,
        (internal_attr,
        (external_attr,
        local_header_offset)))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(CdAttrsSpec::from_structural) ;
        reveal(CdAttrsSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    T3)))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(CdAttrsSpec::from_structural) ;
        reveal(CdAttrsSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            disk_start,
            internal_attr,
            external_attr,
            local_header_offset
        }
        => (disk_start,
        (internal_attr,
        (external_attr,
        local_header_offset))),
    }
   ,
    {
        reveal(CdAttrsSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct CdAttrsForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct CdAttrsReverse ;
impl SpecMap for CdAttrsForward {
    type Input = CdAttrsInner ;
    type Output = CdAttrsSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        CdAttrsSpec::from_structural (input)
    }
}
impl SpecMap for CdAttrsReverse {
    type Input = CdAttrsSpec ;
    type Output = CdAttrsInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `cd_entry`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct CdEntry<'i> {
    pub signature: [u8 ;
    4],
    pub version_made_by: u16,
    pub info: FileInfo,
    pub name_len: u16,
    pub extra_len: u16,
    pub comment_len: u16,
    pub attrs: CdAttrs,
    pub name: &'i [u8],
    pub extra: &'i [u8],
    pub comment: &'i [u8],
}
# [verifier::ext_equal]
pub struct CdEntrySpec < T0 = Seq < u8 >, T1 = u16, T2 = FileInfoSpec, T3 = u16, T4 = u16, T5 = u16, T6 = CdAttrsSpec, T7 = Seq < u8 >, T8 = Seq < u8 >, T9 = Seq < u8 > > {
    pub signature: T0,
    pub version_made_by: T1,
    pub info: T2,
    pub name_len: T3,
    pub extra_len: T4,
    pub comment_len: T5,
    pub attrs: T6,
    pub name: T7,
    pub extra: T8,
    pub comment: T9,
}
pub type CdEntryInner = (Seq < u8 >, (u16, (FileInfoSpec, (u16, (u16, (u16, (CdAttrsSpec, (Seq < u8 >, (Seq < u8 >, Seq < u8 >))))))))) ;
impl<'i> DeepView for CdEntry<'i> {
    type V = CdEntrySpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        CdEntrySpec {
            signature: self.signature.deep_view(),
            version_made_by: self.version_made_by.deep_view(),
            info: self.info.deep_view(),
            name_len: self.name_len.deep_view(),
            extra_len: self.extra_len.deep_view(),
            comment_len: self.comment_len.deep_view(),
            attrs: self.attrs.deep_view(),
            name: self.name.deep_view(),
            extra: self.extra.deep_view(),
            comment: self.comment.deep_view(),
        }
    }
}
impl<'i> CdEntry<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().signature == self.signature.deep_view(),
    self.deep_view().version_made_by == self.version_made_by.deep_view(),
    self.deep_view().info == self.info.deep_view(),
    self.deep_view().name_len == self.name_len.deep_view(),
    self.deep_view().extra_len == self.extra_len.deep_view(),
    self.deep_view().comment_len == self.comment_len.deep_view(),
    self.deep_view().attrs == self.attrs.deep_view(),
    self.deep_view().name == self.name.deep_view(),
    self.deep_view().extra == self.extra.deep_view(),
    self.deep_view().comment == self.comment.deep_view(),
    {
        reveal(< CdEntry as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9 > CdEntrySpec < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    (T6,
    (T7,
    (T8,
    T9)))))))))) -> Self {
        let (signature,
        (version_made_by,
        (info,
        (name_len,
        (extra_len,
        (comment_len,
        (attrs,
        (name,
        (extra,
        comment))))))))) = input ;
        Self {
            signature,
            version_made_by,
            info,
            name_len,
            extra_len,
            comment_len,
            attrs,
            name,
            extra,
            comment
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
    T9))))))))) {
        let Self {
            signature,
            version_made_by,
            info,
            name_len,
            extra_len,
            comment_len,
            attrs,
            name,
            extra,
            comment
        }
        = self ;
        (signature,
        (version_made_by,
        (info,
        (name_len,
        (extra_len,
        (comment_len,
        (attrs,
        (name,
        (extra,
        comment)))))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(CdEntrySpec::from_structural) ;
        reveal(CdEntrySpec::into_structural) ;
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
    T9)))))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(CdEntrySpec::from_structural) ;
        reveal(CdEntrySpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            signature,
            version_made_by,
            info,
            name_len,
            extra_len,
            comment_len,
            attrs,
            name,
            extra,
            comment
        }
        => (signature,
        (version_made_by,
        (info,
        (name_len,
        (extra_len,
        (comment_len,
        (attrs,
        (name,
        (extra,
        comment))))))))),
    }
   ,
    {
        reveal(CdEntrySpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct CdEntryForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct CdEntryReverse ;
impl SpecMap for CdEntryForward {
    type Input = CdEntryInner ;
    type Output = CdEntrySpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        CdEntrySpec::from_structural (input)
    }
}
impl SpecMap for CdEntryReverse {
    type Input = CdEntrySpec ;
    type Output = CdEntryInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `central_directory`."]
# [derive (Debug, PartialEq, Eq, Clone)]
pub struct CentralDirectory<'i> {
    pub entries: Vec < CdEntry<'i> >,
}
# [verifier::ext_equal]
pub struct CentralDirectorySpec < T0 = Seq < CdEntrySpec > > {
    pub entries: T0,
}
pub type CentralDirectoryInner = Seq < CdEntrySpec > ;
impl<'i> DeepView for CentralDirectory<'i> {
    type V = CentralDirectorySpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        CentralDirectorySpec {
            entries: self.entries.deep_view(),
        }
    }
}
impl<'i> CentralDirectory<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().entries == self.entries.deep_view(),
    {
        reveal(< CentralDirectory as DeepView>::deep_view) ;
    }
}
impl < T0 > CentralDirectorySpec < T0 > {
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
        reveal(CentralDirectorySpec::from_structural) ;
        reveal(CentralDirectorySpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: T0) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(CentralDirectorySpec::from_structural) ;
        reveal(CentralDirectorySpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            entries
        }
        => entries,
    }
   ,
    {
        reveal(CentralDirectorySpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct CentralDirectoryForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct CentralDirectoryReverse ;
impl SpecMap for CentralDirectoryForward {
    type Input = CentralDirectoryInner ;
    type Output = CentralDirectorySpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        CentralDirectorySpec::from_structural (input)
    }
}
impl SpecMap for CentralDirectoryReverse {
    type Input = CentralDirectorySpec ;
    type Output = CentralDirectoryInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `local_file`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct LocalFile<'i> {
    pub signature: [u8 ;
    4],
    pub info: FileInfo,
    pub name_len: u16,
    pub extra_len: u16,
    pub name: &'i [u8],
    pub extra: &'i [u8],
    pub data: &'i [u8],
}
# [verifier::ext_equal]
pub struct LocalFileSpec < T0 = Seq < u8 >, T1 = FileInfoSpec, T2 = u16, T3 = u16, T4 = Seq < u8 >, T5 = Seq < u8 >, T6 = Seq < u8 > > {
    pub signature: T0,
    pub info: T1,
    pub name_len: T2,
    pub extra_len: T3,
    pub name: T4,
    pub extra: T5,
    pub data: T6,
}
pub type LocalFileInner = (Seq < u8 >, (FileInfoSpec, (u16, (u16, (Seq < u8 >, (Seq < u8 >, Seq < u8 >)))))) ;
impl<'i> DeepView for LocalFile<'i> {
    type V = LocalFileSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        LocalFileSpec {
            signature: self.signature.deep_view(),
            info: self.info.deep_view(),
            name_len: self.name_len.deep_view(),
            extra_len: self.extra_len.deep_view(),
            name: self.name.deep_view(),
            extra: self.extra.deep_view(),
            data: self.data.deep_view(),
        }
    }
}
impl<'i> LocalFile<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().signature == self.signature.deep_view(),
    self.deep_view().info == self.info.deep_view(),
    self.deep_view().name_len == self.name_len.deep_view(),
    self.deep_view().extra_len == self.extra_len.deep_view(),
    self.deep_view().name == self.name.deep_view(),
    self.deep_view().extra == self.extra.deep_view(),
    self.deep_view().data == self.data.deep_view(),
    {
        reveal(< LocalFile as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6 > LocalFileSpec < T0, T1, T2, T3, T4, T5, T6 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6))))))) -> Self {
        let (signature,
        (info,
        (name_len,
        (extra_len,
        (name,
        (extra,
        data)))))) = input ;
        Self {
            signature,
            info,
            name_len,
            extra_len,
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
    T6)))))) {
        let Self {
            signature,
            info,
            name_len,
            extra_len,
            name,
            extra,
            data
        }
        = self ;
        (signature,
        (info,
        (name_len,
        (extra_len,
        (name,
        (extra,
        data))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(LocalFileSpec::from_structural) ;
        reveal(LocalFileSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(LocalFileSpec::from_structural) ;
        reveal(LocalFileSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            signature,
            info,
            name_len,
            extra_len,
            name,
            extra,
            data
        }
        => (signature,
        (info,
        (name_len,
        (extra_len,
        (name,
        (extra,
        data)))))),
    }
   ,
    {
        reveal(LocalFileSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct LocalFileForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct LocalFileReverse ;
impl SpecMap for LocalFileForward {
    type Input = LocalFileInner ;
    type Output = LocalFileSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        LocalFileSpec::from_structural (input)
    }
}
impl SpecMap for LocalFileReverse {
    type Input = LocalFileSpec ;
    type Output = LocalFileInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `eocd`."]
# [derive (Clone, Copy)]
pub struct EocdFmt ;

pub type EocdFmtSpec = Named < Mapped < Pair < Const < Fixed < 4 >, [u8 ;
4] >, Pair < Const < U16Le, u16 >, Pair < Const < U16Le, u16 >, Pair < U16Le, Pair < U16Le, Pair < U32Le, Pair < U32Le, Bind < U16Le, spec_fn (u16) -> Varied < u16 > > > > > > > > >, BiMap < EocdForward, EocdReverse >> > ;

impl EocdFmt {
    # [doc = "specification constructor for `eocd`."] pub open spec fn spec_inner() -> EocdFmtSpec {
        Named ("eocd",
        Mapped {
            inner: Pair (Const (Fixed::< 4 >,
            [0x50u8, 0x4bu8, 0x05u8, 0x06u8]),
            Pair (Const (U16Le,
            0),
            Pair (Const (U16Le,
            0),
            Pair (U16Le,
            Pair (U16Le,
            Pair (U32Le,
            Pair (U32Le,
            Bind (U16Le,
            | comment_len: u16 | Varied (comment_len))))))))),
            mapper: BiMap (EocdForward,
            EocdReverse),
        }
        )
    }
}


# [doc = "named format combinator for `flags`."]
# [derive (Clone, Copy)]
pub struct FlagsFmt ;

pub type FlagsFmtSpec = Named < Refined < U16Le, PredFnSpec < u16 >> > ;

impl FlagsFmt {
    # [doc = "specification constructor for `flags`."] pub open spec fn spec_inner() -> FlagsFmtSpec {
        Named ("flags",
        Refined (U16Le,
        | x: u16 | x == 0 || x == 2 || x == 4 || x == 6 || x == 2048 || x == 2050 || x == 2052 || x == 2054))
    }
}


# [doc = "named format combinator for `method`."]
# [derive (Clone, Copy)]
pub struct MethodFmt ;

pub type MethodFmtSpec = Named < Refined < U16Le, PredFnSpec < u16 >> > ;

impl MethodFmt {
    # [doc = "specification constructor for `method`."] pub open spec fn spec_inner() -> MethodFmtSpec {
        Named ("method",
        Refined (U16Le,
        | x: u16 | x == 0 || x == 8))
    }
}


# [doc = "named format combinator for `file_info`."]
# [derive (Clone, Copy)]
pub struct FileInfoFmt ;

pub type FileInfoFmtSpec = Named < Mapped < Pair < U16Le, Pair < FlagsFmt, Pair < MethodFmt, Pair < U16Le, Pair < U16Le, Pair < U32Le, Bind < U32Le, spec_fn (u32) -> U32Le > > > > > > >, BiMap < FileInfoForward, FileInfoReverse >> > ;

impl FileInfoFmt {
    # [doc = "specification constructor for `file_info`."] pub open spec fn spec_inner() -> FileInfoFmtSpec {
        Named ("file_info",
        Mapped {
            inner: Pair (U16Le,
            Pair (FlagsFmt,
            Pair (MethodFmt,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U32Le,
            Bind (U32Le,
            | compressed_size: u32 | U32Le))))))),
            mapper: BiMap (FileInfoForward,
            FileInfoReverse),
        }
        )
    }
}


# [doc = "named format combinator for `cd_attrs`."]
# [derive (Clone, Copy)]
pub struct CdAttrsFmt ;

pub type CdAttrsFmtSpec = Named < Mapped < Pair < Const < U16Le, u16 >, Pair < U16Le, Pair < U32Le, U32Le > > >, BiMap < CdAttrsForward, CdAttrsReverse >> > ;

impl CdAttrsFmt {
    # [doc = "specification constructor for `cd_attrs`."] pub open spec fn spec_inner() -> CdAttrsFmtSpec {
        Named ("cd_attrs",
        Mapped {
            inner: Pair (Const (U16Le,
            0),
            Pair (U16Le,
            Pair (U32Le,
            U32Le))),
            mapper: BiMap (CdAttrsForward,
            CdAttrsReverse),
        }
        )
    }
}


# [doc = "named format combinator for `cd_entry`."]
# [derive (Clone, Copy)]
pub struct CdEntryFmt ;

pub type CdEntryFmtSpec = Named < Mapped < Pair < Const < Fixed < 4 >, [u8 ;
4] >, Pair < U16Le, Pair < FileInfoFmt, Bind < U16Le, spec_fn (u16) -> Bind < U16Le, spec_fn (u16) -> Bind < U16Le, spec_fn (u16) -> Pair < CdAttrsFmt, Pair < Varied < u16 >, Pair < Varied < u16 >, Varied < u16 > > > > > > > > > >, BiMap < CdEntryForward, CdEntryReverse >> > ;

impl CdEntryFmt {
    # [doc = "specification constructor for `cd_entry`."] pub open spec fn spec_inner() -> CdEntryFmtSpec {
        Named ("cd_entry",
        Mapped {
            inner: Pair (Const (Fixed::< 4 >,
            [0x50u8, 0x4bu8, 0x01u8, 0x02u8]),
            Pair (U16Le,
            Pair (FileInfoFmt,
            Bind (U16Le,
            | name_len: u16 | Bind (U16Le,
            | extra_len: u16 | Bind (U16Le,
            | comment_len: u16 | Pair (CdAttrsFmt,
            Pair (Varied (name_len),
            Pair (Varied (extra_len),
            Varied (comment_len)))))))))),
            mapper: BiMap (CdEntryForward,
            CdEntryReverse),
        }
        )
    }
}


# [doc = "named format combinator for `central_directory`."]
# [derive (Clone, Copy)]
pub struct CentralDirectoryFmt ;

pub type CentralDirectoryFmtSpec = Named < Mapped < RepeatTillEnd < CdEntryFmt >, BiMap < CentralDirectoryForward, CentralDirectoryReverse >> > ;

impl CentralDirectoryFmt {
    # [doc = "specification constructor for `central_directory`."] pub open spec fn spec_inner() -> CentralDirectoryFmtSpec {
        Named ("central_directory",
        Mapped {
            inner: RepeatTillEnd (CdEntryFmt),
            mapper: BiMap (CentralDirectoryForward,
            CentralDirectoryReverse),
        }
        )
    }
}


# [doc = "named format combinator for `local_file`."]
# [derive (Clone, Copy)]
pub struct LocalFileFmt ;

pub type LocalFileFmtSpec = Named < Mapped < Pair < Const < Fixed < 4 >, [u8 ;
4] >, Bind < FileInfoFmt, spec_fn (FileInfoSpec) -> Bind < U16Le, spec_fn (u16) -> Bind < U16Le, spec_fn (u16) -> Pair < Varied < u16 >, Pair < Varied < u16 >, Varied < u32 > > > > > > >, BiMap < LocalFileForward, LocalFileReverse >> > ;

impl LocalFileFmt {
    # [doc = "specification constructor for `local_file`."] pub open spec fn spec_inner() -> LocalFileFmtSpec {
        Named ("local_file",
        Mapped {
            inner: Pair (Const (Fixed::< 4 >,
            [0x50u8, 0x4bu8, 0x03u8, 0x04u8]),
            Bind (FileInfoFmt,
            | info: FileInfoSpec | Bind (U16Le,
            | name_len: u16 | Bind (U16Le,
            | extra_len: u16 | Pair (Varied (name_len),
            Pair (Varied (extra_len),
            Varied (info.compressed_size))))))),
            mapper: BiMap (LocalFileForward,
            LocalFileReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for EocdFmt {
        type PVal = EocdSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for EocdFmt {
        type Val = EocdSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for EocdFmt {
        type SValue = EocdSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for EocdFmt {
        type SVal = EocdSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for EocdFmt {
        type T = EocdSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

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

    impl SpecParser for FileInfoFmt {
        type PVal = FileInfoSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for FileInfoFmt {
        type Val = FileInfoSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for FileInfoFmt {
        type SValue = FileInfoSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for FileInfoFmt {
        type SVal = FileInfoSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for FileInfoFmt {
        type T = FileInfoSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for CdAttrsFmt {
        type PVal = CdAttrsSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for CdAttrsFmt {
        type Val = CdAttrsSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for CdAttrsFmt {
        type SValue = CdAttrsSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for CdAttrsFmt {
        type SVal = CdAttrsSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for CdAttrsFmt {
        type T = CdAttrsSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for CdEntryFmt {
        type PVal = CdEntrySpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for CdEntryFmt {
        type Val = CdEntrySpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for CdEntryFmt {
        type SValue = CdEntrySpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for CdEntryFmt {
        type SVal = CdEntrySpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for CdEntryFmt {
        type T = CdEntrySpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for CentralDirectoryFmt {
        type PVal = CentralDirectorySpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for CentralDirectoryFmt {
        type Val = CentralDirectorySpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for CentralDirectoryFmt {
        type SValue = CentralDirectorySpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for CentralDirectoryFmt {
        type SVal = CentralDirectorySpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for CentralDirectoryFmt {
        type T = CentralDirectorySpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for LocalFileFmt {
        type PVal = LocalFileSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for LocalFileFmt {
        type Val = LocalFileSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for LocalFileFmt {
        type SValue = LocalFileSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for LocalFileFmt {
        type SVal = LocalFileSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for LocalFileFmt {
        type T = LocalFileSpec ;
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
        EocdSpec::lemma_from_into,
        EocdSpec::lemma_into_from,
        FileInfoSpec::lemma_from_into,
        FileInfoSpec::lemma_into_from,
        CdAttrsSpec::lemma_from_into,
        CdAttrsSpec::lemma_into_from,
        CdEntrySpec::lemma_from_into,
        CdEntrySpec::lemma_into_from,
        CentralDirectorySpec::lemma_from_into,
        CentralDirectorySpec::lemma_into_from,
        LocalFileSpec::lemma_from_into,
        LocalFileSpec::lemma_into_from,
    };

    impl SafeParser for EocdFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< EocdFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for EocdFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< EocdFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for EocdFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< EocdFmt as SpecParser>::spec_parse) ;
            reveal(< EocdFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: EocdInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                EocdSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< EocdFmt as SpecParser>::spec_parse) ;
            reveal(< EocdFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: EocdInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                EocdSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for EocdFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< EocdFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< EocdFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< EocdFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for EocdFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< EocdFmt as SpecSerializer>::spec_serialize) ;
            reveal(< EocdFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for EocdFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< EocdFmt as SpecParser>::spec_parse) ;
            reveal(< EocdFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< EocdFmt as Consistency>::consistent) ;
            reveal(< EocdFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: EocdSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                EocdSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for EocdFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< EocdFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: EocdInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                EocdSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for EocdFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< EocdFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< EocdFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for EocdFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< EocdFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< EocdFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

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
            let fmt = Self::spec_inner() ;
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            reveal(< FlagsFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.sound_inv()) ;
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
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            reveal(< FlagsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FlagsFmt as Consistency>::consistent) ;
            reveal(< FlagsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for FlagsFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< FlagsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.nonmal_inv()) ;
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
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< MethodFmt as SpecParser>::spec_parse) ;
            reveal(< MethodFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
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

    impl SafeParser for FileInfoFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for FileInfoFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for FileInfoFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecParser>::spec_parse) ;
            reveal(< FileInfoFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FileInfoInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                FileInfoSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecParser>::spec_parse) ;
            reveal(< FileInfoFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FileInfoInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                FileInfoSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for FileInfoFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FileInfoFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for FileInfoFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< FileInfoFmt as SpecSerializer>::spec_serialize) ;
            reveal(< FileInfoFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for FileInfoFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecParser>::spec_parse) ;
            reveal(< FileInfoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FileInfoFmt as Consistency>::consistent) ;
            reveal(< FileInfoFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: FileInfoSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                FileInfoSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for FileInfoFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: FileInfoInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                FileInfoSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for FileInfoFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< FileInfoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FileInfoFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for FileInfoFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< FileInfoFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< FileInfoFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for CdAttrsFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for CdAttrsFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for CdAttrsFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecParser>::spec_parse) ;
            reveal(< CdAttrsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CdAttrsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CdAttrsSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecParser>::spec_parse) ;
            reveal(< CdAttrsFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CdAttrsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CdAttrsSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for CdAttrsFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CdAttrsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for CdAttrsFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< CdAttrsFmt as SpecSerializer>::spec_serialize) ;
            reveal(< CdAttrsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for CdAttrsFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecParser>::spec_parse) ;
            reveal(< CdAttrsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CdAttrsFmt as Consistency>::consistent) ;
            reveal(< CdAttrsFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: CdAttrsSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                CdAttrsSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for CdAttrsFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CdAttrsInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CdAttrsSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for CdAttrsFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< CdAttrsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CdAttrsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for CdAttrsFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< CdAttrsFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CdAttrsFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for CdEntryFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for CdEntryFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for CdEntryFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecParser>::spec_parse) ;
            reveal(< CdEntryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CdEntryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CdEntrySpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecParser>::spec_parse) ;
            reveal(< CdEntryFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CdEntryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CdEntrySpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for CdEntryFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CdEntryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for CdEntryFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< CdEntryFmt as SpecSerializer>::spec_serialize) ;
            reveal(< CdEntryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for CdEntryFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecParser>::spec_parse) ;
            reveal(< CdEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CdEntryFmt as Consistency>::consistent) ;
            reveal(< CdEntryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: CdEntrySpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                CdEntrySpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for CdEntryFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CdEntryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CdEntrySpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for CdEntryFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< CdEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CdEntryFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for CdEntryFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< CdEntryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CdEntryFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for CentralDirectoryFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< CentralDirectoryFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for CentralDirectoryFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< CentralDirectoryFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for CentralDirectoryFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< CentralDirectoryFmt as SpecParser>::spec_parse) ;
            reveal(< CentralDirectoryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CentralDirectoryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CentralDirectorySpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< CentralDirectoryFmt as SpecParser>::spec_parse) ;
            reveal(< CentralDirectoryFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CentralDirectoryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CentralDirectorySpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl GoodSerializer for CentralDirectoryFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< CentralDirectoryFmt as SpecSerializer>::spec_serialize) ;
            reveal(< CentralDirectoryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for CentralDirectoryFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< CentralDirectoryFmt as SpecParser>::spec_parse) ;
            reveal(< CentralDirectoryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CentralDirectoryFmt as Consistency>::consistent) ;
            reveal(< CentralDirectoryFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: CentralDirectorySpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                CentralDirectorySpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for CentralDirectoryFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< CentralDirectoryFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: CentralDirectoryInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                CentralDirectorySpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializers for CentralDirectoryFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< CentralDirectoryFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< CentralDirectoryFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for LocalFileFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for LocalFileFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for LocalFileFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecParser>::spec_parse) ;
            reveal(< LocalFileFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LocalFileInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LocalFileSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecParser>::spec_parse) ;
            reveal(< LocalFileFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LocalFileInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LocalFileSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for LocalFileFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LocalFileFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for LocalFileFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< LocalFileFmt as SpecSerializer>::spec_serialize) ;
            reveal(< LocalFileFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for LocalFileFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecParser>::spec_parse) ;
            reveal(< LocalFileFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LocalFileFmt as Consistency>::consistent) ;
            reveal(< LocalFileFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: LocalFileSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                LocalFileSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for LocalFileFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: LocalFileInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                LocalFileSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for LocalFileFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< LocalFileFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LocalFileFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for LocalFileFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< LocalFileFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< LocalFileFmt as SpecSerializer>::spec_serialize) ;
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

    impl<'i> Parser<&'i [u8]> for EocdFmt {
        type PT = Eocd<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<EocdFmt as SpecParser>::spec_parse);
            reveal(<Eocd as DeepView>::deep_view);
            reveal(EocdSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, signature) = Const (Fixed::< 4 >, [0x50, 0x4b, 0x05, 0x06])
            .parse(&rest)?;
            let rest = rest.skip(n1);
            let (n2, disk) = Const (U16Le, 0).parse(&rest)?;
            let rest = rest.skip(n2);
            let (n3, cd_disk) = Const (U16Le, 0).parse(&rest)?;
            let rest = rest.skip(n3);
            let (n4, entries_disk) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, entries_total) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, cd_size) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, cd_offset) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, comment_len) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n8);
            let (n9, comment) = (Varied (comment_len)).parse (& rest) ?;
            let rest = rest.skip(n9);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9;
            let final_v = Eocd {
                signature,
                disk,
                cd_disk,
                entries_disk,
                entries_total,
                cd_size,
                cd_offset,
                comment_len,
                comment,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Eocd<'i>> for EocdFmt {
        fn serialize_into(&self, v: &Eocd<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<EocdFmt as SpecSerializer>::spec_serialize);
            reveal(<EocdFmt as SpecByteLen>::byte_len);
            reveal(<Eocd as DeepView>::deep_view);
            reveal(EocdSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Eocd {
                signature,
                disk,
                cd_disk,
                entries_disk,
                entries_total,
                cd_size,
                cd_offset,
                comment_len,
                comment,
            } = v;
            Fixed::< 4 >.serialize_into(signature, obuf);
            U16Le.serialize_into(disk, obuf);
            U16Le.serialize_into(cd_disk, obuf);
            U16Le.serialize_into(entries_disk, obuf);
            U16Le.serialize_into(entries_total, obuf);
            U32Le.serialize_into(cd_size, obuf);
            U32Le.serialize_into(cd_offset, obuf);
            U16Le.serialize_into(comment_len, obuf);
            Varied (* comment_len).serialize_into(* comment, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Eocd<'i>> for EocdFmt {
        fn prepare(&self, v: &Eocd<'i>) -> Result<usize, PreSerializeError> {
            reveal(<EocdFmt as SpecByteLen>::byte_len);
            reveal(<Eocd as DeepView>::deep_view);
            reveal(EocdSpec::into_structural);
            let Eocd {
                signature,
                disk,
                cd_disk,
                entries_disk,
                entries_total,
                cd_size,
                cd_offset,
                comment_len,
                comment,
            } = v;
            let l1 = (Const (Fixed::< 4 >, [0x50, 0x4b, 0x05, 0x06])).prepare (signature) ?;
            let l2 = (Const (U16Le, 0)).prepare (disk) ?;
            let l3 = (Const (U16Le, 0)).prepare (cd_disk) ?;
            let l4 = (U16Le).prepare (entries_disk) ?;
            let l5 = (U16Le).prepare (entries_total) ?;
            let l6 = (U32Le).prepare (cd_size) ?;
            let l7 = (U32Le).prepare (cd_offset) ?;
            let l8 = (U16Le).prepare (comment_len) ?;
            let l9 = (Varied (* comment_len)).prepare (comment) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l9).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for FlagsFmt {
        type PT = Flags;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<FlagsFmt as SpecParser>::spec_parse);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U16Le.parse(ibuf)?;
            if !(v == 0 || v == 2 || v == 4 || v == 6 || v == 2048 || v == 2050 || v == 2052 || v == 2054) {
                return Err(ParseError::predicate_failed());
            }
            assert(self.spec_parse(ibuf@) == Some((n as int, v.deep_view())));
            Ok((n, v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Flags> for FlagsFmt {
        fn serialize_into(&self, v: &Flags, obuf: &mut Output) {
            reveal(<FlagsFmt as SpecSerializer>::spec_serialize);
            reveal(<FlagsFmt as SpecByteLen>::byte_len);
            let ghost old_obuf = obuf@;

            U16Le.serialize_into(v, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Flags> for FlagsFmt {
        fn prepare(&self, v: &Flags) -> Result<usize, PreSerializeError> {
            reveal(<FlagsFmt as SpecByteLen>::byte_len);
            {
        if ! (*v == 0 || *v == 2 || *v == 4 || *v == 6 || *v == 2048 || *v == 2050 || *v == 2052 || *v == 2054) {
            Err (PreSerializeError::not_compliant (ComplianceErrorKind::PredicateFailed))
        }
        else {
            (U16Le).prepare (v)
        }
    }
        }
    }



    impl<'i> Parser<&'i [u8]> for MethodFmt {
        type PT = Method;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<MethodFmt as SpecParser>::spec_parse);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U16Le.parse(ibuf)?;
            if !(v == 0 || v == 8) {
                return Err(ParseError::predicate_failed());
            }
            assert(self.spec_parse(ibuf@) == Some((n as int, v.deep_view())));
            Ok((n, v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Method> for MethodFmt {
        fn serialize_into(&self, v: &Method, obuf: &mut Output) {
            reveal(<MethodFmt as SpecSerializer>::spec_serialize);
            reveal(<MethodFmt as SpecByteLen>::byte_len);
            let ghost old_obuf = obuf@;

            U16Le.serialize_into(v, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Method> for MethodFmt {
        fn prepare(&self, v: &Method) -> Result<usize, PreSerializeError> {
            reveal(<MethodFmt as SpecByteLen>::byte_len);
            {
        if ! (*v == 0 || *v == 8) {
            Err (PreSerializeError::not_compliant (ComplianceErrorKind::PredicateFailed))
        }
        else {
            (U16Le).prepare (v)
        }
    }
        }
    }



    impl<'i> Parser<&'i [u8]> for FileInfoFmt {
        type PT = FileInfo;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<FileInfoFmt as SpecParser>::spec_parse);
            reveal(<FileInfo as DeepView>::deep_view);
            reveal(FileInfoSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, version_needed) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, flags) = (Named ("flags", FlagsFmt)).parse (& rest) ?;
            if !(flags == 0 || flags == 2 || flags == 4 || flags == 6 || flags == 2048 || flags == 2050 || flags == 2052 || flags == 2054) {
                return Err(ParseError::predicate_failed());
            }
            let rest = rest.skip(n2);
            let (n3, method) = (Named ("method", MethodFmt)).parse (& rest) ?;
            if !(method == 0 || method == 8) {
                return Err(ParseError::predicate_failed());
            }
            let rest = rest.skip(n3);
            let (n4, mod_time) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, mod_date) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, crc32) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, compressed_size) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, uncompressed_size) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n8);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8;
            let final_v = FileInfo {
                version_needed,
                flags,
                method,
                mod_time,
                mod_date,
                crc32,
                compressed_size,
                uncompressed_size,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, FileInfo> for FileInfoFmt {
        fn serialize_into(&self, v: &FileInfo, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<FileInfoFmt as SpecSerializer>::spec_serialize);
            reveal(<FileInfoFmt as SpecByteLen>::byte_len);
            reveal(<FileInfo as DeepView>::deep_view);
            reveal(FileInfoSpec::into_structural);
            let ghost old_obuf = obuf@;

            let FileInfo {
                version_needed,
                flags,
                method,
                mod_time,
                mod_date,
                crc32,
                compressed_size,
                uncompressed_size,
            } = v;
            U16Le.serialize_into(version_needed, obuf);
            FlagsFmt.serialize_into(flags, obuf);
            MethodFmt.serialize_into(method, obuf);
            U16Le.serialize_into(mod_time, obuf);
            U16Le.serialize_into(mod_date, obuf);
            U32Le.serialize_into(crc32, obuf);
            U32Le.serialize_into(compressed_size, obuf);
            U32Le.serialize_into(uncompressed_size, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<FileInfo> for FileInfoFmt {
        fn prepare(&self, v: &FileInfo) -> Result<usize, PreSerializeError> {
            reveal(<FileInfoFmt as SpecByteLen>::byte_len);
            reveal(<FileInfo as DeepView>::deep_view);
            reveal(FileInfoSpec::into_structural);
            let FileInfo {
                version_needed,
                flags,
                method,
                mod_time,
                mod_date,
                crc32,
                compressed_size,
                uncompressed_size,
            } = v;
            let l1 = (U16Le).prepare (version_needed) ?;
            let l2 = {
                if ! (* flags == 0 || * flags == 2 || * flags == 4 || * flags == 6 || * flags == 2048 || * flags == 2050 || * flags == 2052 || * flags == 2054) {
                    Err (PreSerializeError::not_compliant (ComplianceErrorKind::PredicateFailed))
                }
                else {
                    (Named ("flags",
                    FlagsFmt)).prepare (flags)
                }
            }
            ?;
            let l3 = {
                if ! (* method == 0 || * method == 8) {
                    Err (PreSerializeError::not_compliant (ComplianceErrorKind::PredicateFailed))
                }
                else {
                    (Named ("method",
                    MethodFmt)).prepare (method)
                }
            }
            ?;
            let l4 = (U16Le).prepare (mod_time) ?;
            let l5 = (U16Le).prepare (mod_date) ?;
            let l6 = (U32Le).prepare (crc32) ?;
            let l7 = (U32Le).prepare (compressed_size) ?;
            let l8 = (U32Le).prepare (uncompressed_size) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for CdAttrsFmt {
        type PT = CdAttrs;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<CdAttrsFmt as SpecParser>::spec_parse);
            reveal(<CdAttrs as DeepView>::deep_view);
            reveal(CdAttrsSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, disk_start) = Const (U16Le, 0).parse(&rest)?;
            let rest = rest.skip(n1);
            let (n2, internal_attr) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, external_attr) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, local_header_offset) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let total_n = n1 + n2 + n3 + n4;
            let final_v = CdAttrs {
                disk_start,
                internal_attr,
                external_attr,
                local_header_offset,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, CdAttrs> for CdAttrsFmt {
        fn serialize_into(&self, v: &CdAttrs, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<CdAttrsFmt as SpecSerializer>::spec_serialize);
            reveal(<CdAttrsFmt as SpecByteLen>::byte_len);
            reveal(<CdAttrs as DeepView>::deep_view);
            reveal(CdAttrsSpec::into_structural);
            let ghost old_obuf = obuf@;

            let CdAttrs {
                disk_start,
                internal_attr,
                external_attr,
                local_header_offset,
            } = v;
            U16Le.serialize_into(disk_start, obuf);
            U16Le.serialize_into(internal_attr, obuf);
            U32Le.serialize_into(external_attr, obuf);
            U32Le.serialize_into(local_header_offset, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<CdAttrs> for CdAttrsFmt {
        fn prepare(&self, v: &CdAttrs) -> Result<usize, PreSerializeError> {
            reveal(<CdAttrsFmt as SpecByteLen>::byte_len);
            reveal(<CdAttrs as DeepView>::deep_view);
            reveal(CdAttrsSpec::into_structural);
            let CdAttrs {
                disk_start,
                internal_attr,
                external_attr,
                local_header_offset,
            } = v;
            let l1 = (Const (U16Le, 0)).prepare (disk_start) ?;
            let l2 = (U16Le).prepare (internal_attr) ?;
            let l3 = (U32Le).prepare (external_attr) ?;
            let l4 = (U32Le).prepare (local_header_offset) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for CdEntryFmt {
        type PT = CdEntry<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<CdEntryFmt as SpecParser>::spec_parse);
            reveal(<CdEntry as DeepView>::deep_view);
            reveal(CdEntrySpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, signature) = Const (Fixed::< 4 >, [0x50, 0x4b, 0x01, 0x02])
            .parse(&rest)?;
            let rest = rest.skip(n1);
            let (n2, version_made_by) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, info) = (Named ("file_info", FileInfoFmt)).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, name_len) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, extra_len) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, comment_len) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, attrs) = (Named ("cd_attrs", CdAttrsFmt)).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, name) = (Varied (name_len)).parse (& rest) ?;
            let rest = rest.skip(n8);
            let (n9, extra) = (Varied (extra_len)).parse (& rest) ?;
            let rest = rest.skip(n9);
            let (n10, comment) = (Varied (comment_len)).parse (& rest) ?;
            let rest = rest.skip(n10);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9 + n10;
            let final_v = CdEntry {
                signature,
                version_made_by,
                info,
                name_len,
                extra_len,
                comment_len,
                attrs,
                name,
                extra,
                comment,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, CdEntry<'i>> for CdEntryFmt {
        fn serialize_into(&self, v: &CdEntry<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<CdEntryFmt as SpecSerializer>::spec_serialize);
            reveal(<CdEntryFmt as SpecByteLen>::byte_len);
            reveal(<CdEntry as DeepView>::deep_view);
            reveal(CdEntrySpec::into_structural);
            let ghost old_obuf = obuf@;

            let CdEntry {
                signature,
                version_made_by,
                info,
                name_len,
                extra_len,
                comment_len,
                attrs,
                name,
                extra,
                comment,
            } = v;
            Fixed::< 4 >.serialize_into(signature, obuf);
            U16Le.serialize_into(version_made_by, obuf);
            FileInfoFmt.serialize_into(info, obuf);
            U16Le.serialize_into(name_len, obuf);
            U16Le.serialize_into(extra_len, obuf);
            U16Le.serialize_into(comment_len, obuf);
            CdAttrsFmt.serialize_into(attrs, obuf);
            Varied (* name_len).serialize_into(* name, obuf);
            Varied (* extra_len).serialize_into(* extra, obuf);
            Varied (* comment_len).serialize_into(* comment, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<CdEntry<'i>> for CdEntryFmt {
        fn prepare(&self, v: &CdEntry<'i>) -> Result<usize, PreSerializeError> {
            reveal(<CdEntryFmt as SpecByteLen>::byte_len);
            reveal(<CdEntry as DeepView>::deep_view);
            reveal(CdEntrySpec::into_structural);
            let CdEntry {
                signature,
                version_made_by,
                info,
                name_len,
                extra_len,
                comment_len,
                attrs,
                name,
                extra,
                comment,
            } = v;
            let l1 = (Const (Fixed::< 4 >, [0x50, 0x4b, 0x01, 0x02])).prepare (signature) ?;
            let l2 = (U16Le).prepare (version_made_by) ?;
            let l3 = (Named ("file_info", FileInfoFmt)).prepare (info) ?;
            let l4 = (U16Le).prepare (name_len) ?;
            let l5 = (U16Le).prepare (extra_len) ?;
            let l6 = (U16Le).prepare (comment_len) ?;
            let l7 = (Named ("cd_attrs", CdAttrsFmt)).prepare (attrs) ?;
            let l8 = (Varied (* name_len)).prepare (name) ?;
            let l9 = (Varied (* extra_len)).prepare (extra) ?;
            let l10 = (Varied (* comment_len)).prepare (comment) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l9).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l10).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for CentralDirectoryFmt {
        type PT = CentralDirectory<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<CentralDirectoryFmt as SpecParser>::spec_parse);
            reveal(<CentralDirectory as DeepView>::deep_view);
            reveal(CentralDirectorySpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, entries) = (RepeatTillEnd (CdEntryFmt)).parse (& rest) ?;
            let rest = rest.skip(n1);
            let total_n = n1;
            let final_v = CentralDirectory {
                entries,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, CentralDirectory<'i>> for CentralDirectoryFmt {
        fn serialize_into(&self, v: &CentralDirectory<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<CentralDirectoryFmt as SpecSerializer>::spec_serialize);
            reveal(<CentralDirectoryFmt as SpecByteLen>::byte_len);
            reveal(<CentralDirectory as DeepView>::deep_view);
            reveal(CentralDirectorySpec::into_structural);
            let ghost old_obuf = obuf@;

            let CentralDirectory {
                entries,
            } = v;
            Star (CdEntryFmt).serialize_into(entries, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<CentralDirectory<'i>> for CentralDirectoryFmt {
        fn prepare(&self, v: &CentralDirectory<'i>) -> Result<usize, PreSerializeError> {
            reveal(<CentralDirectoryFmt as SpecByteLen>::byte_len);
            reveal(<CentralDirectory as DeepView>::deep_view);
            reveal(CentralDirectorySpec::into_structural);
            let CentralDirectory {
                entries,
            } = v;
            let l1 = (Star (CdEntryFmt)).prepare (entries) ?;
            let total_len = l1;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for LocalFileFmt {
        type PT = LocalFile<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<LocalFileFmt as SpecParser>::spec_parse);
            reveal(<LocalFile as DeepView>::deep_view);
            reveal(LocalFileSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, signature) = Const (Fixed::< 4 >, [0x50, 0x4b, 0x03, 0x04])
            .parse(&rest)?;
            let rest = rest.skip(n1);
            let (n2, info) = (Named ("file_info", FileInfoFmt)).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, name_len) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, extra_len) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, name) = (Varied (name_len)).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, extra) = (Varied (extra_len)).parse (& rest) ?;
            let rest = rest.skip(n6);
            proof {
                info.lemma_deep_view_fields();
                info.deep_view().lemma_into_structural_fields();
            }

            let (n7, data) = (Varied (info.compressed_size)).parse (& rest) ?;
            let rest = rest.skip(n7);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7;
            let final_v = LocalFile {
                signature,
                info,
                name_len,
                extra_len,
                name,
                extra,
                data,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, LocalFile<'i>> for LocalFileFmt {
        fn serialize_into(&self, v: &LocalFile<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<LocalFileFmt as SpecSerializer>::spec_serialize);
            reveal(<LocalFileFmt as SpecByteLen>::byte_len);
            reveal(<LocalFile as DeepView>::deep_view);
            reveal(LocalFileSpec::into_structural);
            let ghost old_obuf = obuf@;

            let LocalFile {
                signature,
                info,
                name_len,
                extra_len,
                name,
                extra,
                data,
            } = v;
            proof {
                info.lemma_deep_view_fields();
                info.deep_view().lemma_into_structural_fields();
            }

            Fixed::< 4 >.serialize_into(signature, obuf);
            FileInfoFmt.serialize_into(info, obuf);
            U16Le.serialize_into(name_len, obuf);
            U16Le.serialize_into(extra_len, obuf);
            Varied (* name_len).serialize_into(* name, obuf);
            Varied (* extra_len).serialize_into(* extra, obuf);
            Varied (info.compressed_size).serialize_into(* data, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<LocalFile<'i>> for LocalFileFmt {
        fn prepare(&self, v: &LocalFile<'i>) -> Result<usize, PreSerializeError> {
            reveal(<LocalFileFmt as SpecByteLen>::byte_len);
            reveal(<LocalFile as DeepView>::deep_view);
            reveal(LocalFileSpec::into_structural);
            let LocalFile {
                signature,
                info,
                name_len,
                extra_len,
                name,
                extra,
                data,
            } = v;
            proof {
                info.lemma_deep_view_fields();
                info.deep_view().lemma_into_structural_fields();
            }

            let l1 = (Const (Fixed::< 4 >, [0x50, 0x4b, 0x03, 0x04])).prepare (signature) ?;
            let l2 = (Named ("file_info", FileInfoFmt)).prepare (info) ?;
            let l3 = (U16Le).prepare (name_len) ?;
            let l4 = (U16Le).prepare (extra_len) ?;
            let l5 = (Varied (* name_len)).prepare (name) ?;
            let l6 = (Varied (* extra_len)).prepare (extra) ?;
            let l7 = (Varied (info.compressed_size)).prepare (data) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }

}
}
