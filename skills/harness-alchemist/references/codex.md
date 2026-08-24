# Codex and ChatGPT Plugins

## Required Shape

```text
.codex-plugin/plugin.json
skills/<name>/SKILL.md
```

For a repository marketplace, add `.agents/plugins/marketplace.json`. Marketplace entries need `name`, `source`, `policy`, and `category`.

Only `plugin.json` belongs inside `.codex-plugin/`. Keep skills, hooks, MCP files, app mappings, and assets at the plugin root.

## Components

- `skills/<name>/SKILL.md`.
- `hooks/hooks.json`.
- `.mcp.json` for bundled MCP servers.
- `.app.json` for registered ChatGPT MCP connection mappings.
- `assets/` for icons, logos, and screenshots.

Manifest component paths start with `./` and must stay inside the plugin root. Codex also checks the default `hooks/hooks.json` path when the manifest omits `hooks`.

The rich `interface` object is recommended for public presentation but is not required for a minimal local plugin. Never generate fake privacy, terms, connector, or application URLs.

## Distribution

```bash
codex plugin marketplace add owner/repo
codex plugin add <plugin-name>@<marketplace-name>
codex plugin list
```

The `@marketplace` suffix is required for `codex plugin add`; a bare plugin
name is rejected. Headless install works without opening `/plugins`, then
start a new session. Codex npm marketplace sources download packages without running lifecycle scripts, so published packages must already contain built runtime files.

Official references:

- https://learn.chatgpt.com/docs/plugins
- https://learn.chatgpt.com/plugins/build/plugins.md
