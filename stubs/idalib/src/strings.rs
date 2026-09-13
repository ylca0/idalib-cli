use std::marker::PhantomData;

use crate::Address;

pub struct StringList<'a>(pub(crate) PhantomData<&'a ()>);

impl<'a> StringList<'a> {
    pub fn rebuild(&self) {}
    pub fn clear(&self) {}
    pub fn get_by_index(&self, _index: usize) -> Option<String> {
        None
    }
    pub fn get_address_by_index(&self, _index: usize) -> Option<Address> {
        None
    }
    pub fn len(&self) -> usize {
        0
    }
    pub fn is_empty(&self) -> bool {
        true
    }
    pub fn iter(&self) -> StringListIter<'_, 'a> {
        StringListIter {
            list: self,
            current: 0,
        }
    }
}

pub struct StringListIter<'s, 'a> {
    list: &'s StringList<'a>,
    current: usize,
}

impl<'s, 'a> Iterator for StringListIter<'s, 'a> {
    type Item = (Address, String);
    fn next(&mut self) -> Option<Self::Item> {
        let _ = &self.list;
        None
    }
}
