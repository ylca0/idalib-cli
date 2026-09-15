//! Direct FFI extensions to the IDA kernel (B-tier capabilities).
//!
//! These symbols are exported by the user's IDA installation (`libida.dylib`
//! / `libida.so` / `ida.dll`) but are not in `idalib-sys`'s autocxx
//! allowlist for 0.6.1. We declare them by hand and link via the same
//! `-l ida` + RPATH that `idalib-build` sets up.
//!
//! ABI notes: these are classic plain-C APIs (pointer + scalars only), which
//! are stable across IDA 9.x. Re-verify signatures against the SDK headers
//! when bumping the IDA version.
//!
//! Under the `stub-idalib` feature (SDK-free dev builds) the functions are
//! no-ops so everything keeps linking.

use std::os::raw::{c_char, c_int, c_uchar};

/// `idaman bool ida_export set_name(ea_t ea, const char *name, int flags=0);`
/// flags == 0 -> SN_CHECK (validate identifier).
///
/// # Safety
/// `name` must point to a valid NUL-terminated C string.
#[cfg(not(feature = "stub-idalib"))]
pub unsafe fn set_name(ea: u64, name: *const c_char, flags: c_int) -> bool {
    unsafe { raw::set_name(ea, name, flags) }
}

/// Stub no-op: always reports failure.
///
/// # Safety
/// Arguments are ignored; provided for signature parity with the real FFI.
#[cfg(feature = "stub-idalib")]
pub unsafe fn set_name(_ea: u64, _name: *const c_char, _flags: c_int) -> bool {
    false
}

/// `idaman ea_t ida_export bin_search(ea_t start, ea_t end, const uchar *data,
///                                     uchar *mask, size_t len, int flags);`
/// flags == 0 -> BIN_SEARCH_FORWARD.
///
/// # Safety
/// `data` (and `mask` when non-null) must reference `len` readable bytes.
#[cfg(not(feature = "stub-idalib"))]
pub unsafe fn bin_search(
    start: u64,
    end: u64,
    data: *const c_uchar,
    mask: *const c_uchar,
    len: usize,
    flags: c_int,
) -> u64 {
    unsafe { raw::bin_search(start, end, data, mask, len, flags) }
}

/// Stub no-op: reports "not found".
///
/// # Safety
/// Arguments are ignored; provided for signature parity with the real FFI.
#[cfg(feature = "stub-idalib")]
pub unsafe fn bin_search(
    #[allow(unused_variables)] _start: u64,
    _end: u64,
    _data: *const c_uchar,
    _mask: *const c_uchar,
    _len: usize,
    _flags: c_int,
) -> u64 {
    u64::MAX
}

/// `idaman bool ida_export apply_cdecl(til_t *til, ea_t ea,
///                                     const char *decl, int flags=0);`
/// flags == 0 -> TINFO_DEFINITE (always passed upstream).
/// `til` is the global type library handle from `get_idati()`.
#[cfg(not(feature = "stub-idalib"))]
pub unsafe fn apply_cdecl(ea: u64, decl: *const c_char, flags: c_int) -> bool {
    unsafe { raw::apply_cdecl(raw::get_idati(), ea, decl, flags) }
}

/// Stub no-op: always reports failure.
///
/// # Safety
/// Arguments are ignored; provided for signature parity with the real FFI.
#[cfg(feature = "stub-idalib")]
pub unsafe fn apply_cdecl(_ea: u64, _decl: *const c_char, _flags: c_int) -> bool {
    false
}

/// Raw C layout of the SDK `xrefblk_t` (fields exactly as in xref.hpp).
#[cfg(not(feature = "stub-idalib"))]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XrefBlk {
    pub from: u64,
    pub to: u64,
    pub iscode: bool,
    pub type_: u8,
    pub user: bool,
    pub flags: u8,
}

#[cfg(not(feature = "stub-idalib"))]
pub unsafe fn xrefblk_first_from(blk: *mut XrefBlk, from: u64, flags: i32) -> bool {
    unsafe { raw::xrefblk_t_first_from(blk, from, flags) }
}

#[cfg(not(feature = "stub-idalib"))]
pub unsafe fn xrefblk_next_from(blk: *mut XrefBlk) -> bool {
    unsafe { raw::xrefblk_t_next_from(blk) }
}

#[cfg(not(feature = "stub-idalib"))]
mod raw {
    use std::os::raw::{c_char, c_int, c_uchar, c_void};

    unsafe extern "C" {
        pub fn set_name(ea: u64, name: *const c_char, flags: c_int) -> bool;
        pub fn bin_search(
            start: u64,
            end: u64,
            data: *const c_uchar,
            mask: *const c_uchar,
            len: usize,
            flags: c_int,
        ) -> u64;
        pub fn xrefblk_t_first_from(blk: *mut super::XrefBlk, from: u64, flags: c_int) -> bool;
        pub fn xrefblk_t_next_from(blk: *mut super::XrefBlk) -> bool;
        pub fn get_idati() -> *mut c_void;
        pub fn apply_cdecl(til: *mut c_void, ea: u64, decl: *const c_char, flags: c_int) -> bool;
    }
}
