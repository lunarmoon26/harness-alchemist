import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

import { Plugin } from "@opencode/plugin"
import { createSkillRuntime } from "@lunarmoon26/agent-skill-runtime"
import type { JsonValue } from "@lunarmoon26/agent-skill-runtime"

const pluginRoot = join(dirname(fileURLToPath(import.meta.url)), "..")

function renderResult(result: JsonValue): string {
  return typeof result === "string" ? result : JSON.stringify(result)
}

const plugin = Plugin.define({
  id: {{NAME_JSON}},
  async setup(ctx) {
    const runtime = await createSkillRuntime({
      pluginRoot,
      manifestPaths: ["skills/{{NAME}}/skill-runtime.json"],
    })

    await ctx.tool.transform((editor) => {
      for (const loaded of runtime.listTools()) {
        if (loaded.tool.inputSchema.type !== "object") continue

        editor.add({
          name: loaded.tool.name,
          description: loaded.tool.description,
          input: loaded.tool.inputSchema as never,
          output: loaded.tool.outputSchema as never,
          options: loaded.tool.capabilities?.length
            ? { permission: `skill-runtime:${loaded.tool.name}` }
            : undefined,
          execute: async (input, context) => {
            const result = await runtime.execute(loaded.tool.name, input as JsonValue, {
              cwd: ctx.location.directory,
              signal: context.signal,
              approve: async () => true,
            })
            return { content: renderResult(result) }
          },
        })
      }
    })
  },
})

export default plugin
