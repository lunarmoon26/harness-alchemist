# Compatibility Model

## Canonical Project Shape

Use a universal repository root with shared skills and separate manifests/runtime entrypoints:

```text
project/
├── .agents/plugins/marketplace.json
├── .agents/skills/develop-<name>/
├── .claude-plugin/marketplace.json
├── .claude-plugin/plugin.json
├── .codex-plugin/plugin.json
├── skills/<name>/SKILL.md
├── skills/<name>/scripts/main.mjs
├── skills/<name>/scripts/main.py
├── skills/<name>/references/tool-contract.md
├── src/opencode.ts
├── src/deepseek.ts
├── cordis.patch.yml
├── plugin.json
└── package.json
```

Product skills own their runtime: `skills/<name>/scripts/` holds behavioral
`.mjs`/`.py` twins (one JSON object in on stdin, one JSON result out on
stdout), and `src/opencode.ts` and `src/deepseek.ts` stay thin adapters that
spawn those scripts. See the generated project's tool contract reference for
details.

## Existing repositories

Existing monorepos may declare `pluginRoot` and an optional `./server` OpenCode
export in root `alchemy.json`. The repository root keeps marketplaces
and maintenance guidance; the plugin package keeps product skills, host
manifests, adapter sources, Cordis patch, and npm metadata. Generated repositories
remain single-package and need no configuration file.

The optional `runtime` field adapts the contract to the repository's language:
`"npm"` (default) requires the full generated package; `"skills"` accepts
skills and harness manifests alone — no npm package, adapters, Cordis patch,
or `.mjs`/`.py` twins — so polyglot repositories can publish workflows to
Claude Code, Codex, Antigravity, and DeepSeek without a JavaScript runtime.

## Capability Matrix

| Artifact | Claude | Codex | OpenCode | Antigravity | DeepSeek |
| --- | --- | --- | --- | --- | --- |
| `skills/<name>/SKILL.md` | Plugin-native | Plugin-native | Install separately | Plugin-native | Install separately |
| `.agents/skills/` | Discoverable by installers | Project-native | Project-native | Project-native | Project-native |
| npm JavaScript entrypoint | Optional dependency support | Optional npm source | Native plugin | No | Bundle module |
| Hooks | Claude schema | Codex schema | Plugin callbacks | Antigravity schema | Cordis events |
| MCP | `.mcp.json` | `.mcp.json` / `.app.json` | Config or plugin tools | `mcp_config.json` | Cordis MCP client row |

Only skills follow a broadly shared specification. Hook, agent, permission, MCP, and runtime APIs are not universal merely because their names are similar.

## Naming

- Use lowercase letters, digits, and single hyphens.
- Keep the plugin name at 56 characters or fewer so `develop-<name>` remains a valid 64-character Agent Skill name.
- Make the npm package basename equal the plugin name. `@scope/my-plugin` is valid for plugin `my-plugin`.
- Keep every skill frontmatter `name` equal to its parent directory.

## Source Baseline

The design was derived from these inspected snapshots on 2026-08-23:

- `anthropics/claude-plugins-official` commit `340e33aef211d95769d252324854497af871dafe`.
- `anthropics/knowledge-work-plugins` commit `5267cf7bff3031921d4474b8e8f86ad02d2b8f6d`.
- `openai/plugins` commit `11c74d6ba24d3a6d48f54a194cd00ef3beea18f9`.
- Local OpenCode examples at commits `22124b8e52e6e94b1cccf75b4c501cebe5ddf2df`, `cf5a390bcdac4f775e6e7e9b559e411b48ff5a93`, and `dbdbc7f713b5032aeb75e3755d31e851c91ab63e`.
- OpenPackage commit `399187d6cc4f4c86391fa07ff23b5387f898d2bf`.
- DeepSeek Harness tag `dsh-v0.1.1-rc.2`, commit `b150a551b8d465e31e418e1b2eaf5e79bbb7d28e`.
- Vercel Skills commit `435076e78988e1e6ec40d00b0b1d76bdbbc5419a`, package version `1.5.23`.

Recheck official documentation before adding a newer or experimental component.
