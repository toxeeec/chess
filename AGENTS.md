## Commands

- Before considering a task finished, run `pnpm fix`, which runs linting and type checking. The task is complete only after the command succeeds.
- Run `pnpm test` when tests are relevant to the change or explicitly requested.
- After modifying code in `game-server/`, run `pnpm build:game-server` to rebuild the WASM output and generated TypeScript bindings.
- Never run `pnpm build` unless explicitly requested.
