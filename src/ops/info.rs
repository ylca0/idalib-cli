use anyhow::Result;

use crate::idalib::{IDAVersion, license, version};

use crate::helpers::json_types as jt;
use crate::ops::top::Out;

pub fn version_info() -> Result<jt::VersionInfo> {
    let ida_version = version()
        .ok()
        .map(|v: IDAVersion| format!("{}.{}.{}", v.major(), v.minor(), v.build()));

    let license = if license::is_valid_license() {
        license::license_id().ok().map(|l| format!("{l}"))
    } else {
        None
    };

    Ok(jt::VersionInfo {
        idalib_cli: crate::cli::VERSION.to_string(),
        // The vendored idalib-rs version this build is compiled against.
        idalib_rs: "0.6.1+9.1.250226".to_string(),
        ida_version,
        ida_install_dir: std::env::var("IDADIR").ok(),
        license,
    })
}

pub fn license_info() -> Result<jt::LicenseView> {
    let valid = license::is_valid_license();
    let id = if valid {
        license::license_id().ok().map(|l| format!("{l}"))
    } else {
        None
    };
    Ok(jt::LicenseView {
        valid,
        id,
        error: None,
    })
}

pub fn ida_version() -> Result<jt::VersionView> {
    let v = version()?;
    Ok(jt::VersionView {
        major: v.major(),
        minor: v.minor(),
        build: v.build(),
        string: format!("{}.{}.{}", v.major(), v.minor(), v.build()),
    })
}

pub fn run(i: &crate::cli::InfoCmd) -> Result<Out> {
    if i.version || i.all {
        Ok(Out::Version(ida_version()?))
    } else if i.ida {
        Ok(Out::License(license_info()?))
    } else {
        Ok(Out::VersionInfo(version_info()?))
    }
}
