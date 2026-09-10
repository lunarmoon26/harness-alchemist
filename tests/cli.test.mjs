import assert from "node:assert/strict"
import { mkdir, readFile, readdir, rename, rm, symlink, writeFile } from "node:fs/promises"
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

  const executable = process.platform === "win32" ? "harness-alchemist.exe" : "harness-alchemist"
  const native = spawnSync(join(root, "target", "release", executable), ["version"], {
    encoding: "utf8",
  })
  assert.equal(native.status, 0, native.stderr)
  assert.equal(native.stdout.trim(), packageJson.version)

  const help = run([])
  assert.equal(help.status, 0, help.stderr)
  assert.match(help.stdout, /Commands:\n  create <directory>/)
  assert.match(help.stdout, /verify\n {24}discovery/)
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
    "mcp.json",
    "plugin.json",
    "skills/recursive-plugin/scripts/main.mjs",
    "skills/recursive-plugin/scripts/main.py",
    "skills/recursive-plugin/references/tool-contract.md",
    "skills/recursive-plugin/skill-runtime.json",
  ]) {
    await readFile(join(output, relative), "utf8")
  }
  assert.equal(
    await readFile(join(output, ".agents/skills/develop-recursive-plugin/scripts/validate.mjs"), "utf8"),
    await readFile(join(root, "templates/v0.1.0/generated/validate.mjs"), "utf8"),
  )
  assert.match(
    await readFile(join(output, "src/opencode.ts"), "utf8"),
    /createOpenCodePlugin/,
  )
  const runtimeManifest = JSON.parse(
    await readFile(join(output, "skills/recursive-plugin/skill-runtime.json"), "utf8"),
  )
  assert.equal(runtimeManifest.tools[0].name, "recursive_plugin_run")
  assert.equal(runtimeManifest.tools[0].entrypoint.path, "scripts/main.mjs")
  const generatedPackage = JSON.parse(await readFile(join(output, "package.json"), "utf8"))
  assert.equal(generatedPackage.dependencies["@lunarmoon26/agent-skill-runtime"], "0.1.0")
  assert.equal(
    JSON.parse(await readFile(join(output, "plugin.json"), "utf8")).$schema,
    "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
  )
  const publishWorkflow = await readFile(
    join(output, ".github/workflows/npm-publish.yml"),
    "utf8",
  )
  assert.match(publishWorkflow, /id-token: write/)
  assert.match(publishWorkflow, /must match package version/)
  assert.match(publishWorkflow, /git merge-base --is-ancestor/)
  assert.match(publishWorkflow, /npm publish --access public/)
  assert.doesNotMatch(publishWorkflow, /NPM_TOKEN|npm version/)
  const nativePublishWorkflow = await readFile(join(root, ".github/workflows/npm-publish.yml"), "utf8")
  assert.match(nativePublishWorkflow, /build-native:/)
  assert.match(nativePublishWorkflow, /__stage-native-packages/)
  assert.match(nativePublishWorkflow, /Publish platform packages/)
  assert.match(nativePublishWorkflow, /id-token: write/)
  assert.doesNotMatch(nativePublishWorkflow, /NPM_TOKEN|npm version/)
  const layout = JSON.parse(await readFile(join(output, "alchemy.json"), "utf8"))
  assert.equal(layout.runtime, "npm")
  assert.equal(layout.template, "v0.1.0")
  assert.equal(layout.generator, "harness-alchemist")
  assert.match(layout.generatorVersion, /^\d+\.\d+\.\d+/)
  assert.equal(new Date(layout.createdAt).getTime(), new Date(layout.createdAt).getTime())

  const validation = run(["validate", output])
  assert.equal(validation.status, 0, validation.stderr)
  assert.match(validation.stdout, /Validated universal plugin scaffold/)

  const mcpPath = join(output, "mcp.json")
  const mcpManifest = JSON.parse(await readFile(mcpPath, "utf8"))
  mcpManifest.mcpServers["recursive-plugin"].args.push("--allow", "network")
  await writeFile(mcpPath, `${JSON.stringify(mcpManifest, null, 2)}\n`)
  const privilegedMcp = run(["validate", output])
  assert.notEqual(privilegedMcp.status, 0)
  assert.match(privilegedMcp.stderr, /mcp\.json must configure 'recursive-plugin'/)
  mcpManifest.mcpServers["recursive-plugin"].args.splice(-2)
  await writeFile(mcpPath, `${JSON.stringify(mcpManifest, null, 2)}\n`)

  const manifestPath = join(output, "skills/recursive-plugin/skill-runtime.json")
  await writeFile(join(output, "skills/outside.mjs"), "")
  runtimeManifest.tools[0].entrypoint.path = "../outside.mjs"
  await writeFile(manifestPath, `${JSON.stringify(runtimeManifest, null, 2)}\n`)
  const escapingRuntime = run(["validate", output])
  assert.notEqual(escapingRuntime.status, 0)
  assert.match(escapingRuntime.stderr, /entrypoint\.path.*skill root/)
  runtimeManifest.tools[0].entrypoint.path = "scripts/main.mjs"
  await writeFile(manifestPath, `${JSON.stringify(runtimeManifest, null, 2)}\n`)

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

