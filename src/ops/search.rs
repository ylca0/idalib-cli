//! Search, raw-byte access and renaming.
//!
//! A 档能力走 `idalib::ffi` 的公开模块（search/bytes/util）；B 档（set_name、
//! bin_search）通过 `crate::ffi_ext` 手写 extern 声明直连 IDA 内核导出符号。

use std::ffi::CString;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::idalib::{Address, ffi, idb::IDB};

// Normalise between the real FFI (autocxx newtypes exposing `.0`) and the
// dev stub (plain ints) at the call sites below.
#[cfg(feature = "stub-idalib")]
pub mod conv {
    pub fn a64(v: u64) -> u64 {
        v
    }
    pub fn i32v(v: i32) -> i32 {
        v
    }
    pub fn ea_in(v: u64) -> u64 {
        v
    }
    pub fn u32v(v: u64) -> u32 {
        v as u32
    }
    pub fn iflags(v: i32) -> i32 {
        v
    }
}

#[cfg(not(feature = "stub-idalib"))]
pub mod conv {
    // autocxx ctype newtypes: `pub struct c_x(pub std::os::raw::c_x)` with
    // `From<c_x>`, publicly reachable via the `autocxx` crate itself.
    pub fn a64(v: autocxx::c_ulonglong) -> u64 {
        u64::from(v)
    }
    pub fn i32v(v: autocxx::c_int) -> i32 {
        i32::from(v)
    }
    pub fn ea_in(v: u64) -> autocxx::c_ulonglong {
        autocxx::c_ulonglong::from(v)
    }
    pub fn u32v(v: u64) -> autocxx::c_uint {
        autocxx::c_uint::from(v as u32)
    }
    pub fn iflags(v: i32) -> autocxx::c_int {
        autocxx::c_int::from(v)
    }
}

use crate::ffi_ext;
use crate::helpers::json_types as jt;
use crate::ops::top::Out;

// ---------------------------------------------------------------------------
// find
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug)]
pub struct FindHit {
    pub address: String,
    pub kind: String,
    pub detail: String,
}

/// Search text (upstream find_text semantics), an immediate, or a byte
/// pattern. Iterates from `start` to the end of the database, collecting up
/// to `max` hits.
pub fn find(
    idb: &IDB,
    text: Option<&str>,
    imm: Option<u64>,
    pattern: Option<&str>,
    start: Option<Address>,
    max: usize,
) -> Result<Out> {
    let matches = count(text, imm, pattern)?;
    if matches != 1 {
        bail!("specify exactly one of --text, --imm or --pattern");
    }

    let mut hits = Vec::new();
    match (text, imm, pattern) {
        (Some(t), _, _) => {
            let ctext = CString::new(t).context("text contains NUL")?;
            let mut ea = start.unwrap_or(min_addr(idb));
            loop {
                let found =
                    unsafe { ffi::search::idalib_find_text(conv::ea_in(ea), ctext.as_ptr()) };
                let f64v = conv::a64(found);
                if f64v == u64::MAX {
                    break;
                }
                let addr: Address = f64v;
                hits.push(FindHit {
                    address: format!("0x{addr:x}"),
                    kind: "text".into(),
                    detail: t.to_string(),
                });
                if hits.len() >= max {
                    break;
                }
                ea = addr + 1;
            }
        }
        (_, Some(v), _) => {
            let mut ea = start.unwrap_or(min_addr(idb));
            loop {
                let found = unsafe { ffi::search::idalib_find_imm(conv::ea_in(ea), conv::u32v(v)) };
                let f64v = conv::a64(found);
                if f64v == u64::MAX {
                    break;
                }
                let addr: Address = f64v;
                hits.push(FindHit {
                    address: format!("0x{addr:x}"),
                    kind: "imm".into(),
                    detail: format!("0x{v:x}"),
                });
                if hits.len() >= max {
                    break;
                }
                ea = addr + 1;
            }
        }
        (_, _, Some(p)) => {
            let bytes = parse_hex_pattern(p)?;
            let end = max_addr(idb);
            let mut ea = start.unwrap_or(min_addr(idb));
            while ea < end && hits.len() < max {
                let found = unsafe {
                    ffi_ext::bin_search(ea, end, bytes.as_ptr(), std::ptr::null(), bytes.len(), 0)
                };
                if found == u64::MAX {
                    break;
                }
                let addr: Address = found;
                hits.push(FindHit {
                    address: format!("0x{addr:x}"),
                    kind: "bytes".into(),
                    detail: p.to_string(),
                });
                ea = addr + bytes.len() as Address;
            }
        }
        _ => unreachable!(),
    }

    Ok(Out::Value(serde_json::json!({
        "query": {
            "text": text, "imm": imm.map(|v| format!("0x{v:x}")), "pattern": pattern,
        },
        "count": hits.len(),
        "hits": hits,
    })))
}

