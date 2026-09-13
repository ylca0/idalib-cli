use bitflags::bitflags;
use std::marker::PhantomData;

use crate::Address;

pub type Register = u16;
pub type Phrase = u16;
pub type InsnType = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandType {
    Reg,
    Mem,
    Phrase,
    Displ,
    Imm,
    Far,
    Near,
    IdpSpec0,
    IdpSpec1,
    IdpSpec2,
    IdpSpec3,
    IdpSpec4,
    IdpSpec5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandDataType {
    Byte,
    Word,
    DWord,
    Float,
    Double,
    TByte,
    PackReal,
    QWord,
    Byte16,
    Code,
    Void,
    FWord,
    Bitfield,
    String,
    Unicode,
    LongDouble,
    Byte32,
    Byte64,
    Half,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct OperandFlags: u32 {
        const NONE = 0;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct IsReturnFlags: u8 {
        const NONE = 0;
    }
}

#[derive(Debug, Clone)]
pub struct Operand {
    n: usize,
    kind: OperandType,
    dtype: OperandDataType,
}

impl Operand {
    pub fn flags(&self) -> OperandFlags {
        OperandFlags::empty()
    }
    pub fn offb(&self) -> i8 {
        0
    }
    pub fn offo(&self) -> i8 {
        0
    }
    pub fn n(&self) -> usize {
        self.n
    }
    pub fn number(&self) -> usize {
        self.n
    }
    pub fn type_(&self) -> OperandType {
        self.kind
    }
    pub fn dtype(&self) -> OperandDataType {
        self.dtype
    }
    pub fn reg(&self) -> Option<Register> {
        None
    }
    pub fn register(&self) -> Option<Register> {
        None
    }
    pub fn phrase(&self) -> Option<Phrase> {
        None
    }
    pub fn value(&self) -> Option<u64> {
        None
    }
    pub fn outer_displacement(&self) -> Option<u64> {
        None
    }
    pub fn address(&self) -> Option<Address> {
        None
    }
    pub fn addr(&self) -> Option<Address> {
        None
    }
    pub fn processor_specific(&self) -> Option<u64> {
        None
    }
    pub fn is_processor_specific(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct Insn {
    ea: Address,
    size: usize,
}

impl Insn {
    pub fn address(&self) -> Address {
        self.ea
    }
    pub fn itype(&self) -> InsnType {
        0
    }
    pub fn operand(&self, n: usize) -> Option<Operand> {
        let _ = n;
        None
    }
    pub fn operand_count(&self) -> usize {
        0
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        true
    }
    pub fn is_basic_block_end(&self, _call_stops_block: bool) -> bool {
        false
    }
    pub fn is_call(&self) -> bool {
        false
    }
    pub fn is_indirect_jump(&self) -> bool {
        false
    }
    pub fn is_ret(&self) -> bool {
        false
    }
    pub fn is_ret_with(&self, _iri: IsReturnFlags) -> bool {
        false
    }
}
