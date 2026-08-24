#!/usr/bin/env node

import { randomUUID } from "node:crypto"
import { chmod, existsSync } from "node:fs"
import { cp, mkdir, readdir, readFile, rename, rm, rmdir, stat, writeFile } from "node:fs/promises"
import { basename, dirname, join, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { spawnSync } from "node:child_process"

const NAME_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/
const PACKAGE_PATTERN = /^(?:@[a-z0-9][a-z0-9._-]*\/)?[a-z0-9]+(?:-[a-z0-9]+)*$/
const LICENSES = new Set(["MIT", "Apache-2.0", "UNLICENSED"])
const CANONICAL_VERSION = "v0.1.0"

export function listTemplates() {
  return `${CANONICAL_VERSION} (canonical)`
}

function usage() {
  return `Usage: harness-alchemist create <output-directory> [options]

Create one TypeScript repository for Claude Code, Codex/ChatGPT, OpenCode,
Google Antigravity, and DeepSeek Harness/Cordis.

Required:
  --description <text>       One-sentence plugin description.
  --author <name>            Author or team name.
  --repository <source>      GitHub owner/repo or a full Git URL.

Options:
  --name <name>              Plugin name; defaults to output directory name.
  --package <name>           npm package; defaults to plugin name. Its basename
                             must equal the plugin name.
  --display-name <text>      Human-readable name; defaults from plugin name.
  --marketplace <name>       Claude/Codex marketplace name; defaults to
                              <plugin-name>-plugins.
  --template <version>       Canonical template version (default: v0.1.0).
  --license <id>             MIT (default), Apache-2.0, or UNLICENSED.
  --dry-run                  Validate inputs and print the files without writing.
  --help                     Show this help.

The destination must be missing or empty. Existing content is never replaced.`
}

function parseArgs(argv) {
  const options = { license: "MIT", templateVersion: CANONICAL_VERSION, dryRun: false }
  const valueOptions = new Map([
    ["--name", "name"],
    ["--description", "description"],
    ["--package", "packageName"],
    ["--author", "author"],
    ["--repository", "repository"],
    ["--display-name", "displayName"],
    ["--marketplace", "marketplace"],
    ["--template", "templateVersion"],
    ["--license", "license"],
  ])

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]
    if (arg === "--help" || arg === "-h") return { help: true }
    if (arg === "--dry-run") {
      options.dryRun = true
      continue
    }
    if (valueOptions.has(arg)) {
      const value = argv[index + 1]
      if (!value || value.startsWith("--")) throw new Error(`${arg} requires a value`)
      options[valueOptions.get(arg)] = value
      index += 1
      continue
    }
    if (arg.startsWith("-")) throw new Error(`Unknown option: ${arg}`)
    if (options.output) throw new Error("Only one output directory may be supplied")
    options.output = arg
  }

  return { ...options, help: false }
}

function titleCase(name) {
  return name
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ")
}

function normalizeRepository(source) {
  if (/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(source)) {
    return { installSource: source, url: `https://github.com/${source}` }
  }
  try {
    const url = new URL(source)
    const githubMatch = url.hostname === "github.com" && url.pathname.match(/^\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/)
    return {
      installSource: githubMatch ? `${githubMatch[1]}/${githubMatch[2]}` : source,
      url: source.replace(/\.git$/, ""),
    }
  } catch {
    if (/^git@[^:]+:.+/.test(source)) return { installSource: source, url: source }
    throw new Error("--repository must be GitHub owner/repo or a full Git URL")
  }
}

