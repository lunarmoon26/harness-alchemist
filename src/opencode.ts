import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

import type { Plugin } from "@opencode-ai/plugin"
import { createOpenCodePlugin } from "@lunarmoon26/agent-skill-runtime/opencode"

const pluginRoot = join(dirname(fileURLToPath(import.meta.url)), "..")

const plugin: Plugin = createOpenCodePlugin({
  pluginRoot,
  manifestPaths: ["skills/harness-alchemist/skill-runtime.json"],
})

export default plugin
