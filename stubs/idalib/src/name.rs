use std::marker::PhantomData;

use crate::Address;

pub type NameIndex = usize;

pub struct NameList<'a>(pub(crate) PhantomData<&'a ()>);

impl<'a> NameList<'a> {
    pub fn get_by_index(&self, _index: NameIndex) -> Option<Name> {
        None
    }
    pub fn get_closest_by_address(&self, _address: Address) -> Option<Name> {
        None
    }
    pub fn get_address_by_index(&self, _index: NameIndex) -> Option<Address> {
        None
    }
    pub fn has_name(&self, _address: Address) -> bool {
        false
    }
    pub fn len(&self) -> usize {
        0
    }
    pub fn is_empty(&self) -> bool {
        true
    }
    pub fn iter(&self) -> NameListIter<'_, 'a> {
        NameListIter {
            list: self,
            current: 0,
        }
    }
}

pub struct Name {
    address: Address,
    name: String,
    is_public: bool,
    is_weak: bool,
}

impl Name {
    pub fn address(&self) -> Address {
        self.address
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_public(&self) -> bool {
        self.is_public
    }
    pub fn is_weak(&self) -> bool {
        self.is_weak
    }
}

pub struct NameListIter<'s, 'a> {
    list: &'s NameList<'a>,
    current: usize,
}

impl<'s, 'a> Iterator for NameListIter<'s, 'a> {
    type Item = Name;
    fn next(&mut self) -> Option<Self::Item> {
        let _ = &self.list;
        None
    }
}
