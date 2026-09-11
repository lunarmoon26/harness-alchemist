# Repository Guidance

This package ships one plugin layout across Claude Code, Codex/ChatGPT, OpenCode, Antigravity, and DeepSeek Harness/Cordis.

## Contracts

- Load `.agents/skills/develop-harness-alchemist/` before changing scaffold paths or manifests; its compatibility reference defines the cross-harness contract.
- Keep the three guidance audiences separate: this repository's maintainer skill, `skills/harness-alchemist/` for CLI users, and each generated project's `.agents/skills/develop-<name>/`. See `docs/skill-tiers.md`.
- `templates/v0.1.0/` is the unreleased canonical scaffold. Update it in place until release; version incompatible generated-layout changes after release.
- `alchemy.json` is the explicit adaptation boundary for existing monorepos. Keep generated projects strict by default; adapted `pluginRoot` packages choose `runtime: "npm"` (full generated contract) or `runtime: "skills"` (runtime manifests and skills only, any language) and still satisfy the shared skill, manifest, and payload contracts for their mode.
- Keep the npm platform launcher in `bin/` and the native CLI implementation in `rust/`. The Rust validator owns CLI-time manifest checks; `templates/v0.1.0/generated/validate.mjs` remains the frozen local validator emitted into `v0.1.0` projects. OpenCode and DeepSeek adapters remain TypeScript.
- Keep portable workflows and skill-owned resources in `skills/`; executable skills own a `skill-runtime.json` that fixes the model-callable entrypoint. Additional prose-only skills may omit the manifest in adapted `runtime: "skills"` repositories, but the primary MCP-targeted skill must keep one. Product skills may keep a behavioral `.py` twin for direct compatibility testing. OpenCode runtime code in `src/opencode.ts` and Cordis runtime code in `src/deepseek.ts` are thin shared-runtime adapters, with named exports only (no default export in `src/deepseek.ts`).
- `package.json` owns shared metadata. After changing it, run `npm run sync` to propagate values, including the matching Cargo package version, without replacing harness-specific fields.
- Keep manifest paths repository-relative; plugin hosts may cache or copy payloads. Do not invent connector IDs, credentials, legal URLs, or presentation assets.

## Verification

- Node.js 22.20+, Bun 1.2+, and the pinned Rust toolchain are required for the full gate. Run `npm run verify` before reporting completion (`check`, native and adapter builds/tests, scaffold validation, npm-payload check).
- For focused coverage, use `node --test tests/cli.test.mjs`; runtime-entrypoint tests require `npm run build && node --test tests/runtimes.test.mjs` because they import `dist/`.
- When changing the CLI or templates, exercise a real recursive `create` then `validate` flow with both Node and Bun; creation never overwrites non-empty destinations.
