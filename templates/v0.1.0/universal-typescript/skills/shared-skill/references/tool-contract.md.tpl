# {{DISPLAY_NAME}} Tool Contract

The shared skill owns its runtime. Harness entrypoints are thin adapters that
delegate to the scripts in this directory; they must not contain workflow logic.

## Entrypoints

| File | Runtime | Requirement |
| --- | --- | --- |
| `scripts/main.mjs` | Node.js 22+ or Bun 1.2+ | Zero npm dependencies |
| `scripts/main.py` | CPython 3.10+ | Standard library only |

`main.mjs` and `main.py` are behavioral twins. Any change to one must be
mirrored in the other, and both must keep the same name so validators can pair
them.

## I/O Contract

1. Read exactly one JSON object from stdin.
2. Write exactly one JSON result followed by a newline to stdout.
3. On success: `{"ok": true, "plugin": "<plugin-name>", ...}` with exit code 0.
4. On failure: write a diagnostic to stderr, write nothing to stdout, and exit
   with a non-zero code.

## Delegation Rules

- Claude Code, Codex, and Antigravity agents invoke these scripts directly via
  a shell, guided by `SKILL.md`.
- The OpenCode adapter (`src/opencode.ts`) registers a tool that spawns the
  scripts and returns their JSON output.
- The DeepSeek/Cordis adapter (`src/deepseek.ts`) provides a service that
  spawns the scripts; `cordis.patch.yml` loads it from the published package.

Prefer `.mjs` when only a JavaScript runtime is guaranteed; use `.py` when
Python is available or the workflow needs Python-only libraries.
