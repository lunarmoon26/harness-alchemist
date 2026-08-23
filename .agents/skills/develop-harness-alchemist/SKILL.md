---
name: develop-harness-alchemist
description: "Develop, validate, and publish the Harness Alchemist self-hosting CLI and universal coding-agent plugin. Use when modifying its canonical templates, create or validate commands, shared skills, harness manifests, npm package, or runtime entrypoints."
compatibility: Requires Node.js 22.20+ for build and package scripts; the zero-dependency CLI also supports Bun 1.2+.
---

# Develop Harness Alchemist

Maintain this repository as one package with shared Agent Skills and separate harness runtime contracts.

This is the tier-1 maintainer workflow. Keep its self-hosting guidance separate
from the published `skills/harness-alchemist/` CLI-user skill and the concise
maintenance skill generated at `.agents/skills/develop-<name>/`. See
`docs/skill-tiers.md` before moving guidance between those audiences.

## Workflow

1. Read `references/compatibility.md` before changing paths or manifests.
2. Keep the CLI dispatcher in `bin/` and reusable create and validation logic in `lib/`.
3. Treat `templates/v0.1.0/` as the unreleased canonical generated layout. Update it in place until release; add a new version only for incompatible post-release changes.
4. Keep portable workflow instructions in `skills/`.
5. Keep OpenCode code in `src/opencode.ts` and Cordis code in `src/deepseek.ts`.
6. Change canonical metadata in `package.json`, then run `npm run sync`.
7. Run `npm run verify` after every structural or runtime change.
8. Test both `node bin/harness-alchemist.mjs` and `bun bin/harness-alchemist.mjs` after CLI changes.
9. Run `npm pack --dry-run` before publishing and inspect the included files.

Do not add fake legal URLs, application IDs, credentials, assets, or connector metadata. Ask for real values when a publishing surface requires them.

## Commands

```bash
npm run sync
npm run verify
npm run pack:check
npm pack --dry-run
node bin/harness-alchemist.mjs templates
bun bin/harness-alchemist.mjs templates
```

If Claude Code is installed, also run:

```bash
claude plugin validate . --strict
```
