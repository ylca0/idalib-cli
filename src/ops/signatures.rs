use anyhow::{Result, bail};

use crate::idalib::idb::IDB;

use crate::helpers::json_types as jt;

pub fn make_signatures(idb: &mut IDB, only_pat: bool) -> Result<jt::OkView> {
    idb.make_signatures(only_pat)?;
    Ok(jt::OkView {
        ok: true,
        detail: Some(format!("signatures made (only_pat={only_pat})")),
    })
}

pub fn dispatch(idb: &mut IDB, s: &crate::cli::SignaturesCmd) -> Result<jt::OkView> {
    if s.make {
        return make_signatures(idb, s.only_pat);
    }
    bail!("use `--make` to generate signatures")
}
