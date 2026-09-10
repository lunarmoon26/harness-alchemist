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
| Agent Plugins / Antigravity | `plugin.json`, `mcp.json` | Root metadata uses Agent Plugins 1.0; skills use nested `<name>/SKILL.md`. |
| DeepSeek | `src/deepseek.ts`, `cordis.patch.yml` | Function plugin injects `tools`, uses named exports, and has no default export. |

## Skill Script Contract

- `skills/{{NAME}}/skill-runtime.json` owns the portable schema and fixed
  executable. `scripts/` contains the implementation and direct-test twin.
- Contract: one JSON object on stdin, one JSON result plus newline on stdout,
  non-zero exit with a stderr diagnostic on failure. See
  `skills/{{NAME}}/references/tool-contract.md`.
- `src/opencode.ts` and `src/deepseek.ts` are thin adapters over
  `@lunarmoon26/agent-skill-runtime`. Workflow logic never lives in adapters.
- Maintenance scripts under `.agents/skills/develop-{{NAME}}/scripts/` are
  maintainer tooling and are exempt from the twin rule.

## Validation Tiers

- Tier B (always): Agent Skills frontmatter compliance, SKILL.md relative-path
  resolution, runtime-manifest validation, path containment, and twin parity.
- Tier A (when the optional `pyodide` package is installed locally): Python
  entrypoints are additionally compiled and smoke-executed in a WebAssembly
  CPython sandbox without requiring native Python. Pyodide is validation-only;
  this repository does not declare it as a dependency.

## DeepSeek Harness Volatility

DeepSeek Harness (`dsh`) is a developer preview built on the Cordis kernel.
The generated contract pins function-form named-export plugins, `tools`
injection, lifecycle-owned registrations, `cordis.patch.yml` insert entries,
and the `dsh.bundle.patch` package field. Re-verify against official dsh docs
before expanding the integration.

## Metadata

`package.json` owns version, description, author, repository, license, and npm package name. `scripts/sync-metadata.mjs` propagates these values without replacing harness-specific fields.

The npm package basename must remain `{{NAME}}`; the package may be scoped.
