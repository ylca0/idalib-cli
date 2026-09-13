use anyhow::{Context, Result, bail};
use clap::Parser;
use serde::Serialize;

use crate::cli::Cli;
use crate::ops::top;

/// Result of one batched sub-command.
#[derive(Serialize, Debug)]
pub struct BatchItem {
    pub op: String,
    pub ok: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Run a sequence of commands against the `-d/--db` database selected on the
/// parent invocation. Stateless: each sub-command reuses the same open IDB.
pub fn run(cli: &crate::cli::Cli, b: &crate::cli::BatchCmd) -> Result<top::Out> {
    if b.ops.is_empty() {
        bail!("batch requires at least one op");
    }

    let mut items: Vec<BatchItem> = Vec::new();
    let mut last_out: Option<top::Out> = None;

    // Open the IDB once, run every op against it, save once at the end.
    top::with_idb_ref(cli, |idb| {
        for op in &b.ops {
            let mut argv = shell_words_split(op)?;
            argv.insert(0, "idalib-cli".to_string());
            // Parse with a fake global --db so per-op parsing succeeds; we do
            // NOT reopen the DB - ops run against the already-open handle.
            argv.push("-d".to_string());
            argv.push("__BATCH__".to_string());
            let sub = Cli::try_parse_from(argv)
                .with_context(|| format!("failed to parse batch op: {op}"))?;

            match top::run_db_op(&sub, idb) {
                Ok(out) => {
                    let json = serde_json::to_value(&out).unwrap_or(serde_json::Value::Null);
                    last_out = Some(out);
                    items.push(BatchItem {
                        op: op.clone(),
                        ok: true,
                        output: Some(json),
                        error: None,
                    });
                }
                Err(e) => {
                    items.push(BatchItem {
                        op: op.clone(),
                        ok: false,
                        output: None,
                        error: Some(format!("{e:#}")),
                    });
                }
            }
        }
        Ok(())
    })?;

    Ok(top::Out::Value(serde_json::json!({
        "results": items,
        "last": last_out.map(|o| serde_json::to_value(&o).unwrap_or(serde_json::Value::Null)),
    })))
}

/// Minimal POSIX-style shell tokenisation for batch ops. Splits on whitespace,
/// respects single and double quotes and backslash escapes.
pub fn shell_words_split(s: &str) -> Result<Vec<String>> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();
    let mut in_word = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_word = true;
                while let Some(&c2) = chars.peek() {
                    if c2 == '\'' {
                        chars.next();
                        break;
                    }
                    cur.push(c2);
                    chars.next();
                }
            }
            '"' => {
                in_word = true;
                while let Some(&c2) = chars.peek() {
                    if c2 == '"' {
                        chars.next();
                        break;
                    }
                    cur.push(c2);
                    chars.next();
                }
            }
            '\\' => {
                if let Some(&c2) = chars.peek() {
                    cur.push(c2);
                    chars.next();
                    in_word = true;
                }
            }
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            c => {
                in_word = true;
                cur.push(c);
            }
        }
    }
    if in_word {
        words.push(cur);
    }
    if words.is_empty() {
        bail!("empty op string");
    }
    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::shell_words_split;

    #[test]
    fn splits_simple() {
        let v = shell_words_split("functions -u").unwrap();
        assert_eq!(v, vec!["functions", "-u"]);
    }

    #[test]
    fn splits_quoted() {
        let v = shell_words_split("comments set -a 0x401000 -c \"hello world\"").unwrap();
        assert_eq!(
            v,
            vec!["comments", "set", "-a", "0x401000", "-c", "hello world"]
        );
    }

    #[test]
    fn empty_fails() {
        assert!(shell_words_split("").is_err());
    }
}
