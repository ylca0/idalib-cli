use bitflags::bitflags;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::func::{Function, FunctionId};
use crate::Address;

#[derive(Debug)]
pub struct IDB {
    pub(crate) path: PathBuf,
    pub(crate) save: bool,
    pub(crate) _marker: PhantomData<()>,
}

impl IDB {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, crate::IDAError> {
        Ok(Self {
            path: path.as_ref().to_owned(),
            save: true,
            _marker: PhantomData,
        })
    }
    pub fn open_with(
        path: impl AsRef<Path>,
        _auto_analyse: bool,
        save: bool,
    ) -> Result<Self, crate::IDAError> {
        Ok(Self {
            path: path.as_ref().to_owned(),
            save,
            _marker: PhantomData,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn save_on_close(&mut self, status: bool) {
        self.save = status;
    }
    pub fn auto_wait(&mut self) -> bool {
        true
    }
    pub fn set_screen_address(&mut self, _ea: Address) {}
    pub fn make_signatures(&mut self, _only_pat: bool) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn decompiler_available(&self) -> bool {
        true
    }
    pub fn meta(&self) -> crate::meta::Metadata {
        crate::meta::Metadata(PhantomData)
    }
    pub fn meta_mut(&mut self) -> crate::meta::MetadataMut {
        crate::meta::MetadataMut(PhantomData)
    }
    pub fn processor(&self) -> crate::processor::Processor {
        crate::processor::Processor(PhantomData)
    }
    pub fn entries(&self) -> EntryPointIter {
        EntryPointIter {
            index: 0,
            limit: 0,
            _marker: PhantomData,
        }
    }
    pub fn function_at(&self, _ea: Address) -> Option<Function> {
        None
    }
    pub fn next_head(&self, _ea: Address) -> Option<Address> {
        None
    }
    pub fn next_head_with(&self, _ea: Address, _max_ea: Address) -> Option<Address> {
        None
    }
    pub fn prev_head(&self, _ea: Address) -> Option<Address> {
        None
    }
    pub fn prev_head_with(&self, _ea: Address, _min_ea: Address) -> Option<Address> {
        None
    }
    pub fn insn_at(&self, _ea: Address) -> Option<crate::insn::Insn> {
        None
    }
    pub fn decompile<'a>(
        &'a self,
        _f: &Function<'a>,
    ) -> Result<crate::decompiler::CFunction<'a>, crate::IDAError> {
        Ok(crate::decompiler::CFunction(PhantomData))
    }
    pub fn decompile_with<'a>(
        &'a self,
        _f: &Function<'a>,
        _all_blocks: bool,
    ) -> Result<crate::decompiler::CFunction<'a>, crate::IDAError> {
        Ok(crate::decompiler::CFunction(PhantomData))
    }
    pub fn function_by_id(&self, _id: FunctionId) -> Option<Function> {
        None
    }
    pub fn functions<'a>(&'a self) -> impl Iterator<Item = (FunctionId, Function)> {
        std::iter::empty()
    }
    pub fn function_count(&self) -> usize {
        0
    }
    pub fn segment_at(&self, _ea: Address) -> Option<crate::segment::Segment> {
        None
    }
    pub fn segment_by_id(&self, _id: crate::segment::SegmentId) -> Option<crate::segment::Segment> {
        None
    }
    pub fn segment_by_name(&self, _name: impl AsRef<str>) -> Option<crate::segment::Segment> {
        None
    }
    pub fn segments<'a>(
        &'a self,
    ) -> impl Iterator<Item = (crate::segment::SegmentId, crate::segment::Segment)> {
        std::iter::empty()
    }
    pub fn segment_count(&self) -> usize {
        0
    }
    pub fn register_by_name(&self, _name: impl AsRef<str>) -> Option<crate::insn::Register> {
        None
    }
    pub fn insn_alignment_at(&self, _ea: Address) -> Option<usize> {
        None
    }
    pub fn first_xref_from(
        &self,
        _ea: Address,
        _flags: crate::xref::XRefQuery,
    ) -> Option<crate::xref::XRef> {
        None
    }
    pub fn first_xref_to(
        &self,
        _ea: Address,
        _flags: crate::xref::XRefQuery,
    ) -> Option<crate::xref::XRef> {
        None
    }
    pub fn get_cmt(&self, _ea: Address) -> Option<String> {
        None
    }
    pub fn get_cmt_with(&self, _ea: Address, _rptble: bool) -> Option<String> {
        None
    }
    pub fn set_cmt(&self, _ea: Address, _comm: impl AsRef<str>) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn set_cmt_with(
        &self,
        _ea: Address,
        _comm: impl AsRef<str>,
        _rptble: bool,
    ) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn append_cmt(&self, _ea: Address, _comm: impl AsRef<str>) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn append_cmt_with(
        &self,
        _ea: Address,
        _comm: impl AsRef<str>,
        _rptble: bool,
    ) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn remove_cmt(&self, _ea: Address) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn remove_cmt_with(&self, _ea: Address, _rptble: bool) -> Result<(), crate::IDAError> {
        Ok(())
    }
    pub fn bookmarks(&self) -> crate::bookmarks::Bookmarks {
        crate::bookmarks::Bookmarks(PhantomData)
    }
    pub fn find_text(&self, _start_ea: Address, _text: impl AsRef<str>) -> Option<Address> {
        None
    }
    pub fn find_text_iter<'a, T>(&'a self, _text: T) -> impl Iterator<Item = Address>
    where
        T: AsRef<str>,
    {
        std::iter::empty()
    }
    pub fn find_imm(&self, _start_ea: Address, _imm: u32) -> Option<Address> {
        None
    }
    pub fn find_imm_iter<'a>(&'a self, _imm: u32) -> impl Iterator<Item = Address> {
        std::iter::empty()
    }
    pub fn find_defined(&self, _start_ea: Address) -> Option<Address> {
        None
    }
    pub fn strings(&self) -> crate::strings::StringList {
        crate::strings::StringList(PhantomData)
    }
    pub fn names(&self) -> crate::name::NameList {
        crate::name::NameList(PhantomData)
    }
    pub fn address_to_string(&self, _ea: Address) -> Option<String> {
        None
    }
}

pub struct EntryPointIter<'a> {
    index: usize,
    limit: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for EntryPointIter<'a> {
    type Item = Address;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct IDBOpenOptions {
    idb: Option<PathBuf>,
    save: bool,
    auto_analyse: bool,
}

impl Default for IDBOpenOptions {
    fn default() -> Self {
        Self {
            idb: None,
            save: false,
            auto_analyse: true,
        }
    }
}

impl IDBOpenOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn idb(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.idb = Some(path.as_ref().to_owned());
        self
    }
    pub fn save(&mut self, save: bool) -> &mut Self {
        self.save = save;
        self
    }
    pub fn auto_analyse(&mut self, auto_analyse: bool) -> &mut Self {
        self.auto_analyse = auto_analyse;
        self
    }
    pub fn open(&self, path: impl AsRef<Path>) -> Result<IDB, crate::IDAError> {
        IDB::open_with(path, self.auto_analyse, self.save)
    }
}
