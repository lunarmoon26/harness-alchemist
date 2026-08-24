---
name: harness-alchemist
description: "Use the Harness Alchemist CLI to create or validate universal coding-agent plugins for Claude Code, Codex/ChatGPT, OpenCode, Google Antigravity, and DeepSeek Harness/Cordis."
compatibility: Requires Node.js 22.20+ or Bun 1.2+ for the CLI. Platform CLIs are optional and only needed for live installation checks.
---

# Harness Alchemist

Use the CLI to create or validate a universal plugin repository. This is the
tier-2 end-user workflow: it does not maintain Harness Alchemist itself or
replace the generated repository's local maintenance skill.

## Start Here

1. Inspect the target directory before changing it.
2. Read [references/compatibility.md](references/compatibility.md) before choosing paths or manifests.
3. For a new project, run `harness-alchemist create --help` and use the CLI instead of recreating the structure manually.
4. After creation, load `.agents/skills/develop-<name>/` in the generated project before maintaining its files.
5. Run `harness-alchemist validate` and the generated package's `verify` script before reporting completion.

## Create A Project

Gather a lowercase kebab-case plugin name, description, npm package name, author, repository, and license. Then run:

```bash
harness-alchemist create /absolute/path/to/project \
  --name my-plugin \
  --description "What the plugin does" \
  --package @scope/my-plugin \
  --author "Example Team" \
  --repository example/my-plugin
```

Use `bunx harness-alchemist create ...` or `npx harness-alchemist create ...` when the CLI is not installed. The CLI only writes to a missing or empty destination.

## After Generation

The generated `skills/<name>/SKILL.md` is the plugin's end-user workflow
starter. The generated `.agents/skills/develop-<name>/` is its maintenance
guide; use that local skill for file ownership, metadata, and harness changes.

List supported template versions with:

```bash
harness-alchemist templates
```

## Verify

```bash
harness-alchemist validate /path/to/project
cd /path/to/project
npm install
npm run verify
npm pack --dry-run
```

Static validation is structural evidence, not proof that remote services or host-specific lifecycle events work.

## Skill Runtime

`scripts/main.mjs` (Node/Bun) and `scripts/main.py` (CPython 3.10+) are
behavioral twins demonstrating this repository's own tool contract: one JSON
object on stdin, one JSON result plus newline on stdout, non-zero exit with a
stderr diagnostic on failure. The `src/` adapters delegate to them; keep any
contract change mirrored in both twins and in
[references/compatibility.md](references/compatibility.md).
