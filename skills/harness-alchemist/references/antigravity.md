# Google Antigravity Plugins

## Plugin Shape

```text
plugin.json
skills/<name>/SKILL.md
rules/<name>.md
hooks.json
mcp_config.json
```

Only `plugin.json` is required. The CLI manifest schema accepts a machine-readable name and optional description. Keep the root manifest separate from Claude and Codex manifests.

Current Antigravity Agent Skills use the open Agent Skills directory format with nested `<name>/SKILL.md`. This is also the documented plugin skill shape.

## Installation

Antigravity 2 scans workspace plugins under `.agents/plugins/` or `_agents/plugins/`, and global plugins under `~/.gemini/config/plugins/`.

Antigravity CLI stages installed plugins under `~/.gemini/antigravity-cli/plugins/<name>/` and supports:

```bash
agy plugin install /path/to/plugin
agy plugin list
agy plugin disable plugin-name
agy plugin enable plugin-name
agy plugin uninstall plugin-name
```

The CLI documentation also shows a legacy flat Markdown workspace-skill example. Prefer the nested Agent Skills shape because the current Antigravity Skills and plugin documentation both specify it.

Official references:

- https://antigravity.google/docs/plugins/
- https://antigravity.google/docs/skills/
- https://antigravity.google/docs/cli/plugins/
