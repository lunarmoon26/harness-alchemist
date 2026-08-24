#!/usr/bin/env node

import { existsSync } from "node:fs"
import { readFile, writeFile } from "node:fs/promises"
import { dirname, join, resolve } from "node:path"
import { fileURLToPath } from "node:url"

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

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"))
}

async function writeJson(path, value) {
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`)
}

function authorObject(author, repository) {
  if (typeof author === "string") return { name: author }
  if (author && typeof author === "object") return author
  return { name: "Plugin contributors", url: repository }
}

const scriptDirectory = dirname(fileURLToPath(import.meta.url))
const root = findRoot(process.cwd()) ?? findRoot(scriptDirectory)
if (!root) throw new Error("Could not find the plugin project root")
const packagePath = join(root, "package.json")
const packageJson = await readJson(packagePath)
const pluginName = packageJson.name?.split("/").at(-1)

if (!pluginName || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(pluginName)) {
  throw new Error("package.json name must have a lowercase kebab-case basename")
}

const repository = typeof packageJson.repository === "string"
  ? packageJson.repository
  : packageJson.repository?.url
const author = authorObject(packageJson.author, repository)

if (!packageJson.version || !packageJson.description || !repository || !packageJson.license) {
  throw new Error("package.json requires version, description, repository, and license")
}

const claudePath = join(root, ".claude-plugin/plugin.json")
const claude = await readJson(claudePath)
Object.assign(claude, {
  name: pluginName,
  version: packageJson.version,
  description: packageJson.description,
  author,
  homepage: repository,
  repository,
  license: packageJson.license,
})
await writeJson(claudePath, claude)

const codexPath = join(root, ".codex-plugin/plugin.json")
const codex = await readJson(codexPath)
Object.assign(codex, {
  name: pluginName,
  version: packageJson.version,
  description: packageJson.description,
  author: { ...author, url: author.url ?? repository },
  homepage: repository,
  repository,
  license: packageJson.license,
})
await writeJson(codexPath, codex)

const antigravityPath = join(root, "plugin.json")
const antigravity = await readJson(antigravityPath)
Object.assign(antigravity, { name: pluginName, description: packageJson.description })
await writeJson(antigravityPath, antigravity)

const claudeMarketplacePath = join(root, ".claude-plugin/marketplace.json")
const claudeMarketplace = await readJson(claudeMarketplacePath)
claudeMarketplace.owner = author
const claudeEntry = claudeMarketplace.plugins?.find((entry) => entry.name === pluginName)
if (!claudeEntry) throw new Error("Claude marketplace does not contain the package plugin")
Object.assign(claudeEntry, { description: packageJson.description, author })
await writeJson(claudeMarketplacePath, claudeMarketplace)

const codexMarketplacePath = join(root, ".agents/plugins/marketplace.json")
const codexMarketplace = await readJson(codexMarketplacePath)
const codexEntry = codexMarketplace.plugins?.find((entry) => entry.name === pluginName)
if (!codexEntry) throw new Error("Codex marketplace does not contain the package plugin")
await writeJson(codexMarketplacePath, codexMarketplace)

const cordisPath = join(root, "cordis.patch.yml")
const cordisLines = (await readFile(cordisPath, "utf8")).split("\n")
let updatedCordisEntry = false

for (let index = 0; index < cordisLines.length; index += 1) {
  const idMatch = cordisLines[index].match(/^(\s*)-\s+id:\s*['\"]?([^'\"\s]+)['\"]?\s*$/)
  if (!idMatch || idMatch[2] !== pluginName) continue

  const idIndent = idMatch[1].length
  for (let next = index + 1; next < cordisLines.length; next += 1) {
    const line = cordisLines[next]
    if (line.trim() && line.search(/\S/) <= idIndent) break
    const nameMatch = line.match(/^(\s*)name:\s*.+$/)
    if (nameMatch) {
      cordisLines[next] = `${nameMatch[1]}name: '${packageJson.name}/deepseek'`
      updatedCordisEntry = true
      break
    }
  }
  break
}

if (!updatedCordisEntry) throw new Error(`Could not find Cordis entry '${pluginName}' to update`)
await writeFile(cordisPath, cordisLines.join("\n"))

const layoutPath = join(root, "harness-alchemist.json")
if (existsSync(layoutPath)) {
  const layout = await readJson(layoutPath)
  if (layout.generatorVersion !== packageJson.version) {
    layout.generatorVersion = packageJson.version
    await writeJson(layoutPath, layout)
  }
}

console.log(`Synchronized plugin manifests from ${packagePath}`)
