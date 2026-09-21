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
# [doc = "data type for `class`."]
# [repr (u8)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum Class {
    None = 0,
    Elf32 = 1,
    Elf64 = 2,
    Unknown (u8),
}
pub type ClassSpec = Class ;
pub type ClassInner = Sum < u8, u8 > ;
impl DeepView for Class {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Class {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Class as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: ClassInner) -> bool {
        match input {
            L (x) => x == 0 || x == 1 || x == 2,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: ClassInner) -> Self {
        match input {
            L (x) => match x {
                0 => Self::None,
                1 => Self::Elf32,
                2 => Self::Elf64,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> ClassInner {
        match self {
            Self::None => L (0),
            Self::Elf32 => L (1),
            Self::Elf64 => L (2),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Class::from_structural) ;
        reveal(Class::into_structural) ;
        match self {
            Self::None => {
            }
           ,
            Self::Elf32 => {
            }
           ,
            Self::Elf64 => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: ClassInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Class::from_structural) ;
        reveal(Class::into_structural) ;
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
pub struct ClassForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ClassReverse ;
impl SpecMap for ClassForward {
    type Input = ClassInner ;
    type Output = ClassSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Class::from_structural (input)
    }
}
impl SpecMap for ClassReverse {
    type Input = ClassSpec ;
    type Output = ClassInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for Class {
}

# [doc = "data type for `data`."]
# [repr (u8)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum Data {
    None = 0,
    Lsb = 1,
    Msb = 2,
    Unknown (u8),
}
pub type DataSpec = Data ;
pub type DataInner = Sum < u8, u8 > ;
impl DeepView for Data {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl Data {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< Data as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: DataInner) -> bool {
        match input {
            L (x) => x == 0 || x == 1 || x == 2,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: DataInner) -> Self {
        match input {
            L (x) => match x {
                0 => Self::None,
                1 => Self::Lsb,
                2 => Self::Msb,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> DataInner {
        match self {
            Self::None => L (0),
            Self::Lsb => L (1),
            Self::Msb => L (2),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Data::from_structural) ;
        reveal(Data::into_structural) ;
        match self {
            Self::None => {
            }
           ,
            Self::Lsb => {
            }
           ,
            Self::Msb => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: DataInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Data::from_structural) ;
        reveal(Data::into_structural) ;
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
pub struct DataForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct DataReverse ;
impl SpecMap for DataForward {
    type Input = DataInner ;
    type Output = DataSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Data::from_structural (input)
    }
}
impl SpecMap for DataReverse {
    type Input = DataSpec ;
    type Output = DataInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for Data {
}

# [doc = "data type for `ident`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Ident<'i> {
    pub magic: &'i [u8],
    pub class: Class,
    pub data: Data,
    pub version: u8,
    pub osAbi: u8,
    pub abiVersion: u8,
    pub _pad1: &'i [u8],
}
# [verifier::ext_equal]
pub struct IdentSpec < T0 = Seq < u8 >, T1 = ClassSpec, T2 = DataSpec, T3 = u8, T4 = u8, T5 = u8, T6 = Seq < u8 > > {
    pub magic: T0,
    pub class: T1,
    pub data: T2,
    pub version: T3,
    pub osAbi: T4,
    pub abiVersion: T5,
    pub _pad1: T6,
}
pub type IdentInner = (Seq < u8 >, (ClassSpec, (DataSpec, (u8, (u8, (u8, Seq < u8 >)))))) ;
impl<'i> DeepView for Ident<'i> {
    type V = IdentSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        IdentSpec {
            magic: self.magic.deep_view(),
            class: self.class.deep_view(),
            data: self.data.deep_view(),
            version: self.version.deep_view(),
            osAbi: self.osAbi.deep_view(),
            abiVersion: self.abiVersion.deep_view(),
            _pad1: self._pad1.deep_view(),
        }
    }
}
impl<'i> Ident<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().magic == self.magic.deep_view(),
    self.deep_view().class == self.class.deep_view(),
    self.deep_view().data == self.data.deep_view(),
    self.deep_view().version == self.version.deep_view(),
    self.deep_view().osAbi == self.osAbi.deep_view(),
    self.deep_view().abiVersion == self.abiVersion.deep_view(),
    self.deep_view()._pad1 == self._pad1.deep_view(),
    {
        reveal(< Ident as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6 > IdentSpec < T0, T1, T2, T3, T4, T5, T6 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6))))))) -> Self {
        let (magic,
        (class,
        (data,
        (version,
        (osAbi,
        (abiVersion,
        _pad1)))))) = input ;
        Self {
            magic,
            class,
            data,
            version,
            osAbi,
            abiVersion,
            _pad1
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
            magic,
            class,
            data,
            version,
            osAbi,
            abiVersion,
            _pad1
        }
        = self ;
        (magic,
        (class,
        (data,
        (version,
        (osAbi,
        (abiVersion,
        _pad1))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(IdentSpec::from_structural) ;
        reveal(IdentSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    (T1,
    (T2,
    (T3,
    (T4,
    (T5,
    T6))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(IdentSpec::from_structural) ;
        reveal(IdentSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            magic,
            class,
            data,
            version,
            osAbi,
            abiVersion,
            _pad1
        }
        => (magic,
        (class,
        (data,
        (version,
        (osAbi,
        (abiVersion,
        _pad1)))))),
    }
   ,
    {
        reveal(IdentSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct IdentForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct IdentReverse ;
impl SpecMap for IdentForward {
    type Input = IdentInner ;
    type Output = IdentSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        IdentSpec::from_structural (input)
    }
}
impl SpecMap for IdentReverse {
    type Input = IdentSpec ;
    type Output = IdentInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `object_type`."]
# [repr (u16)]
# [derive (Debug, PartialEq, Eq, Clone, Copy, StructuralEq)]
pub enum ObjectType {
    None = 0,
    Rel = 1,
    Exec = 2,
    Dyn = 3,
    Core = 4,
    Unknown (u16),
}
pub type ObjectTypeSpec = ObjectType ;
pub type ObjectTypeInner = Sum < u16, u16 > ;
impl DeepView for ObjectType {
    type V = Self ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        * self
    }
}
impl ObjectType {
    pub proof fn lemma_deep_view (& self) ensures self.deep_view() == * self,
    {
        reveal(< ObjectType as DeepView>::deep_view) ;
    }
    pub open spec fn structural_valid (input: ObjectTypeInner) -> bool {
        match input {
            L (x) => x == 0 || x == 1 || x == 2 || x == 3 || x == 4,
            R (x) => true,
        }
    }
    # [verifier::opaque] pub open spec fn from_structural (input: ObjectTypeInner) -> Self {
        match input {
            L (x) => match x {
                0 => Self::None,
                1 => Self::Rel,
                2 => Self::Exec,
                3 => Self::Dyn,
                4 => Self::Core,
                _ => arbitrary(),
            }
           ,
            R (x) => Self::Unknown (x),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> ObjectTypeInner {
        match self {
            Self::None => L (0),
            Self::Rel => L (1),
            Self::Exec => L (2),
            Self::Dyn => L (3),
            Self::Core => L (4),
            Self::Unknown (x) => R (x),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(ObjectType::from_structural) ;
        reveal(ObjectType::into_structural) ;
        match self {
            Self::None => {
            }
           ,
            Self::Rel => {
            }
           ,
            Self::Exec => {
            }
           ,
            Self::Dyn => {
            }
           ,
            Self::Core => {
            }
           ,
            Self::Unknown (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: ObjectTypeInner) requires Self::structural_valid (input),
    ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(ObjectType::from_structural) ;
        reveal(ObjectType::into_structural) ;
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
pub struct ObjectTypeForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ObjectTypeReverse ;
impl SpecMap for ObjectTypeForward {
    type Input = ObjectTypeInner ;
    type Output = ObjectTypeSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        ObjectType::from_structural (input)
    }
}
impl SpecMap for ObjectTypeReverse {
    type Input = ObjectTypeSpec ;
    type Output = ObjectTypeInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}
# [cfg (not (verus_keep_ghost))] unsafe impl Structural for ObjectType {
}

# [doc = "data type for `header32`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Header32 {
    pub type_: ObjectType,
    pub machine: u16,
    pub version: u32,
    pub entry: u32,
    pub phoff: u32,
    pub shoff: u32,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}
# [verifier::ext_equal]
pub struct Header32Spec < T0 = ObjectTypeSpec, T1 = u16, T2 = u32, T3 = u32, T4 = u32, T5 = u32, T6 = u32, T7 = u16, T8 = u16, T9 = u16, T10 = u16, T11 = u16, T12 = u16 > {
    pub type_: T0,
    pub machine: T1,
    pub version: T2,
    pub entry: T3,
    pub phoff: T4,
    pub shoff: T5,
    pub flags: T6,
    pub ehsize: T7,
    pub phentsize: T8,
    pub phnum: T9,
    pub shentsize: T10,
    pub shnum: T11,
    pub shstrndx: T12,
}
pub type Header32Inner = (ObjectTypeSpec, (u16, (u32, (u32, (u32, (u32, (u32, (u16, (u16, (u16, (u16, (u16, u16)))))))))))) ;
impl DeepView for Header32 {
    type V = Header32Spec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        Header32Spec {
            type_: self.type_.deep_view(),
            machine: self.machine.deep_view(),
            version: self.version.deep_view(),
            entry: self.entry.deep_view(),
            phoff: self.phoff.deep_view(),
            shoff: self.shoff.deep_view(),
            flags: self.flags.deep_view(),
            ehsize: self.ehsize.deep_view(),
            phentsize: self.phentsize.deep_view(),
            phnum: self.phnum.deep_view(),
            shentsize: self.shentsize.deep_view(),
            shnum: self.shnum.deep_view(),
            shstrndx: self.shstrndx.deep_view(),
        }
    }
}
impl Header32 {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().type_ == self.type_.deep_view(),
    self.deep_view().machine == self.machine.deep_view(),
    self.deep_view().version == self.version.deep_view(),
    self.deep_view().entry == self.entry.deep_view(),
    self.deep_view().phoff == self.phoff.deep_view(),
    self.deep_view().shoff == self.shoff.deep_view(),
    self.deep_view().flags == self.flags.deep_view(),
    self.deep_view().ehsize == self.ehsize.deep_view(),
    self.deep_view().phentsize == self.phentsize.deep_view(),
    self.deep_view().phnum == self.phnum.deep_view(),
    self.deep_view().shentsize == self.shentsize.deep_view(),
    self.deep_view().shnum == self.shnum.deep_view(),
    self.deep_view().shstrndx == self.shstrndx.deep_view(),
    {
        reveal(< Header32 as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 > Header32Spec < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 > {
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
    T12))))))))))))) -> Self {
        let (type_,
        (machine,
        (version,
        (entry,
        (phoff,
        (shoff,
        (flags,
        (ehsize,
        (phentsize,
        (phnum,
        (shentsize,
        (shnum,
        shstrndx)))))))))))) = input ;
        Self {
            type_,
            machine,
            version,
            entry,
            phoff,
            shoff,
            flags,
            ehsize,
            phentsize,
            phnum,
            shentsize,
            shnum,
            shstrndx
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
    T12)))))))))))) {
        let Self {
            type_,
            machine,
            version,
            entry,
            phoff,
            shoff,
            flags,
            ehsize,
            phentsize,
            phnum,
            shentsize,
            shnum,
            shstrndx
        }
        = self ;
        (type_,
        (machine,
        (version,
        (entry,
        (phoff,
        (shoff,
        (flags,
        (ehsize,
        (phentsize,
        (phnum,
        (shentsize,
        (shnum,
        shstrndx))))))))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Header32Spec::from_structural) ;
        reveal(Header32Spec::into_structural) ;
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
    T12))))))))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Header32Spec::from_structural) ;
        reveal(Header32Spec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            type_,
            machine,
            version,
            entry,
            phoff,
            shoff,
            flags,
            ehsize,
            phentsize,
            phnum,
            shentsize,
            shnum,
            shstrndx
        }
        => (type_,
        (machine,
        (version,
        (entry,
        (phoff,
        (shoff,
        (flags,
        (ehsize,
        (phentsize,
        (phnum,
        (shentsize,
        (shnum,
        shstrndx)))))))))))),
    }
   ,
    {
        reveal(Header32Spec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct Header32Forward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct Header32Reverse ;
impl SpecMap for Header32Forward {
    type Input = Header32Inner ;
    type Output = Header32Spec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Header32Spec::from_structural (input)
    }
}
impl SpecMap for Header32Reverse {
    type Input = Header32Spec ;
    type Output = Header32Inner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `header64`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Header64 {
    pub type_: ObjectType,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}
# [verifier::ext_equal]
pub struct Header64Spec < T0 = ObjectTypeSpec, T1 = u16, T2 = u32, T3 = u64, T4 = u64, T5 = u64, T6 = u32, T7 = u16, T8 = u16, T9 = u16, T10 = u16, T11 = u16, T12 = u16 > {
    pub type_: T0,
    pub machine: T1,
    pub version: T2,
    pub entry: T3,
    pub phoff: T4,
    pub shoff: T5,
    pub flags: T6,
    pub ehsize: T7,
    pub phentsize: T8,
    pub phnum: T9,
    pub shentsize: T10,
    pub shnum: T11,
    pub shstrndx: T12,
}
pub type Header64Inner = (ObjectTypeSpec, (u16, (u32, (u64, (u64, (u64, (u32, (u16, (u16, (u16, (u16, (u16, u16)))))))))))) ;
impl DeepView for Header64 {
    type V = Header64Spec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        Header64Spec {
            type_: self.type_.deep_view(),
            machine: self.machine.deep_view(),
            version: self.version.deep_view(),
            entry: self.entry.deep_view(),
            phoff: self.phoff.deep_view(),
            shoff: self.shoff.deep_view(),
            flags: self.flags.deep_view(),
            ehsize: self.ehsize.deep_view(),
            phentsize: self.phentsize.deep_view(),
            phnum: self.phnum.deep_view(),
            shentsize: self.shentsize.deep_view(),
            shnum: self.shnum.deep_view(),
            shstrndx: self.shstrndx.deep_view(),
        }
    }
}
impl Header64 {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().type_ == self.type_.deep_view(),
    self.deep_view().machine == self.machine.deep_view(),
    self.deep_view().version == self.version.deep_view(),
    self.deep_view().entry == self.entry.deep_view(),
    self.deep_view().phoff == self.phoff.deep_view(),
    self.deep_view().shoff == self.shoff.deep_view(),
    self.deep_view().flags == self.flags.deep_view(),
    self.deep_view().ehsize == self.ehsize.deep_view(),
    self.deep_view().phentsize == self.phentsize.deep_view(),
    self.deep_view().phnum == self.phnum.deep_view(),
    self.deep_view().shentsize == self.shentsize.deep_view(),
    self.deep_view().shnum == self.shnum.deep_view(),
    self.deep_view().shstrndx == self.shstrndx.deep_view(),
    {
        reveal(< Header64 as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 > Header64Spec < T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 > {
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
    T12))))))))))))) -> Self {
        let (type_,
        (machine,
        (version,
        (entry,
        (phoff,
        (shoff,
        (flags,
        (ehsize,
        (phentsize,
        (phnum,
        (shentsize,
        (shnum,
        shstrndx)))))))))))) = input ;
        Self {
            type_,
            machine,
            version,
            entry,
            phoff,
            shoff,
            flags,
            ehsize,
            phentsize,
            phnum,
            shentsize,
            shnum,
            shstrndx
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
    T12)))))))))))) {
        let Self {
            type_,
            machine,
            version,
            entry,
            phoff,
            shoff,
            flags,
            ehsize,
            phentsize,
            phnum,
            shentsize,
            shnum,
            shstrndx
        }
        = self ;
        (type_,
        (machine,
        (version,
        (entry,
        (phoff,
        (shoff,
        (flags,
        (ehsize,
        (phentsize,
        (phnum,
        (shentsize,
        (shnum,
        shstrndx))))))))))))
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(Header64Spec::from_structural) ;
        reveal(Header64Spec::into_structural) ;
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
    T12))))))))))))) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(Header64Spec::from_structural) ;
        reveal(Header64Spec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            type_,
            machine,
            version,
            entry,
            phoff,
            shoff,
            flags,
            ehsize,
            phentsize,
            phnum,
            shentsize,
            shnum,
            shstrndx
        }
        => (type_,
        (machine,
        (version,
        (entry,
        (phoff,
        (shoff,
        (flags,
        (ehsize,
        (phentsize,
        (phnum,
        (shentsize,
        (shnum,
        shstrndx)))))))))))),
    }
   ,
    {
        reveal(Header64Spec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct Header64Forward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct Header64Reverse ;
impl SpecMap for Header64Forward {
    type Input = Header64Inner ;
    type Output = Header64Spec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        Header64Spec::from_structural (input)
    }
}
impl SpecMap for Header64Reverse {
    type Input = Header64Spec ;
    type Output = Header64Inner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `elf`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub struct Elf<'i> {
    pub ident: Ident<'i>,
    pub header: ElfHeader,
}
# [verifier::ext_equal]
pub struct ElfSpec < T0 = IdentSpec, T1 = ElfHeaderSpec > {
    pub ident: T0,
    pub header: T1,
}
pub type ElfInner = (IdentSpec, ElfHeaderSpec) ;
impl<'i> DeepView for Elf<'i> {
    type V = ElfSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        ElfSpec {
            ident: self.ident.deep_view(),
            header: self.header.deep_view(),
        }
    }
}
impl<'i> Elf<'i> {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view().ident == self.ident.deep_view(),
    self.deep_view().header == self.header.deep_view(),
    {
        reveal(< Elf as DeepView>::deep_view) ;
    }
}
impl < T0, T1 > ElfSpec < T0, T1 > {
    # [verifier::opaque] pub open spec fn from_structural (input: (T0,
    T1)) -> Self {
        let (ident,
        header) = input ;
        Self {
            ident,
            header
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> (T0,
    T1) {
        let Self {
            ident,
            header
        }
        = self ;
        (ident,
        header)
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(ElfSpec::from_structural) ;
        reveal(ElfSpec::into_structural) ;
    }
    pub broadcast proof fn lemma_into_from (input: (T0,
    T1)) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(ElfSpec::from_structural) ;
        reveal(ElfSpec::into_structural) ;
    }
    pub proof fn lemma_into_structural_fields (self) ensures Self::into_structural (self) == match self {
        Self {
            ident,
            header
        }
        => (ident,
        header),
    }
   ,
    {
        reveal(ElfSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ElfForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ElfReverse ;
impl SpecMap for ElfForward {
    type Input = ElfInner ;
    type Output = ElfSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        ElfSpec::from_structural (input)
    }
}
impl SpecMap for ElfReverse {
    type Input = ElfSpec ;
    type Output = ElfInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

# [doc = "data type for `elf_header`."]
# [derive (Debug, PartialEq, Eq, Clone, Copy)]
pub enum ElfHeader {
    Elf32 (Header32),
    Elf64 (Header64),
    Default (()),
}
# [verifier::ext_equal]
pub enum ElfHeaderSpec < T0 = Header32Spec, T1 = Header64Spec, T2 =() > {
    Elf32 (T0),
    Elf64 (T1),
    Default (T2),
}
pub type ElfHeaderInner = Sum < Header32Spec, Sum < Header64Spec,() > > ;
impl DeepView for ElfHeader {
    type V = ElfHeaderSpec ;
    # [verifier::opaque] open spec fn deep_view (& self) -> Self::V {
        match self {
            ElfHeader::Elf32 (v) => ElfHeaderSpec::Elf32 (v.deep_view()),
            ElfHeader::Elf64 (v) => ElfHeaderSpec::Elf64 (v.deep_view()),
            ElfHeader::Default (v) => ElfHeaderSpec::Default (v.deep_view()),
        }
    }
}
impl ElfHeader {
    pub proof fn lemma_deep_view_fields (& self) ensures self.deep_view() == match self {
        ElfHeader::Elf32 (v) => ElfHeaderSpec::Elf32 (v.deep_view()),
        ElfHeader::Elf64 (v) => ElfHeaderSpec::Elf64 (v.deep_view()),
        ElfHeader::Default (v) => ElfHeaderSpec::Default (v.deep_view()),
    }
   ,
    {
        reveal(< ElfHeader as DeepView>::deep_view) ;
    }
}
impl < T0, T1, T2 > ElfHeaderSpec < T0, T1, T2 > {
    # [verifier::opaque] pub open spec fn from_structural (input: Sum < T0,
    Sum < T1,
    T2 > >) -> Self {
        match input {
            L (value) => Self::Elf32 (value),
            R (L (value)) => Self::Elf64 (value),
            R (R (value)) => Self::Default (value),
        }
    }
    # [verifier::opaque] pub open spec fn into_structural (self) -> Sum < T0,
    Sum < T1,
    T2 > > {
        match self {
            Self::Elf32 (value) => L (value),
            Self::Elf64 (value) => R (L (value)),
            Self::Default (value) => R (R (value)),
        }
    }
    pub broadcast proof fn lemma_from_into (self) ensures # [trigger] Self::from_structural (Self::into_structural (self)) == self,
    {
        reveal(ElfHeaderSpec::from_structural) ;
        reveal(ElfHeaderSpec::into_structural) ;
        match self {
            Self::Elf32 (_) => {
            }
           ,
            Self::Elf64 (_) => {
            }
           ,
            Self::Default (_) => {
            }
           ,
        }
    }
    pub broadcast proof fn lemma_into_from (input: Sum < T0,
    Sum < T1,
    T2 > >) ensures # [trigger] Self::into_structural (Self::from_structural (input)) == input,
    {
        reveal(ElfHeaderSpec::from_structural) ;
        reveal(ElfHeaderSpec::into_structural) ;
        match input {
            L (_) => {
            }
           ,
            R (L (_)) => {
            }
           ,
            R (R (_)) => {
            }
           ,
        }
    }
    pub proof fn lemma_into_structural_variant (self) ensures Self::into_structural (self) == match self {
        Self::Elf32 (value) => L (value),
        Self::Elf64 (value) => R (L (value)),
        Self::Default (value) => R (R (value)),
    }
   ,
    {
        reveal(ElfHeaderSpec::into_structural) ;
    }
}
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ElfHeaderForward ;
# [derive (Clone, Copy)]
# [doc (hidden)]
pub struct ElfHeaderReverse ;
impl SpecMap for ElfHeaderForward {
    type Input = ElfHeaderInner ;
    type Output = ElfHeaderSpec ;
    open spec fn spec_map (& self,
    input: Self::Input) -> Self::Output {
        ElfHeaderSpec::from_structural (input)
    }
}
impl SpecMap for ElfHeaderReverse {
    type Input = ElfHeaderSpec ;
    type Output = ElfHeaderInner ;
    open spec fn spec_map (& self,
    value: Self::Input) -> Self::Output {
        value.into_structural()
    }
}

// ============================================================
// Format Specifications
// ============================================================
# [doc = "named format combinator for `class`."]
# [derive (Clone, Copy)]
pub struct ClassFmt ;

pub type ClassFmtSpec = Named < Mapped < Choice < Refined < U8, PredFnSpec < u8 >>, Refined < U8, PredFnSpec < u8 >> >, BiMap < ClassForward, ClassReverse >> > ;

impl ClassFmt {
    # [doc = "specification constructor for `class`."] pub open spec fn spec_inner() -> ClassFmtSpec {
        Named ("class",
        Mapped {
            inner: Choice (Refined (U8,
            | x: u8 | ((x == 0) || (x == 1)) || (x == 2)),
            Refined (U8,
            | x: u8 | ((x != 0) && (x != 1)) && (x != 2))),
            mapper: BiMap (ClassForward,
            ClassReverse),
        }
        )
    }
}