function validateOptions(options) {
  if (!options.output) throw new Error("An output directory is required")
  const name = options.name ?? basename(resolve(options.output))
  if (!NAME_PATTERN.test(name) || name.length > 56) {
    throw new Error("Plugin name must be <=56 characters using lowercase letters, digits, and single hyphens")
  }
  const packageName = options.packageName ?? name
  if (!PACKAGE_PATTERN.test(packageName) || packageName.split("/").at(-1) !== name) {
    throw new Error("npm package must be unscoped or @scoped and have the plugin name as its basename")
  }
  if (!options.description?.trim()) throw new Error("--description is required")
  if (options.description.includes("\n")) throw new Error("--description must be one line")
  if (!options.author?.trim()) throw new Error("--author is required")
  if (!options.repository?.trim()) throw new Error("--repository is required")
  if (!LICENSES.has(options.license)) {
    throw new Error("--license must be MIT, Apache-2.0, or UNLICENSED")
  }
  if (options.templateVersion !== CANONICAL_VERSION) {
    throw new Error(`Unknown template version '${options.templateVersion}'. Available: ${CANONICAL_VERSION}`)
  }
  const marketplace = options.marketplace ?? `${name}-plugins`
  if (!NAME_PATTERN.test(marketplace) || marketplace.length > 64) {
    throw new Error("Marketplace name must be <=64 characters in lowercase kebab-case")
  }

  return {
    ...options,
    output: resolve(options.output),
    name,
    packageName,
    marketplace,
    displayName: options.displayName?.trim() || titleCase(name),
    description: options.description.trim(),
    author: options.author.trim(),
    repository: normalizeRepository(options.repository.trim()),
  }
}

async function destinationState(path) {
  if (!existsSync(path)) return "missing"
  const info = await stat(path)
  if (!info.isDirectory()) return "not-directory"
  return (await readdir(path)).length === 0 ? "empty" : "non-empty"
}

function replaceTokens(content, tokens) {
  return content.replace(/\{\{([A-Z0-9_]+)\}\}/g, (match, key) => {
    if (!(key in tokens)) throw new Error(`Unknown template token ${match}`)
    return tokens[key]
  })
}

function mapSegment(segment, metadata) {
  if (segment === "shared-skill") return metadata.name
  if (segment === "develop-template") return `develop-${metadata.name}`
  return segment.endsWith(".tpl") ? segment.slice(0, -4) : segment
}

async function renderTemplates(source, destination, metadata, tokens, relative = []) {
  for (const entry of await readdir(source, { withFileTypes: true })) {
    const nextRelative = [...relative, mapSegment(entry.name, metadata)]
    const target = join(destination, ...nextRelative)
    const input = join(source, entry.name)
    if (entry.isDirectory()) {
      await mkdir(target, { recursive: true })
      await renderTemplates(input, destination, metadata, tokens, nextRelative)
    } else if (entry.isFile()) {
      const content = replaceTokens(await readFile(input, "utf8"), tokens)
      await mkdir(dirname(target), { recursive: true })
      await writeFile(target, content)
    }
  }
}

async function licenseText(skillRoot, metadata) {
  if (metadata.license === "UNLICENSED") {
    return `Copyright (c) ${new Date().getFullYear()} ${metadata.author}\n\nAll rights reserved. This project is not licensed for redistribution.\n`
  }
  if (metadata.license === "Apache-2.0") {
    return readFile(join(skillRoot, "licenses/Apache-2.0.txt"), "utf8")
  }
  return `MIT License

Copyright (c) ${new Date().getFullYear()} ${metadata.author}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
`
}

async function listTemplateFiles(root, metadata, relative = []) {
  const files = []
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const next = [...relative, mapSegment(entry.name, metadata)]
    if (entry.isDirectory()) files.push(...(await listTemplateFiles(join(root, entry.name), metadata, next)))
    else if (entry.isFile()) files.push(next.join("/"))
  }
  files.push(`.agents/skills/develop-${metadata.name}/scripts/validate.mjs`, "LICENSE")
  return files.sort()
}

