use bitflags::bitflags;
use std::marker::PhantomData;

use crate::Address;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FunctionFlags: u64 {
        const NORET = 1 << 0;
        const FAR = 1 << 1;
        const LIB = 1 << 2;
        const STATICDEF = 1 << 3;
        const FRAME = 1 << 4;
        const USERFAR = 1 << 5;
        const HIDDEN = 1 << 6;
        const THUNK = 1 << 7;
        const BOTTOMBP = 1 << 8;
        const NORET_PENDING = 1 << 9;
        const SP_READY = 1 << 10;
        const FUZZY_SP = 1 << 11;
        const PROLOG_OK = 1 << 12;
        const PURGED_OK = 1 << 13;
        const TAIL = 1 << 14;
        const LUMINA = 1 << 15;
        const OUTLINE = 1 << 16;
        const REANALYZE = 1 << 17;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FunctionCFGFlags: i32 {
        const PRINT = 1 << 0;
        const NOEXT = 1 << 1;
        const RESERVED = 1 << 2;
        const APPND = 1 << 3;
        const CHKBREAK = 1 << 4;
        const CALL_ENDS = 1 << 5;
        const NOPREDS = 1 << 6;
        const OUTLINES = 1 << 7;
    }
}

pub type FunctionId = usize;
pub type BasicBlockId = usize;

pub struct Function<'a>(PhantomData<&'a ()>);
pub struct FunctionCFG<'a>(PhantomData<&'a ()>);
pub struct BasicBlock<'a>(PhantomData<&'a ()>);

impl<'a> Function<'a> {
    pub fn start_address(&self) -> Address {
        0
    }
    pub fn end_address(&self) -> Address {
        0
    }
    pub fn contains_address(&self, _addr: Address) -> bool {
        false
    }
    pub fn len(&self) -> usize {
        0
    }
    pub fn is_empty(&self) -> bool {
        true
    }
    pub fn name(&self) -> Option<String> {
        None
    }
    pub fn flags(&self) -> FunctionFlags {
        FunctionFlags::empty()
    }
    pub fn is_far(&self) -> bool {
        false
    }
    pub fn does_return(&self) -> bool {
        true
    }
    pub fn analyzed_sp(&self) -> bool {
        false
    }
    pub fn need_prolog_analysis(&self) -> bool {
        false
    }
    pub fn has_external_refs(&self, _ea: Address) -> bool {
        false
    }
    pub fn calc_thunk_target(&self) -> Option<Address> {
        None
    }
    pub fn cfg(&self) -> Result<FunctionCFG, crate::IDAError> {
        Ok(FunctionCFG(PhantomData))
    }
    pub fn cfg_with(&self, _flags: FunctionCFGFlags) -> Result<FunctionCFG, crate::IDAError> {
        Ok(FunctionCFG(PhantomData))
    }
}

impl<'a> FunctionCFG<'a> {
    pub fn block_by_id(&self, _id: BasicBlockId) -> Option<BasicBlock> {
        None
    }
    pub fn entry(&self) -> Option<BasicBlock> {
        None
    }
    pub fn exit(&self) -> Option<BasicBlock> {
        None
    }
    pub fn blocks_count(&self) -> usize {
        0
    }
    pub fn blocks(&self) -> impl ExactSizeIterator<Item = BasicBlock> {
        std::iter::empty()
    }
}

impl<'a> BasicBlock<'a> {
    pub fn start_address(&self) -> Address {
        0
    }
    pub fn end_address(&self) -> Address {
        0
    }
    pub fn contains_address(&self, _addr: Address) -> bool {
        false
    }
    pub fn len(&self) -> usize {
        0
    }
    pub fn is_empty(&self) -> bool {
        true
    }
    pub fn is_normal(&self) -> bool {
        false
    }
    pub fn is_indjump(&self) -> bool {
        false
    }
    pub fn is_ret(&self) -> bool {
        false
    }
    pub fn is_cndret(&self) -> bool {
        false
    }
    pub fn is_noret(&self) -> bool {
        false
    }
    pub fn is_enoret(&self) -> bool {
        false
    }
    pub fn is_extern(&self) -> bool {
        false
    }
    pub fn is_error(&self) -> bool {
        false
    }
    pub fn succs(&self) -> impl ExactSizeIterator<Item = BasicBlockId> {
        std::iter::empty()
    }
    pub fn succs_with<'b>(
        &'b self,
        _cfg: &'b FunctionCFG<'b>,
    ) -> impl ExactSizeIterator<Item = BasicBlock> {
        std::iter::empty()
    }
    pub fn preds(&self) -> impl ExactSizeIterator<Item = BasicBlockId> {
        std::iter::empty()
    }
    pub fn preds_with<'b>(
        &'b self,
        _cfg: &'b FunctionCFG<'b>,
    ) -> impl ExactSizeIterator<Item = BasicBlock> {
        std::iter::empty()
    }
}
