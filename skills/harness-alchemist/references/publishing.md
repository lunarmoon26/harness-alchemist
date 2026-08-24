# Publishing and Installation

## Before Publishing

Run the full verification ladder before distributing:

1. `harness-alchemist validate <project>` — static contract for all harnesses.
2. The project's own `verify` script — build plus runtime delegation tests.
3. `harness-alchemist install-check <project>` — drives the local harness CLIs
   (claude, codex, agy, opencode, dsh) and asserts real discovery. Claude,
   Codex, and Antigravity verify in both runtimes; the OpenCode plugin leg and
   DeepSeek Cordis check need npm mode with built adapters. Missing CLIs are
   reported as skipped. Use `--keep` to leave installs in place for manual
   inspection, and `--harness <id>` to target one harness.

Run:

```bash
npm install
npm run sync
npm run verify
npm pack --dry-run
```

Inspect the dry-run payload. It must contain:

- `dist/opencode.js` and `dist/deepseek.js`.
- `cordis.patch.yml`.
- `.claude-plugin/plugin.json`.
- `.codex-plugin/plugin.json`.
- Root `plugin.json`.
- Shared `skills/`.
- README and license.

Marketplace catalogs are Git repository entrypoints and do not need to ship in the npm tarball.

## Git Repository Distribution

```bash
claude plugin marketplace add owner/repo
codex plugin marketplace add owner/repo
npx skills add owner/repo
```

Claude and Codex marketplace files both point to the repository root. Vercel Skills discovers both `skills/` and `.agents/skills/`.

## npm Distribution

```bash
npm publish
```

OpenCode users add the package name to `opencode.json`. DeepSeek users add the same package to a profile with `dsh plugin`.

The package root exports OpenCode. The `./deepseek` subpath exports Cordis. Keep both compiled and included in `files`.

## GitHub Release Publishing

`.github/workflows/npm-publish.yml` publishes when a `vX.Y.Z` tag is pushed
(or a GitHub release with a semver tag such as `vX.Y.Z-rc.1` is published);
the workflow applies that version to `package.json`, runs
`npm run sync`, verifies the package, and publishes it with npm provenance.

Configure the repository `NPM_TOKEN` secret with an npm publish token before
pushing the tag. The workflow never publishes from pull requests or branch
pushes.

## Public Catalog Metadata

Minimal manifests are suitable for development and private distribution. Before public submission, add only verified metadata:

- Homepage and repository URLs.
- Publisher contact details.
- Privacy policy and terms URLs when required.
- Codex presentation icons, screenshots, capabilities, and starter prompts.
- Connector or application IDs created by the target platform.

Do not fabricate these values to satisfy a checker.