# [doc = "named format combinator for `data`."]
# [derive (Clone, Copy)]
pub struct DataFmt ;

pub type DataFmtSpec = Named < Mapped < Choice < Refined < U8, PredFnSpec < u8 >>, Refined < U8, PredFnSpec < u8 >> >, BiMap < DataForward, DataReverse >> > ;

impl DataFmt {
    # [doc = "specification constructor for `data`."] pub open spec fn spec_inner() -> DataFmtSpec {
        Named ("data",
        Mapped {
            inner: Choice (Refined (U8,
            | x: u8 | ((x == 0) || (x == 1)) || (x == 2)),
            Refined (U8,
            | x: u8 | ((x != 0) && (x != 1)) && (x != 2))),
            mapper: BiMap (DataForward,
            DataReverse),
        }
        )
    }
}


# [doc = "named format combinator for `ident`."]
# [derive (Clone, Copy)]
pub struct IdentFmt ;

pub type IdentFmtSpec = Named < Mapped < Pair < Fixed < 4 >, Pair < ClassFmt, Pair < DataFmt, Pair < U8, Pair < U8, Pair < U8, Fixed < 7 > > > > > > >, BiMap < IdentForward, IdentReverse >> > ;

impl IdentFmt {
    # [doc = "specification constructor for `ident`."] pub open spec fn spec_inner() -> IdentFmtSpec {
        Named ("ident",
        Mapped {
            inner: Pair (Fixed::< 4 >,
            Pair (ClassFmt,
            Pair (DataFmt,
            Pair (U8,
            Pair (U8,
            Pair (U8,
            Fixed::< 7 >)))))),
            mapper: BiMap (IdentForward,
            IdentReverse),
        }
        )
    }
}


