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

Harness Alchemist scaffolds one TypeScript plugin repository that installs natively into **Claude Code**, **Codex/ChatGPT**, **OpenCode**, **Google Antigravity**, and **DeepSeek Harness/Cordis**. Each product skill declares its fixed executable and portable tool schema in `skill-runtime.json`; host entrypoints delegate through `@lunarmoon26/agent-skill-runtime`.

## Install

```bash
npm install -g harness-alchemist
# or run it without installing:
npx harness-alchemist@latest create my-plugin --help
```

The CLI implementation is a native Rust binary. npm and Bun installations use a
small launcher to select a prebuilt macOS, Linux, or Windows binary for x64 or
arm64. No Rust toolchain or install script is required by CLI users. Repository
development requires the pinned Rust toolchain in `rust-toolchain.toml`. The npm
launcher requires Node.js 22.20+; invoking it with Bun 1.2+ is also supported.

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
Add `alchemy.json` at the repository root:

```json
{
  "$schema": "https://unpkg.com/harness-alchemist/alchemy.schema.json",
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

- `"npm"` (default) — the full generated contract: npm metadata, portable
  runtime manifests, OpenCode and Cordis adapters, and the Cordis patch.
- `"skills"` — skills, portable runtime manifests, and harness manifests only.
  No npm package, adapters, or Cordis patch are required, and single-language scripts are allowed, so
  Python, Go, Rust, Java, C#, or Swift repositories can expose their workflows
  to Claude Code, Codex, Antigravity, and DeepSeek's filesystem skill roots
  without adopting a JavaScript runtime.

Generated projects include a `alchemy.json` manifest recording their
`runtime`, canonical `template` version, `generator`, `generatorVersion`, and
`createdAt`; `npm run sync` keeps `generatorVersion` aligned with the package
version.

## What you get

| Surface | Purpose |
| --- | --- |
| `skills/<name>/` | Agent Skills workflow, portable `skill-runtime.json`, fixed executable, optional tested twin, and tool-contract reference |
| `src/opencode.ts` | OpenCode plugin projecting the portable runtime manifest as native tools |
| `src/deepseek.ts` + `cordis.patch.yml` | Cordis function plugin registering the same portable tools |
| `.claude-plugin/`, `.codex-plugin/`, `.agents/plugins/`, `plugin.json`, `mcp.json` | Native and Agent Plugins manifests for Claude Code, Codex, Antigravity, and MCP hosts |
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
| DeepSeek Harness | profile filesystem roots | Cordis tool plugin | `dsh plugin --profile demo add <package-or-path>` |

## Skill script contract

Product skills own their logic; adapters never do.

`skills/<name>/skill-runtime.json` declares the model-facing schema and selects
one author-controlled entrypoint. Hosts validate the same manifest and never
accept an executable path from model input.

```bash
echo '{"request": "hello"}' | node skills/<name>/scripts/main.mjs
# {"ok":true,"plugin":"my-plugin","echo":{"request":"hello"}}
```

- One JSON object on stdin, one JSON result plus newline on stdout.
- Non-zero exit with a stderr diagnostic on failure.
- `scripts/main.py` is a stdlib-only behavioral twin for direct compatibility
  testing; the starter manifest selects `scripts/main.mjs` for host tools.

## Install-level verification

`install-check` drives your local harness CLIs against the project and asserts
each one can discover the plugin — the same checks a user's install would
perform, automated and cleaned up afterwards:

```bash
npx harness-alchemist@latest install-check /path/to/project
npx harness-alchemist@latest install-check . --harness agy --json
```

| Harness | Verified by | Isolation |
| --- | --- | --- |
| Claude Code | marketplace add → install → `plugin details` skill inventory | user scope, auto-removed |
| Codex | marketplace add → plugin add → `plugin list` enabled | plugin cache, auto-removed |
| Antigravity | `plugin validate` → install → `plugin list` | staged, auto-uninstalled |
| OpenCode | skills discovery via `debug skill` + plugin startup | temp `XDG_CONFIG_HOME` |
| DeepSeek | Cordis bundle composed into profile (`--dump-config`) | temp `DSH_HOME` |

Claude, Codex, and Antigravity run in both runtimes; the OpenCode plugin leg
and the DeepSeek Cordis check require npm mode with built adapters
(`npm run build` first). Missing CLIs are reported as skipped, not failures.

## Validation tiers

`npm run validate` always enforces Agent Skills frontmatter compliance, SKILL.md
reference resolution, runtime-manifest validation, fixed contained entrypoints,
npm-mode twin parity, and Python syntax through the native Rust parser. It does
not execute project Python during structural validation; JavaScript/Python
behavioral parity is exercised by the runtime tests.

## Release automation

The main package and six platform packages use GitHub Actions workflow
`npm-publish.yml` in `lunarmoon26/harness-alchemist` as their Trusted Publisher.
The workflow builds each native target on its matching hosted runner, publishes
platform packages first, then publishes the main package. It uses OIDC instead
of an `NPM_TOKEN`, requires the tag to match both `package.json` and
`Cargo.toml`, and accepts only commits contained in `main`.

Trusted Publisher configuration is package-scoped. Before the first native
release, create each new platform package once using an interactive npm session
with 2FA (a reserved `0.0.0` bootstrap version keeps real releases on OIDC), then
configure its publisher with npm 11.5.1 or newer:

```bash
npm trust github @lunarmoon26/harness-alchemist-darwin-arm64 \
  --repo lunarmoon26/harness-alchemist --file npm-publish.yml \
  --allow-publish --yes
```

Repeat that command for all six platform package names, then set each package's
publishing access to **Require two-factor authentication and disallow tokens**.
The existing main package must retain the same workflow identity and policy.

```bash
# bump package.json version, then:
npm run sync && npm run verify && git commit -am "Release vX.Y.Z" && git push
# after the release commit is merged to main:
git tag vX.Y.Z && git push origin vX.Y.Z   # publishes through npm Trusted Publishing
```

## License

[MIT](./LICENSE)
