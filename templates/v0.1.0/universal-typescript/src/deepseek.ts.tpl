import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

import type { Context } from "@deepseek-ai/cordis"
import { registerDeepSeekTools } from "@lunarmoon26/agent-skill-runtime/deepseek"

export const name = {{NAME_JSON}}
export const inject = ["tools"]

const pluginRoot = join(dirname(fileURLToPath(import.meta.url)), "..")

export async function apply(context: Context): Promise<void> {
  await registerDeepSeekTools(context, {
    pluginRoot,
    manifestPaths: ["skills/{{NAME}}/skill-runtime.json"],
  })
}
