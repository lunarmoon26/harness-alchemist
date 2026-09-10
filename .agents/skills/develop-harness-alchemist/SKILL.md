---
name: develop-harness-alchemist
description: "Develop, validate, and publish the Harness Alchemist self-hosting CLI and universal coding-agent plugin. Use when modifying its canonical templates, create or validate commands, shared skills, harness manifests, npm package, or runtime entrypoints."
compatibility: Requires Node.js 22.20+, Bun 1.2+, and the pinned Rust toolchain for the full repository gate.
---

# Develop Harness Alchemist

Maintain this repository as one main npm package, its native platform artifacts,
shared Agent Skills, and separate harness runtime contracts.

This is the tier-1 maintainer workflow. Keep its self-hosting guidance separate
from the published `skills/harness-alchemist/` CLI-user skill and the concise
maintenance skill generated at `.agents/skills/develop-<name>/`. See
`docs/skill-tiers.md` before moving guidance between those audiences.

## Workflow

1. Read `references/compatibility.md` before changing paths or manifests.
2. Keep the npm platform launcher in `bin/`, the native CLI in `rust/`, and required JavaScript host adapters in `src/`.
3. Treat `templates/v0.1.0/` as the unreleased canonical generated layout. Update it in place until release; add a new version only for incompatible post-release changes.
4. Keep portable workflow instructions in `skills/`.
5. Keep OpenCode code in `src/opencode.ts` and Cordis code in `src/deepseek.ts`.
6. Change canonical metadata in `package.json`, then run `npm run sync`.
7. Run `npm run verify` after every structural or runtime change.
8. Test the native binary directly and through both `node bin/harness-alchemist.mjs` and `bun bin/harness-alchemist.mjs` after CLI changes.
9. Keep each product skill's `skill-runtime.json` authoritative for executable paths and portable tool schemas; adapters must not accept a model-selected script path.
10. Run `npm run pack:check` before publishing; the source-tree tarball is not the staged native package.

Do not add fake legal URLs, application IDs, credentials, assets, or connector metadata. Ask for real values when a publishing surface requires them.

## Commands

```bash
npm run sync
npm run verify
npm run pack:check
cargo test --locked
target/release/harness-alchemist templates
node bin/harness-alchemist.mjs templates
bun bin/harness-alchemist.mjs templates
```

If Claude Code is installed, also run:

```bash
claude plugin validate . --strict
```
