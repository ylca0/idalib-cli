use serde::Serialize;

use crate::idalib::{
    func::{BasicBlock, FunctionCFG},
    insn::{Insn, Operand, OperandDataType, OperandType},
    segment::Segment,
    xref::{CodeRef, DataRef, XRef, XRefType},
};

pub const FORMAT_VERSION: u8 = 1;

#[derive(Serialize, Debug)]
pub struct VersionInfo {
    pub idalib_cli: String,
    pub idalib_rs: String,
    pub ida_version: Option<String>,
    pub ida_install_dir: Option<String>,
    pub license: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Error {
    pub code: String,
    pub message: String,
}

impl Error {
    pub fn from_anyhow(e: &anyhow::Error) -> Self {
        Self {
            code: "error".to_string(),
            message: format!("{e:#}"),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct InsnView {
    pub address: String,
    pub size: usize,
    pub operand_count: usize,
    pub operands: Vec<OperandView>,
}

impl InsnView {
    pub fn from_insn(i: &Insn) -> Self {
        let operands = (0..i.operand_count())
            .filter_map(|n| i.operand(n))
            .map(|o| OperandView::from_operand(&o))
            .collect();
        Self {
            address: format!("0x{:x}", i.address()),
            size: i.len(),
            operand_count: i.operand_count(),
            operands,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct OperandView {
    pub index: usize,
    pub kind: String,
    pub reg: Option<String>,
    pub value: Option<String>,
    pub address: Option<String>,
    pub dtype: String,
}

impl OperandView {
    pub fn from_operand(o: &Operand) -> Self {
        let kind = match o.type_() {
            OperandType::Reg => "register".to_string(),
            OperandType::Imm => "immediate".to_string(),
            OperandType::Displ => "displacement".to_string(),
            OperandType::Near => "near".to_string(),
            OperandType::Far => "far".to_string(),
            OperandType::Phrase => "phrase".to_string(),
            OperandType::Mem => "memory".to_string(),
            OperandType::IdpSpec0 => "idpspec0".to_string(),
            OperandType::IdpSpec1 => "idpspec1".to_string(),
            OperandType::IdpSpec2 => "idpspec2".to_string(),
            OperandType::IdpSpec3 => "idpspec3".to_string(),
            OperandType::IdpSpec4 => "idpspec4".to_string(),
            OperandType::IdpSpec5 => "idpspec5".to_string(),
        };
        let dtype = match o.dtype() {
            OperandDataType::Byte => "byte".to_string(),
            OperandDataType::Word => "word".to_string(),
            OperandDataType::DWord => "dword".to_string(),
            OperandDataType::QWord => "qword".to_string(),
            OperandDataType::Float => "float".to_string(),
            OperandDataType::Double => "double".to_string(),
            OperandDataType::TByte => "tbyte".to_string(),
            OperandDataType::PackReal => "packed_real".to_string(),
            OperandDataType::Byte16 => "byte16".to_string(),
            OperandDataType::Code => "code".to_string(),
            OperandDataType::Void => "void".to_string(),
            OperandDataType::FWord => "fword".to_string(),
            OperandDataType::Bitfield => "bitfield".to_string(),
            OperandDataType::String => "string".to_string(),
            OperandDataType::Unicode => "unicode".to_string(),
            OperandDataType::LongDouble => "long_double".to_string(),
            OperandDataType::Byte32 => "byte32".to_string(),
            OperandDataType::Byte64 => "byte64".to_string(),
            OperandDataType::Half => "half".to_string(),
        };
        Self {
            index: o.n(),
            kind,
            reg: o.reg().map(|r| format!("r{r}")),
            value: o.value().map(|v| format!("0x{v:x}")),
            address: o.address().map(|a| format!("0x{a:x}")),
            dtype,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct SegmentView {
    pub id: usize,
    pub start: String,
    pub end: String,
    pub size: usize,
    pub name: Option<String>,
    pub alignment: String,
    pub permissions: String,
    pub bitness: usize,
    pub kind: String,
    pub is_hidden: bool,
    pub is_16bit: bool,
    pub is_32bit: bool,
    pub is_64bit: bool,
    pub is_code: bool,
    pub is_data: bool,
    pub is_import: bool,
}

impl SegmentView {
    pub fn from_segment(id: usize, s: &Segment) -> Self {
        let kind = if s.r#type().is_code() {
            "code"
        } else if s.r#type().is_data() {
            "data"
        } else if s.r#type().is_import() {
            "import"
        } else {
            "other"
        };
        Self {
            id,
            start: format!("0x{:x}", s.start_address()),
            end: format!("0x{:x}", s.end_address()),
            size: s.len(),
            name: s.name(),
            alignment: format!("{:?}", s.alignment()),
            permissions: format!("{:?}", s.permissions()),
            bitness: s.bitness(),
            kind: kind.to_string(),
            is_hidden: s.is_hidden(),
            is_16bit: s.is_16bit(),
            is_32bit: s.is_32bit(),
            is_64bit: s.is_64bit(),
            is_code: s.r#type().is_code(),
            is_data: s.r#type().is_data(),
            is_import: s.r#type().is_import(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct FunctionView {
    pub id: usize,
    pub start: String,
    pub end: String,
    pub size: usize,
    pub name: Option<String>,
    pub flags: u64,
    pub is_lib: bool,
    pub is_thunk: bool,
    pub is_tail: bool,
    pub is_far: bool,
    pub does_return: bool,
    pub blocks: Option<usize>,
    pub decompiled: bool,
    pub pseudocode: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct BasicBlockView {
    pub id: usize,
    pub start: String,
    pub end: String,
    pub size: usize,
    pub is_normal: bool,
    pub is_ret: bool,
    pub is_cndret: bool,
    pub is_noret: bool,
    pub is_enoret: bool,
    pub is_extern: bool,
    pub is_error: bool,
    pub is_indjump: bool,
    pub preds: Vec<String>,
    pub succs: Vec<String>,
}

impl BasicBlockView {
    pub fn from_block(blk: &BasicBlock, cfg: &FunctionCFG) -> Self {
        Self {
            id: 0,
            start: format!("0x{:x}", blk.start_address()),
            end: format!("0x{:x}", blk.end_address()),
            size: blk.len(),
            is_normal: blk.is_normal(),
            is_ret: blk.is_ret(),
            is_cndret: blk.is_cndret(),
            is_noret: blk.is_noret(),
            is_enoret: blk.is_enoret(),
            is_extern: blk.is_extern(),
            is_error: blk.is_error(),
            is_indjump: blk.is_indjump(),
            preds: blk
                .preds_with(cfg)
                .map(|b| format!("0x{:x}", b.start_address()))
                .collect(),
            succs: blk
                .succs_with(cfg)
                .map(|b| format!("0x{:x}", b.start_address()))
                .collect(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct FunctionDetailView {
    pub function: FunctionView,
    pub blocks: Vec<BasicBlockView>,
    pub entry: Option<String>,
    pub exit: Option<String>,
    pub xrefs_to: Vec<XRefView>,
}

#[derive(Serialize, Debug)]
pub struct XRefView {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub is_code: bool,
    pub is_data: bool,
    pub is_user_defined: bool,
}

impl XRefView {
    pub fn from_xref(x: &XRef) -> Self {
        let kind = match x.type_() {
            XRefType::Code(c) => match c {
                CodeRef::NearCall => "near_call",
                CodeRef::FarCall => "far_call",
                CodeRef::NearJump => "near_jump",
                CodeRef::FarJump => "far_jump",
                CodeRef::Flow => "flow",
                CodeRef::Unknown => "code_unknown",
                CodeRef::Obsolete => "code_obsolete",
            }
            .to_string(),
            XRefType::Data(d) => match d {
                DataRef::Offset => "offset",
                DataRef::Write => "write",
                DataRef::Read => "read",
                DataRef::Text => "text",
                DataRef::Informational => "informational",
                DataRef::EnumMember => "enum_member",
                DataRef::Unknown => "data_unknown",
            }
            .to_string(),
        };
        Self {
            from: format!("0x{:x}", x.from()),
            to: format!("0x{:x}", x.to()),
            kind,
            is_code: x.is_code(),
            is_data: x.is_data(),
            is_user_defined: x.is_user_defined(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct NameView {
    pub address: String,
    pub name: String,
    pub is_public: bool,
    pub is_weak: bool,
}

#[derive(Serialize, Debug)]
pub struct StringView {
    pub index: usize,
    pub address: String,
    pub value: String,
}

#[derive(Serialize, Debug)]
pub struct MetadataView {
    pub procname: Option<String>,
    pub filetype: Option<String>,
    pub compiler: Option<String>,
    pub bitness: Option<u32>,
    pub app_bitness: Option<u32>,
    pub is_dll: Option<bool>,
    pub is_be: Option<bool>,
    pub is_16bit: Option<bool>,
    pub is_32bit: Option<bool>,
    pub is_64bit: Option<bool>,
    pub database_change_count: Option<u32>,
    pub is_auto_enabled: Option<bool>,
    pub is_readonly_idb: Option<bool>,
    pub is_kernel_mode: Option<bool>,
    pub is_snapshot: Option<bool>,
}

#[derive(Serialize, Debug)]
pub struct EntryPointView {
    pub ordinal: usize,
    pub address: String,
}

#[derive(Serialize, Debug)]
pub struct BookmarkView {
    pub index: u32,
    pub address: String,
    pub description: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum BookmarksOut {
    One(BookmarkView),
    Many(Vec<BookmarkView>),
}

#[derive(Serialize, Debug)]
pub struct CommentView {
    pub address: String,
    pub comment: String,
    pub repeatable: bool,
}

#[derive(Serialize, Debug)]
pub struct SessionView {
    pub id: String,
    pub name: String,
    pub state: String,
    pub created_at: String,
    pub idb: Option<String>,
    pub binary: Option<String>,
    pub auto_analyse: Option<bool>,
    pub save: Option<bool>,
    pub ready: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct OkView {
    pub ok: bool,
    pub detail: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct IdbView {
    pub path: String,
    pub save_on_close: bool,
    pub auto_analyse: bool,
}

#[derive(Serialize, Debug)]
pub struct ProcessorView {
    pub family: String,
    pub short_name: String,
    pub long_name: String,
}

#[derive(Serialize, Debug)]
pub struct LicenseView {
    pub valid: bool,
    pub id: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct VersionView {
    pub major: i32,
    pub minor: i32,
    pub build: i32,
    pub string: String,
}

#[derive(Serialize, Debug)]
pub struct CmtView {
    pub address: String,
    pub comment: String,
    pub repeatable: bool,
}
