use std::path::PathBuf;

use anyhow::Result;

/// Runtime configuration for `idalib-cli`, loaded from
/// `$IDALIB_CLI_HOME/config.toml` (default `~/.idapro/idalib-cli/config.toml`).
///
/// The config lets agents/users set defaults without repeating flags:
///
/// ```toml
/// [defaults]
/// idadir  = "/Applications/IDA Professional 9.1.app/Contents/MacOS"
/// idb_dir = "~/ida_out"
/// save    = true
/// auto_analyse = true
/// ```
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub idadir: Option<PathBuf>,
    pub idb_dir: Option<PathBuf>,
    pub save: Option<bool>,
    pub auto_analyse: Option<bool>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        parse_tomlish(&raw)
    }
}

pub fn config_path() -> Result<PathBuf> {
    Ok(crate::session::storage::base_dir()?.join("config.toml"))
}

/// A tiny TOML subset parser sufficient for our config shape (no external dep).
fn parse_tomlish(raw: &str) -> Result<Config> {
    let mut cfg = Config::default();
    let mut section = String::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            if section != "defaults" {
                continue;
            }
            match k {
                "idadir" => cfg.idadir = Some(expand_tilde(v).into()),
                "idb_dir" => cfg.idb_dir = Some(expand_tilde(v).into()),
                "save" => cfg.save = Some(v.eq_ignore_ascii_case("true")),
                "auto_analyse" => cfg.auto_analyse = Some(v.eq_ignore_ascii_case("true")),
                _ => {}
            }
        }
    }
    Ok(cfg)
}

fn expand_tilde(p: &str) -> String {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    p.to_string()
}

#[cfg(test)]
mod tests {
    use super::parse_tomlish;

    #[test]
    fn parses_defaults() {
        let raw = r#"
            # comment
            [defaults]
            idadir = "/opt/ida"
            save = true
            auto_analyse = false
        "#;
        let cfg = parse_tomlish(raw).unwrap();
        assert_eq!(cfg.idadir.unwrap(), std::path::PathBuf::from("/opt/ida"));
        assert_eq!(cfg.save, Some(true));
        assert_eq!(cfg.auto_analyse, Some(false));
    }

    #[test]
    fn ignores_other_sections() {
        let raw = "[other]\nfoo = 1\n[defaults]\nidb_dir = \"~/out\"\n";
        let cfg = parse_tomlish(raw).unwrap();
        assert!(cfg.idb_dir.is_some());
        assert!(cfg.save.is_none());
    }
}