# [doc = "named format combinator for `object_type`."]
# [derive (Clone, Copy)]
pub struct ObjectTypeFmt ;

pub type ObjectTypeFmtSpec = Named < Mapped < Choice < Refined < U16Le, PredFnSpec < u16 >>, Refined < U16Le, PredFnSpec < u16 >> >, BiMap < ObjectTypeForward, ObjectTypeReverse >> > ;

impl ObjectTypeFmt {
    # [doc = "specification constructor for `object_type`."] pub open spec fn spec_inner() -> ObjectTypeFmtSpec {
        Named ("object_type",
        Mapped {
            inner: Choice (Refined (U16Le,
            | x: u16 | ((((x == 0) || (x == 1)) || (x == 2)) || (x == 3)) || (x == 4)),
            Refined (U16Le,
            | x: u16 | ((((x != 0) && (x != 1)) && (x != 2)) && (x != 3)) && (x != 4))),
            mapper: BiMap (ObjectTypeForward,
            ObjectTypeReverse),
        }
        )
    }
}


# [doc = "named format combinator for `header32`."]
# [derive (Clone, Copy)]
pub struct Header32Fmt ;

pub type Header32FmtSpec = Named < Mapped < Pair < ObjectTypeFmt, Pair < U16Le, Pair < U32Le, Pair < U32Le, Pair < U32Le, Pair < U32Le, Pair < U32Le, Pair < U16Le, Pair < U16Le, Pair < U16Le, Pair < U16Le, Pair < U16Le, U16Le > > > > > > > > > > > >, BiMap < Header32Forward, Header32Reverse >> > ;

