use bitflags::bitflags;
use std::marker::PhantomData;

use crate::Address;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct SegmentPermissions: u16 {
        const READ = 1;
        const WRITE = 2;
        const EXECUTE = 4;
    }
}

impl SegmentPermissions {
    pub fn is_executable(&self) -> bool {
        self.contains(SegmentPermissions::EXECUTE)
    }
    pub fn is_writable(&self) -> bool {
        self.contains(SegmentPermissions::WRITE)
    }
    pub fn is_readable(&self) -> bool {
        self.contains(SegmentPermissions::READ)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentAlignment {
    Abs,
    RelByte,
    RelWord,
    RelPara,
    RelPage,
    RelDble,
    Rel4k,
    Group,
    Rel32Bytes,
    Rel64Bytes,
    RelQword,
    Rel128Bytes,
    Rel512Bytes,
    Rel1024Bytes,
    Rel2048Bytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    Normal,
    Extern,
    Code,
    Data,
    Import,
    Group,
    Bss,
    Null,
    Absym,
    Comm,
    Imem,
    Undefined,
}

impl SegmentType {
    pub fn is_normal(&self) -> bool {
        matches!(self, SegmentType::Normal)
    }
    pub fn is_extern(&self) -> bool {
        matches!(self, SegmentType::Extern)
    }
    pub fn is_code(&self) -> bool {
        matches!(self, SegmentType::Code)
    }
    pub fn is_data(&self) -> bool {
        matches!(self, SegmentType::Data)
    }
    pub fn is_import(&self) -> bool {
        matches!(self, SegmentType::Import)
    }
    pub fn is_group(&self) -> bool {
        matches!(self, SegmentType::Group)
    }
    pub fn is_bss(&self) -> bool {
        matches!(self, SegmentType::Bss)
    }
    pub fn is_null(&self) -> bool {
        matches!(self, SegmentType::Null)
    }
    pub fn is_absym(&self) -> bool {
        matches!(self, SegmentType::Absym)
    }
    pub fn is_comm(&self) -> bool {
        matches!(self, SegmentType::Comm)
    }
    pub fn is_imem(&self) -> bool {
        matches!(self, SegmentType::Imem)
    }
    pub fn is_undefined(&self) -> bool {
        matches!(self, SegmentType::Undefined)
    }
}

pub type SegmentId = usize;

pub struct Segment<'a>(pub(crate) PhantomData<&'a ()>);

impl<'a> Segment<'a> {
    pub fn start_address(&self) -> Address {
        0
    }
    pub fn end_address(&self) -> Address {
        0
    }
    pub fn len(&self) -> usize {
        0
    }
    pub fn is_empty(&self) -> bool {
        true
    }
    pub fn contains_address(&self, _addr: Address) -> bool {
        false
    }
    pub fn name(&self) -> Option<String> {
        None
    }
    pub fn alignment(&self) -> SegmentAlignment {
        SegmentAlignment::Abs
    }
    pub fn permissions(&self) -> SegmentPermissions {
        SegmentPermissions::empty()
    }
    pub fn bitness(&self) -> usize {
        0
    }
    pub fn r#type(&self) -> SegmentType {
        SegmentType::Normal
    }
    pub fn bytes(&self) -> Vec<u8> {
        Vec::new()
    }
    pub fn address_bits(&self) -> u32 {
        0
    }
    pub fn address_bytes(&self) -> usize {
        0
    }
    pub fn is_16bit(&self) -> bool {
        false
    }
    pub fn is_32bit(&self) -> bool {
        false
    }
    pub fn is_64bit(&self) -> bool {
        false
    }
    pub fn is_hidden(&self) -> bool {
        false
    }
}
