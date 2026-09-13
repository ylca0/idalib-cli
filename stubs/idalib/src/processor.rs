use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorFamily {
    X86,
    Arm,
    Mips,
    Ppc,
    Unknown,
}

pub struct Processor<'a>(pub(crate) PhantomData<&'a ()>);

impl<'a> Processor<'a> {
    pub fn family(&self) -> ProcessorFamily {
        ProcessorFamily::Unknown
    }
    pub fn long_name(&self) -> String {
        String::new()
    }
    pub fn short_name(&self) -> String {
        String::new()
    }
}
