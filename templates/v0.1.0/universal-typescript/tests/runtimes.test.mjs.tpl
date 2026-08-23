import assert from "node:assert/strict"
import test from "node:test"

import opencodePlugin from "../dist/opencode.js"
import * as deepseekPlugin from "../dist/deepseek.js"

test("OpenCode entrypoint returns an inert hooks object", async () => {
  assert.deepEqual(await opencodePlugin({}), {})
})

test("DeepSeek entrypoint exposes a Cordis namespace plugin", () => {
  assert.equal(deepseekPlugin.name, {{NAME_JSON}})
  assert.equal(typeof deepseekPlugin.apply, "function")
  assert.equal("default" in deepseekPlugin, false)
})
