# {{DISPLAY_NAME}}

{{DESCRIPTION}}

This repository packages one shared Agent Skill and a portable executable manifest for Claude Code, Codex/ChatGPT, OpenCode, Google Antigravity, and DeepSeek Harness/Cordis.

## Structure

```text
.claude-plugin/              Claude plugin and marketplace manifests
.codex-plugin/               Codex/ChatGPT plugin manifest
.agents/plugins/             Codex repository marketplace
.agents/skills/              Project development skill
.github/workflows/           GitHub release publishing
skills/                      Shared installable Agent Skills
skills/{{NAME}}/skill-runtime.json  Portable tool schema and fixed executable
src/opencode.ts              OpenCode npm plugin entrypoint
src/deepseek.ts              Cordis plugin entrypoint
cordis.patch.yml             DeepSeek Harness bundle layer
plugin.json                  Agent Plugins metadata
mcp.json                     Agent Plugins MCP server
```

The runtime entrypoints are thin adapters that project `skills/{{NAME}}/skill-runtime.json`; see its tool contract before extending it.

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
(including a prerelease tag such as `vX.Y.Z-rc.1`). The workflow requires the
tag to match the committed `package.json` version, checks that the tagged
commit is contained in `main`, verifies synchronized metadata and the package,
then publishes through npm Trusted Publishing with provenance.

Before the first release, configure the npm package's Trusted Publisher for
GitHub Actions with the repository owner, repository name, workflow filename
`npm-publish.yml`, and no environment. Do not create an `NPM_TOKEN` secret.

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
