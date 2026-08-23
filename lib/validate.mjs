#!/usr/bin/env node

import { existsSync } from "node:fs"
import { readdir, readFile, realpath } from "node:fs/promises"
import { basename, dirname, join, resolve, sep } from "node:path"
import { fileURLToPath } from "node:url"
import { spawnSync } from "node:child_process"

const REQUIRED_FILES = [
  ".agents/plugins/marketplace.json",
  ".claude-plugin/marketplace.json",
  ".claude-plugin/plugin.json",
  ".codex-plugin/plugin.json",
  ".github/workflows/npm-publish.yml",
  ".gitignore",
  "AGENTS.md",
  "LICENSE",
  "README.md",
  "cordis.patch.yml",
  "package.json",
  "plugin.json",
  "src/deepseek.ts",
  "src/opencode.ts",
  "tsconfig.json",
]

function usage() {
  return `Usage: harness-alchemist validate [project-directory] [--external] [--json]

Validates a universal Claude, Codex, OpenCode, Antigravity, and DeepSeek
plugin scaffold. By default the project is resolved from the current working
directory or from the script's containing generated project.

Options:
  --external  Run installed platform validators, currently Claude Code.
  --json      Print a machine-readable result.
  --help      Show this help.`
}

function findProjectRoot(start) {
  let current = resolve(start)
  while (true) {
    if (
      existsSync(join(current, "package.json")) &&
      existsSync(join(current, ".claude-plugin")) &&
      existsSync(join(current, ".codex-plugin"))
    ) {
      return current
    }
    const parent = dirname(current)
    if (parent === current) return undefined
    current = parent
  }
}

function parseArgs(argv) {
  let project
  let external = false
  let json = false

  for (const arg of argv) {
    if (arg === "--help" || arg === "-h") return { help: true }
    if (arg === "--external") external = true
    else if (arg === "--json") json = true
    else if (arg.startsWith("-")) throw new Error(`Unknown option: ${arg}`)
    else if (project) throw new Error("Only one project directory may be supplied")
    else project = arg
  }

  return { project, external, json, help: false }
}

async function readJson(path, errors) {
  try {
    return JSON.parse(await readFile(path, "utf8"))
  } catch (error) {
    errors.push(`${path}: invalid JSON (${error.message})`)
    return undefined
  }
}

function unquote(value) {
  const trimmed = value.trim()
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1)
  }
  return trimmed
}

function parseFrontmatter(content) {
  if (!content.startsWith("---\n")) return undefined
  const end = content.indexOf("\n---", 4)
  if (end < 0) return undefined
  const block = content.slice(4, end)
  const values = {}
  for (const line of block.split("\n")) {
    const match = line.match(/^([a-zA-Z0-9-]+):\s*(.*)$/)
    if (match) values[match[1]] = unquote(match[2])
  }
  return values
}

function parseYamlScalar(value) {
  const scalar = value.trim()
  if (scalar.startsWith("'") && scalar.endsWith("'")) {
    return scalar.slice(1, -1).replaceAll("''", "'")
  }
  if (scalar.startsWith('"') && scalar.endsWith('"')) {
    try {
      return JSON.parse(scalar)
    } catch {
      return undefined
    }
  }
  return /^[A-Za-z0-9@/_.-]+$/.test(scalar) ? scalar : undefined
}

function parseCordisInsertEntries(content) {
  if (content.includes("\t")) return []
  const lines = content.split("\n")
  const entries = []

  for (let index = 0; index < lines.length; index += 1) {
    const insertMatch = lines[index].match(/^(\s*)-\s+insert:\s*(?:#.*)?$/)
    if (!insertMatch || insertMatch[1].length !== 0) continue

    let entryIndent
    for (let next = index + 1; next < lines.length; next += 1) {
      const line = lines[next]
      if (line.trim() && line.search(/\S/) === 0) break
      if (!line.trim() || line.trimStart().startsWith("#")) continue
      const idMatch = line.match(/^(\s*)-\s+id:\s*(.+?)\s*$/)
      if (entryIndent === undefined) {
        if (!idMatch || idMatch[1].length === 0) break
        entryIndent = idMatch[1].length
      }
      if (!idMatch || idMatch[1].length !== entryIndent) continue
      const id = parseYamlScalar(idMatch[2])
      if (!id) continue

      const idIndent = idMatch[1].length
      let name
      let nameFields = 0
      for (let field = next + 1; field < lines.length; field += 1) {
        const fieldLine = lines[field]
        const fieldIndent = fieldLine.search(/\S/)
        if (fieldLine.trim() && fieldIndent <= idIndent) break
        const nameMatch = fieldLine.match(/^(\s*)name:\s*(.+?)\s*$/)
        if (nameMatch && nameMatch[1].length === idIndent + 2) {
          nameFields += 1
          name = parseYamlScalar(nameMatch[2])
        }
      }
      entries.push({ id, name, valid: name !== undefined && nameFields === 1 })
    }
  }

  return entries
}

async function collectSkillFiles(root, relativeRoot) {
  const start = join(root, relativeRoot)
  if (!existsSync(start)) return []
  const results = []

  async function walk(directory, depth) {
    if (depth > 5) return
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name)
      if (entry.isDirectory()) await walk(path, depth + 1)
      else if (entry.isFile() && entry.name === "SKILL.md") results.push(path)
    }
  }

  await walk(start, 0)
  return results
}