export async function runCreate(argv) {
  let options
  try {
    const parsed = parseArgs(argv)
    if (parsed.help) {
      console.log(usage())
      return 0
    }
    options = validateOptions(parsed)
  } catch (error) {
    console.error(error.message)
    console.error(usage())
    return 2
  }

  const scriptPath = fileURLToPath(import.meta.url)
  const projectRoot = dirname(dirname(scriptPath))
  const canonicalRoot = join(projectRoot, "templates", options.templateVersion)
  const templateRoot = join(canonicalRoot, "universal-typescript")
  const state = await destinationState(options.output)
  if (state === "not-directory" || state === "non-empty") {
    console.error(`Destination must be missing or empty: ${options.output}`)
    return 1
  }

  const tokens = {
    NAME: options.name,
    NAME_JSON: JSON.stringify(options.name),
    DISPLAY_NAME: options.displayName,
    DISPLAY_NAME_JSON: JSON.stringify(options.displayName),
    DESCRIPTION: options.description,
    DESCRIPTION_JSON: JSON.stringify(options.description),
    SHARED_SKILL_DESCRIPTION_JSON: JSON.stringify(
      `${options.description} Use when the user requests ${options.displayName} workflows or explicitly asks to use the ${options.name} plugin.`,
    ),
    DEVELOPMENT_SKILL_DESCRIPTION_JSON: JSON.stringify(
      `Develop, validate, and publish the ${options.displayName} universal coding-agent plugin. Use when modifying its shared skills, Claude or Codex manifests, OpenCode npm entrypoint, Antigravity bundle, or DeepSeek Cordis integration.`,
    ),
    PACKAGE_NAME: options.packageName,
    PACKAGE_NAME_JSON: JSON.stringify(options.packageName),
    AUTHOR: options.author,
    AUTHOR_JSON: JSON.stringify(options.author),
    REPOSITORY_SOURCE: options.repository.installSource,
    REPOSITORY_SOURCE_JSON: JSON.stringify(options.repository.installSource),
    REPOSITORY_URL: options.repository.url,
    REPOSITORY_URL_JSON: JSON.stringify(options.repository.url),
    MARKETPLACE: options.marketplace,
    MARKETPLACE_JSON: JSON.stringify(options.marketplace),
    LICENSE: options.license,
    LICENSE_JSON: JSON.stringify(options.license),
    YEAR: String(new Date().getFullYear()),
  }

  if (options.dryRun) {
    console.log(JSON.stringify({
      output: options.output,
      template: options.templateVersion,
      plugin: options.name,
      package: options.packageName,
      files: await listTemplateFiles(templateRoot, options),
    }, null, 2))
    return 0
  }

  await mkdir(dirname(options.output), { recursive: true })
  const staging = join(dirname(options.output), `.${basename(options.output)}.scaffold-${randomUUID()}`)

  try {
    await mkdir(staging)
    await renderTemplates(templateRoot, staging, options, tokens)
    const localScripts = join(staging, ".agents/skills", `develop-${options.name}`, "scripts")
    await mkdir(localScripts, { recursive: true })
    await cp(join(projectRoot, "lib/validate.mjs"), join(localScripts, "validate.mjs"))
    const skillScripts = join(staging, "skills", options.name, "scripts")
    if (existsSync(skillScripts)) {
      for (const entry of await readdir(skillScripts)) {
        await chmod(join(skillScripts, entry), 0o755)
      }
    }
    await writeFile(join(staging, "LICENSE"), await licenseText(canonicalRoot, options))

    const validation = spawnSync(process.execPath, [join(projectRoot, "lib/validate.mjs"), staging], {
      encoding: "utf8",
    })
    if (validation.status !== 0) {
      throw new Error(`Generated project failed validation:\n${validation.stderr || validation.stdout}`)
    }

    if (state === "empty") await rmdir(options.output)
    await rename(staging, options.output)

    console.log(`Created universal plugin scaffold at ${options.output}`)
    console.log(`Next: cd ${JSON.stringify(options.output)} && npm install && npm run verify`)
    console.log(`Bun:  cd ${JSON.stringify(options.output)} && bun install && bun run verify`)
    return 0
  } catch (error) {
    await rm(staging, { recursive: true, force: true })
    console.error(error.message)
    return 1
  }
}

if (resolve(process.argv[1] ?? "") === fileURLToPath(import.meta.url)) {
  process.exitCode = await runCreate(process.argv.slice(2))
}