impl Header32Fmt {
    # [doc = "specification constructor for `header32`."] pub open spec fn spec_inner() -> Header32FmtSpec {
        Named ("header32",
        Mapped {
            inner: Pair (ObjectTypeFmt,
            Pair (U16Le,
            Pair (U32Le,
            Pair (U32Le,
            Pair (U32Le,
            Pair (U32Le,
            Pair (U32Le,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U16Le,
            U16Le)))))))))))),
            mapper: BiMap (Header32Forward,
            Header32Reverse),
        }
        )
    }
}


# [doc = "named format combinator for `header64`."]
# [derive (Clone, Copy)]
pub struct Header64Fmt ;

pub type Header64FmtSpec = Named < Mapped < Pair < ObjectTypeFmt, Pair < U16Le, Pair < U32Le, Pair < U64Le, Pair < U64Le, Pair < U64Le, Pair < U32Le, Pair < U16Le, Pair < U16Le, Pair < U16Le, Pair < U16Le, Pair < U16Le, U16Le > > > > > > > > > > > >, BiMap < Header64Forward, Header64Reverse >> > ;

impl Header64Fmt {
    # [doc = "specification constructor for `header64`."] pub open spec fn spec_inner() -> Header64FmtSpec {
        Named ("header64",
        Mapped {
            inner: Pair (ObjectTypeFmt,
            Pair (U16Le,
            Pair (U32Le,
            Pair (U64Le,
            Pair (U64Le,
            Pair (U64Le,
            Pair (U32Le,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U16Le,
            Pair (U16Le,
            U16Le)))))))))))),
            mapper: BiMap (Header64Forward,
            Header64Reverse),
        }
        )
    }
}


# [doc = "named format combinator for `elf`."]
# [derive (Clone, Copy)]
pub struct ElfFmt ;

pub type ElfFmtSpec = Named < Mapped < Bind < IdentFmt, spec_fn (IdentSpec) -> ElfHeaderFmtSpec >, BiMap < ElfForward, ElfReverse >> > ;

impl ElfFmt {
    # [doc = "specification constructor for `elf`."] pub open spec fn spec_inner() -> ElfFmtSpec {
        Named ("elf",
        Mapped {
            inner: Bind (IdentFmt,
            | ident: IdentSpec | ElfHeaderFmt::spec_inner (ident)),
            mapper: BiMap (ElfForward,
            ElfReverse),
        }
        )
    }
}


# [doc = "named format combinator for `elf_header`."]
# [derive (Clone, Copy)]
pub struct ElfHeaderFmt<'i> {
    ident: Ident<'i>,
}
impl<'i> ElfHeaderFmt<'i> {
    # [verifier::type_invariant] spec fn wf (& self) -> bool {
        IdentFmt.consistent (self.ident.deep_view())
    }
    pub closed spec fn ident_spec (& self) -> IdentSpec {
        self.ident.deep_view()
    }
    pub closed spec fn spec (ident: Ident<'i>) -> Self {
        ElfHeaderFmt {
            ident
        }
    }
}

pub type ElfHeaderFmtSpec = Named < Mapped < Sum < Header32Fmt, Sum < Header64Fmt, Empty > >, BiMap < ElfHeaderForward, ElfHeaderReverse >> > ;

impl<'i> ElfHeaderFmt<'i> {
    # [doc = "specification constructor for `elf_header`."] pub open spec fn spec_inner (ident: IdentSpec) -> ElfHeaderFmtSpec {
        Named ("elf_header",
        Mapped {
            inner: match ident.class {
                ClassSpec::Elf32 => L (Header32Fmt),
                ClassSpec::Elf64 => R (L (Header64Fmt)),
                _ => R (R (Empty)),
            }
           ,
            mapper: BiMap (ElfHeaderForward,
            ElfHeaderReverse),
        }
        )
    }
}

// ============================================================
// Derived Parser, Serializer, Length, and Consistency Specifications
// ============================================================
mod derived_specs {
    use super::*;

