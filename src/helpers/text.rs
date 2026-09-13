//! Human-readable rendering helpers (used when `--json` is not set).

pub fn pretty_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default()
}
