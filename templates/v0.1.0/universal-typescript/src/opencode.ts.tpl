import { spawn } from "node:child_process"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

import { tool } from "@opencode-ai/plugin"
import type { Plugin } from "@opencode-ai/plugin"

const scriptsDirectory = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "skills/{{NAME}}/scripts",
)

export function runSkillScript(script: string, payload: string): Promise<string> {
  return new Promise((resolveResult, rejectResult) => {
    const runtime = script.endsWith(".py") ? "python3" : process.execPath
    const child = spawn(runtime, [join(scriptsDirectory, script)], {
      stdio: ["pipe", "pipe", "pipe"],
    })
    let stdout = ""
    let stderr = ""
    child.stdout.on("data", (chunk) => {
      stdout += chunk
    })
    child.stderr.on("data", (chunk) => {
      stderr += chunk
    })
    child.on("error", (error) => {
      rejectResult(new Error(`Could not run ${script} via ${runtime}: ${error.message}`))
    })
    child.on("close", (code) => {
      if (code !== 0) {
        rejectResult(new Error(stderr.trim() || `${script} exited with code ${code}`))
      } else {
        resolveResult(stdout)
      }
    })
    child.stdin.end(payload)
  })
}

const plugin = (async () => {
  return {
    tool: {
      {{NAME}}_run: tool({
        description:
          "Run the {{DISPLAY_NAME}} shared-skill workflow. Delegates to skills/{{NAME}}/scripts.",
        args: {
          input: tool.schema
            .string()
            .describe("JSON object passed on stdin to the skill entrypoint"),
          runtime: tool.schema
            .enum(["node", "python"])
            .optional()
            .describe("Entrypoint runtime; defaults to node"),
        },
        execute: async (args) => {
          const script = args.runtime === "python" ? "main.py" : "main.mjs"
          const output = await runSkillScript(script, args.input)
          return JSON.stringify(JSON.parse(output))
        },
      }),
    },
  }
}) satisfies Plugin

export default plugin
