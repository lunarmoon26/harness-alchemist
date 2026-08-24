<p align="center">
  <a href="https://github.com/lunarmoon26/harness-alchemist">
    <picture>
      <source srcset="assets/harness-alchemist_dark.svg" media="(prefers-color-scheme: dark)">
      <source srcset="assets/harness-alchemist_light.svg" media="(prefers-color-scheme: light)">
      <img src="assets/harness-alchemist_light.svg" alt="Harness Alchemist logo" height="110">
    </picture>
  </a>
</p>

<p align="center">The universal scaffold for coding-agent plugins. One repository, five harnesses, skills that own their runtime.</p>
<p align="center">
  <a href="https://www.npmjs.com/package/harness-alchemist" target="blank">
    <img src="https://img.shields.io/npm/v/harness-alchemist?style=flat-square" alt="Npm package for Harness Alchemist">
  </a>
  <a href="./LICENSE">
    <img alt="License: MIT" src="https://img.shields.io/github/license/lunarmoon26/harness-alchemist?style=flat-square" />
  </a>
  <a href="https://github.com/lunarmoon26/harness-alchemist/actions/workflows/ci.yml" target="blank">
    <img src="https://img.shields.io/github/actions/workflow/status/lunarmoon26/harness-alchemist/ci.yml?branch=main&style=flat-square&label=CI" alt="CI status for Harness Alchemist">
  </a>
  <br /><br />
</p>

<p align="center">
  Follow <a href="https://x.com/haochuanzero">@haochuanzero on X</a> for updates · Start from the <a href="https://blog.haochuanz.net/harness-alchemist/">one-page tour</a>.
</p>

---

Harness Alchemist scaffolds one TypeScript plugin repository that installs natively into **Claude Code**, **Codex/ChatGPT**, **OpenCode**, **Google Antigravity**, and **DeepSeek Harness/Cordis**. Product skills ship behavioral `.mjs`/`.py` script twins; every harness entrypoint is a thin adapter that delegates to them.

## Install

```bash
npm install -g harness-alchemist
# or run it without installing:
npx harness-alchemist@latest create my-plugin --help
```

Requires Node.js 22.20+ (Bun 1.2+ also supported). Zero runtime dependencies.

## Quick start

```bash
npx harness-alchemist@latest create my-plugin \
  --description "What the plugin does" \
  --author "Example Team" \
  --repository example/my-plugin

cd my-plugin && npm install && npm run verify
```

Creation only writes to a missing or empty destination. The generated project passes its own gate out of the box: TypeScript checks, runtime delegation tests, scaffold validation, and an npm-payload audit.

## Existing monorepos and SDK packages

`validate` also supports an adapted plugin package inside an existing repository.
Add `harness-alchemist.json` at the repository root:

```json
{
  "$schema": "https://unpkg.com/harness-alchemist/harness-alchemist.schema.json",
  "pluginRoot": "packages/my-sdk",
  "opencodeExport": "./server"
}
```

The manifest is JSON-Schema-validated; the schema ships in the npm package and
is referenced through `$schema`, so editors autocomplete and check every field.

`pluginRoot` contains the canonical plugin manifests, shared skills, Cordis
patch, adapter sources, and publishable package metadata. Repository marketplace
manifests remain at the project root and point at that package directory.
`opencodeExport: "./server"` preserves an SDK at the package root while exposing
the OpenCode adapter through the modern server entrypoint. Omitting the file
retains the strict generated single-package layout.

The optional `runtime` field selects what the adapted package must contain:

- `"npm"` (default) — the full generated contract: npm metadata, OpenCode and
  Cordis adapters, Cordis patch, and `.mjs`/`.py` script twins.
- `"skills"` — skills and harness manifests only. No npm package, adapters, or
  Cordis patch are required, and single-language scripts are allowed, so
  Python, Go, Rust, Java, C#, or Swift repositories can expose their workflows
  to Claude Code, Codex, Antigravity, and DeepSeek's filesystem skill roots
  without adopting a JavaScript runtime.

Generated projects include a `harness-alchemist.json` manifest recording their
`runtime`, canonical `template` version, `generator`, `generatorVersion`, and
`createdAt`; `npm run sync` keeps `generatorVersion` aligned with the package
version.

## What you get

| Surface | Purpose |
| --- | --- |
| `skills/<name>/` | Agent Skills spec skill with `.mjs`/`.py` script twins under `scripts/` and a tool-contract reference |
| `src/opencode.ts` | OpenCode plugin registering tools that spawn the skill scripts |
| `src/deepseek.ts` + `cordis.patch.yml` | Cordis function plugin providing a service over the same scripts |
| `.claude-plugin/`, `.codex-plugin/`, `.agents/plugins/`, `plugin.json` | Native manifests for Claude Code, Codex, and Antigravity marketplaces |
| `.agents/skills/develop-<name>/` | Repository-maintenance skill so agents can develop the project recursively |
| `.github/workflows/` | CI plus tag-triggered publish (`vX.Y.Z` → verify → npm provenance) |

## Supported harnesses

Install paths below are verified against real CLIs before shipping in the template.

| Harness | Skills | Runtime | Install |
| --- | --- | --- | --- |
| Claude Code | bundled natively | — | `claude plugin install <name>@<marketplace>` |
| Codex/ChatGPT | bundled natively | — | `codex plugin add <name>@<marketplace>` |
| OpenCode | via `~/.agents/skills/` | npm package hooks | `"plugin": ["<package>"]` in `opencode.json` |
| Google Antigravity | nested bundle | — | `agy plugin validate . && agy plugin install .` |
| DeepSeek Harness | profile filesystem roots | Cordis service plugin | `dsh plugin --profile demo add <package-or-path>` |

## Skill script contract

Product skills own their logic; adapters never do.

```bash
echo '{"request": "hello"}' | node skills/<name>/scripts/main.mjs
# {"ok":true,"plugin":"my-plugin","echo":{"request":"hello"}}
```

- One JSON object on stdin, one JSON result plus newline on stdout.
- Non-zero exit with a stderr diagnostic on failure.
- `scripts/main.py` is a stdlib-only behavioral twin of `scripts/main.mjs`.

## Validation tiers

`npm run validate` always enforces Agent Skills frontmatter compliance, SKILL.md reference resolution, and twin parity. With the optional `pyodide` devDependency installed, Python entrypoints are additionally compiled and smoke-executed inside a WebAssembly CPython sandbox — no native Python required.

## Release automation

```bash
# bump package.json version, then:
npm run sync && npm run verify && git commit -am "Release vX.Y.Z" && git push
git tag vX.Y.Z && git push origin vX.Y.Z   # publishes automatically
```

## License

[MIT](./LICENSE)
