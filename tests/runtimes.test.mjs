import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import test from "node:test"

import opencodePlugin from "../dist/opencode.js"
import * as deepseekPlugin from "../dist/deepseek.js"

const pythonAvailable = spawnSync("python3", ["--version"], { encoding: "utf8" }).status === 0

test("OpenCode tool delegates to the shared skill script", async () => {
  const hooks = await opencodePlugin({})
  const entry = hooks.tool?.["harness_alchemist_run"]
  assert.equal(typeof entry?.execute, "function")
  const result = await entry.execute({ input: '{"hello":"world"}' })
  assert.deepEqual(JSON.parse(result), {
    ok: true,
    plugin: "harness-alchemist",
    echo: { hello: "world" },
  })
})

test("DeepSeek entrypoint exposes a Cordis namespace plugin", () => {
  assert.equal(deepseekPlugin.name, "harness-alchemist")
  assert.equal(typeof deepseekPlugin.apply, "function")
  assert.equal("default" in deepseekPlugin, false)
})

test("DeepSeek adapter delegates to the shared skill script", () => {
  const result = deepseekPlugin.runSkillScript("main.mjs", '{"hello":"world"}')
  assert.deepEqual(result, {
    ok: true,
    plugin: "harness-alchemist",
    echo: { hello: "world" },
  })
})

test(
  "Python twin honors the same tool contract",
  { skip: !pythonAvailable },
  () => {
    const result = deepseekPlugin.runSkillScript("main.py", '{"hello":"world"}')
    assert.deepEqual(result, {
      ok: true,
      plugin: "harness-alchemist",
      echo: { hello: "world" },
    })
  },
)
