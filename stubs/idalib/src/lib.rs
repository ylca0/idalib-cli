//! DEV-ONLY stub of the `idalib` crate.
//!
//! Mirrors the public API of idalib-rs 0.6.1 (IDA Pro 9.1) with enough shape
//! for `idalib-cli` to typecheck and unit-test without the IDA SDK installed.
//! This crate is NEVER compiled into release builds of `idalib-cli`.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(clippy::all)]
#![allow(clippy::needless_lifetimes)]
#![allow(mismatched_lifetime_syntaxes)]
#![allow(elided_lifetimes_in_paths)]

pub mod bookmarks;
pub mod decompiler;
pub mod func;
pub mod idb;
pub mod insn;
pub mod license;
pub mod meta;
pub mod name;
pub mod processor;
pub mod segment;
pub mod strings;
pub mod xref;

pub use idb::{IDBOpenOptions, IDB};
pub use license::{is_valid_license, license_id, LicenseId};

pub type Address = u64;

#[derive(Debug)]
pub struct IDAVersion {
    pub major: i32,
    pub minor: i32,
    pub build: i32,
}

impl IDAVersion {
    pub fn major(&self) -> i32 {
        self.major
    }
    pub fn minor(&self) -> i32 {
        self.minor
    }
    pub fn build(&self) -> i32 {
        self.build
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IDAError {
    #[error("ffi error: {0}")]
    Ffi(String),
    #[error("could not initialise IDA")]
    Init(i32),
    #[error("input file `{0}` not found")]
    FileNotFound(String),
    #[error("could not open IDA database: {0:x}")]
    OpenDb(i32),
    #[error("could not close IDA database: {0:x}")]
    CloseDb(i32),
    #[error("invalid license")]
    InvalidLicense,
    #[error("could not generate pattern or signature files")]
    MakeSigs,
    #[error("could not get library version")]
    GetVersion,
    #[error("no decompiler available")]
    NoDecompiler,
}

pub fn force_batch_mode() {}
pub fn init_library() -> &'static std::sync::Mutex<()> {
    static M: std::sync::Mutex<()> = std::sync::Mutex::new(());
    &M
}
pub fn enable_console_messages(_enabled: bool) {}
pub fn version() -> Result<IDAVersion, IDAError> {
    Ok(IDAVersion {
        major: 9,
        minor: 1,
        build: 250226,
    })
}

pub struct AddressFlags<'a>(std::marker::PhantomData<&'a ()>);
impl<'a> AddressFlags<'a> {
    pub fn is_code(&self) -> bool {
        false
    }
    pub fn is_data(&self) -> bool {
        false
    }
}

pub mod ffi {
    use std::os::raw::{c_char, c_int, c_uchar};

    pub mod entry {
        pub unsafe fn get_entry_qty() -> usize {
            0
        }
        pub unsafe fn get_entry_ordinal(_idx: usize) -> std::ffi::c_ulonglong {
            0
        }
        pub unsafe fn get_entry(_ord: std::ffi::c_ulonglong) -> std::ffi::c_ulonglong {
            0
        }
    }

    pub mod xref {
        use std::os::raw::c_int;
        pub const XREF_ALL: i32 = 0;
        pub const XREF_FAR: i32 = 1;
        pub const XREF_DATA: i32 = 2;

        #[derive(Debug, Clone, Copy)]
        #[repr(C)]
        pub struct xrefblk_t {
            pub from: u64,
            pub to: u64,
            pub iscode: bool,
            pub type_: i32,
            pub user: bool,
            pub _flags: c_int,
        }

        unsafe extern "C" {
            pub fn xrefblk_t_first_from(xb: *mut xrefblk_t, ea: u64, flags: c_int) -> bool;
            pub fn xrefblk_t_next_from(xb: *mut xrefblk_t) -> bool;
        }
    }

    pub mod search {
        use std::os::raw::c_char;
        pub unsafe fn idalib_find_text(
            _ea: std::ffi::c_ulonglong,
            _t: *const c_char,
        ) -> std::ffi::c_ulonglong {
            std::ffi::c_ulonglong::MAX
        }
        pub unsafe fn idalib_find_imm(
            _ea: std::ffi::c_ulonglong,
            _i: std::ffi::c_uint,
        ) -> std::ffi::c_ulonglong {
            std::ffi::c_ulonglong::MAX
        }
        pub unsafe fn idalib_find_defined(_ea: std::ffi::c_ulonglong) -> std::ffi::c_ulonglong {
            std::ffi::c_ulonglong::MAX
        }
    }

    pub mod bytes {
        pub unsafe fn idalib_get_byte(_ea: std::ffi::c_ulonglong) -> u8 {
            0
        }
        pub unsafe fn idalib_get_word(_ea: std::ffi::c_ulonglong) -> u16 {
            0
        }
        pub unsafe fn idalib_get_dword(_ea: std::ffi::c_ulonglong) -> u32 {
            0
        }
        pub unsafe fn idalib_get_qword(_ea: std::ffi::c_ulonglong) -> u64 {
            0
        }
        pub unsafe fn idalib_get_bytes(_ea: std::ffi::c_ulonglong, _buf: &mut Vec<u8>) -> usize {
            0
        }
        pub unsafe fn get_flags(_ea: std::ffi::c_ulonglong) -> u64 {
            0
        }
        pub unsafe fn is_code(_ea: std::ffi::c_ulonglong) -> bool {
            false
        }
        pub unsafe fn is_data(_ea: std::ffi::c_ulonglong) -> bool {
            false
        }
    }

    pub mod util {
        use std::os::raw::{c_int, c_uchar};
        pub unsafe fn is_call_insn(_ea: std::ffi::c_ulonglong) -> bool {
            false
        }
        pub unsafe fn is_ret_insn(_ea: std::ffi::c_ulonglong, _strict: c_uchar) -> bool {
            false
        }
        pub unsafe fn is_indirect_jump_insn(_ea: std::ffi::c_ulonglong) -> bool {
            false
        }
        pub unsafe fn is_align_insn(_ea: std::ffi::c_ulonglong) -> c_int {
            0
        }
        pub unsafe fn next_head(
            _ea: std::ffi::c_ulonglong,
            _m: std::ffi::c_ulonglong,
        ) -> std::ffi::c_ulonglong {
            std::ffi::c_ulonglong::MAX
        }
        pub unsafe fn prev_head(
            _ea: std::ffi::c_ulonglong,
            _m: std::ffi::c_ulonglong,
        ) -> std::ffi::c_ulonglong {
            std::ffi::c_ulonglong::MAX
        }
    }
}
