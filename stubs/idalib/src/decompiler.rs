use std::marker::PhantomData;

pub struct CFunction<'a>(pub(crate) PhantomData<&'a ()>);
pub struct CBlock<'a>(pub(crate) PhantomData<&'a ()>);
pub struct CBlockIter<'a>(pub(crate) PhantomData<&'a ()>);
pub struct CInsn<'a>(pub(crate) PhantomData<&'a ()>);

impl<'a> CFunction<'a> {
    pub fn pseudocode(&self) -> String {
        String::new()
    }
    pub fn body(&self) -> CBlock {
        CBlock(PhantomData)
    }
}

impl<'a> CBlock<'a> {
    pub fn iter(&self) -> CBlockIter {
        CBlockIter(PhantomData)
    }
    pub fn len(&self) -> usize {
        0
    }
    pub fn is_empty(&self) -> bool {
        true
    }
}

impl<'a> Iterator for CBlockIter<'a> {
    type Item = CInsn<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
