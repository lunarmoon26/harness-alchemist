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

const result = spawnSync(
  "npm",
  ["pack", "--dry-run", "--json", "--ignore-scripts"],
  { cwd: root, encoding: "utf8" },
)

if (result.error) throw result.error
if (result.status !== 0) throw new Error(result.stderr || result.stdout || "npm pack failed")

const report = JSON.parse(result.stdout)
const files = new Set(report[0]?.files?.map((file) => file.path) ?? [])
const packageJson = JSON.parse(await readFile(join(root, "package.json"), "utf8"))

for (const required of [
  ".claude-plugin/plugin.json",
  ".codex-plugin/plugin.json",
  "cordis.patch.yml",
  "dist/deepseek.js",
  "dist/opencode.js",
  "plugin.json",
]) {
  if (!files.has(required)) throw new Error(`npm package is missing ${required}`)
}

for (const target of exportTargets(packageJson.exports)) {
  if (!target.startsWith("./") || target.includes("..")) {
    throw new Error(`Invalid package export target: ${target}`)
  }
  const path = target.slice(2)
  if (!files.has(path)) throw new Error(`Package export target is not packed: ${target}`)
}

console.log(`Validated ${files.size} files in the npm dry-run payload`)
