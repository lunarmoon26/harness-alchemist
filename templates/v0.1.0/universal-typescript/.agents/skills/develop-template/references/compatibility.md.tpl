# {{DISPLAY_NAME}} Compatibility

## Shared Contract

- `skills/{{NAME}}/SKILL.md` is bundled natively by Claude Code, Codex, and Antigravity.
- `.agents/skills/develop-{{NAME}}/` is repository-local development guidance and is discoverable by Vercel Skills.
- OpenCode and DeepSeek users install shared skills separately from the npm runtime plugin.

## Layout Manifest

`alchemy.json` records this repository's layout: `runtime: "npm"`,
the canonical `template` version, `generator`, `generatorVersion`, and
`createdAt`. It is validated against the published JSON Schema referenced by
`$schema`. `npm run sync` refreshes `generatorVersion` from `package.json`.
If this project ever moves into a monorepo, add `pluginRoot` (and optionally
`opencodeExport: "./server"` for SDK packages); `runtime: "skills"` adapts
non-JavaScript repositories to skills-only validation.

## Harness-Specific Contract

| Harness | Files | Rule |
| --- | --- | --- |
| Claude Code | `.claude-plugin/` | Components stay at repository root; marketplace source is `./`. |
| Codex/ChatGPT | `.codex-plugin/`, `.agents/plugins/` | Local marketplace entries require policy and category. |
| OpenCode | `src/opencode.ts` | Package root exports a plugin function returning hooks. |
| Antigravity | `plugin.json` | Skills use nested `<name>/SKILL.md`. |
| DeepSeek | `src/deepseek.ts`, `cordis.patch.yml` | Function plugin uses named exports and no default export. |

## Skill Script Contract

- `skills/{{NAME}}/scripts/` owns the skill runtime. Entrypoints come in
  behavioral `.mjs` and `.py` twins with identical base names.
- Contract: one JSON object on stdin, one JSON result plus newline on stdout,
  non-zero exit with a stderr diagnostic on failure. See
  `skills/{{NAME}}/references/tool-contract.md`.
- `src/opencode.ts` and `src/deepseek.ts` are thin adapters that spawn these
  scripts. Workflow logic never lives in the adapters.
- Maintenance scripts under `.agents/skills/develop-{{NAME}}/scripts/` are
  maintainer tooling and are exempt from the twin rule.

## Validation Tiers

- Tier B (always): Agent Skills frontmatter compliance, SKILL.md relative-path
  resolution, and twin parity for product skills.
- Tier A (when the optional `pyodide` package is installed locally): Python
  entrypoints are additionally compiled and smoke-executed in a WebAssembly
  CPython sandbox without requiring native Python. This repository does not
  declare pyodide as a dependency; install it ad hoc for deeper checks.

## DeepSeek Harness Volatility

DeepSeek Harness (`dsh`) is a developer preview built on the Cordis kernel.
The generated contract pins only the verified surface: function-form named-
export plugins, `cordis.patch.yml` insert entries resolving npm module names,
and the `dsh.bundle.patch` package field. Re-verify against official dsh docs
before adopting deeper Cordis services.

## Metadata

`package.json` owns version, description, author, repository, license, and npm package name. `scripts/sync-metadata.mjs` propagates these values without replacing harness-specific fields.

The npm package basename must remain `{{NAME}}`; the package may be scoped.
