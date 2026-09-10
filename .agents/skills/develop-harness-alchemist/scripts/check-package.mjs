#!/usr/bin/env node

import { existsSync } from "node:fs"
import { readFile } from "node:fs/promises"
import { dirname, join, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { spawnSync } from "node:child_process"

function findRoot(start) {
  let current = resolve(start)
  while (true) {
    if (existsSync(join(current, "package.json")) && existsSync(join(current, ".claude-plugin"))) {
      return current
    }
    const parent = dirname(current)
    if (parent === current) return undefined
    current = parent
  }
}

function run(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: "utf8" })
  if (result.error) throw result.error
  if (result.status !== 0) throw new Error(result.stderr || result.stdout || `${command} failed`)
  return result.stdout
}

function packFiles(directory) {
  const report = JSON.parse(run("npm", ["pack", "--dry-run", "--json", "--ignore-scripts"], directory))
  return new Map(report[0]?.files?.map((file) => [file.path, file]) ?? [])
}

function exportTargets(exports) {
  const targets = []
  for (const value of Object.values(exports ?? {})) {
    if (typeof value === "string") targets.push(value)
    else if (value && typeof value === "object") {
      if (typeof value.import === "string") targets.push(value.import)
      if (typeof value.types === "string") targets.push(value.types)
    }
  }
  return targets
}

const scriptDirectory = dirname(fileURLToPath(import.meta.url))
const root = findRoot(process.cwd()) ?? findRoot(scriptDirectory)
if (!root) throw new Error("Could not find the plugin project root")

run(process.execPath, [
  join(root, "bin/harness-alchemist.mjs"),
  "__stage-native-packages",
  "--output",
  join(root, ".native-packages"),
], root)

const staging = join(root, ".native-packages")
const mainDirectory = join(staging, "main")
const mainManifest = JSON.parse(await readFile(join(mainDirectory, "package.json"), "utf8"))
const mainFiles = packFiles(mainDirectory)

for (const required of [
  ".claude-plugin/plugin.json",
  ".codex-plugin/plugin.json",
  "alchemy.json",
  "alchemy.schema.json",
  "bin/harness-alchemist.mjs",
  "cordis.patch.yml",
  "dist/deepseek.js",
  "dist/opencode.js",
  "mcp.json",
  "plugin.json",
  "skills/harness-alchemist/skill-runtime.json",
  "templates/v0.1.0/generated/validate.mjs",
  "templates/v0.1.0/universal-typescript/.github/workflows/npm-publish.yml.tpl",
  "templates/v0.1.0/universal-typescript/alchemy.json.tpl",
  "templates/v0.1.0/universal-typescript/mcp.json.tpl",
  "templates/v0.1.0/universal-typescript/package.json.tpl",
  "templates/v0.1.0/universal-typescript/skills/shared-skill/skill-runtime.json.tpl",
  "templates/v0.1.0/template.json",
]) {
  if (!mainFiles.has(required)) throw new Error(`main npm package is missing ${required}`)
}

for (const target of exportTargets(mainManifest.exports)) {
  if (!target.startsWith("./") || target.includes("..")) {
    throw new Error(`Invalid package export target: ${target}`)
  }
  if (!mainFiles.has(target.slice(2))) throw new Error(`Package export target is not packed: ${target}`)
}
if (process.platform !== "win32" && !(mainFiles.get("bin/harness-alchemist.mjs")?.mode & 0o111)) {
  throw new Error("main npm launcher must be executable")
}

const expectedPlatformPackages = [
  "@lunarmoon26/harness-alchemist-darwin-arm64",
  "@lunarmoon26/harness-alchemist-darwin-x64",
  "@lunarmoon26/harness-alchemist-linux-arm64",
  "@lunarmoon26/harness-alchemist-linux-x64",
  "@lunarmoon26/harness-alchemist-win32-arm64",
  "@lunarmoon26/harness-alchemist-win32-x64",
]
if (mainManifest.scripts) throw new Error("published main package must not contain lifecycle scripts")
for (const packageName of expectedPlatformPackages) {
  if (mainManifest.optionalDependencies?.[packageName] !== mainManifest.version) {
    throw new Error(`main package must pin ${packageName} to ${mainManifest.version}`)
  }
}

const host = `${process.platform}-${process.arch}`
const packageBasename = {
  "darwin-arm64": "harness-alchemist-darwin-arm64",
  "darwin-x64": "harness-alchemist-darwin-x64",
  "linux-arm64": "harness-alchemist-linux-arm64",
  "linux-x64": "harness-alchemist-linux-x64",
  "win32-arm64": "harness-alchemist-win32-arm64",
  "win32-x64": "harness-alchemist-win32-x64",
}[host]
if (!packageBasename) throw new Error(`Unsupported package-audit host: ${host}`)

const platformDirectory = join(staging, "platform", packageBasename)
const platformManifest = JSON.parse(await readFile(join(platformDirectory, "package.json"), "utf8"))
const platformFiles = packFiles(platformDirectory)
const executable = process.platform === "win32" ? "harness-alchemist.exe" : "harness-alchemist"
if (!platformFiles.has(executable)) throw new Error(`platform package is missing ${executable}`)
if (process.platform !== "win32" && !(platformFiles.get(executable)?.mode & 0o111)) {
  throw new Error("native package binary must be executable")
}
if (platformManifest.version !== mainManifest.version) throw new Error("platform package version differs from main package")
if (platformManifest.scripts || platformManifest.dependencies) {
  throw new Error("platform package must not contain scripts or dependencies")
}
const version = run(join(platformDirectory, executable), ["version"], platformDirectory).trim()
if (version !== mainManifest.version) throw new Error(`native binary reports ${version}, expected ${mainManifest.version}`)

console.log(`Validated ${mainFiles.size} main-package files and ${platformFiles.size} native-package files`)