async function scanForTokens(root) {
  const matches = []
  const ignored = new Set([".git", "dist", "node_modules", "coverage", "templates"])

  async function walk(directory, depth) {
    if (depth > 8) return
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      if (ignored.has(entry.name)) continue
      const path = join(directory, entry.name)
      if (entry.isDirectory()) {
        await walk(path, depth + 1)
      } else if (entry.isFile()) {
        const content = await readFile(path, "utf8").catch(() => "")
        if (/\{\{[A-Z0-9_]+\}\}/.test(content)) matches.push(path)
      }
    }
  }

  await walk(root, 0)
  return matches
}

function requirePath(root, relativePath, errors) {
  if (!existsSync(join(root, relativePath))) errors.push(`Missing required file: ${relativePath}`)
}

async function requireManifestPath(root, value, label, errors) {
  if (typeof value !== "string" || !value.startsWith("./")) {
    errors.push(`${label} must be a ./-relative path`)
    return
  }
  const target = resolve(root, value)
  if (!(target === root || target.startsWith(`${root}${sep}`))) {
    errors.push(`${label} escapes the plugin root`)
  } else if (!existsSync(target)) {
    errors.push(`${label} points to missing path ${value}`)
  } else {
    const [canonicalRoot, canonicalTarget] = await Promise.all([realpath(root), realpath(target)])
    if (!(canonicalTarget === canonicalRoot || canonicalTarget.startsWith(`${canonicalRoot}${sep}`))) {
      errors.push(`${label} resolves through a symlink outside the plugin root`)
    }
  }
}

function exportTarget(exports, key, field = "import") {
  const value = exports?.[key]
  if (typeof value === "string") return value
  if (value && typeof value === "object") return value[field]
  return undefined
}

