# Harness Alchemist

Scaffold, validate, and publish portable coding-agent plugins across Claude Code, Codex/ChatGPT, OpenCode, Google Antigravity, and DeepSeek Harness/Cordis.

Harness Alchemist is a zero-runtime-dependency CLI for Node.js and Bun. Its own repository follows the same universal plugin layout that it generates. The first frozen scaffold is `templates/v0.1.0`.

## Quick Start

Run with Node.js:

```bash
npx harness-alchemist create ./my-plugin \
  --description "What the plugin does" \
  --author "Example Team" \
  --repository example/my-plugin
```

Or with Bun:

```bash
bunx harness-alchemist create ./my-plugin \
  --description "What the plugin does" \
  --author "Example Team" \
  --repository example/my-plugin
```

The destination must be missing or empty. Harness Alchemist renders into a staging directory, validates the result, and moves it into place without replacing existing content.

## Commands

```text
harness-alchemist create <directory> [options]
harness-alchemist validate [directory] [--external] [--json]
harness-alchemist templates
harness-alchemist version
```

`init` and `new` are aliases for `create`. Run `harness-alchemist create --help` for metadata and license options.

New projects use the unreleased canonical `v0.1.0` template. Unknown versions
fail instead of silently selecting a different layout.

## Generated Layout

```text
.agents/
  plugins/                       Codex repository marketplace
  skills/develop-<name>/         Project-local maintenance skill and scripts
.claude-plugin/                  Claude plugin and marketplace manifests
.codex-plugin/                   Codex/ChatGPT plugin manifest
skills/<name>/SKILL.md           Shared installable Agent Skill
src/opencode.ts                  OpenCode npm plugin entrypoint
src/deepseek.ts                  Cordis plugin entrypoint
cordis.patch.yml                 DeepSeek Harness bundle layer
plugin.json                      Antigravity plugin manifest
package.json                     Canonical package metadata
```

Generated repositories start at version `0.1.0` with inert OpenCode and Cordis entrypoints. Add only the runtime hooks or services the plugin actually needs.

The recursive part is intentional: Harness Alchemist itself uses this layout, and every generated repository receives `.agents/skills/develop-<name>/` with its own sync, validation, package-payload, and compatibility workflow.

## Guidance Tiers

The repository-maintainer skill, the published CLI skill, and a generated
project's local maintenance skill have different audiences and scopes. See
[docs/skill-tiers.md](docs/skill-tiers.md) before moving guidance between them.

## Canonical Template

`templates/v0.1.0/universal-typescript/` is the unreleased canonical scaffold. It includes the generated project's local maintenance skill, scoped to that project's ownership boundary. `lib/create.mjs` supplies the version-specific validator and license during the atomic render.

Future incompatible scaffold changes should create a new `templates/<version>/`
snapshot instead of mutating `v0.1.0` after release.

## Local Development

Requires Node.js 22.20 or newer for the full build and package checks. The scaffolding and validation CLI also runs directly with Bun 1.2 or newer.

```bash
npm install
npm run verify
node ./bin/harness-alchemist.mjs templates
bun ./bin/harness-alchemist.mjs templates
```

After changing package metadata:

```bash
npm run sync
npm run verify
```

Inspect the publish payload before release:

```bash
npm pack --dry-run
```

## Package Surfaces

- npm binary: `harness-alchemist`
- Shared Agent Skill: `skills/harness-alchemist/SKILL.md`
- OpenCode package-root export: `harness-alchemist`
- Cordis export: `harness-alchemist/deepseek`
- DeepSeek bundle patch: `harness-alchemist/cordis.patch.yml`

The runtime entrypoints are intentionally inert in v0.1.0. The CLI and Agent Skill are the implemented product surfaces.

## Harness Installation

Claude Code:

```bash
claude plugin marketplace add lunarmoon26/harness-alchemist
claude plugin install harness-alchemist@harness-alchemist-plugins
```

Codex and ChatGPT:

```bash
codex plugin marketplace add lunarmoon26/harness-alchemist
```

OpenCode shared skill:

```bash
npx skills add lunarmoon26/harness-alchemist --agent opencode
```

DeepSeek Harness after npm publication:

```bash
dsh plugin --profile demo add harness-alchemist
dsh --profile demo --dump-config
```

## License

MIT
