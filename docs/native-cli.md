# Native Rust CLI

Status: Accepted

## Decision

Harness Alchemist implements its command-line interface as a native Rust binary.
The public `harness-alchemist` npm package remains the cross-platform entrypoint:
a minimal JavaScript launcher selects and executes an exact-version native binary
from an npm optional dependency. OpenCode and DeepSeek package exports remain
JavaScript because those hosts load JavaScript plugin modules.

The package does not use WebAssembly or lifecycle scripts to install binaries.
Platform packages contain binaries at publish time and use npm `os` and `cpu`
constraints so an install selects only the compatible artifact.

## Compatibility Contract

- `version`, `templates`, `create`, `validate`, and `install-check` retain their
  command names, arguments, output shapes, and exit-code meanings.
- `init` and `new` remain aliases for `create`.
- Generated `v0.1.0` projects retain their existing file layout and local
  JavaScript maintenance scripts. The native rewrite does not silently redirect
  or mutate the versioned scaffold contract.
- Canonical generated files come from the published template payload. Untracked
  operating-system metadata such as `.DS_Store` is never emitted.
- Scaffold creation remains no-overwrite, stages beside the destination, validates
  the generated tree, and renames it into place only after validation succeeds.
- Path containment checks continue to reject lexical and symlink escapes.
- `install-check` continues to execute installed harness CLIs and clean up temporary
  or installed state unless `--keep` is supplied.

## Python Validation

The native validator parses Python twins with a Rust Python parser. It does not
execute project Python inside Pyodide and does not execute untrusted project code
through the host Python installation. Behavioral parity between generated
JavaScript and Python twins remains an automated test responsibility.

This intentionally replaces the optional Pyodide validation tier. It keeps the
CLI runtime fully native, avoids bundling an interpreter, and preserves the
validator's security boundary.

## Distribution

The unscoped `harness-alchemist` package remains the package users install. It
depends optionally on exact-version packages under the `@lunarmoon26` scope for:

- macOS x64 and arm64;
- Linux x64 and arm64, using static musl binaries that also run on glibc systems;
- Windows x64 and arm64.

The JavaScript launcher is distribution glue only. Direct release binaries may
also be installed without the launcher when minimum process startup time matters.

Every platform package is built before publication. Release automation publishes
platform packages first and the main package last, uses npm Trusted Publishing,
and never downloads executable code during package installation.
Prerelease versions use npm's `next` dist-tag so the existing stable `latest`
release remains the default install.

Trusted Publisher relationships can only be configured from an existing npm
package. Each platform name therefore needs a one-time interactive 2FA bootstrap
before the first native release. Prefer a reserved `0.0.0` package so every real
binary version is subsequently published by OIDC with automatic provenance.
Configure each package for repository `lunarmoon26/harness-alchemist`, workflow
`npm-publish.yml`, no environment, and `npm publish` permission, then require 2FA
and disallow traditional publishing tokens in its npm package settings.

## Development Benchmark

A warmed local startup comparison on an Apple M4 Pro running macOS 26.6.2,
Node.js 24.19.0, and Rust 1.96.1 measured 50 `version` invocations through the
same `spawnSync` harness:

| Entrypoint | Median | p95 |
| --- | ---: | ---: |
| Published `0.1.10` JavaScript CLI | 67.34 ms | 70.71 ms |
| Native npm launcher | 29.68 ms | 31.89 ms |
| Direct Rust binary | 2.89 ms | 3.56 ms |

The npm path reduced median startup by about 56%, while direct execution reduced
it by about 96%. These development measurements include process-spawn overhead
and are directional rather than a release performance guarantee.

## Acceptance

- Existing CLI compatibility tests pass against the native binary.
- Generated trees remain byte-compatible except for volatile timestamps.
- Rust tests cover parsing, validation, path containment, and command failures.
- npm launcher tests cover native selection, argument forwarding, signals, and
  unsupported platforms.
- CI builds and smoke-tests every supported target family.
- Package audits prove the main package contains the launcher and each platform
  package contains exactly its expected native executable and metadata.
- Release checks require one version across `package.json`, `Cargo.toml`, the tag,
  and every generated npm package.

## Consequences

- Normal npm execution still pays the startup cost of a small Node launcher.
- The release publishes seven npm packages per version instead of one.
- Trusted Publisher and publishing-access policy must be configured for every new
  platform package before automated releases can publish them.
- Rust becomes a required maintainer and CI toolchain; Node and Bun remain required
  for host-adapter builds, tests, and the npm launcher.
