# idalib-analysis

Analyse a binary with IDA Pro's IDALib through the `idalib-cli` tool: inspect
segments, functions, disassembly, decompiled pseudo-code, strings, names and
cross-references, and edit database state (comments/bookmarks).

This skill is agent-first: it tells you (the agent) exactly which commands to
run and how to interpret the structured JSON output.

## When to use

Use this whenever you need to reverse-engineer, triage or deeply inspect a
binary, function, string or xref, or when you must annotate an IDB.

## Setup (one time)

1. `idalib-cli` must be built with the IDA SDK (see README "Build" section).
   Confirm it works:

   ```sh
   idalib-cli info
   ```

2. Open a session for the target binary. A session is an independent, stateful
   IDA/IDALib process backed by its own IDB file:

   ```sh
   idalib-cli session open -b ./target.bin -o ./target.i64
   # -> {"id": 1, "state": "ready", "idb": "./target.i64", ...}
   ```

   Reopening later is cheap: `idalib-cli session open -b ./target.bin -o ./target.i64`
   reuses the existing IDB (all state persists).

## Command reference

All commands emit JSON by default. Target a session with `-s <id>` (global flag).

| Goal | Command |
| --- | --- |
| Show runtime / version / license | `idalib-cli info` |
| Create a session | `idalib-cli session open -b <bin> -o <idb> [-n name]` |
| List sessions | `idalib-cli session list` |
| Session details | `idalib-cli session show -s <id>` |
| Close / save / remove a session | `idalib-cli session close -s <id>` / `save` / `remove` |
| Segments | `idalib-cli segments -s <id>` |
| Functions | `idalib-cli functions -s <id>` (add `-u` for user code only) |
| One function (CFG/blocks/xrefs) | `idalib-cli function -s <id> -a 0x401000` |
| Disassemble | `idalib-cli disasm -s <id> -a 0x401000 -n 20` |
| Decompile (Hex-Rays) | `idalib-cli decompile -s <id> -a 0x401000` |
| Strings | `idalib-cli strings -s <id>` |
| Names | `idalib-cli names -s <id>` |
| Xrefs to an address | `idalib-cli xrefs -s <id> -a 0x401000 --all` |
| Entry points | `idalib-cli entries -s <id>` |
| DB metadata (compiler/filetype/bitness) | `idalib-cli meta -s <id>` |
| Processor info | `idalib-cli processor -s <id>` |
| One instruction | `idalib-cli insn -s <id> -a 0x401000` |
| Read a comment | `idalib-cli comments get -s <id> -a 0x401000` |
| Set / append / remove comment | `idalib-cli comments set/append/remove -s <id> -a 0x401000 -c "..."` |
| List / add / remove bookmarks | `idalib-cli bookmarks list/add/remove -s <id> -a 0x401000` |
| Generate FLIRT sigs | `idalib-cli signatures --make -s <id>` |

## Recommended analysis workflow

1. **Triage**: `session open` -> `meta` -> `segments` -> `strings`.
2. **Get the lay of the land**: `functions -u` to list user functions, note the
   interesting names and addresses.
3. **Focus**: `decompile -a <fn>` to read the pseudo-code; `function -a <fn>`
   for CFG + xrefs; `disasm -a <ea>` for raw instructions.
4. **Follow the data**: `xrefs -a <ea> --all` to find who references a function
   or string; `strings` to locate message text.
5. **Annotate**: set comments/bookmarks so the state persists for other agents
   or later sessions.

## Batched / parallel queries

- Run several queries in one call (one process, one session):
  ```sh
  idalib-cli batch -- "meta" "segments" "decompile -a 0x401000"
  ```
- Run the same query across many sessions concurrently (multi-process):
  ```sh
  idalib-cli parallel -- "decompile -a 0x401000"     # all sessions
  idalib-cli parallel -S 1,2,3 -- "functions -u"     # specific sessions
  ```

## Tips for agents

- Addresses are accepted in hex (`0x401000` or `401000`).
- Use `-u` on `functions` to skip library/thunk noise.
- `decompile` returns the full pseudo-code string in `pseudocode`.
- Prefer `batch` when you need several facts about one binary — it avoids
  reopening the IDB repeatedly.
- `parallel` is ideal for mass-analysis of many samples: create one session per
  sample, then fan out a query across all of them.
- Session state (comments, bookmarks, names) persists in the `.i64`, so annotate
  once and reuse.
