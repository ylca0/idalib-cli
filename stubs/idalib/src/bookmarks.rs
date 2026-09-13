use std::marker::PhantomData;

use crate::Address;

pub type BookmarkIndex = u32;

pub struct Bookmarks<'a>(pub(crate) PhantomData<&'a ()>);

impl<'a> Bookmarks<'a> {
    pub fn mark(
        &self,
        _ea: Address,
        _desc: impl AsRef<str>,
    ) -> Result<BookmarkIndex, crate::IDAError> {
        Ok(0)
    }
    pub fn mark_with(
        &self,
        _ea: Address,
        _desc: impl AsRef<str>,
        _repeatable: bool,
    ) -> Result<BookmarkIndex, crate::IDAError> {
        Ok(0)
    }
    pub fn get_description(&self, _ea: Address) -> Option<String> {
        None
    }
    pub fn get_address(&self, _idx: BookmarkIndex) -> Option<Address> {
        None
    }
    pub fn get_description_by_index(&self, _idx: BookmarkIndex) -> Option<String> {
        None
    }
    pub fn erase(&self, _ea: Address) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn erase_by_index(&self, _idx: BookmarkIndex) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn find_index(&self, _ea: Address) -> Option<BookmarkIndex> {
        None
    }
    pub fn len(&self) -> BookmarkIndex {
        0
    }
    pub fn is_empty(&self) -> bool {
        true
    }
}
