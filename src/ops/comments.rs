use anyhow::Result;

use crate::idalib::idb::IDB;

use crate::helpers::json_types as jt;

pub fn get_cmt(idb: &IDB, ea: u64) -> Result<jt::CmtView> {
    let comment = idb.get_cmt(ea).unwrap_or_default();
    Ok(jt::CmtView {
        address: format!("0x{ea:x}"),
        comment,
        repeatable: false,
    })
}

pub fn set_cmt(idb: &IDB, ea: u64, comment: &str) -> Result<jt::CmtView> {
    idb.set_cmt(ea, comment)?;
    Ok(jt::CmtView {
        address: format!("0x{ea:x}"),
        comment: comment.to_string(),
        repeatable: false,
    })
}

pub fn append_cmt(idb: &IDB, ea: u64, comment: &str) -> Result<jt::CmtView> {
    idb.append_cmt(ea, comment)?;
    Ok(jt::CmtView {
        address: format!("0x{ea:x}"),
        comment: comment.to_string(),
        repeatable: false,
    })
}

pub fn remove_cmt(idb: &IDB, ea: u64) -> Result<jt::CmtView> {
    idb.remove_cmt(ea)?;
    Ok(jt::CmtView {
        address: format!("0x{ea:x}"),
        comment: String::new(),
        repeatable: false,
    })
}

pub fn dispatch(idb: &mut IDB, c: &crate::cli::CommentsCmd) -> Result<jt::CmtView> {
    let idb: &IDB = idb;
    match &c.command {
        crate::cli::CommentsAction::Get(g) => get_cmt(idb, g.address),
        crate::cli::CommentsAction::Set(s) => set_cmt(idb, s.address, &s.comment),
        crate::cli::CommentsAction::Append(a) => append_cmt(idb, a.address, &a.comment),
        crate::cli::CommentsAction::Remove(r) => remove_cmt(idb, r.address),
    }
}