test("dry-run lists the embedded scaffold once without writing", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-dry-"))
  const output = join(parent, "dry-plugin")
  const result = run([
    "create",
    output,
    "--description",
    "Inspect the native embedded template.",
    "--author",
    "Example Team",
    "--repository",
    "example/dry-plugin",
    "--dry-run",
  ])

  assert.equal(result.status, 0, result.stderr)
  const report = JSON.parse(result.stdout)
  assert.deepEqual(Object.keys(report), ["output", "template", "plugin", "package", "files"])
  assert.equal(report.output, output)
  assert.equal(report.files.length, new Set(report.files).size)
  assert.ok(report.files.includes(".agents/skills/develop-dry-plugin/scripts/validate.mjs"))
  assert.ok(!report.files.some((path) => path.endsWith(".DS_Store")))
  await assert.rejects(readFile(output), /ENOENT/)
})

test("native validator rejects invalid Python syntax without executing it", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-python-"))
  const output = join(parent, "python-plugin")
  const creation = run([
    "create",
    output,
    "--description",
    "Exercise native Python parsing.",
    "--author",
    "Example Team",
    "--repository",
    "example/python-plugin",
  ])
  assert.equal(creation.status, 0, creation.stderr)

  await writeFile(join(output, "skills/python-plugin/scripts/main.py"), "def broken(:\n")
  const validation = run(["validate", output])
  assert.equal(validation.status, 1)
  assert.match(validation.stderr, /Python syntax check failed/)

  const jsonValidation = run(["validate", output, "--json"])
  assert.equal(jsonValidation.status, 1)
  assert.deepEqual(Object.keys(JSON.parse(jsonValidation.stdout)), [
    "valid",
    "root",
    "errors",
    "warnings",
  ])
})

test("npm launcher forwards arguments to an explicit native binary", { skip: process.platform === "win32" }, async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-launcher-"))
  const binary = join(parent, "fake-native")
  await writeFile(binary, "#!/bin/sh\nprintf '%s\\n' \"$*\"\nprintf '%s\\n' \"$HARNESS_ALCHEMIST_PACKAGE_ROOT\"\n", {
    mode: 0o755,
  })
  const result = run(["first", "second"], {
    env: { ...process.env, HARNESS_ALCHEMIST_BINARY: binary },
  })
  assert.equal(result.status, 0, result.stderr)
  assert.deepEqual(result.stdout.trim().split("\n"), ["first second", root])
})

test("npm launcher mirrors native termination signals", { skip: process.platform === "win32" }, async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-signal-"))
  const binary = join(parent, "signal-native")
  await writeFile(binary, "#!/bin/sh\nkill -TERM $$\n", { mode: 0o755 })

  const result = run([], {
    env: { ...process.env, HARNESS_ALCHEMIST_BINARY: binary },
  })
  assert.equal(result.status, null)
  assert.equal(result.signal, "SIGTERM")
})

