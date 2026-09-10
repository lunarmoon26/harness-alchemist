import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"
import test from "node:test"

import opencodePlugin from "../dist/opencode.js"
import * as deepseekPlugin from "../dist/deepseek.js"

const pythonAvailable = spawnSync("python3", ["--version"], { encoding: "utf8" }).status === 0
const root = dirname(dirname(fileURLToPath(import.meta.url)))

function executionContext() {
  return {
    directory: root,
    worktree: root,
    abort: new AbortController().signal,
    ask: async () => {},
    metadata: () => {},
    sessionID: "session",
    messageID: "message",
    agent: "test",
  }
}

test("OpenCode tool delegates to the shared skill script", async () => {
  const hooks = await opencodePlugin({})
  const entry = hooks.tool?.[{{TOOL_NAME_JSON}}]
  assert.equal(typeof entry?.execute, "function")
  const result = await entry.execute({ request: "hello" }, executionContext())
  assert.deepEqual(JSON.parse(result), {
    ok: true,
    plugin: "{{NAME}}",
    echo: { request: "hello" },
  })
})

test("DeepSeek entrypoint exposes a Cordis namespace plugin", () => {
  assert.equal(deepseekPlugin.name, {{NAME_JSON}})
  assert.deepEqual(deepseekPlugin.inject, ["tools"])
  assert.equal(typeof deepseekPlugin.apply, "function")
  assert.equal("default" in deepseekPlugin, false)
})

test("DeepSeek adapter registers the manifest tool", async () => {
  const definitions = []
  await deepseekPlugin.apply({
    tools: {
      register(definition) {
        definitions.push(definition)
        return () => {}
      },
    },
  })
  const definition = definitions.find((candidate) => candidate.name === {{TOOL_NAME_JSON}})
  assert.ok(definition)
  const result = await definition.execute(
    { request: "hello" },
    { signal: new AbortController().signal },
  )
  assert.deepEqual(result, {
    ok: true,
    plugin: "{{NAME}}",
    echo: { request: "hello" },
  })
})

test(
  "Python twin honors the same tool contract",
  { skip: !pythonAvailable },
  () => {
    const result = spawnSync("python3", [join(root, "skills/{{NAME}}/scripts/main.py")], {
      input: '{"request":"hello"}\n',
      encoding: "utf8",
    })
    assert.equal(result.status, 0, result.stderr)
    assert.deepEqual(JSON.parse(result.stdout), {
      ok: true,
      plugin: "{{NAME}}",
      echo: { request: "hello" },
    })
  },
)