fn count(text: Option<&str>, imm: Option<u64>, pattern: Option<&str>) -> Result<usize> {
    Ok(usize::from(text.is_some()) + usize::from(imm.is_some()) + usize::from(pattern.is_some()))
}

fn parse_hex_pattern(p: &str) -> Result<Vec<u8>> {
    let hex: String = p.chars().filter(|c| !c.is_whitespace()).collect();
    if hex.len() % 2 != 0 {
        bail!("hex pattern must have an even number of digits");
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| anyhow::anyhow!("bad hex in pattern: {e}"))
        })
        .collect()
}

fn min_addr(idb: &IDB) -> Address {
    idb.segments()
        .map(|(_, s)| s.start_address())
        .min()
        .unwrap_or(0)
}

fn max_addr(idb: &IDB) -> Address {
    idb.segments()
        .map(|(_, s)| s.end_address())
        .max()
        .unwrap_or(u64::MAX)
}

// ---------------------------------------------------------------------------
// bytes
// ---------------------------------------------------------------------------

/// Dump raw bytes (hexdump) or little-endian integers at an address.
pub fn bytes(_idb: &IDB, ea: Address, count: usize, width: Option<&str>) -> Result<Out> {
    let width = width.unwrap_or("byte");
    match width {
        "byte" => {
            // The C++ bridge reads buf.data()/capacity(), so the Vec must be
            // pre-sized before the call.
            let mut buf: Vec<u8> = vec![0u8; count];
            #[cfg(not(feature = "stub-idalib"))]
            let n: usize =
                unsafe { ffi::bytes::idalib_get_bytes(conv::ea_in(ea), &mut buf) }.unwrap_or(0);
            #[cfg(feature = "stub-idalib")]
            let n: usize = unsafe { ffi::bytes::idalib_get_bytes(conv::ea_in(ea), &mut buf) };
            buf.truncate(n.min(count));
            if buf.is_empty() {
                bail!("no readable bytes at {ea:#x}");
            }
            let hexdump: Vec<String> = buf
                .chunks(16)
                .enumerate()
                .map(|(i, row)| {
                    let off = ea + (i * 16) as Address;
                    let hexcells: Vec<String> = row.iter().map(|b| format!("{b:02x}")).collect();
                    let ascii: String = row
                        .iter()
                        .map(|b| {
                            if (0x20..0x7f).contains(b) {
                                *b as char
                            } else {
                                '.'
                            }
                        })
                        .collect();
                    format!("0x{off:04x}: {:<47}  {ascii}", hexcells.join(" "))
                })
                .collect();
            Ok(Out::Value(serde_json::json!({
                "address": format!("0x{ea:x}"),
                "width": "byte",
                "count": buf.len(),
                "hex": hexdump.join("\n"),
                "bytes": buf.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>(),
            })))
        }
        w => {
            let read_one: fn(Address) -> u64 = match w {
                "word" => |a| unsafe { ffi::bytes::idalib_get_word(conv::ea_in(a)) as u64 },
                "dword" => |a| unsafe { ffi::bytes::idalib_get_dword(conv::ea_in(a)) as u64 },
                "qword" => |a| unsafe { ffi::bytes::idalib_get_qword(conv::ea_in(a)) },
                _ => bail!("invalid width {w}"),
            };
            let step: Address = match w {
                "word" => 2,
                "dword" => 4,
                _ => 8,
            };
            let mut values = Vec::new();
            for i in 0..count {
                let a = ea + i as Address * step;
                let v = read_one(a);
                values.push(serde_json::json!({
                    "address": format!("0x{a:x}"),
                    "value": format!("0x{v:x}"),
                }));
            }
            Ok(Out::Value(serde_json::json!({
                "address": format!("0x{ea:x}"),
                "width": w,
                "count": values.len(),
                "values": values,
            })))
        }
    }
}

// ---------------------------------------------------------------------------
// xrefs from
// ---------------------------------------------------------------------------

/// Xrefs FROM an address (outgoing references). Uses our own extern binding:
/// the upstream 0.6.1 cxx bridge for `xrefblk_t_first_from` misbehaves
/// (returns incoming refs), so we call the kernel directly.
pub fn xrefs_from(
    _idb: &IDB,
    #[cfg_attr(feature = "stub-idalib", allow(unused_variables))] ea: Address,
    all: bool,
) -> Result<Vec<jt::XRefView>> {
    #[cfg_attr(feature = "stub-idalib", allow(unused_variables))]
    let flags: i32 = if all { 0x00 } else { 0x01 }; // XREF_FLOW / XREF_NOFLOW

    #[cfg_attr(feature = "stub-idalib", allow(unused_mut))]
    let mut out = Vec::new();
    #[cfg(not(feature = "stub-idalib"))]
    unsafe {
        let mut blk = ffi_ext::XrefBlk {
            from: 0,
            to: 0,
            iscode: false,
            type_: 0,
            user: false,
            flags: 0,
        };
        let mut found = ffi_ext::xrefblk_first_from(&mut blk, ea, flags);
        while found {
            out.push(jt::XRefView {
                from: format!("0x{:x}", blk.from),
                to: format!("0x{:x}", blk.to),
                kind: xref_kind(blk.type_ as i32, blk.iscode),
                is_code: blk.iscode,
                is_data: !blk.iscode,
                is_user_defined: blk.user,
            });
            if out.len() >= 4096 {
                break; // safety valve against cyclic graphs
            }
            found = ffi_ext::xrefblk_next_from(&mut blk);
        }
    }
    Ok(out)
}