test("packed npm launcher resolves its optional native package", async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-package-"))
  const staging = join(parent, "staging")
  const archives = join(parent, "archives")
  const consumer = join(parent, "consumer")
  await mkdir(archives)
  await mkdir(consumer)

  const staged = run(["__stage-native-packages", "--output", staging])
  assert.equal(staged.status, 0, staged.stderr)
  const platformEntries = await readdir(join(staging, "platform"), { withFileTypes: true })
  const platformEntry = platformEntries.find((entry) => entry.isDirectory())
  assert.ok(platformEntry)
  const platformDirectory = join(staging, "platform", platformEntry.name)
  const platformManifest = JSON.parse(
    await readFile(join(platformDirectory, "package.json"), "utf8"),
  )
  const mainManifestPath = join(staging, "main", "package.json")
  const mainManifest = JSON.parse(await readFile(mainManifestPath, "utf8"))
  mainManifest.optionalDependencies = {
    [platformManifest.name]: platformManifest.version,
  }
  await writeFile(mainManifestPath, `${JSON.stringify(mainManifest, null, 2)}\n`)

  const npm = process.platform === "win32" ? "npm.cmd" : "npm"
  const pack = (directory) => {
    const result = spawnSync(
      npm,
      ["pack", directory, "--pack-destination", archives, "--ignore-scripts", "--json"],
      { cwd: root, encoding: "utf8" },
    )
    assert.equal(result.status, 0, result.stderr)
    return join(archives, JSON.parse(result.stdout)[0].filename)
  }
  const platformArchive = pack(platformDirectory)
  const mainArchive = pack(join(staging, "main"))
  const runtimeArchive = pack(join(root, "node_modules/@lunarmoon26/agent-skill-runtime"))
  await writeFile(
    join(consumer, "package.json"),
    `${JSON.stringify(
      {
        private: true,
        dependencies: {
          "@lunarmoon26/agent-skill-runtime": `file:${runtimeArchive}`,
          [platformManifest.name]: `file:${platformArchive}`,
          "harness-alchemist": `file:${mainArchive}`,
        },
      },
      null,
      2,
    )}\n`,
  )
  const installation = spawnSync(
    npm,
    [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--package-lock=false",
      "--omit=peer",
    ],
    { cwd: consumer, encoding: "utf8" },
  )
  assert.equal(installation.status, 0, installation.stderr)

  const shim = join(
    consumer,
    "node_modules/.bin",
    process.platform === "win32" ? "harness-alchemist.cmd" : "harness-alchemist",
  )
  const version = spawnSync(shim, ["version"], {
    encoding: "utf8",
    shell: process.platform === "win32",
  })
  assert.equal(version.status, 0, version.stderr)
  assert.equal(version.stdout.trim(), mainManifest.version)
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
    "mcp.json",
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
  packageJson.files.push("alchemy.json")
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
  await writeFile(join(output, "alchemy.json"), `${JSON.stringify({
    pluginRoot: pluginRelative,
    opencodeExport: "./server",
  }, null, 2)}\n`)

  const validation = run(["validate", output])
  assert.equal(validation.status, 0, validation.stderr)
  assert.match(validation.stdout, /Validated universal plugin scaffold/)

  const nestedValidation = run(["validate"], { cwd: pluginRoot })
  assert.equal(nestedValidation.status, 0, nestedValidation.stderr)
  assert.match(nestedValidation.stdout, /Validated universal plugin scaffold at .*[/\\]sdk-monorepo/)

  await writeFile(join(output, "alchemy.json"), `${JSON.stringify({
    pluginRoot: pluginRelative,
    opencodeExport: "./server",
    createdAt: "2024-01-01T12:30",
  }, null, 2)}\n`)
  const timestampWithoutSeconds = run(["validate", output])
  assert.equal(timestampWithoutSeconds.status, 0, timestampWithoutSeconds.stderr)

  for (const [field, value, message] of [
    ["pluginRoot", 42, /pluginRoot must be a non-empty relative path/],
    ["runtime", 42, /runtime must be one of/],
    ["opencodeExport", 42, /opencodeExport must be/],
  ]) {
    await writeFile(join(output, "alchemy.json"), `${JSON.stringify({
      pluginRoot: pluginRelative,
      opencodeExport: "./server",
      [field]: value,
    }, null, 2)}\n`)
    const invalidType = run(["validate", output])
    assert.equal(invalidType.status, 1)
    assert.match(invalidType.stderr, message)
  }

  await writeFile(join(output, "alchemy.json"), "null\n")
  const nullLayout = run(["validate", output])
  assert.notEqual(nullLayout.status, 0)
  assert.match(nullLayout.stderr, /must contain a JSON object/)

  const externalPlugin = join(parent, "external-plugin")
  const escapedPlugin = join(output, "packages/escaped")
  await mkdir(externalPlugin)
  await symlink(externalPlugin, escapedPlugin, "dir")
  await writeFile(join(output, "alchemy.json"), `${JSON.stringify({
    pluginRoot: "packages/escaped",
    opencodeExport: "./server",
  }, null, 2)}\n`)
  const symlinkLayout = run(["validate", output])
  assert.notEqual(symlinkLayout.status, 0)
  assert.match(symlinkLayout.stderr, /symlink outside the project root/)

  await writeFile(join(output, "alchemy.json"), `${JSON.stringify({
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
    "mcp.json",
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
    writeFile(join(output, "alchemy.json"), `${JSON.stringify(layout, null, 2)}\n`)

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
  const runtimeManifestPath = join(pluginRoot, "skills/polyglot-monorepo/skill-runtime.json")
  const runtimeManifest = JSON.parse(await readFile(runtimeManifestPath, "utf8"))
  runtimeManifest.tools[0].entrypoint = { engine: "python", path: "scripts/main.py" }
  await writeFile(runtimeManifestPath, `${JSON.stringify(runtimeManifest, null, 2)}\n`)
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

  await writeLayout({ pluginRoot: pluginRelative, runtime: "skills" })
  const installCheck = run(["install-check", output, "--harness", "opencode", "--json"])
  assert.equal(installCheck.status, 0, installCheck.stderr)
  const parsed = JSON.parse(installCheck.stdout)
  assert.equal(parsed.plugin, "polyglot-monorepo")
  assert.equal(parsed.runtime, "skills")
  assert.equal(parsed.results.length, 1)
  assert.ok(["pass", "skip"].includes(parsed.results[0].status))
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

test("install-check reports a plan and rejects unknown harnesses", async () => {
  const help = run(["install-check", "--help"])
  assert.equal(help.status, 0, help.stderr)
  assert.match(help.stdout, /--harness <id>/)

  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-check-"))
  const output = join(parent, "checked-plugin")
  const creation = run([
    "create",
    output,
    "--description",
    "Exercise install-check planning.",
    "--author",
    "Example Team",
    "--repository",
    "example/checked-plugin",
  ])
  assert.equal(creation.status, 0, creation.stderr)

  const unknown = run(["install-check", output, "--harness", "cursor"])
  assert.equal(unknown.status, 2)
  assert.match(unknown.stderr, /Unknown harness/)
})

test("install-check cleans up successful Claude mutations after a later failure", { skip: process.platform === "win32" }, async () => {
  const parent = await mkdtemp(join(tmpdir(), "harness-alchemist-cleanup-"))
  const output = join(parent, "cleanup-plugin")
  const creation = run([
    "create",
    output,
    "--description",
    "Exercise native cleanup behavior.",
    "--author",
    "Example Team",
    "--repository",
    "example/cleanup-plugin",
  ])
  assert.equal(creation.status, 0, creation.stderr)

  const fakeBin = join(parent, "bin")
  const calls = join(parent, "calls.txt")
  await mkdir(fakeBin)
  await writeFile(join(fakeBin, "claude"), `#!/bin/sh
printf '%s\n' "$*" >> "$HARNESS_TEST_LOG"
case "$*" in
  "plugin details "*) exit 9 ;;
esac
`, { mode: 0o755 })

  const check = run(["install-check", output, "--harness", "claude", "--json"], {
    env: {
      ...process.env,
      PATH: `${fakeBin}${delimiter}${process.env.PATH ?? ""}`,
      HARNESS_TEST_LOG: calls,
    },
  })
  assert.equal(check.status, 1)
  const report = JSON.parse(check.stdout)
  assert.deepEqual(Object.keys(report), ["project", "plugin", "runtime", "results", "summary"])
  assert.deepEqual(Object.keys(report.results[0]), ["harness", "status", "details"])
  assert.deepEqual(Object.keys(report.summary), ["pass", "fail", "skip"])
  assert.equal(report.results[0].status, "fail")
  const commands = (await readFile(calls, "utf8")).trim().split("\n")
  assert.match(commands[0], /^plugin validate .* --strict$/)
  assert.deepEqual(commands.slice(1), [
    `plugin marketplace add ${output}`,
    "plugin install cleanup-plugin@cleanup-plugin-plugins",
    "plugin details cleanup-plugin",
    "plugin uninstall cleanup-plugin@cleanup-plugin-plugins",
    "plugin marketplace remove cleanup-plugin-plugins",
  ])
})
