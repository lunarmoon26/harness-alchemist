# Harness Alchemist Compatibility

## Shared Contract

- `skills/harness-alchemist/SKILL.md` is bundled natively by Claude Code, Codex, and Antigravity.
- `.agents/skills/develop-harness-alchemist/` is repository-local development guidance and is discoverable by Vercel Skills.
- OpenCode and DeepSeek users install shared skills separately from the npm runtime plugin.

## Self-Hosting Contract

- The repository root follows the universal layout produced by `templates/v0.1.0/universal-typescript/`, the unreleased canonical scaffold.
- `bin/harness-alchemist.mjs` is the npm executable; `lib/create.mjs` and `lib/validate.mjs` remain zero-dependency Node/Bun modules.
- Every generated repository receives `.agents/skills/develop-<name>/`, so its own metadata and harness contracts can be maintained locally.
- Template identifiers include the `v` prefix. Never silently redirect a requested template version.
- Update v0.1.0 in place until release; preserve its generated behavior and create a new template directory for incompatible changes after release.

## Existing repository adaptation

An existing monorepo may place `harness-alchemist.json` at its root with a
repository-relative `pluginRoot`. Project marketplaces and maintenance guidance
remain at the repository root; package manifests, product skills, runtime
sources, Cordis patch, and npm metadata live under `pluginRoot`.

`opencodeExport` defaults to `.`. The explicit value `./server` allows an SDK to
retain its package-root export while publishing the OpenCode adapter from
`dist/opencode.js`. This mode relaxes only the generated package's exact Node
engine declaration; all shared-skill, manifest, Cordis, metadata, and payload
checks remain active.

## Harness-Specific Contract

| Harness | Files | Rule |
| --- | --- | --- |
| Claude Code | `.claude-plugin/` | Components stay at repository root; marketplace source is `./`. |
| Codex/ChatGPT | `.codex-plugin/`, `.agents/plugins/` | Local marketplace entries require policy and category. |
| OpenCode | `src/opencode.ts` | Package root exports a plugin function returning hooks. |
| Antigravity | `plugin.json` | Skills use nested `<name>/SKILL.md`. |
| DeepSeek | `src/deepseek.ts`, `cordis.patch.yml` | Function plugin uses named exports and no default export. |

## Skill Script Contract

- Product skills (`skills/<name>/`) own their runtime in `scripts/`.
  Entrypoints come in behavioral `.mjs`/`.py` twins with identical names.
- The I/O contract lives in the generated
  `skills/<name>/references/tool-contract.md`: one JSON object on stdin, one
  JSON result plus newline on stdout, non-zero exit with a stderr diagnostic
  on failure.
- Harness entrypoints are thin adapters that spawn these scripts; workflow
  logic never lives in `src/opencode.ts` or `src/deepseek.ts`.
- Maintenance skills under `.agents/skills/` are exempt from the twin rule;
  they may ship single-language maintainer tooling.

## Validation Tiers

- Tier B (always): Agent Skills frontmatter compliance, SKILL.md relative-path
  resolution, and `.mjs`/`.py` twin parity for product skills.
- Tier A (when the optional `pyodide` devDependency resolves): Python
  entrypoints are additionally compiled and smoke-executed against the tool
  contract inside a WebAssembly CPython sandbox. Missing pyodide degrades to a
  warning, never an error. Generated projects do not depend on pyodide.

## DeepSeek Harness Volatility

DeepSeek Harness (`dsh`) is a developer preview built on the Cordis kernel.
The generated contract pins only the verified surface: function-form named-
export plugins, `cordis.patch.yml` insert entries resolving npm module names,
and the `dsh.bundle.patch` package field. Re-verify these against the official
docs before relying on deeper Cordis services such as tool registration.

## Metadata

`package.json` owns version, description, author, repository, license, and npm package name. `scripts/sync-metadata.mjs` propagates these values without replacing harness-specific fields.

The npm package basename must remain `harness-alchemist`; the package may be scoped.
