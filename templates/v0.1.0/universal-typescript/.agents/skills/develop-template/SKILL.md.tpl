---
name: develop-{{NAME}}
description: {{DEVELOPMENT_SKILL_DESCRIPTION_JSON}}
compatibility: Requires Node.js 22.20 or newer for project scripts.
---

# Maintain {{DISPLAY_NAME}}

Use this tier-3 skill to maintain this generated plugin repository. It does not
describe development of the Harness Alchemist CLI or its canonical templates.

## Repository Layout

- `skills/{{NAME}}/SKILL.md` is the plugin's end-user workflow. Replace the
  starter procedure with the capability's domain-specific instructions.
- `skills/{{NAME}}/scripts/` owns the skill runtime as `.mjs`/`.py` twins;
  see `skills/{{NAME}}/references/tool-contract.md` before changing them.
- `.agents/skills/develop-{{NAME}}/` is this repository's maintenance guidance
  and validation tooling.
- `.claude-plugin/`, `.codex-plugin/`, `.agents/plugins/`, root `plugin.json`,
  and `cordis.patch.yml` are harness-specific manifests and bundles.
- `src/opencode.ts` and `src/deepseek.ts` contain separate OpenCode and Cordis
  runtime behavior.
- `package.json` owns shared metadata; use `npm run sync` after changing it.

## Maintenance

1. Read `references/compatibility.md` before changing manifests or entrypoints.
2. Put portable end-user workflows and skill-owned scripts in `skills/` and
   host-specific runtime code, kept to thin delegation adapters, in its
   corresponding entrypoint.
3. Run `npm run verify` before reporting completion. Before publishing, inspect
   `npm pack --dry-run` and run `claude plugin validate . --strict` when Claude
   Code is installed.

Do not add invented credentials, connector IDs, legal URLs, or public metadata.