#[cfg_attr(feature = "stub-idalib", allow(dead_code))]
fn xref_kind(t: i32, iscode: bool) -> String {
    // fl_* code types and dr_* data types mirror the SDK constants.
    if iscode {
        match t {
            0x00 => "unknown",
            0x10 => "far_call",
            0x11 => "near_call",
            0x12 => "far_jump",
            0x13 => "near_jump",
            0x15 => "flow",
            0x16 => "call_supplementary",
            0x18 => "user_defined",
            _ => "code",
        }
    } else {
        match t {
            0x01 => "offset",
            0x02 => "write",
            0x03 => "read",
            0x04 => "text",
            0x05 => "informational",
            0x06 => "enum_member",
            _ => "data",
        }
    }
    .to_string()
}

// ---------------------------------------------------------------------------
// insn enhancements
// ---------------------------------------------------------------------------

/// Enrich an `InsnView` with classification flags + mnemonic group from the
/// `ffi::util` classifiers.
pub fn insn_enriched(idb: &IDB, ea: Address) -> Result<Out> {
    let i = idb.insn_at(ea).context("no instruction at address")?;
    let mut v = jt::InsnView::from_insn(&i);

    #[cfg(not(feature = "stub-idalib"))]
    let (is_call, is_ret, is_indirect, is_align) = {
        // The classifiers take a decoded `insn_t` reference, not an address.
        let insn =
            crate::idalib::ffi::insn::decode(ea.into()).context("no instruction at address")?;
        unsafe {
            (
                ffi::util::is_call_insn(&insn),
                ffi::util::is_ret_insn(&insn, 0),
                ffi::util::is_indirect_jump_insn(&insn),
                i32::from(ffi::util::is_align_insn(ea.into())) != 0,
            )
        }
    };

    #[cfg(feature = "stub-idalib")]
    let (is_call, is_ret, is_indirect, is_align) = (false, false, false, false);

    let group = if is_call {
        "call"
    } else if is_ret {
        "return"
    } else if is_indirect {
        "indirect_jump"
    } else if is_align {
        "alignment"
    } else {
        "other"
    };

    v.is_call = Some(is_call);
    v.is_ret = Some(is_ret);
    v.is_indirect_jump = Some(is_indirect);
    v.group = Some(group.to_string());
    Ok(Out::Insn(v))
}

// ---------------------------------------------------------------------------
// rename (B 档：ffi_ext::set_name)
// ---------------------------------------------------------------------------

/// Apply a C type declaration at `ea` (function prototype or data type)
/// via the IDA kernel's `apply_cdecl`. The declaration is plain C, e.g.
/// `int f(const char *, int)` for a prototype or `char arr[16]` for data.
pub fn set_type(_idb: &IDB, ea: Address, decl: &str) -> Result<Out> {
    let decl = decl.trim();
    if decl.is_empty() {
        bail!("type declaration must not be empty");
    }
    let c = CString::new(decl).context("declaration contains NUL")?;
    let ok = unsafe { ffi_ext::apply_cdecl(ea, c.as_ptr(), 0) };
    if !ok {
        bail!(
            "apply_cdecl failed at {ea:#x} for {decl:?} (parse error, unknown type, or bad address; \
             declarations must be valid C, end with ';', and use unnamed parameters)"
        );
    }
    Ok(Out::Value(serde_json::json!({
        "ok": true,
        "address": format!("0x{ea:x}"),
        "type": decl,
    })))
}

/// Rename the item at `ea` via the IDA kernel's `set_name`.
pub fn rename(_idb: &IDB, ea: Address, name: &str) -> Result<Out> {
    if name.is_empty() {
        bail!("name must not be empty");
    }
    let c = CString::new(name).context("name contains NUL")?;
    let ok = unsafe { ffi_ext::set_name(ea, c.as_ptr(), 0) };
    if !ok {
        bail!("set_name failed for {ea:#x} -> {name:?} (invalid name or address)");
    }
    Ok(Out::Value(serde_json::json!({
        "ok": true,
        "address": format!("0x{ea:x}"),
        "name": name,
    })))
}
