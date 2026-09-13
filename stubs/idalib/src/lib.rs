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
}