    impl SpecParser for ClassFmt {
        type PVal = ClassSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ClassFmt {
        type Val = ClassSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ClassFmt {
        type SValue = ClassSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ClassFmt {
        type SVal = ClassSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ClassFmt {
        type T = ClassSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for DataFmt {
        type PVal = DataSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for DataFmt {
        type Val = DataSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for DataFmt {
        type SValue = DataSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for DataFmt {
        type SVal = DataSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for DataFmt {
        type T = DataSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for IdentFmt {
        type PVal = IdentSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for IdentFmt {
        type Val = IdentSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for IdentFmt {
        type SValue = IdentSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for IdentFmt {
        type SVal = IdentSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for IdentFmt {
        type T = IdentSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for ObjectTypeFmt {
        type PVal = ObjectTypeSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ObjectTypeFmt {
        type Val = ObjectTypeSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ObjectTypeFmt {
        type SValue = ObjectTypeSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ObjectTypeFmt {
        type SVal = ObjectTypeSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ObjectTypeFmt {
        type T = ObjectTypeSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for Header32Fmt {
        type PVal = Header32Spec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for Header32Fmt {
        type Val = Header32Spec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for Header32Fmt {
        type SValue = Header32Spec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for Header32Fmt {
        type SVal = Header32Spec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for Header32Fmt {
        type T = Header32Spec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for Header64Fmt {
        type PVal = Header64Spec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for Header64Fmt {
        type Val = Header64Spec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for Header64Fmt {
        type SValue = Header64Spec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for Header64Fmt {
        type SVal = Header64Spec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for Header64Fmt {
        type T = Header64Spec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl SpecParser for ElfFmt {
        type PVal = ElfSpec ;
        # [verifier::opaque] open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner().spec_parse (ibuf)
        }
    }
    impl Consistency for ElfFmt {
        type Val = ElfSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner().consistent (v)
        }
    }
    impl SpecSerializerDps for ElfFmt {
        type SValue = ElfSpec ;
        # [verifier::opaque] open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner().spec_serialize_dps (v,
            obuf)
        }
    }
    impl SpecSerializer for ElfFmt {
        type SVal = ElfSpec ;
        # [verifier::opaque] open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner().spec_serialize (v)
        }
    }
    impl SpecByteLen for ElfFmt {
        type T = ElfSpec ;
        # [verifier::opaque] open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner().byte_len (v)
        }
    }

    impl<'i> SpecParser for ElfHeaderFmt<'i> {
        type PVal = ElfHeaderSpec ;
        open spec fn spec_parse (& self,
        ibuf: Seq < u8 >) -> Option < (int,
        Self::PVal) > {
            Self::spec_inner (self.ident_spec()).spec_parse (ibuf)
        }
    }
    impl<'i> Consistency for ElfHeaderFmt<'i> {
        type Val = ElfHeaderSpec ;
        open spec fn consistent (& self,
        v: Self::Val) -> bool {
            Self::spec_inner (self.ident_spec()).consistent (v)
        }
    }
    impl<'i> SpecSerializerDps for ElfHeaderFmt<'i> {
        type SValue = ElfHeaderSpec ;
        open spec fn spec_serialize_dps (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) -> Seq < u8 > {
            Self::spec_inner (self.ident_spec()).spec_serialize_dps (v,
            obuf)
        }
    }
    impl<'i> SpecSerializer for ElfHeaderFmt<'i> {
        type SVal = ElfHeaderSpec ;
        open spec fn spec_serialize (& self,
        v: Self::SVal) -> Seq < u8 > {
            Self::spec_inner (self.ident_spec()).spec_serialize (v)
        }
    }
    impl<'i> SpecByteLen for ElfHeaderFmt<'i> {
        type T = ElfHeaderSpec ;
        open spec fn byte_len (& self,
        v: Self::T) -> nat {
            Self::spec_inner (self.ident_spec()).byte_len (v)
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
        Class::lemma_from_into,
        Class::lemma_into_from,
        Data::lemma_from_into,
        Data::lemma_into_from,
        IdentSpec::lemma_from_into,
        IdentSpec::lemma_into_from,
        ObjectType::lemma_from_into,
        ObjectType::lemma_into_from,
        Header32Spec::lemma_from_into,
        Header32Spec::lemma_into_from,
        Header64Spec::lemma_from_into,
        Header64Spec::lemma_into_from,
        ElfSpec::lemma_from_into,
        ElfSpec::lemma_into_from,
        ElfHeaderSpec::lemma_from_into,
        ElfHeaderSpec::lemma_into_from,
    };

    impl SafeParser for ClassFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ClassFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ClassFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ClassFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ClassFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ClassFmt as SpecParser>::spec_parse) ;
            reveal(< ClassFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ClassInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Class::structural_valid (input)) ;
                Class::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ClassFmt as SpecParser>::spec_parse) ;
            reveal(< ClassFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ClassInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Class::structural_valid (input)) ;
                Class::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ClassFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ClassFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ClassFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ClassFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ClassFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ClassFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ClassFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ClassFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ClassFmt as SpecParser>::spec_parse) ;
            reveal(< ClassFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ClassFmt as Consistency>::consistent) ;
            reveal(< ClassFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ClassSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Class::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ClassFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ClassFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ClassInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Class::structural_valid (input)) ;
                Class::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ClassFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ClassFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ClassFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ClassFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ClassFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ClassFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for DataFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< DataFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for DataFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< DataFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for DataFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< DataFmt as SpecParser>::spec_parse) ;
            reveal(< DataFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: DataInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Data::structural_valid (input)) ;
                Data::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< DataFmt as SpecParser>::spec_parse) ;
            reveal(< DataFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: DataInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Data::structural_valid (input)) ;
                Data::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for DataFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< DataFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< DataFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< DataFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for DataFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< DataFmt as SpecSerializer>::spec_serialize) ;
            reveal(< DataFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for DataFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< DataFmt as SpecParser>::spec_parse) ;
            reveal(< DataFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< DataFmt as Consistency>::consistent) ;
            reveal(< DataFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: DataSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Data::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for DataFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< DataFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: DataInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (Data::structural_valid (input)) ;
                Data::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for DataFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< DataFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< DataFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for DataFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< DataFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< DataFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for IdentFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< IdentFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for IdentFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< IdentFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for IdentFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< IdentFmt as SpecParser>::spec_parse) ;
            reveal(< IdentFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IdentInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IdentSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< IdentFmt as SpecParser>::spec_parse) ;
            reveal(< IdentFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IdentInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IdentSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for IdentFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< IdentFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< IdentFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IdentFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for IdentFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< IdentFmt as SpecSerializer>::spec_serialize) ;
            reveal(< IdentFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for IdentFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< IdentFmt as SpecParser>::spec_parse) ;
            reveal(< IdentFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IdentFmt as Consistency>::consistent) ;
            reveal(< IdentFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: IdentSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                IdentSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for IdentFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< IdentFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: IdentInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                IdentSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for IdentFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< IdentFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IdentFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for IdentFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< IdentFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< IdentFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for ObjectTypeFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ObjectTypeFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ObjectTypeFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ObjectTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ObjectTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ObjectType::structural_valid (input)) ;
                ObjectType::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ObjectTypeFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ObjectTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ObjectType::structural_valid (input)) ;
                ObjectType::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ObjectTypeFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ObjectTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ObjectTypeFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ObjectTypeFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ObjectTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ObjectTypeFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecParser>::spec_parse) ;
            reveal(< ObjectTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ObjectTypeFmt as Consistency>::consistent) ;
            reveal(< ObjectTypeFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ObjectTypeSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                ObjectType::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ObjectTypeFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ObjectTypeInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                assert (ObjectType::structural_valid (input)) ;
                ObjectType::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ObjectTypeFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ObjectTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ObjectTypeFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ObjectTypeFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ObjectTypeFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ObjectTypeFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for Header32Fmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< Header32Fmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for Header32Fmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< Header32Fmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for Header32Fmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< Header32Fmt as SpecParser>::spec_parse) ;
            reveal(< Header32Fmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: Header32Inner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                Header32Spec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< Header32Fmt as SpecParser>::spec_parse) ;
            reveal(< Header32Fmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: Header32Inner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                Header32Spec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for Header32Fmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< Header32Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< Header32Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< Header32Fmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for Header32Fmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< Header32Fmt as SpecSerializer>::spec_serialize) ;
            reveal(< Header32Fmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for Header32Fmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< Header32Fmt as SpecParser>::spec_parse) ;
            reveal(< Header32Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< Header32Fmt as Consistency>::consistent) ;
            reveal(< Header32Fmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: Header32Spec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Header32Spec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for Header32Fmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< Header32Fmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: Header32Inner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                Header32Spec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for Header32Fmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< Header32Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< Header32Fmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for Header32Fmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< Header32Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< Header32Fmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for Header64Fmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< Header64Fmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for Header64Fmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< Header64Fmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for Header64Fmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< Header64Fmt as SpecParser>::spec_parse) ;
            reveal(< Header64Fmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: Header64Inner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                Header64Spec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< Header64Fmt as SpecParser>::spec_parse) ;
            reveal(< Header64Fmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: Header64Inner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                Header64Spec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for Header64Fmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< Header64Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< Header64Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< Header64Fmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for Header64Fmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< Header64Fmt as SpecSerializer>::spec_serialize) ;
            reveal(< Header64Fmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for Header64Fmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< Header64Fmt as SpecParser>::spec_parse) ;
            reveal(< Header64Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< Header64Fmt as Consistency>::consistent) ;
            reveal(< Header64Fmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: Header64Spec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                Header64Spec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for Header64Fmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< Header64Fmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: Header64Inner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                Header64Spec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for Header64Fmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< Header64Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< Header64Fmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for Header64Fmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< Header64Fmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< Header64Fmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl SafeParser for ElfFmt {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            reveal(< ElfFmt as SpecParser>::spec_parse) ;
            Self::spec_inner().lemma_parse_safe (ibuf) ;
        }
    }
    impl Productive for ElfFmt {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner().productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            reveal(< ElfFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl SoundParser for ElfFmt {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            reveal(< ElfFmt as SpecParser>::spec_parse) ;
            reveal(< ElfFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ElfInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ElfSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            reveal(< ElfFmt as SpecParser>::spec_parse) ;
            reveal(< ElfFmt as Consistency>::consistent) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ElfInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ElfSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl NonTailFmt for ElfFmt {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ElfFmt as SpecSerializerDps>::spec_serialize_dps) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            reveal(< ElfFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ElfFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl GoodSerializer for ElfFmt {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            reveal(< ElfFmt as SpecSerializer>::spec_serialize) ;
            reveal(< ElfFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl SPRoundTripDps for ElfFmt {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            reveal(< ElfFmt as SpecParser>::spec_parse) ;
            reveal(< ElfFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ElfFmt as Consistency>::consistent) ;
            reveal(< ElfFmt as SpecByteLen>::byte_len) ;
            let fmt = Self::spec_inner() ;
            assert forall | output: ElfSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                ElfSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl NonMalleable for ElfFmt {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            reveal(< ElfFmt as SpecParser>::spec_parse) ;
            let fmt = Self::spec_inner() ;
            assert forall | input: ElfInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ElfSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl EquivSerializersGeneral for ElfFmt {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            reveal(< ElfFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ElfFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl EquivSerializers for ElfFmt {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            reveal(< ElfFmt as SpecSerializerDps>::spec_serialize_dps) ;
            reveal(< ElfFmt as SpecSerializer>::spec_serialize) ;
            let fmt = Self::spec_inner() ;
            assert (fmt.equiv_inv()) ;
            fmt.lemma_serialize_equiv_on_empty (v) ;
        }
    }

    impl<'i> SafeParser for ElfHeaderFmt<'i> {
        proof fn lemma_parse_safe (& self,
        ibuf: Seq < u8 >) {
            Self::spec_inner (self.ident_spec()).lemma_parse_safe (ibuf) ;
        }
    }
    impl<'i> Productive for ElfHeaderFmt<'i> {
        open spec fn productive_inv (& self) -> bool {
            Self::spec_inner (self.ident_spec()).productive_inv()
        }
        proof fn lemma_productive (& self,
        s: Seq < u8 >) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert (fmt.productive_inv()) ;
            fmt.lemma_productive (s) ;
        }
    }
    impl<'i> SoundParser for ElfHeaderFmt<'i> {
        proof fn lemma_parse_sound_consumption (& self,
        ibuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert forall | input: ElfHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ElfHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_consumption (ibuf) ;
        }
        proof fn lemma_parse_sound_value (& self,
        ibuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert forall | input: ElfHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ElfHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.sound_inv()) ;
            fmt.lemma_parse_sound_value (ibuf) ;
        }
    }
    impl<'i> NonTailFmt for ElfHeaderFmt<'i> {
        proof fn lemma_serialize_dps_prepend (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_prepend (v,
            obuf) ;
        }
        proof fn lemma_serialize_dps_len (& self,
        v: Self::SValue,
        obuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert (fmt.serialize_dps_inv()) ;
            fmt.lemma_serialize_dps_len (v,
            obuf) ;
        }
    }
    impl<'i> GoodSerializer for ElfHeaderFmt<'i> {
        proof fn lemma_serialize_len (& self,
        v: Self::SVal) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert (fmt.serialize_inv()) ;
            fmt.lemma_serialize_len (v) ;
        }
    }
    impl<'i> SPRoundTripDps for ElfHeaderFmt<'i> {
        proof fn theorem_serialize_dps_parse_roundtrip (& self,
        v: Self::T,
        obuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert forall | output: ElfHeaderSpec | # [trigger] fmt.1.consistent (output) implies fmt.1.mapper.sound (output) by {
                ElfHeaderSpec::lemma_from_into (output) ;
            }
            assert (fmt.unambiguous()) ;
            fmt.theorem_serialize_dps_parse_roundtrip (v,
            obuf) ;
        }
    }
    impl<'i> NonMalleable for ElfHeaderFmt<'i> {
        proof fn lemma_parse_non_malleable (& self,
        buf1: Seq < u8 >,
        buf2: Seq < u8 >) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert forall | input: ElfHeaderInner | # [trigger] fmt.1.inner.consistent (input) implies fmt.1.mapper.lossless (input) by {
                ElfHeaderSpec::lemma_into_from (input) ;
            }
            assert (fmt.nonmal_inv()) ;
            fmt.lemma_parse_non_malleable (buf1,
            buf2) ;
        }
    }
    impl<'i> EquivSerializersGeneral for ElfHeaderFmt<'i> {
        proof fn lemma_serialize_equiv (& self,
        v: Self::SVal,
        obuf: Seq < u8 >) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
            assert (fmt.equiv_general_inv()) ;
            fmt.lemma_serialize_equiv (v,
            obuf) ;
        }
    }
    impl<'i> EquivSerializers for ElfHeaderFmt<'i> {
        proof fn lemma_serialize_equiv_on_empty (& self,
        v: Self::SVal) {
            let fmt = Self::spec_inner (self.ident_spec()) ;
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

    impl<'i> Parser<&'i [u8]> for ClassFmt {
        type PT = Class;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ClassFmt as SpecParser>::spec_parse);
            reveal(<Class as DeepView>::deep_view);
            reveal(Class::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U8.parse(&rest)?;
            let enum_val = match v {
                0 => Class::None,
                1 => Class::Elf32,
                2 => Class::Elf64,
                x => Class::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Class> for ClassFmt {
        fn serialize_into(&self, v: &Class, obuf: &mut Output) {
            reveal(<ClassFmt as SpecSerializer>::spec_serialize);
            reveal(<ClassFmt as SpecByteLen>::byte_len);
            reveal(<Class as DeepView>::deep_view);
            reveal(Class::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                Class::None => 0,
                Class::Elf32 => 1,
                Class::Elf64 => 2,
                Class::Unknown (x) => x,
            };
            U8.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Class> for ClassFmt {
        fn prepare(&self, v: &Class) -> Result<usize, PreSerializeError> {
            reveal(<ClassFmt as SpecByteLen>::byte_len);
            reveal(<Class as DeepView>::deep_view);
            reveal(Class::into_structural);
            let tag = match *v {
                Class::None => 0,
                Class::Elf32 => 1,
                Class::Elf64 => 2,
                Class::Unknown (x) if x != 0 && x != 1 && x != 2 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U8.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for DataFmt {
        type PT = Data;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<DataFmt as SpecParser>::spec_parse);
            reveal(<Data as DeepView>::deep_view);
            reveal(Data::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U8.parse(&rest)?;
            let enum_val = match v {
                0 => Data::None,
                1 => Data::Lsb,
                2 => Data::Msb,
                x => Data::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Data> for DataFmt {
        fn serialize_into(&self, v: &Data, obuf: &mut Output) {
            reveal(<DataFmt as SpecSerializer>::spec_serialize);
            reveal(<DataFmt as SpecByteLen>::byte_len);
            reveal(<Data as DeepView>::deep_view);
            reveal(Data::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                Data::None => 0,
                Data::Lsb => 1,
                Data::Msb => 2,
                Data::Unknown (x) => x,
            };
            U8.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Data> for DataFmt {
        fn prepare(&self, v: &Data) -> Result<usize, PreSerializeError> {
            reveal(<DataFmt as SpecByteLen>::byte_len);
            reveal(<Data as DeepView>::deep_view);
            reveal(Data::into_structural);
            let tag = match *v {
                Data::None => 0,
                Data::Lsb => 1,
                Data::Msb => 2,
                Data::Unknown (x) if x != 0 && x != 1 && x != 2 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U8.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for IdentFmt {
        type PT = Ident<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<IdentFmt as SpecParser>::spec_parse);
            reveal(<Ident as DeepView>::deep_view);
            reveal(IdentSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, magic) = (Fixed::< 4 >).parse (& rest) ?;
            let rest = rest.skip(n1);
            let (n2, class) = (Named ("class", ClassFmt)).parse (& rest) ?;
            proof {
                class.lemma_deep_view();
            }
            let rest = rest.skip(n2);
            let (n3, data) = (Named ("data", DataFmt)).parse (& rest) ?;
            proof {
                data.lemma_deep_view();
            }
            let rest = rest.skip(n3);
            let (n4, version) = (U8).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, osAbi) = (U8).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, abiVersion) = (U8).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, _pad1) = (Fixed::< 7 >).parse (& rest) ?;
            let rest = rest.skip(n7);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7;
            let final_v = Ident {
                magic,
                class,
                data,
                version,
                osAbi,
                abiVersion,
                _pad1,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Ident<'i>> for IdentFmt {
        fn serialize_into(&self, v: &Ident<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<IdentFmt as SpecSerializer>::spec_serialize);
            reveal(<IdentFmt as SpecByteLen>::byte_len);
            reveal(<Ident as DeepView>::deep_view);
            reveal(IdentSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Ident {
                magic,
                class,
                data,
                version,
                osAbi,
                abiVersion,
                _pad1,
            } = v;
            proof {
                class.lemma_deep_view();
                data.lemma_deep_view();
            }

            Fixed::< 4 >.serialize_into(* magic, obuf);
            ClassFmt.serialize_into(class, obuf);
            DataFmt.serialize_into(data, obuf);
            U8.serialize_into(version, obuf);
            U8.serialize_into(osAbi, obuf);
            U8.serialize_into(abiVersion, obuf);
            Fixed::< 7 >.serialize_into(* _pad1, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Ident<'i>> for IdentFmt {
        fn prepare(&self, v: &Ident<'i>) -> Result<usize, PreSerializeError> {
            reveal(<IdentFmt as SpecByteLen>::byte_len);
            reveal(<Ident as DeepView>::deep_view);
            reveal(IdentSpec::into_structural);
            let Ident {
                magic,
                class,
                data,
                version,
                osAbi,
                abiVersion,
                _pad1,
            } = v;
            proof {
                class.lemma_deep_view();
                data.lemma_deep_view();
            }

            let l1 = (Fixed::< 4 >).prepare (magic) ?;
            let l2 = (Named ("class", ClassFmt)).prepare (class) ?;
            let l3 = (Named ("data", DataFmt)).prepare (data) ?;
            let l4 = (U8).prepare (version) ?;
            let l5 = (U8).prepare (osAbi) ?;
            let l6 = (U8).prepare (abiVersion) ?;
            let l7 = (Fixed::< 7 >).prepare (_pad1) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for ObjectTypeFmt {
        type PT = ObjectType;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ObjectTypeFmt as SpecParser>::spec_parse);
            reveal(<ObjectType as DeepView>::deep_view);
            reveal(ObjectType::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n, v) = U16Le.parse(&rest)?;
            let enum_val = match v {
                0 => ObjectType::None,
                1 => ObjectType::Rel,
                2 => ObjectType::Exec,
                3 => ObjectType::Dyn,
                4 => ObjectType::Core,
                x => ObjectType::Unknown (x),
            };
            assert (self.spec_parse (ibuf @) == Some ((n as int, enum_val.deep_view()))) ;
            Ok((n, enum_val))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, ObjectType> for ObjectTypeFmt {
        fn serialize_into(&self, v: &ObjectType, obuf: &mut Output) {
            reveal(<ObjectTypeFmt as SpecSerializer>::spec_serialize);
            reveal(<ObjectTypeFmt as SpecByteLen>::byte_len);
            reveal(<ObjectType as DeepView>::deep_view);
            reveal(ObjectType::into_structural);
            let ghost old_obuf = obuf@;

            let tag = match *v {
                ObjectType::None => 0,
                ObjectType::Rel => 1,
                ObjectType::Exec => 2,
                ObjectType::Dyn => 3,
                ObjectType::Core => 4,
                ObjectType::Unknown (x) => x,
            };
            U16Le.serialize_into(&tag, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<ObjectType> for ObjectTypeFmt {
        fn prepare(&self, v: &ObjectType) -> Result<usize, PreSerializeError> {
            reveal(<ObjectTypeFmt as SpecByteLen>::byte_len);
            reveal(<ObjectType as DeepView>::deep_view);
            reveal(ObjectType::into_structural);
            let tag = match *v {
                ObjectType::None => 0,
                ObjectType::Rel => 1,
                ObjectType::Exec => 2,
                ObjectType::Dyn => 3,
                ObjectType::Core => 4,
                ObjectType::Unknown (x) if x != 0 && x != 1 && x != 2 && x != 3 && x != 4 => x, _ => return Err (PreSerializeError::not_compliant (ComplianceErrorKind::InvalidTag)),
            };
            U16Le.prepare(&tag)
        }
    }



    impl<'i> Parser<&'i [u8]> for Header32Fmt {
        type PT = Header32;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<Header32Fmt as SpecParser>::spec_parse);
            reveal(<Header32 as DeepView>::deep_view);
            reveal(Header32Spec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, type_) = (Named ("object_type", ObjectTypeFmt)).parse (& rest) ?;
            proof {
                type_.lemma_deep_view();
            }
            let rest = rest.skip(n1);
            let (n2, machine) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, version) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, entry) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, phoff) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, shoff) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, flags) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, ehsize) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n8);
            let (n9, phentsize) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n9);
            let (n10, phnum) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n10);
            let (n11, shentsize) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n11);
            let (n12, shnum) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n12);
            let (n13, shstrndx) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n13);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9 + n10 + n11 + n12 + n13;
            let final_v = Header32 {
                type_,
                machine,
                version,
                entry,
                phoff,
                shoff,
                flags,
                ehsize,
                phentsize,
                phnum,
                shentsize,
                shnum,
                shstrndx,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Header32> for Header32Fmt {
        fn serialize_into(&self, v: &Header32, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<Header32Fmt as SpecSerializer>::spec_serialize);
            reveal(<Header32Fmt as SpecByteLen>::byte_len);
            reveal(<Header32 as DeepView>::deep_view);
            reveal(Header32Spec::into_structural);
            let ghost old_obuf = obuf@;

            let Header32 {
                type_,
                machine,
                version,
                entry,
                phoff,
                shoff,
                flags,
                ehsize,
                phentsize,
                phnum,
                shentsize,
                shnum,
                shstrndx,
            } = v;
            proof {
                type_.lemma_deep_view();
            }

            ObjectTypeFmt.serialize_into(type_, obuf);
            U16Le.serialize_into(machine, obuf);
            U32Le.serialize_into(version, obuf);
            U32Le.serialize_into(entry, obuf);
            U32Le.serialize_into(phoff, obuf);
            U32Le.serialize_into(shoff, obuf);
            U32Le.serialize_into(flags, obuf);
            U16Le.serialize_into(ehsize, obuf);
            U16Le.serialize_into(phentsize, obuf);
            U16Le.serialize_into(phnum, obuf);
            U16Le.serialize_into(shentsize, obuf);
            U16Le.serialize_into(shnum, obuf);
            U16Le.serialize_into(shstrndx, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Header32> for Header32Fmt {
        fn prepare(&self, v: &Header32) -> Result<usize, PreSerializeError> {
            reveal(<Header32Fmt as SpecByteLen>::byte_len);
            reveal(<Header32 as DeepView>::deep_view);
            reveal(Header32Spec::into_structural);
            let Header32 {
                type_,
                machine,
                version,
                entry,
                phoff,
                shoff,
                flags,
                ehsize,
                phentsize,
                phnum,
                shentsize,
                shnum,
                shstrndx,
            } = v;
            proof {
                type_.lemma_deep_view();
            }

            let l1 = (Named ("object_type", ObjectTypeFmt)).prepare (type_) ?;
            let l2 = (U16Le).prepare (machine) ?;
            let l3 = (U32Le).prepare (version) ?;
            let l4 = (U32Le).prepare (entry) ?;
            let l5 = (U32Le).prepare (phoff) ?;
            let l6 = (U32Le).prepare (shoff) ?;
            let l7 = (U32Le).prepare (flags) ?;
            let l8 = (U16Le).prepare (ehsize) ?;
            let l9 = (U16Le).prepare (phentsize) ?;
            let l10 = (U16Le).prepare (phnum) ?;
            let l11 = (U16Le).prepare (shentsize) ?;
            let l12 = (U16Le).prepare (shnum) ?;
            let l13 = (U16Le).prepare (shstrndx) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l9).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l10).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l11).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l12).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l13).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for Header64Fmt {
        type PT = Header64;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<Header64Fmt as SpecParser>::spec_parse);
            reveal(<Header64 as DeepView>::deep_view);
            reveal(Header64Spec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, type_) = (Named ("object_type", ObjectTypeFmt)).parse (& rest) ?;
            proof {
                type_.lemma_deep_view();
            }
            let rest = rest.skip(n1);
            let (n2, machine) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n2);
            let (n3, version) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n3);
            let (n4, entry) = (U64Le).parse (& rest) ?;
            let rest = rest.skip(n4);
            let (n5, phoff) = (U64Le).parse (& rest) ?;
            let rest = rest.skip(n5);
            let (n6, shoff) = (U64Le).parse (& rest) ?;
            let rest = rest.skip(n6);
            let (n7, flags) = (U32Le).parse (& rest) ?;
            let rest = rest.skip(n7);
            let (n8, ehsize) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n8);
            let (n9, phentsize) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n9);
            let (n10, phnum) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n10);
            let (n11, shentsize) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n11);
            let (n12, shnum) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n12);
            let (n13, shstrndx) = (U16Le).parse (& rest) ?;
            let rest = rest.skip(n13);
            let total_n = n1 + n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9 + n10 + n11 + n12 + n13;
            let final_v = Header64 {
                type_,
                machine,
                version,
                entry,
                phoff,
                shoff,
                flags,
                ehsize,
                phentsize,
                phnum,
                shentsize,
                shnum,
                shstrndx,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Header64> for Header64Fmt {
        fn serialize_into(&self, v: &Header64, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<Header64Fmt as SpecSerializer>::spec_serialize);
            reveal(<Header64Fmt as SpecByteLen>::byte_len);
            reveal(<Header64 as DeepView>::deep_view);
            reveal(Header64Spec::into_structural);
            let ghost old_obuf = obuf@;

            let Header64 {
                type_,
                machine,
                version,
                entry,
                phoff,
                shoff,
                flags,
                ehsize,
                phentsize,
                phnum,
                shentsize,
                shnum,
                shstrndx,
            } = v;
            proof {
                type_.lemma_deep_view();
            }

            ObjectTypeFmt.serialize_into(type_, obuf);
            U16Le.serialize_into(machine, obuf);
            U32Le.serialize_into(version, obuf);
            U64Le.serialize_into(entry, obuf);
            U64Le.serialize_into(phoff, obuf);
            U64Le.serialize_into(shoff, obuf);
            U32Le.serialize_into(flags, obuf);
            U16Le.serialize_into(ehsize, obuf);
            U16Le.serialize_into(phentsize, obuf);
            U16Le.serialize_into(phnum, obuf);
            U16Le.serialize_into(shentsize, obuf);
            U16Le.serialize_into(shnum, obuf);
            U16Le.serialize_into(shstrndx, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Header64> for Header64Fmt {
        fn prepare(&self, v: &Header64) -> Result<usize, PreSerializeError> {
            reveal(<Header64Fmt as SpecByteLen>::byte_len);
            reveal(<Header64 as DeepView>::deep_view);
            reveal(Header64Spec::into_structural);
            let Header64 {
                type_,
                machine,
                version,
                entry,
                phoff,
                shoff,
                flags,
                ehsize,
                phentsize,
                phnum,
                shentsize,
                shnum,
                shstrndx,
            } = v;
            proof {
                type_.lemma_deep_view();
            }

            let l1 = (Named ("object_type", ObjectTypeFmt)).prepare (type_) ?;
            let l2 = (U16Le).prepare (machine) ?;
            let l3 = (U32Le).prepare (version) ?;
            let l4 = (U64Le).prepare (entry) ?;
            let l5 = (U64Le).prepare (phoff) ?;
            let l6 = (U64Le).prepare (shoff) ?;
            let l7 = (U32Le).prepare (flags) ?;
            let l8 = (U16Le).prepare (ehsize) ?;
            let l9 = (U16Le).prepare (phentsize) ?;
            let l10 = (U16Le).prepare (phnum) ?;
            let l11 = (U16Le).prepare (shentsize) ?;
            let l12 = (U16Le).prepare (shnum) ?;
            let l13 = (U16Le).prepare (shstrndx) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l3).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l4).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l5).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l6).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l7).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l8).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l9).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l10).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l11).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l12).ok_or (PreSerializeError::length_too_large()) ?.checked_add (l13).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for ElfFmt {
        type PT = Elf<'i>;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            broadcast use vest_lib::core::spec::SafeParser::lemma_parse_safe;
            broadcast use vest_lib::core::spec::SoundParser::lemma_parse_sound_value;

            reveal(<ElfFmt as SpecParser>::spec_parse);
            reveal(<Elf as DeepView>::deep_view);
            reveal(ElfSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            let (n1, ident) = (Named ("ident", IdentFmt)).parse (& rest) ?;
            let rest = rest.skip(n1);
            proof {
                ident.lemma_deep_view_fields();
                ident.deep_view().lemma_into_structural_fields();
            }

            let (n2, header) = (Named ("elf_header", ElfHeaderFmt {
                ident: ident
            }
            )).parse (& rest) ?;
            let rest = rest.skip(n2);
            let total_n = n1 + n2;
            let final_v = Elf {
                ident,
                header,
            };
            assert(self.spec_parse(ibuf@) == Some((total_n as int, final_v.deep_view())));
            Ok((total_n, final_v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, Elf<'i>> for ElfFmt {
        fn serialize_into(&self, v: &Elf<'i>, obuf: &mut Output) {
            broadcast use vest_lib::core::exec::output::outbuf_lemmas;
            reveal(<ElfFmt as SpecSerializer>::spec_serialize);
            reveal(<ElfFmt as SpecByteLen>::byte_len);
            reveal(<Elf as DeepView>::deep_view);
            reveal(ElfSpec::into_structural);
            let ghost old_obuf = obuf@;

            let Elf {
                ident,
                header,
            } = v;
            proof {
                ident.lemma_deep_view_fields();
                ident.deep_view().lemma_into_structural_fields();
            }

            IdentFmt.serialize_into(ident, obuf);
            ElfHeaderFmt {
                ident: * ident
            }
            .serialize_into(header, obuf);

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<Elf<'i>> for ElfFmt {
        fn prepare(&self, v: &Elf<'i>) -> Result<usize, PreSerializeError> {
            reveal(<ElfFmt as SpecByteLen>::byte_len);
            reveal(<Elf as DeepView>::deep_view);
            reveal(ElfSpec::into_structural);
            let Elf {
                ident,
                header,
            } = v;
            proof {
                ident.lemma_deep_view_fields();
                ident.deep_view().lemma_into_structural_fields();
            }

            let l1 = (Named ("ident", IdentFmt)).prepare (ident) ?;
            let l2 = (Named ("elf_header", ElfHeaderFmt {
                ident: * ident
            }
            )).prepare (header) ?;
            let total_len = l1.checked_add (l2).ok_or (PreSerializeError::length_too_large()) ?;
            Ok(total_len)
        }
    }



    impl<'i> Parser<&'i [u8]> for ElfHeaderFmt<'i> {
        type PT = ElfHeader;

        fn parse(&self, ibuf: &&'i [u8]) -> PResult<Self::PT> {
            reveal(<ElfHeaderFmt as SpecParser>::spec_parse);
            reveal(<ElfHeader as DeepView>::deep_view);
            reveal(ElfHeaderSpec::from_structural);
            let _ = ibuf.len();
            let rest = *ibuf;

            proof {
                use_type_invariant(self);
            }

            proof {
                self.ident.lemma_deep_view_fields();
                self.ident.deep_view().lemma_into_structural_fields();
                self.ident.class.lemma_deep_view();
            }

            let (n, v) = match self.ident.class {
                Class::Elf32 => {
                    let (n,
                    v) = (Named ("header32",
                    Header32Fmt)).parse (& rest) ?;
                    (n,
                    ElfHeader::Elf32 (v))
                }
                ,
                Class::Elf64 => {
                    let (n,
                    v) = (Named ("header64",
                    Header64Fmt)).parse (& rest) ?;
                    (n,
                    ElfHeader::Elf64 (v))
                }
                ,
                _ => {
                    let (n,
                    v) = (Empty).parse (& rest) ?;
                    (n,
                    ElfHeader::Default (v))
                }
                ,
            };
            assert(self.spec_parse(ibuf@) == Some((n as int, v.deep_view())));
            Ok((n, v))
        }
    }

    impl<Output: OutputBuf, 'i> Serializer<Output, ElfHeader> for ElfHeaderFmt<'i> {
        fn serialize_into(&self, v: &ElfHeader, obuf: &mut Output) {
            reveal(<ElfHeaderFmt as SpecSerializer>::spec_serialize);
            reveal(<ElfHeaderFmt as SpecByteLen>::byte_len);
            reveal(<ElfHeader as DeepView>::deep_view);
            reveal(ElfHeaderSpec::into_structural);
            proof {
                use_type_invariant(self);
            }

            let ghost old_obuf = obuf@;

            proof {
                self.ident.lemma_deep_view_fields();
                self.ident.deep_view().lemma_into_structural_fields();
                self.ident.class.lemma_deep_view();
            }

            match (self.ident.class, v) {
                (Class::Elf32, ElfHeader::Elf32 (v)) => {
                    (Header32Fmt).serialize_into (v,
                    obuf) ;
                }
                ,
                (Class::Elf64, ElfHeader::Elf64 (v)) => {
                    (Header64Fmt).serialize_into (v,
                    obuf) ;
                }
                ,
                (_, ElfHeader::Default (v)) => {
                    (Empty).serialize_into (v,
                    obuf) ;
                }
                ,
                _ => {},
            }

            assert(obuf@ == old_obuf + self.spec_serialize(v.deep_view()));
        }
    }

    impl<'i> Prepare<ElfHeader> for ElfHeaderFmt<'i> {
        fn prepare(&self, v: &ElfHeader) -> Result<usize, PreSerializeError> {
            reveal(<ElfHeaderFmt as SpecByteLen>::byte_len);
            reveal(<ElfHeader as DeepView>::deep_view);
            reveal(ElfHeaderSpec::into_structural);
            proof {
                use_type_invariant(self);
            }

            proof {
                self.ident.lemma_deep_view_fields();
                self.ident.deep_view().lemma_into_structural_fields();
                self.ident.class.lemma_deep_view();
            }

            match (self.ident.class, v) {
                (Class::Elf32, ElfHeader::Elf32 (v)) => (Named ("header32", Header32Fmt)).prepare (v),
                (Class::Elf64, ElfHeader::Elf64 (v)) => (Named ("header64", Header64Fmt)).prepare (v),
                (Class::None, ElfHeader::Default (v)) => (Empty).prepare (v),
                (Class::Unknown (x), ElfHeader::Default (v)) if x != 0 && x != 1 && x != 2 => (Empty).prepare (v),
                 _ => Err(PreSerializeError::not_compliant(ComplianceErrorKind::InvalidTag)),
            }
        }
    }

}
}
