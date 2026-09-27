# OpenCode Plugins

## Runtime Contract

An OpenCode V2 plugin is an ESM JavaScript or TypeScript module exporting a plugin definition. The definition has a stable `id` and a `setup(ctx)` function that registers tools, hooks, transforms, or subscriptions through the OpenCode context.

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

Use `import { Plugin } from "@opencode/plugin"` and export `Plugin.define({ id, setup })`. Register custom tools with `ctx.tool.transform(...)` and JSON Schema input definitions.

## Installation

OpenCode accepts npm package names in `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "plugins": ["@scope/my-plugin"]
}
```

OpenCode installs npm plugins at startup. For local development, register an absolute `file://` URL to compiled JavaScript or a direct TypeScript file with all of its dependencies available.

Skills are discovered from `~/.agents/skills/` (and project `.agents/skills/`), not from the OpenCode config directory, and symlinked skill directories are skipped. Install them from the Git repository or a checkout:

```bash
npx skills add owner/repo --agent opencode
# or, from a local clone:
cp -R skills/<name> ~/.agents/skills/
```

Verify discovery with `opencode debug skill`.

## Extension Rules

- Every plugin needs a stable `id`.
- Custom tools are registered through `ctx.tool.transform(...)`.
- Keep transform callbacks synchronous and side-effect-free; load runtime data before registering a transform.
- Use `console` output for simple startup diagnostics or OpenCode context APIs for domain-specific behavior.
- Return a cleanup function from `setup` when a plugin owns long-lived clients.
- Restart OpenCode after plugin or skill installation changes.

Official reference: https://opencode.ai/v2/docs/plugins/
