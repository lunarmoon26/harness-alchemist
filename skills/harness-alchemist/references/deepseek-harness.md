# DeepSeek Harness and Cordis

## Supported Distribution

DeepSeek Harness uses Cordis plugins composed through profile bundles. The removed `.dsh-plugin` repository format is not supported.

A package contributes a bundle through:

```json
{
  "dsh": {
    "bundle": {
      "patch": "./cordis.patch.yml"
    }
  }
}
```

The patch inserts the package's Cordis entrypoint:

```yaml
- insert:
    - id: my-plugin
      name: '@scope/my-plugin/deepseek'
```

## Cordis Entrypoint

Function plugins use named exports and no default export:

```ts
import type { Context } from "@deepseek-ai/cordis"

export const name = "my-plugin"

export function apply(ctx: Context): void {
  // Register lifecycle-owned behavior.
}
```

Loader resolves modules as `exports.default ?? exports`. A default export would hide namespace metadata such as `inject` and `Config` from a named-export function plugin.

Declare required services through `inject`. Use `ctx.get()` for optional services. Register cleanup through Cordis effects or returned disposers.

## Installation

```bash
dsh plugin --profile demo add @scope/my-plugin
dsh --profile demo --dump-config
```

Bundle membership changes require a profile restart. Profile patch edits may hot-reload.

Shared Agent Skills are discovered from project `.agents/skills` and other configured filesystem roots. Package-relative skill assets are not automatically activated by a bundle; install repository skills separately when needed.

DeepSeek Harness remains a developer preview. Reinspect its source contract before adding advanced UI, agent-preset, or dynamic Host/Client templates.
