use anyhow::{Context, Result, bail};

use crate::idalib::{
    Address,
    func::{Function, FunctionFlags},
    idb::IDB,
    xref::XRefQuery,
};

use crate::helpers::json_types as jt;

pub fn segments(idb: &IDB) -> Vec<jt::SegmentView> {
    idb.segments()
        .map(|(sid, s)| jt::SegmentView::from_segment(sid, &s))
        .collect()
}

pub fn segment_at(idb: &IDB, ea: Address) -> Option<jt::SegmentView> {
    idb.segment_at(ea)
        .map(|s| jt::SegmentView::from_segment(0, &s))
}

pub fn functions(idb: &IDB, user_only: bool) -> Vec<jt::FunctionView> {
    idb.functions()
        .filter_map(|(fid, f)| {
            if user_only
                && (f.flags().contains(FunctionFlags::LIB)
                    || f.flags().contains(FunctionFlags::THUNK))
            {
                return None;
            }
            let blocks = f.cfg().ok().map(|c| c.blocks_count());
            let mut v = function_view(fid, &f);
            v.blocks = blocks;
            Some(v)
        })
        .collect()
}

pub fn function(idb: &IDB, ea: Address) -> Result<jt::FunctionDetailView> {
    let f = idb.function_at(ea).context("no function at address")?;
    let cfg = f.cfg()?;
    let xrefs_to = collect_xrefs_to(idb, ea);
    let blocks: Vec<jt::BasicBlockView> = cfg
        .blocks()
        .enumerate()
        .map(|(i, b)| {
            let mut v = jt::BasicBlockView::from_block(&b, &cfg);
            v.id = i;
            v
        })
        .collect();

    let entry = cfg.entry().map(|b| format!("0x{:x}", b.start_address()));
    let exit = cfg.exit().map(|b| format!("0x{:x}", b.start_address()));

    let mut fview = function_view(function_id_for(idb, &f), &f);
    fview.blocks = Some(blocks.len());
    fview.decompiled = idb.decompile(&f).is_ok();

    Ok(jt::FunctionDetailView {
        function: fview,
        blocks,
        entry,
        exit,
        xrefs_to,
    })
}

fn collect_xrefs_to(idb: &IDB, ea: Address) -> Vec<jt::XRefView> {
    let mut out = Vec::new();
    let mut cur = match idb.first_xref_to(ea, XRefQuery::ALL) {
        Some(x) => x,
        None => return out,
    };
    loop {
        out.push(jt::XRefView::from_xref(&cur));
        match cur.next_to() {
            Some(n) => cur = n,
            None => break,
        }
    }
    out
}

pub fn disasm(idb: &IDB, ea: Address, count: usize) -> Result<Vec<jt::InsnView>> {
    let mut cur = ea;
    let mut out = Vec::new();
    for _ in 0..count {
        match idb.insn_at(cur) {
            Some(i) => {
                out.push(jt::InsnView::from_insn(&i));
                cur = i.address() + i.len() as u64;
            }
            None => break,
        }
    }
    if out.is_empty() {
        bail!("no instructions found at {ea:#x}");
    }
    Ok(out)
}

pub fn decompile(idb: &IDB, ea: Address, all_blocks: bool) -> Result<jt::FunctionView> {
    let f = idb.function_at(ea).context("no function at address")?;
    let pseudocode = if all_blocks {
        idb.decompile_with(&f, true)?.pseudocode()
    } else {
        idb.decompile(&f)?.pseudocode()
    };
    let blocks = f.cfg().ok().map(|c| c.blocks_count());
    let mut v = function_view(function_id_for(idb, &f), &f);
    v.blocks = blocks;
    v.decompiled = true;
    v.pseudocode = Some(pseudocode);
    Ok(v)
}

/// Sanitise IDA-returned text for JSON output: raw NUL bytes are invalid JSON
/// control characters, so strip everything from the first NUL (C-string
/// semantics) and drop other control characters.
pub fn sanitize(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let s = match s.find('\0') {
        Some(pos) => &s[..pos],
        None => s,
    };
    s.chars().filter(|c| !c.is_control()).collect()
}

