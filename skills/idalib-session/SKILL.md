# idalib-session

Create, inspect, and manage independent, stateful IDA/IDALib analysis sessions
via `idalib-cli`.

## When to use

Use this whenever you need to set up analysis on one or more binaries, manage
the lifecycle of analysis processes, or coordinate multiple concurrent analysis
jobs (multi-agent / multi-process).

## Session model

- A **session** is one analysis target: a binary plus its IDB file (`.i64`).
- Sessions are **independent and stateful**: each has its own IDB state that
  persists across invocations (comments, bookmarks, names, analysis results).
- Sessions are stored in `~/.idapro/idalib-cli/sessions.json` (override with
  `IDALIB_CLI_HOME`). All CLI invocations share this registry.
- Since each invocation is a separate process, concurrency is achieved by
  running multiple invocations in parallel — each on its own session/IDB.

## Commands

```sh
# Create a session (builds the IDB immediately; reuses it if it exists)
idalib-cli session open -b ./target.bin -o ./target.i64 -n mytarget

# List / inspect
idalib-cli session list
idalib-cli session show -s 1

# Lifecycle
idalib-cli session save -s 1          # mark for save on close
idalib-cli session close -s 1         # close (releases the DB)
idalib-cli session analyze -s 1       # (re)run auto-analysis
idalib-cli session remove -s 1        # delete the session record
```

## Multi-session workflows

- **Many samples, one binary each**: create one session per sample, then use
  `parallel` to query all of them at once:
  ```sh
  idalib-cli session open -b a.bin -o a.i64
  idalib-cli session open -b b.bin -o b.i64
  idalib-cli parallel -- "strings"            # runs on all sessions concurrently
  idalib-cli parallel -S 1,2 -- "decompile -a 0x401000"
  ```

- **Batch on one session** (avoids re-opening the IDB repeatedly):
  ```sh
  idalib-cli batch -s 1 -- "meta" "segments" "functions -u"
  ```

## Tips for agents

- Default IDB path is `<binary>.i64` next to the binary if `-o` is omitted —
  prefer giving an explicit `-o` to keep things tidy.
- Sessions survive across agent turns and CLI invocations, so you can annotate
  in one step and query in the next.
- Use `session show -s <id>` to confirm a session is `ready` before querying.
- To analyse one binary with several independent views, open multiple sessions
  pointing at the same binary with different `-o` IDB paths.
