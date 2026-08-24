import assert from "node:assert/strict"
import { readFile, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"
import { mkdtemp } from "node:fs/promises"
import { spawnSync } from "node:child_process"
import test from "node:test"

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const cli = join(root, "bin/harness-alchemist.mjs")

function run(args, options = {}) {
  return spawnSync(options.runtime ?? process.execPath, [cli, ...args], {
    cwd: options.cwd ?? root,
    encoding: "utf8",
  })
}

test("reports its version and canonical templates", () => {
  const version = run(["version"])
  assert.equal(version.status, 0, version.stderr)
  assert.equal(version.stdout.trim(), "0.1.0")

  const templates = run(["templates"])
  assert.equal(templates.status, 0, templates.stderr)
  assert.equal(templates.stdout.trim(), "v0.1.0 (canonical)")
})

test("creates and validates a recursively agent-developable project", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-"))
  const output = join(parent, "recursive-plugin")
  const creation = run([
    "create",
    output,
    "--description",
    "Exercise the canonical recursive scaffold.",
    "--author",
    "Example Team",
    "--repository",
    "example/recursive-plugin",
  ])

  assert.equal(creation.status, 0, creation.stderr)
  assert.match(creation.stdout, /Created universal plugin scaffold/)
  assert.equal(
    JSON.parse(await readFile(join(output, "package.json"), "utf8")).version,
    "0.1.0",
  )
  assert.match(
    await readFile(join(output, ".agents/skills/develop-recursive-plugin/SKILL.md"), "utf8"),
    /## Repository Layout/,
  )
  assert.match(
    await readFile(join(output, "skills/recursive-plugin/SKILL.md"), "utf8"),
    /name: recursive-plugin/,
  )
  for (const relative of [
    "skills/recursive-plugin/scripts/main.mjs",
    "skills/recursive-plugin/scripts/main.py",
    "skills/recursive-plugin/references/tool-contract.md",
  ]) {
    await readFile(join(output, relative), "utf8")
  }
  assert.match(
    await readFile(join(output, "src/opencode.ts"), "utf8"),
    /"recursive-plugin_run": tool\(/,
  )
  assert.match(
    await readFile(join(output, ".github/workflows/npm-publish.yml"), "utf8"),
    /npm publish --provenance --access public/,
  )

  const validation = run(["validate", output])
  assert.equal(validation.status, 0, validation.stderr)
  assert.match(validation.stdout, /Validated universal plugin scaffold/)

  await writeFile(
    join(output, "cordis.patch.yml"),
    "- insert\n    - id: recursive-plugin\n      name: 'recursive-plugin/deepseek'\n",
  )
  const malformedPatch = run(["validate", output])
  assert.notEqual(malformedPatch.status, 0)
  assert.match(malformedPatch.stderr, /valid insert entry/)

  for (const invalidPatch of [
    "- insert:\n    - id: recursive-plugin\n      options:\n        name: 'recursive-plugin/deepseek'\n",
    "- insert:\n    - id: recursive-plugin\n      name: 'recursive-plugin/deepseek'\n      name: 'recursive-plugin/deepseek'\n",
    "- insert:\n    options:\n      - id: recursive-plugin\n        name: 'recursive-plugin/deepseek'\n",
  ]) {
    await writeFile(join(output, "cordis.patch.yml"), invalidPatch)
    const invalidStructure = run(["validate", output])
    assert.notEqual(invalidStructure.status, 0)
    assert.match(invalidStructure.stderr, /valid insert entry/)
  }
})

test("does not replace non-empty destinations", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-safe-"))
  const output = join(parent, "occupied-plugin")
  await writeFile(output, "keep me")

  const result = run([
    "create",
    output,
    "--description",
    "Must not replace existing content.",
    "--author",
    "Example Team",
    "--repository",
    "example/occupied-plugin",
  ])

  assert.notEqual(result.status, 0)
  assert.equal(await readFile(output, "utf8"), "keep me")
})

test("rejects unknown canonical template versions", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-version-"))
  const result = run([
    "create",
    join(parent, "future-plugin"),
    "--description",
    "Reject unknown snapshots.",
    "--author",
    "Example Team",
    "--repository",
    "example/future-plugin",
    "--template",
    "0.2.0",
  ])

  assert.equal(result.status, 2)
  assert.match(result.stderr, /Unknown template version/)
})

test("creates an Apache-2.0 project from the canonical license source", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-apache-"))
  const output = join(parent, "apache-plugin")
  const creation = run([
    "create",
    output,
    "--description",
    "Exercise Apache license generation.",
    "--author",
    "Example Team",
    "--repository",
    "example/apache-plugin",
    "--license",
    "Apache-2.0",
  ])

  assert.equal(creation.status, 0, creation.stderr)
  assert.match(await readFile(join(output, "LICENSE"), "utf8"), /Apache License/)
  assert.equal(
    JSON.parse(await readFile(join(output, "package.json"), "utf8")).license,
    "Apache-2.0",
  )
})

const bunAvailable = spawnSync("bun", ["--version"], { encoding: "utf8" }).status === 0

test("creates and validates a project under Bun", { skip: !bunAvailable }, async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-bun-"))
  const output = join(parent, "bun-plugin")
  const creation = run([
    "create",
    output,
    "--description",
    "Exercise the Bun CLI runtime.",
    "--author",
    "Example Team",
    "--repository",
    "example/bun-plugin",
  ], { runtime: "bun" })

  assert.equal(creation.status, 0, creation.stderr)
  const validation = run(["validate", output], { runtime: "bun" })
  assert.equal(validation.status, 0, validation.stderr)
})
