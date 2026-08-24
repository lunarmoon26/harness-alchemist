import { spawnSync } from "node:child_process"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

import type { Context } from "@deepseek-ai/cordis"

export const name = "harness-alchemist"

const scriptsDirectory = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "skills/harness-alchemist/scripts",
)

export function runSkillScript(script: string, payload: string): unknown {
  const runtime = script.endsWith(".py") ? "python3" : process.execPath
  const result = spawnSync(runtime, [join(scriptsDirectory, script)], {
    input: payload,
    encoding: "utf8",
  })
  if (result.error) {
    throw new Error(`Could not run ${script} via ${runtime}: ${result.error.message}`)
  }
  if (result.status !== 0) {
    const diagnostic = result.stderr.trim() || `${script} exited with code ${result.status}`
    throw new Error(diagnostic)
  }
  return JSON.parse(result.stdout)
}

export function apply(context: Context): void {
  context.provide(name, {
    run: (payload: string, runtime: "node" | "python" = "node") =>
      runSkillScript(runtime === "python" ? "main.py" : "main.mjs", payload),
  })
}
