# Claude Code Plugins

## Required Shape

For a manifest-backed plugin:

```text
.claude-plugin/plugin.json
skills/<name>/SKILL.md
```

For repository installation, add `.claude-plugin/marketplace.json`. A one-plugin repository may use `"source": "./"`.

Only `plugin.json` belongs inside `.claude-plugin/`. Components remain at the plugin root.

## Supported Components

- `skills/<name>/SKILL.md`; prefer this over legacy `commands/*.md`.
- `agents/*.md`.
- `hooks/hooks.json`.
- `.mcp.json`.
- `.lsp.json`.
- `monitors/monitors.json`.
- `bin/` for executables added to the Bash tool path.
- Root `settings.json` for supported plugin defaults.

Use `${CLAUDE_PLUGIN_ROOT}` for files within the copied plugin. Do not reference `../` paths outside the repository payload.

## Development

```bash
claude --plugin-dir /absolute/path/to/plugin
claude plugin validate /absolute/path/to/plugin --strict
```

Use `/reload-plugins` after component changes when supported.

## Distribution

```bash
claude plugin marketplace add owner/repo
claude plugin install plugin-name@marketplace-name
```

Marketplace installation copies plugins into a cache. Versioned manifests require a version bump for updates.

Official references:

- https://code.claude.com/docs/en/plugins
- https://code.claude.com/docs/en/plugins-reference
- https://code.claude.com/docs/en/plugin-marketplaces
