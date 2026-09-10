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
import { registerDeepSeekTools } from "@lunarmoon26/agent-skill-runtime/deepseek"

export const name = "my-plugin"
export const inject = ["tools"]

export async function apply(ctx: Context): Promise<void> {
  await registerDeepSeekTools(ctx, { pluginRoot: "/resolved/plugin/root" })
}
```

Loader resolves modules as `exports.default ?? exports`. A default export would hide namespace metadata such as `inject` and `Config` from a named-export function plugin.

Declare required services through `inject`. Use `ctx.get()` for optional services. The Harness tool registry owns each portable tool registration as a Cordis effect, so unload removes it.

## Installation

```bash
dsh plugin --profile demo add @scope/my-plugin
dsh --profile demo --dump-config
```

`dsh plugin add` forwards to pnpm in the profile directory, so an unpublished checkout can be installed by absolute path and stays linked to the working tree:

```bash
dsh plugin --profile demo add /absolute/path/to/my-plugin
```

Bundle membership changes require a profile restart. Profile patch edits may hot-reload.

Shared Agent Skills are discovered from project `.agents/skills` and other configured filesystem roots. Package-relative skill assets are not automatically activated by a bundle; install repository skills separately when needed.

DeepSeek Harness remains a developer preview. Reinspect its source contract before adding advanced UI, agent-preset, or dynamic Host/Client templates.
