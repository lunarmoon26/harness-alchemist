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

## Harness-Specific Contract

| Harness | Files | Rule |
| --- | --- | --- |
| Claude Code | `.claude-plugin/` | Components stay at repository root; marketplace source is `./`. |
| Codex/ChatGPT | `.codex-plugin/`, `.agents/plugins/` | Local marketplace entries require policy and category. |
| OpenCode | `src/opencode.ts` | Package root exports a plugin function returning hooks. |
| Antigravity | `plugin.json` | Skills use nested `<name>/SKILL.md`. |
| DeepSeek | `src/deepseek.ts`, `cordis.patch.yml` | Function plugin uses named exports and no default export. |

## Metadata

`package.json` owns version, description, author, repository, license, and npm package name. `scripts/sync-metadata.mjs` propagates these values without replacing harness-specific fields.

The npm package basename must remain `harness-alchemist`; the package may be scoped.
