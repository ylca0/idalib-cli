use anyhow::Result;

use crate::idalib::idb::IDB;

use crate::helpers::json_types as jt;
use crate::session::session::Address;

pub fn bookmarks(idb: &IDB) -> Vec<jt::BookmarkView> {
    let b = idb.bookmarks();
    (0..b.len())
        .filter_map(|idx| {
            let addr = b.get_address(idx)?;
            Some(jt::BookmarkView {
                index: idx,
                address: format!("0x{addr:x}"),
                description: b.get_description_by_index(idx),
            })
        })
        .collect()
}

pub fn mark(idb: &IDB, ea: Address, desc: &str) -> Result<jt::BookmarkView> {
    let idx = idb.bookmarks().mark(ea, desc)?;
    Ok(jt::BookmarkView {
        index: idx,
        address: format!("0x{ea:x}"),
        description: Some(desc.to_string()),
    })
}

pub fn get(idb: &IDB, ea: Address) -> Result<jt::BookmarkView> {
    let desc = idb.bookmarks().get_description(ea);
    Ok(jt::BookmarkView {
        index: 0,
        address: format!("0x{ea:x}"),
        description: desc,
    })
}

pub fn erase(idb: &IDB, ea: Address) -> Result<jt::BookmarkView> {
    idb.bookmarks().erase(ea)?;
    Ok(jt::BookmarkView {
        index: 0,
        address: format!("0x{ea:x}"),
        description: None,
    })
}

pub fn dispatch(idb: &mut IDB, b: &crate::cli::BookmarksCmd) -> Result<jt::BookmarksOut> {
    let idb: &IDB = idb;
    match &b.command {
        crate::cli::BookmarksAction::Add(a) => {
            Ok(jt::BookmarksOut::One(mark(idb, a.address, &a.description)?))
        }
        crate::cli::BookmarksAction::Get(g) => Ok(jt::BookmarksOut::One(get(idb, g.address)?)),
        crate::cli::BookmarksAction::Remove(r) => Ok(jt::BookmarksOut::One(erase(idb, r.address)?)),
        crate::cli::BookmarksAction::List(_) => Ok(jt::BookmarksOut::Many(bookmarks(idb))),
    }
}
