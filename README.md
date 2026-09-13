<div align="center">

# idalib-cli

**Agent-native CLI for the IDA Pro IDALib — built on [idalib-rs](https://github.com/idalib-rs/idalib)**

[![IDA](https://img.shields.io/badge/IDA_Pro-9.1-blue)](https://hex-rays.com/ida-pro)
[![idalib-rs](https://img.shields.io/badge/idalib--rs-0.6.1-orange)](https://github.com/idalib-rs/idalib)
[![Version](https://img.shields.io/badge/version-0.9.1-green)](#versions--branches)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-lightgrey)](#license)

**English** · [简体中文](README.zh-CN.md)

</div>

A single-binary CLI that exposes IDA analysis as structured JSON — designed to
be driven by code agents (Codex, Claude Code, OpenCode) or humans. Manage
multiple independent, stateful analysis sessions, each backed by its own IDB,
and run them concurrently across processes.

---

## Features

| | |
| --- | --- |
| 💾 **Sessions** | Independent, stateful analysis sessions; registry persisted in `~/.idapro/idalib-cli/sessions.json` |
| 🗄️ **Persistent IDB** | Each session maps to an `.i64` file — comments, bookmarks, names, analysis survive across processes |
| 🧩 **Full IDALib coverage** | Segments, functions, CFG, disassembly, Hex-Rays, strings, names, xrefs, entries, metadata, comments, bookmarks, FLIRT |
| ⚡ **Batch & parallel** | `batch` = several commands in one process; `parallel` = one command fanned out to many sessions |
| 🤖 **Agent-first** | JSON on stdout everywhere; global `-s/--session`; skills included for agents |

## Install

> **Prerequisites**
> 1. IDA Pro 9.1 installed & launched once (valid license)
> 2. IDA 9.1 SDK unpacked (from your Hex-Rays account) — **build time only**
> 3. Rust toolchain + LLVM/Clang ([bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html))

```sh
export IDADIR="/Applications/IDA Professional 9.1.app/Contents/MacOS"  # IDA install dir
export IDASDKDIR=$HOME/idasdk91                                        # unpacked SDK

git clone <this-repo> && cd idalib-cli
cargo install --path .

idalib-cli info    # ✅ verify: tool version, IDA version, license
```

<details>
<summary>Notes & dev checks without an SDK</summary>

- `IDADIR` is auto-detected from common locations if unset.
- The binary links your local `libida`/`libidalib` at runtime; the SDK is
  never embedded or redistributed.
- Code-level checks without an SDK (dev only):

  ```sh
  cargo check --no-default-features --features stub-idalib
  cargo test  --no-default-features --features stub-idalib
  cargo fmt --all --check
  ```

</details>

## Quick start

```sh
idalib-cli session open -b ./target.bin -o ./target.i64   # 1. create a session
idalib-cli batch -- "meta" "segments" "strings" "functions -u"   # 2. overview
idalib-cli decompile -s 1 -a 0x401000                     # 3. Hex-Rays pseudo-code
idalib-cli comments set -s 1 -a 0x401000 -c "note"        # 4. annotate (persisted)
idalib-cli parallel -- "decompile -a 0x401000"            # 5. fan out to all sessions
```

## Command reference

> Global: `-s/--session <ID>` selects the session (falls back to the first
> ready one) · `-j/--json` forces JSON (already the default) · addresses
> accept `0x401000` or `401000`

<details>
<summary><b>Session management</b></summary>

```sh
idalib-cli session open    -b <bin> [-o <idb>] [-n name] [--save] [--auto-analyse]
idalib-cli session list
idalib-cli session show    -s <id>
idalib-cli session close   -s <id> [--save true|false]
idalib-cli session save    -s <id>
idalib-cli session analyze -s <id> [--wait]
idalib-cli session remove  -s <id>
```

</details>

<details>
<summary><b>Database queries</b></summary>

```sh
idalib-cli segments                     # all segments
idalib-cli segments-by-range -a <ea>    # segment containing an address
idalib-cli functions [-u]               # -u = user code only (skip lib/thunk)
idalib-cli function  -a <ea>            # details: CFG, blocks, xrefs
idalib-cli disasm    -a <ea> [-n N]     # disassemble N instructions (default 8)
idalib-cli decompile -a <ea> [--all-blocks]   # Hex-Rays pseudo-code
idalib-cli strings | names | entries
idalib-cli xrefs [-a <ea>] [--all]      # to an address, or across all functions
idalib-cli meta                         # filetype / compiler / bitness
idalib-cli processor
idalib-cli insn      -a <ea>            # single instruction
```

</details>

<details>
<summary><b>Database edits</b> (persisted in the IDB)</summary>

```sh
idalib-cli comments get|set|append|remove -a <ea> [-c "text"]
idalib-cli bookmarks list|add|get|remove  -a <ea> [-d "desc"]
idalib-cli signatures --make [--only-pat]      # generate FLIRT signatures
```

</details>

<details>
<summary><b>Batch & parallel</b></summary>

```sh
# sequential, one process, one session
idalib-cli batch -- "meta" "segments" "decompile -a 0x401000"

# one command, many sessions (one subprocess per session)
idalib-cli parallel -- "functions -u"           # all sessions
idalib-cli parallel -S 1,2,3 --jobs 4 -- "strings"
```

</details>

<details>
<summary><b>Sample output</b> (<code>decompile</code>)</summary>

```json
{
  "id": 7,
  "start": "0x401000",
  "end": "0x401080",
  "size": 128,
  "name": "main",
  "blocks": 3,
  "decompiled": true,
  "pseudocode": "int __cdecl main(...) { ... }"
}
```

</details>

Agent-oriented workflow guides live in [`skills/`](skills/) — useful for
humans too. A runnable end-to-end example: [`examples/workflow.sh`](examples/workflow.sh).

## Configuration

Optional `~/.idapro/idalib-cli/config.toml` (base dir overridable via
`IDALIB_CLI_HOME`):

```toml
[defaults]
idadir = "/Applications/IDA Professional 9.1.app/Contents/MacOS"
idb_dir = "~/ida_out"      # default IDB location (otherwise next to the binary)
save = true
auto_analyse = true
```

By default an IDB is created next to the analysed binary (`<binary>.i64`).

## Architecture notes

Flow: **load config → restore session registry → run op (open IDB from disk →
query → save) → persist registry**.

- A **session** = one binary + one IDB file + a registry record. The IDB file
  is the source of truth → sessions are independent, stateful across
  invocations, and safe to run concurrently.
- `parallel` isolates by process (one subprocess per session) — IDALib is not
  thread-safe for concurrent in-process DB use.
- Output contract: one JSON document on stdout; errors to stderr, non-zero exit.
- Bundled upstream workarounds: `EntryPointIter` infinite-loop fix (0.6.1),
  NUL-padding sanitisation for JSON-safe strings, IDA farewell-message
  suppression.

## Versions & branches

Tool versions are `x.y.z`; one dev + one release branch per minor:

```
main             latest development
v0.9_dev         development branch (tool 0.9.x)
v0.9_release     stable branch
v0.9.1 tag       release point
```

Supporting a new IDA version = bump the `idalib` dependency, update
`[package.metadata.ida]`, open a new branch line (`v0.10_*`).

## Distribution note

This repository contains no Hex-Rays code. The build links SDK stub libraries
(link-time shells); at runtime the binary loads the user's own IDA libraries
and validates their license. **Never commit or redistribute the IDA SDK.**

## License

MIT OR Apache-2.0, matching [idalib-rs](https://github.com/idalib-rs/idalib).