use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    OldEXE,
    OldCOM,
    BIN,
    DRV,
    WIN,
    HEX,
    MEX,
    LX,
    LE,
    NLM,
    COFF,
    PE,
    OMF,
    SREC,
    ZIP,
    OMFLIB,
    AR,
    LOADER,
    ELF,
    W32RUN,
    AOUT,
    PRC,
    EXE,
    COM,
    AIXAR,
    MACHO,
    PSXOBJ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compiler {
    UNK,
    MS,
    BC,
    WATCOM,
    GNU,
    VISAGE,
    BP,
    UNSURE,
}

pub struct Metadata<'a>(pub(crate) PhantomData<&'a ()>);
pub struct MetadataMut<'a>(pub(crate) PhantomData<&'a ()>);

impl<'a> Metadata<'a> {
    pub fn procname(&self) -> String {
        String::new()
    }
    pub fn filetype(&self) -> FileType {
        FileType::BIN
    }
    pub fn cc_id(&self) -> Compiler {
        Compiler::UNK
    }
    pub fn bitness(&self) -> u32 {
        0
    }
    pub fn app_bitness(&self) -> u32 {
        0
    }
    pub fn is_dll(&self) -> bool {
        false
    }
    pub fn is_be(&self) -> bool {
        false
    }
    pub fn is_16bit(&self) -> bool {
        false
    }
    pub fn is_32bit(&self) -> bool {
        false
    }
    pub fn is_32bit_exactly(&self) -> bool {
        false
    }
    pub fn is_64bit(&self) -> bool {
        false
    }
    pub fn database_change_count(&self) -> u32 {
        0
    }
    pub fn is_auto_enabled(&self) -> bool {
        false
    }
    pub fn readonly_idb(&self) -> bool {
        false
    }
    pub fn is_kernel_mode(&self) -> bool {
        false
    }
    pub fn is_snapshot(&self) -> bool {
        false
    }
}
