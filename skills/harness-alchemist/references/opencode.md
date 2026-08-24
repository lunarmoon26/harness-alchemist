# OpenCode Plugins

## Runtime Contract

An OpenCode plugin is an ESM JavaScript or TypeScript module exporting one or more plugin functions. Each function receives the OpenCode context and returns a hooks object.

For npm publication, expose compiled JavaScript from the package root:

```json
{
  "type": "module",
  "exports": {
    ".": {
      "types": "./dist/opencode.d.ts",
      "import": "./dist/opencode.js"
    }
  }
}
```

Use `import type { Plugin } from "@opencode-ai/plugin"` for type-only entrypoints. Import `tool` at runtime only when registering custom tools.

## Installation

OpenCode accepts npm package names in `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "plugin": ["@scope/my-plugin"]
}
```

OpenCode installs npm plugins with Bun at startup. For local development, register an absolute `file://` URL to compiled JavaScript or a direct TypeScript file with all of its dependencies available.

Skills are discovered from `~/.agents/skills/` (and project `.agents/skills/`), not from the OpenCode config directory, and symlinked skill directories are skipped. Install them from the Git repository or a checkout:

```bash
npx skills add owner/repo --agent opencode
# or, from a local clone:
cp -R skills/<name> ~/.agents/skills/
```

Verify discovery with `opencode debug skill`.

## Extension Rules

- Return `{}` when no hooks are registered.
- Custom tools live under the returned `tool` map.
- Mutating hooks alter the provided output object in place.
- Use `client.app.log()` for structured logs.
- Release resources in `dispose` when a plugin owns long-lived clients.
- Restart OpenCode after plugin or skill installation changes.

Official reference: https://opencode.ai/docs/plugins/
