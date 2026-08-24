import assert from "node:assert/strict"
import { mkdir, readFile, rename, rm, symlink, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { delimiter, dirname, join } from "node:path"
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
    env: options.env ?? process.env,
  })
}

test("reports its version and canonical templates", async () => {
  const version = run(["version"])
  assert.equal(version.status, 0, version.stderr)
  const packageJson = JSON.parse(await readFile(join(root, "package.json"), "utf8"))
  assert.equal(version.stdout.trim(), packageJson.version)

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
  const layout = JSON.parse(await readFile(join(output, "harness-alchemist.json"), "utf8"))
  assert.equal(layout.runtime, "npm")
  assert.equal(layout.template, "v0.1.0")
  assert.equal(layout.generator, "harness-alchemist")
  assert.match(layout.generatorVersion, /^\d+\.\d+\.\d+/)
  assert.equal(new Date(layout.createdAt).getTime(), new Date(layout.createdAt).getTime())

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

test("validates an adapted SDK package inside a monorepo", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-monorepo-"))
  const output = join(parent, "sdk-monorepo")
  const creation = run([
    "create",
    output,
    "--description",
    "Exercise an adapted SDK package.",
    "--author",
    "Example Team",
    "--repository",
    "example/sdk-monorepo",
  ])
  assert.equal(creation.status, 0, creation.stderr)

  const pluginRelative = "packages/sdk-monorepo"
  const pluginRoot = join(output, pluginRelative)
  await mkdir(join(pluginRoot, ".claude-plugin"), { recursive: true })
  for (const relative of [
    ".claude-plugin/plugin.json",
    ".codex-plugin",
    "cordis.patch.yml",
    "package.json",
    "plugin.json",
    "skills",
    "src",
    "tsconfig.json",
  ]) {
    await rename(join(output, relative), join(pluginRoot, relative))
  }

  const packageJsonPath = join(pluginRoot, "package.json")
  const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8"))
  packageJson.exports["./server"] = packageJson.exports["."]
  packageJson.exports["."] = {
    import: "./dist/sdk.js",
    types: "./dist/sdk.d.ts",
  }
  packageJson.files.push("harness-alchemist.json")
  delete packageJson.engines
  await writeFile(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`)

  const claudeMarketplacePath = join(output, ".claude-plugin/marketplace.json")
  const claudeMarketplace = JSON.parse(await readFile(claudeMarketplacePath, "utf8"))
  claudeMarketplace.plugins[0].source = `./${pluginRelative}`
  await writeFile(claudeMarketplacePath, `${JSON.stringify(claudeMarketplace, null, 2)}\n`)

  const codexMarketplacePath = join(output, ".agents/plugins/marketplace.json")
  const codexMarketplace = JSON.parse(await readFile(codexMarketplacePath, "utf8"))
  codexMarketplace.plugins[0].source.path = `./${pluginRelative}`
  await writeFile(codexMarketplacePath, `${JSON.stringify(codexMarketplace, null, 2)}\n`)

  await writeFile(join(output, "package.json"), `${JSON.stringify({
    name: "sdk-monorepo-workspace",
    private: true,
    workspaces: ["packages/*"],
  }, null, 2)}\n`)
  await writeFile(join(output, "harness-alchemist.json"), `${JSON.stringify({
    pluginRoot: pluginRelative,
    opencodeExport: "./server",
  }, null, 2)}\n`)

  const validation = run(["validate", output])
  assert.equal(validation.status, 0, validation.stderr)
  assert.match(validation.stdout, /Validated universal plugin scaffold/)

  const nestedValidation = run(["validate"], { cwd: pluginRoot })
  assert.equal(nestedValidation.status, 0, nestedValidation.stderr)
  assert.match(nestedValidation.stdout, /Validated universal plugin scaffold at .*[/\\]sdk-monorepo/)

  await writeFile(join(output, "harness-alchemist.json"), "null\n")
  const nullLayout = run(["validate", output])
  assert.notEqual(nullLayout.status, 0)
  assert.match(nullLayout.stderr, /must contain a JSON object/)

  const externalPlugin = join(parent, "external-plugin")
  const escapedPlugin = join(output, "packages/escaped")
  await mkdir(externalPlugin)
  await symlink(externalPlugin, escapedPlugin, "dir")
  await writeFile(join(output, "harness-alchemist.json"), `${JSON.stringify({
    pluginRoot: "packages/escaped",
    opencodeExport: "./server",
  }, null, 2)}\n`)
  const symlinkLayout = run(["validate", output])
  assert.notEqual(symlinkLayout.status, 0)
  assert.match(symlinkLayout.stderr, /symlink outside the project root/)

  await writeFile(join(output, "harness-alchemist.json"), `${JSON.stringify({
    pluginRoot: pluginRelative,
    opencodeExport: "./server",
  }, null, 2)}\n`)
  const fakeBin = join(parent, "bin")
  const externalArgs = join(parent, "claude-args.txt")
  await mkdir(fakeBin)
  await writeFile(
    join(fakeBin, "claude"),
    "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$HARNESS_CLAUDE_ARGS\"\n",
    { mode: 0o755 },
  )
  const externalValidation = run(["validate", output, "--external"], {
    env: {
      ...process.env,
      PATH: `${fakeBin}${delimiter}${process.env.PATH ?? ""}`,
      HARNESS_CLAUDE_ARGS: externalArgs,
    },
  })
  assert.equal(externalValidation.status, 0, externalValidation.stderr)
  assert.deepEqual(
    (await readFile(externalArgs, "utf8")).trim().split("\n"),
    ["plugin", "validate", pluginRoot, "--strict"],
  )
})

test("validates a skills-only monorepo package without npm metadata", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-skills-"))
  const output = join(parent, "polyglot-monorepo")
  const creation = run([
    "create",
    output,
    "--description",
    "Exercise a skills-only adaptation.",
    "--author",
    "Example Team",
    "--repository",
    "example/polyglot-monorepo",
  ])
  assert.equal(creation.status, 0, creation.stderr)

  const pluginRelative = "services/polyglot-plugin"
  const pluginRoot = join(output, pluginRelative)
  await mkdir(join(pluginRoot, ".claude-plugin"), { recursive: true })
  for (const relative of [
    ".claude-plugin/plugin.json",
    ".codex-plugin",
    "plugin.json",
    "skills",
  ]) {
    await rename(join(output, relative), join(pluginRoot, relative))
  }
  for (const relative of [
    "package.json",
    "cordis.patch.yml",
    "src",
    "tests",
    "tsconfig.json",
  ]) {
    await rm(join(output, relative), { recursive: true, force: true })
  }

  for (const manifest of [
    ".claude-plugin/marketplace.json",
    ".agents/plugins/marketplace.json",
  ]) {
    const path = join(output, manifest)
    const parsed = JSON.parse(await readFile(path, "utf8"))
    if (parsed.plugins[0].source?.source === "local") {
      parsed.plugins[0].source.path = `./${pluginRelative}`
    } else {
      parsed.plugins[0].source = `./${pluginRelative}`
    }
    await writeFile(path, `${JSON.stringify(parsed, null, 2)}\n`)
  }

  const writeLayout = (layout) =>
    writeFile(join(output, "harness-alchemist.json"), `${JSON.stringify(layout, null, 2)}\n`)

  await writeLayout({ pluginRoot: pluginRelative, runtime: "skills" })
  const validation = run(["validate", output])
  assert.equal(validation.status, 0, validation.stderr)
  assert.match(validation.stdout, /Validated universal plugin scaffold/)

  await rm(join(pluginRoot, "skills/polyglot-monorepo/scripts/main.mjs"))
  const skillPath = join(pluginRoot, "skills/polyglot-monorepo/SKILL.md")
  await writeFile(
    skillPath,
    (await readFile(skillPath, "utf8")).replaceAll("scripts/main.mjs", "scripts/main.py"),
  )
  const pythonOnly = run(["validate", output])
  assert.equal(pythonOnly.status, 0, pythonOnly.stderr)

  await writeLayout({ pluginRoot: pluginRelative, runtime: "skills", opencodeExport: "./server" })
  const exportConflict = run(["validate", output])
  assert.notEqual(exportConflict.status, 0)
  assert.match(exportConflict.stderr, /opencodeExport requires runtime 'npm'/)

  await writeLayout({ pluginRoot: pluginRelative, runtime: "native" })
  const unknownRuntime = run(["validate", output])
  assert.notEqual(unknownRuntime.status, 0)
  assert.match(unknownRuntime.stderr, /runtime must be one of/)
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
