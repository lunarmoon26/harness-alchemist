# {{DISPLAY_NAME}}

{{DESCRIPTION}}

This repository packages one shared Agent Skill and separate runtime entrypoints for Claude Code, Codex/ChatGPT, OpenCode, Google Antigravity, and DeepSeek Harness/Cordis.

## Structure

```text
.claude-plugin/              Claude plugin and marketplace manifests
.codex-plugin/               Codex/ChatGPT plugin manifest
.agents/plugins/             Codex repository marketplace
.agents/skills/              Project development skill
.github/workflows/           GitHub release publishing
skills/                      Shared installable Agent Skills
src/opencode.ts              OpenCode npm plugin entrypoint
src/deepseek.ts              Cordis plugin entrypoint
cordis.patch.yml             DeepSeek Harness bundle layer
plugin.json                  Antigravity plugin manifest
```

The runtime entrypoints are thin adapters that delegate to skill scripts; see `skills/{{NAME}}/references/tool-contract.md` before extending them.

## Skill Boundaries

`skills/{{NAME}}/SKILL.md` is the plugin's shared end-user workflow. Replace
its starter procedure with the plugin's domain-specific behavior.

`.agents/skills/develop-{{NAME}}/` is this repository's maintenance skill. It
identifies where shared skills, manifests, runtime code, and metadata belong;
load it before changing those surfaces.

## Development

Requires Node.js 22.20 or newer.

```bash
npm install
npm run verify
```

After changing the version, description, author, repository, or license in `package.json`, synchronize the harness manifests:

```bash
npm run sync
npm run verify
```

Inspect the npm payload before publishing:

```bash
npm pack --dry-run
```

## GitHub Release Publishing

`.github/workflows/npm-publish.yml` publishes when a `vX.Y.Z` tag is pushed
(or a GitHub release with a semver tag such as `vX.Y.Z-rc.1` is published);
the workflow applies that version, synchronizes manifests,
verifies the package, and publishes with npm provenance.

Configure the repository `NPM_TOKEN` secret with an npm publish token before
pushing the tag.

## Claude Code

```bash
claude plugin marketplace add {{REPOSITORY_SOURCE}}
claude plugin install {{NAME}}@{{MARKETPLACE}}
```

For local development:

```bash
claude --plugin-dir .
claude plugin validate . --strict
```

## Codex and ChatGPT

```bash
codex plugin marketplace add {{REPOSITORY_SOURCE}}
codex plugin add {{NAME}}@{{MARKETPLACE}}
codex plugin list
```

The `@{{MARKETPLACE}}` suffix is required; a bare plugin name is rejected.

## OpenCode

After publishing `{{PACKAGE_NAME}}`, add it to `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "plugin": [{{PACKAGE_NAME_JSON}}]
}
```

For an unpublished checkout, point the entry at the built adapter instead:

```json
{
  "plugin": ["file:///absolute/path/to/{{NAME}}/dist/opencode.js"]
}
```

Skills are discovered from `~/.agents/skills/`, not from the OpenCode config
directory, and symlinked directories are skipped. Copy each shared skill you
need:

```bash
cp -R skills/{{NAME}} ~/.agents/skills/
```

Verify with `opencode debug skill`.

## Google Antigravity

Clone the repository and install its root as a plugin:

```bash
agy plugin install /absolute/path/to/{{NAME}}
```

Antigravity also discovers repository-local development skills from `.agents/skills/`.

## DeepSeek Harness

After publishing the npm package:

```bash
dsh plugin --profile demo add {{PACKAGE_NAME}}
dsh --profile demo --dump-config
```

For an unpublished checkout, `dsh plugin add` forwards to pnpm, so a local
path works and stays linked to your working tree:

```bash
dsh plugin --profile demo add /absolute/path/to/{{NAME}}
```

The bundle loads `{{PACKAGE_NAME}}/deepseek` through `package.json`'s
`dsh.bundle.patch` entry. Verify composition with
`dsh --profile demo --dump-config`, which should list an insert for
`{{NAME}}`.

## License

{{LICENSE}}
