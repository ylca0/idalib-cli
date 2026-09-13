# idalib-batch

Combine many IDA/IDALib queries into a single invocation, or fan a query out
across many sessions in parallel. Reduces process spin-up and is the fastest
way to gather a complete picture of one or more binaries.

## When to use

- You need several facts about one binary (batch).
- You have many sessions/samples to analyse with the same query (parallel).
- You are an agent and want to minimise round-trips.

## batch — many ops, one session, one process

```sh
idalib-cli batch -s 1 -- "meta" "segments" "functions -u" "decompile -a 0x401000"
```

- Each op is a full command line (quoted); ops run in order against the same
  session.
- JSON output contains a `results` array — one entry per op with `op`, `ok`,
  `output` and `error` fields — plus the `last` result for convenience.
- Ops that fail do not stop the rest (each is reported independently).

## parallel — one op, many sessions, many processes

```sh
idalib-cli parallel -- "decompile -a 0x401000"     # every session
idalib-cli parallel -S 1,2,3 -- "functions -u"     # selected sessions
idalib-cli parallel --jobs 4 -- "strings"          # limit concurrency
```

- Each session runs the op in its own subprocess, so independent IDB files are
  analysed truly concurrently (great for many-sample mass analysis).
- Default concurrency = number of CPUs; cap with `--jobs`.

## Tips for agents

- Combine `batch` + `parallel`: for N samples, first create N sessions, then
  `idalib-cli parallel -- "batch" ...` — however `parallel` runs a single op, so
  for multi-op fan-out use:
  ```sh
  idalib-cli parallel -- "batch -- <op1> <op2>"   # not recommended
  ```
  Prefer: create sessions per sample, then `parallel` each op you need.
- For triage of one binary in one shot:
  ```sh
  idalib-cli batch -- "meta" "segments" "strings" "functions -u"
  ```
