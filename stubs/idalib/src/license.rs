use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct LicenseId([u8; 6]);

impl Display for LicenseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STUB-LICENSE")
    }
}

pub fn is_valid_license() -> bool {
    true
}

pub fn license_id() -> Result<LicenseId, crate::IDAError> {
    Ok(LicenseId([0; 6]))
}
