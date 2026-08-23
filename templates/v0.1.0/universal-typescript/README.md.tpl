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

The initial runtime entrypoints are intentionally inert. Add only the hooks, tools, or services the plugin actually needs.

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

`.github/workflows/npm-publish.yml` publishes when a GitHub release is
published. Tag the release as `vX.Y.Z` (or a semver prerelease such as
`vX.Y.Z-rc.1`); the workflow applies that version, synchronizes manifests,
verifies the package, and publishes with npm provenance.

Configure the repository `NPM_TOKEN` secret with an npm publish token before
creating a release.

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
```

Open `/plugins`, install **{{DISPLAY_NAME}}**, and start a new session.

## OpenCode

After publishing `{{PACKAGE_NAME}}`, add it to `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "plugin": [{{PACKAGE_NAME_JSON}}]
}
```

Install the shared skill separately:

```bash
npx skills add {{REPOSITORY_SOURCE}} --agent opencode
```

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

The bundle loads `{{PACKAGE_NAME}}/deepseek`. Install the shared Agent Skill separately when the workflow needs it:

```bash
npx skills add {{REPOSITORY_SOURCE}}
```

## License

{{LICENSE}}
