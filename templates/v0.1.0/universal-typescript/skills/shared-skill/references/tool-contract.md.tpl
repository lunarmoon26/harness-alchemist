# {{DISPLAY_NAME}} Tool Contract

The shared skill owns its runtime. `skill-runtime.json` declares its portable
tool schema and author-selected executable. Harness entrypoints project that
manifest and must not contain workflow logic.

## Entrypoints

| File | Runtime | Requirement |
| --- | --- | --- |
| `scripts/main.mjs` | Node.js 22+ or Bun 1.2+ | Zero npm dependencies |
| `scripts/main.py` | CPython 3.10+ | Standard library only |

`main.mjs` is the starter manifest's fixed model-callable entrypoint.
`main.py` is a behavioral twin for direct compatibility testing. Any change to
one must be mirrored in the other so validators can pair them.

## I/O Contract

1. Read exactly one JSON object from stdin.
2. Write exactly one JSON result followed by a newline to stdout.
3. On success: `{"ok": true, "plugin": "<plugin-name>", ...}` with exit code 0.
4. On failure: write a diagnostic to stderr, write nothing to stdout, and exit
   with a non-zero code.

## Delegation Rules

- Claude Code, Codex, and Antigravity agents invoke these scripts directly via
  a shell, guided by `SKILL.md`.
- The OpenCode adapter (`src/opencode.ts`) projects the manifest through the
  shared runtime package.
- The DeepSeek/Cordis adapter (`src/deepseek.ts`) registers the same tool in the
  Harness tool registry; `cordis.patch.yml` loads it from the npm package.

Change `entrypoint.engine` and `entrypoint.path` together when Python or another
supported engine becomes the canonical implementation. Never accept an
executable path from model input.
