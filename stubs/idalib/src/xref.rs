use bitflags::bitflags;
use std::marker::PhantomData;

use crate::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeRef {
    Unknown,
    FarCall,
    NearCall,
    FarJump,
    NearJump,
    Obsolete,
    Flow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataRef {
    Unknown,
    Offset,
    Write,
    Read,
    Text,
    Informational,
    EnumMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRefType {
    Code(CodeRef),
    Data(DataRef),
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct XRefFlags: i32 {
        const USER = 1;
        const TAIL = 2;
        const BASE = 4;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct XRefQuery: i32 {
        const ALL = 0;
        const FAR = 1;
        const DATA = 2;
    }
}

pub struct XRef<'a>(pub(crate) PhantomData<&'a ()>);

impl<'a> Clone for XRef<'a> {
    fn clone(&self) -> Self {
        XRef(PhantomData)
    }
}

impl<'a> XRef<'a> {
    pub fn from(&self) -> Address {
        0
    }
    pub fn to(&self) -> Address {
        0
    }
    pub fn flags(&self) -> XRefFlags {
        XRefFlags::empty()
    }
    pub fn type_(&self) -> XRefType {
        XRefType::Code(CodeRef::Unknown)
    }
    pub fn is_code(&self) -> bool {
        false
    }
    pub fn is_data(&self) -> bool {
        false
    }
    pub fn is_user_defined(&self) -> bool {
        false
    }
    pub fn next_to(&self) -> Option<Self> {
        None
    }
    pub fn next_from(&self) -> Option<Self> {
        None
    }
}
