# spec-spine-cli

The `spec-spine` command-line tool — a thin translation of `spec-spine-core`
results into stdout/stderr and stable exit codes.

```
spec-spine compile                 # specs/*/spec.md -> .derived/spec-registry/registry.json
spec-spine registry list           # list specs (optionally --status)
spec-spine registry show <id>      # show one spec
spec-spine registry status-report  # counts by status
spec-spine registry relationships <id>
```

`cargo install spec-spine-cli` installs the `spec-spine` binary. Exit codes:
`0` ok, `1` validation failure / not found, `2` stale, `3` I/O / parse / schema.
License: Apache-2.0.