export async function validateProject(projectRoot, options = {}) {
  const root = resolve(projectRoot)
  const errors = []
  const warnings = []

  for (const file of REQUIRED_FILES) requirePath(root, file, errors)
  if (errors.length > 0) return { root, errors, warnings }

  const packageJson = await readJson(join(root, "package.json"), errors)
  const claudePlugin = await readJson(join(root, ".claude-plugin/plugin.json"), errors)
  const claudeMarketplace = await readJson(
    join(root, ".claude-plugin/marketplace.json"),
    errors,
  )
  const codexPlugin = await readJson(join(root, ".codex-plugin/plugin.json"), errors)
  const codexMarketplace = await readJson(
    join(root, ".agents/plugins/marketplace.json"),
    errors,
  )
  const antigravityPlugin = await readJson(join(root, "plugin.json"), errors)

  if (errors.length > 0) return { root, errors, warnings }

  const packageBase = packageJson.name?.split("/").at(-1)
  const pluginName = claudePlugin.name
  const validName = /^[a-z0-9]+(?:-[a-z0-9]+)*$/

  if (!validName.test(pluginName ?? "")) {
    errors.push("Plugin name must contain lowercase letters, digits, and single hyphens")
  }
  if ((pluginName?.length ?? 0) > 56) {
    errors.push("Plugin name must be at most 56 characters")
  }
  if (packageBase !== pluginName) {
    errors.push(`npm package basename '${packageBase}' must match plugin name '${pluginName}'`)
  }

  for (const [label, value] of [
    ["Codex plugin", codexPlugin.name],
    ["Antigravity plugin", antigravityPlugin.name],
  ]) {
    if (value !== pluginName) errors.push(`${label} name '${value}' does not match '${pluginName}'`)
  }

  for (const [label, manifest] of [
    ["Claude plugin", claudePlugin],
    ["Codex plugin", codexPlugin],
  ]) {
    if (manifest.version !== packageJson.version) {
      errors.push(`${label} version does not match package.json`)
    }
    if (manifest.description !== packageJson.description) {
      errors.push(`${label} description does not match package.json`)
    }
  }
  if (antigravityPlugin.description !== packageJson.description) {
    errors.push("Antigravity description does not match package.json")
  }

  if (packageJson.type !== "module") errors.push("package.json type must be 'module'")
  if (packageJson.engines?.node !== ">=22.20.0") {
    errors.push("package.json engines.node must be '>=22.20.0'")
  }
  const requiredExports = [
    [".", "import", "./dist/opencode.js"],
    [".", "types", "./dist/opencode.d.ts"],
    ["./deepseek", "import", "./dist/deepseek.js"],
    ["./deepseek", "types", "./dist/deepseek.d.ts"],
    ["./cordis.patch.yml", "import", "./cordis.patch.yml"],
  ]
  for (const [key, field, expected] of requiredExports) {
    const target = exportTarget(packageJson.exports, key, field)
    if (target !== expected) {
      errors.push(`package.json export '${key}' ${field} target must be '${expected}'`)
    }
  }
  for (const entry of [
    "dist",
    "skills",
    "cordis.patch.yml",
    ".claude-plugin/plugin.json",
    ".codex-plugin/plugin.json",
    "plugin.json",
  ]) {
    if (!packageJson.files?.includes(entry)) errors.push(`package.json files is missing '${entry}'`)
  }
  if (packageJson.dsh?.bundle?.patch !== "./cordis.patch.yml") {
    errors.push("package.json dsh.bundle.patch must be './cordis.patch.yml'")
  }

  await requireManifestPath(root, claudePlugin.skills, "Claude skills", errors)
  await requireManifestPath(root, codexPlugin.skills, "Codex skills", errors)

  const claudeEntry = claudeMarketplace.plugins?.find((entry) => entry.name === pluginName)
  if (!claudeEntry) errors.push("Claude marketplace is missing the plugin entry")
  else if (claudeEntry.source !== "./") errors.push("Claude marketplace source must be './'")

  const codexEntry = codexMarketplace.plugins?.find((entry) => entry.name === pluginName)
  if (!codexEntry) errors.push("Codex marketplace is missing the plugin entry")
  else {
    if (codexEntry.source?.source !== "local" || codexEntry.source?.path !== "./") {
      errors.push("Codex marketplace source must be a local './' path")
    }
    if (!codexEntry.policy?.installation || !codexEntry.policy?.authentication) {
      errors.push("Codex marketplace entry requires installation and authentication policies")
    }
    if (!codexEntry.category) errors.push("Codex marketplace entry requires a category")
  }

  const patch = await readFile(join(root, "cordis.patch.yml"), "utf8")
  const expectedModule = `${packageJson.name}/deepseek`
  const cordisEntries = parseCordisInsertEntries(patch)
    .filter((entry) => entry.id === pluginName)
  const cordisEntry = cordisEntries[0]
  if (cordisEntries.length !== 1 || !cordisEntry?.valid) {
    errors.push(`cordis.patch.yml must contain a valid insert entry for '${pluginName}'`)
  } else if (cordisEntry.name !== expectedModule) {
    errors.push(`cordis.patch.yml entry '${pluginName}' must load '${expectedModule}'`)
  }

  const skillFiles = [
    ...(await collectSkillFiles(root, "skills")),
    ...(await collectSkillFiles(root, ".agents/skills")),
  ]
  if (skillFiles.length < 2) errors.push("Expected a shared skill and a project development skill")

  for (const skillFile of skillFiles) {
    const frontmatter = parseFrontmatter(await readFile(skillFile, "utf8"))
    const directoryName = basename(dirname(skillFile))
    if (!frontmatter) errors.push(`${skillFile}: missing YAML frontmatter`)
    else {
      if (frontmatter.name !== directoryName) {
        errors.push(`${skillFile}: frontmatter name must match directory '${directoryName}'`)
      }
      if (!frontmatter.description) errors.push(`${skillFile}: description is required`)
      if (!validName.test(frontmatter.name ?? "") || (frontmatter.name?.length ?? 0) > 64) {
        errors.push(`${skillFile}: invalid Agent Skill name`)
      }
    }
  }

  for (const path of await scanForTokens(root)) {
    errors.push(`${path}: unresolved scaffold token`)
  }

  if (options.external) {
    const result = spawnSync("claude", ["plugin", "validate", root, "--strict"], {
      encoding: "utf8",
    })
    if (result.error?.code === "ENOENT") warnings.push("Claude CLI not found; skipped external validation")
    else if (result.status !== 0) {
      errors.push(`Claude plugin validation failed: ${(result.stderr || result.stdout).trim()}`)
    }
  }

  return { root, errors, warnings }
}

export async function runValidate(argv) {
  let args
  try {
    args = parseArgs(argv)
  } catch (error) {
    console.error(error.message)
    console.error(usage())
    return 2
  }

  if (args.help) {
    console.log(usage())
    return 0
  }

  const scriptDirectory = dirname(fileURLToPath(import.meta.url))
  const root = args.project
    ? resolve(args.project)
    : findProjectRoot(process.cwd()) ?? findProjectRoot(scriptDirectory)

  if (!root) {
    console.error("Could not find a universal plugin project. Pass its directory explicitly.")
    return 2
  }

  const result = await validateProject(root, { external: args.external })
  if (args.json) {
    console.log(JSON.stringify({ valid: result.errors.length === 0, ...result }, null, 2))
  } else if (result.errors.length === 0) {
    console.log(`Validated universal plugin scaffold at ${result.root}`)
    for (const warning of result.warnings) console.warn(`Warning: ${warning}`)
  } else {
    console.error(`Validation failed for ${result.root}:`)
    for (const error of result.errors) console.error(`- ${error}`)
    for (const warning of result.warnings) console.warn(`Warning: ${warning}`)
  }

  return result.errors.length > 0 ? 1 : 0
}

if (resolve(process.argv[1] ?? "") === fileURLToPath(import.meta.url)) {
  process.exitCode = await runValidate(process.argv.slice(2))
}
