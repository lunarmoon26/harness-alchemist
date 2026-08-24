# Repository Guidance

This package ships one plugin layout across Claude Code, Codex/ChatGPT, OpenCode, Antigravity, and DeepSeek Harness/Cordis.

## Contracts

- Load `.agents/skills/develop-harness-alchemist/` before changing scaffold paths or manifests; its compatibility reference defines the cross-harness contract.
- Keep the three guidance audiences separate: this repository's maintainer skill, `skills/harness-alchemist/` for CLI users, and each generated project's `.agents/skills/develop-<name>/`. See `docs/skill-tiers.md`.
- `templates/v0.1.0/` is the unreleased canonical scaffold. Update it in place until release; version incompatible generated-layout changes after release.
- Keep CLI dispatch in `bin/` and reusable, zero-dependency Node/Bun-compatible logic in `lib/`.
- Keep portable workflows and skill-owned scripts in `skills/`; product-skill entrypoints ship as behavioral `.mjs`/`.py` twins per the generated tool contract. OpenCode runtime code in `src/opencode.ts` and Cordis runtime code in `src/deepseek.ts` are thin adapters that delegate to skill scripts, with named exports only (no default export in `src/deepseek.ts`).
- `package.json` owns shared metadata. After changing it, run `npm run sync` to propagate values without replacing harness-specific fields.
- Keep manifest paths repository-relative; plugin hosts may cache or copy payloads. Do not invent connector IDs, credentials, legal URLs, or presentation assets.

## Verification

- Node.js 22.20+ is required for the full gate; Bun 1.2+ is also supported. Run `npm run verify` before reporting completion (`check`, build/tests, scaffold validation, npm-payload check).
- For focused coverage, use `node --test tests/cli.test.mjs`; runtime-entrypoint tests require `npm run build && node --test tests/runtimes.test.mjs` because they import `dist/`.
- When changing the CLI or templates, exercise a real recursive `create` then `validate` flow with both Node and Bun; creation never overwrites non-empty destinations.
