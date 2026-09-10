# Harness Alchemist Compatibility

## Shared Contract

- `skills/harness-alchemist/SKILL.md` is bundled natively by Claude Code, Codex, and Antigravity.
- `.agents/skills/develop-harness-alchemist/` is repository-local development guidance and is discoverable by Vercel Skills.
- OpenCode and DeepSeek users install shared skills separately from the npm runtime plugin.

## Self-Hosting Contract

- The repository root extends the universal layout produced by `templates/v0.1.0/universal-typescript/` with the Rust source and native npm distribution needed by the self-hosting CLI.
- `bin/harness-alchemist.mjs` is a minimal npm platform launcher. `rust/` owns the native `version`, `templates`, `create`, `validate`, and `install-check` implementation. The OpenCode and Cordis exports remain TypeScript because those hosts load JavaScript modules.
- `templates/v0.1.0/generated/validate.mjs` is emitted unchanged into generated `v0.1.0` projects. It is a versioned scaffold asset, not the implementation used by the native CLI.
- Every generated repository receives `.agents/skills/develop-<name>/`, so its own metadata and harness contracts can be maintained locally.
- Template identifiers include the `v` prefix. Never silently redirect a requested template version.
- Update v0.1.0 in place until release; preserve its generated behavior and create a new template directory for incompatible changes after release.

## Layout Manifest

`alchemy.json` is the repository's layout manifest, validated against
`alchemy.schema.json` (shipped in the npm package; `$schema` points at
the published copy). Generated projects carry one with `runtime: "npm"`,
`template`, `generator`, `generatorVersion`, and `createdAt`; `sync-metadata.mjs`
refreshes `generatorVersion` from `package.json`. Adapted repositories add
`pluginRoot` (and, in npm mode, `opencodeExport`). The validator rejects
unknown fields and cross-field conflicts such as `opencodeExport` outside
npm mode.

## Existing repository adaptation

An existing monorepo may place `alchemy.json` at its root with a
repository-relative `pluginRoot`. Project marketplaces and maintenance guidance
remain at the repository root; package manifests, product skills, runtime
sources, Cordis patch, and npm metadata live under `pluginRoot`.

The optional `runtime` field selects the adapted contract:

- `"npm"` (default) — the full generated contract: npm package, portable
  runtime manifests, OpenCode and Cordis adapters, and the Cordis patch.
- `"skills"` — skills and portable runtime manifests only. npm metadata,
  adapters, the Cordis patch, and twin parity are not required, enabling polyglot
  repositories (Python, Go, Rust, Java, C#, Swift) to expose skills without a
  JavaScript runtime. `opencodeExport` is rejected in this mode.

`opencodeExport` defaults to `.`. The explicit value `./server` allows an SDK to
retain its package-root export while publishing the OpenCode adapter from
`dist/opencode.js`. This mode relaxes only the generated package's exact Node
engine declaration; all shared-skill, manifest, Cordis, metadata, and payload
checks remain active.

## Harness-Specific Contract

| Harness | Files | Rule |
| --- | --- | --- |
| Claude Code | `.claude-plugin/` | Components stay at repository root; marketplace source is `./`. |
| Codex/ChatGPT | `.codex-plugin/`, `.agents/plugins/` | Local marketplace entries require policy and category. |
| OpenCode | `src/opencode.ts` | Package root exports a plugin function returning hooks. |
| Agent Plugins / Antigravity | `plugin.json`, `mcp.json` | Root metadata uses Agent Plugins 1.0; skills use nested `<name>/SKILL.md`. |
| DeepSeek | `src/deepseek.ts`, `cordis.patch.yml` | Function plugin injects `tools`, uses named exports, and has no default export. |

## Skill Script Contract

- Product skills (`skills/<name>/`) own `skill-runtime.json`, which declares
  portable schemas and an author-selected executable under `scripts/`.
- Generated npm projects include a behavioral `.py` twin for direct
  compatibility testing; it is not selected through model input.
- The I/O contract lives in the generated
  `skills/<name>/references/tool-contract.md`: one JSON object on stdin, one
  JSON result plus newline on stdout, non-zero exit with a stderr diagnostic
  on failure.
- Harness entrypoints are thin adapters over the shared runtime package;
  workflow logic never lives in `src/opencode.ts` or `src/deepseek.ts`.
- Maintenance skills under `.agents/skills/` are exempt from the twin rule;
  they may ship single-language maintainer tooling.

## Validation Tiers

- Tier B (always): Agent Skills frontmatter compliance, SKILL.md relative-path
  resolution, portable runtime-manifest validation, path containment, and
  npm-mode `.mjs`/`.py` twin parity.
- The native CLI parses Python twins with Rust and never executes project Python.
  Behavioral parity remains covered by runtime tests. Generated `v0.1.0`
  projects retain their local optional-Pyodide validator as part of that
  immutable scaffold contract.

## Native npm Distribution

- The `harness-alchemist` package contains the launcher, templates, skills, and
  JavaScript host adapters. Exact-version optional dependencies select one
  native package for macOS, static-musl Linux, or Windows on x64 or arm64.
- Platform packages are under `@lunarmoon26`, contain no install scripts or
  runtime dependencies, and are published before the main package.
- Every package uses the same `npm-publish.yml` Trusted Publisher and disallows
  traditional publishing tokens. See `docs/native-cli.md` for the accepted
  architecture and release consequences.

## DeepSeek Harness Volatility

DeepSeek Harness (`dsh`) is a developer preview built on the Cordis kernel.
The generated contract pins function-form named-export plugins, `tools`
injection, lifecycle-owned tool registration, `cordis.patch.yml` insert entries
resolving npm module names, and the `dsh.bundle.patch` package field. Re-verify
these against official Harness releases before expanding the integration.

## Metadata

`package.json` owns version, description, author, repository, license, and npm package name. `scripts/sync-metadata.mjs` propagates these values without replacing harness-specific fields and keeps `Cargo.toml` at the same version.

The npm package basename must remain `harness-alchemist`; the package may be scoped.