pub fn strings(idb: &IDB, all: bool) -> Vec<jt::StringView> {
    let _ = all;
    idb.strings()
        .iter()
        .enumerate()
        .map(|(i, (addr, value))| jt::StringView {
            index: i,
            address: format!("0x{addr:x}"),
            value: sanitize(value),
        })
        .collect()
}

pub fn names(idb: &IDB) -> Vec<jt::NameView> {
    idb.names()
        .iter()
        .map(|n| jt::NameView {
            address: format!("0x{:x}", n.address()),
            name: sanitize(n.name()),
            is_public: n.is_public(),
            is_weak: n.is_weak(),
        })
        .collect()
}

pub fn xrefs(idb: &IDB, ea: Option<Address>, all: bool) -> Result<Vec<jt::XRefView>> {
    let _flags = if all {
        XRefQuery::ALL
    } else {
        XRefQuery::FAR | XRefQuery::DATA
    };
    match ea {
        Some(ea) => Ok(collect_xrefs_to(idb, ea)),
        None => {
            let mut out = Vec::new();
            for (_fid, f) in idb.functions() {
                let ea = f.start_address();
                out.extend(collect_xrefs_to(idb, ea));
            }
            Ok(out)
        }
    }
}

pub fn entries(idb: &IDB) -> Vec<jt::EntryPointView> {
    // NOTE: upstream `EntryPointIter` (idalib-rs 0.6.1) never advances its
    // index on a valid address, so `idb.entries()` loops forever. Call the
    // FFI directly with a bounded loop instead.
    let _ = idb;
    let qty = unsafe { crate::idalib::ffi::entry::get_entry_qty() };
    (0..qty)
        .map(|idx| {
            let ord = unsafe { crate::idalib::ffi::entry::get_entry_ordinal(idx) };
            #[allow(clippy::useless_conversion)]
            let addr: u64 = unsafe { crate::idalib::ffi::entry::get_entry(ord) }.into();
            jt::EntryPointView {
                ordinal: idx,
                address: format!("0x{addr:x}"),
            }
        })
        .collect()
}

pub fn meta(idb: &IDB) -> jt::MetadataView {
    let m = idb.meta();
    jt::MetadataView {
        procname: Some(m.procname()),
        filetype: Some(format!("{:?}", m.filetype())),
        compiler: Some(format!("{:?}", m.cc_id())),
        bitness: Some(m.app_bitness()),
        app_bitness: Some(m.app_bitness()),
        is_dll: Some(m.is_dll()),
        is_be: Some(m.is_be()),
        is_16bit: Some(m.is_16bit()),
        is_32bit: Some(m.is_32bit_exactly()),
        is_64bit: Some(m.is_64bit()),
        database_change_count: Some(m.database_change_count()),
        is_auto_enabled: Some(m.is_auto_enabled()),
        is_readonly_idb: Some(m.readonly_idb()),
        is_kernel_mode: Some(m.is_kernel_mode()),
        is_snapshot: Some(m.is_snapshot()),
    }
}

pub fn processor(idb: &IDB) -> jt::ProcessorView {
    let p = idb.processor();
    jt::ProcessorView {
        family: format!("{:?}", p.family()),
        short_name: p.short_name(),
        long_name: p.long_name(),
    }
}

pub fn insn(idb: &IDB, ea: Address) -> Result<jt::InsnView> {
    let i = idb.insn_at(ea).context("no instruction at address")?;
    Ok(jt::InsnView::from_insn(&i))
}

fn function_view(fid: usize, f: &Function) -> jt::FunctionView {
    jt::FunctionView {
        id: fid,
        start: format!("0x{:x}", f.start_address()),
        end: format!("0x{:x}", f.end_address()),
        size: f.len(),
        name: f.name(),
        flags: f.flags().bits(),
        is_lib: f.flags().contains(FunctionFlags::LIB),
        is_thunk: f.flags().contains(FunctionFlags::THUNK),
        is_tail: f.flags().contains(FunctionFlags::TAIL),
        is_far: f.is_far(),
        does_return: f.does_return(),
        blocks: None,
        decompiled: false,
        pseudocode: None,
    }
}

fn function_id_for(idb: &IDB, f: &Function) -> usize {
    idb.functions()
        .find_map(|(fid, ff)| {
            if ff.start_address() == f.start_address() {
                Some(fid)
            } else {
                None
            }
        })
        .unwrap_or(0)
}
