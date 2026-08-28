## Commands

- Before considering a task finished, run `pnpm fix`, which runs linting and type checking. The task is complete only after the command succeeds.
- Run `pnpm test` when tests are relevant to the change or explicitly requested.
- After modifying code in `game-server/`, run `pnpm build:game-server` to rebuild the WASM output and generated TypeScript bindings.
- Never run `pnpm build` unless explicitly requested.

## Performance Commits

- Before suggesting or creating a `perf:` commit, run each benchmark once against the committed `HEAD` and once against the current worktree:

```sh
pnpm bench games --target wasm
pnpm bench games --target native
pnpm bench perft suite --target wasm
pnpm bench perft suite --target native
```

Use this commit message template:

```text
perf: <message>

Games WASM: <change>
Games native: <change>
Perft suite WASM: <change>
Perft suite native: <change>
```
