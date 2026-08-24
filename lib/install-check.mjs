#!/usr/bin/env node

import { existsSync, readdirSync } from "node:fs"
import { cp, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { spawnSync } from "node:child_process"

const TIMEOUT_MS = 300_000

function findProjectRoot(start) {
  let current = resolve(start)
  let conventionalRoot
  while (true) {
    if (existsSync(join(current, "alchemy.json"))) return current
    if (
      !conventionalRoot &&
      existsSync(join(current, "package.json")) &&
      existsSync(join(current, ".claude-plugin")) &&
      existsSync(join(current, ".codex-plugin"))
    ) {
      conventionalRoot = current
    }
    const parent = dirname(current)
    if (parent === current) return conventionalRoot
    current = parent
  }
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    timeout: TIMEOUT_MS,
    ...options,
  })
  if (result.error?.code === "ENOENT") return { missing: true }
  return {
    ok: result.status === 0,
    status: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  }
}

function firstLine(text) {
  return text.trim().split("\n")[0] ?? ""
}

function usage() {
  return `Usage: harness-alchemist install-check [project-directory] [options]

Install-level verification: drives the local harness CLIs (claude, codex,
agy, opencode, dsh) against the project's plugin package and asserts each
harness can discover it. Static validation is a prerequisite; run
'harness-alchemist validate' first.

Options:
  --harness <id>  Limit to one harness (claude, codex, agy, opencode, dsh).
                  Repeatable.
  --keep          Keep installed plugins and marketplaces after the check.
  --json          Print a machine-readable result.
  --help          Show this help.`
}

function parseArgs(argv) {
  const options = { harnesses: [], keep: false, json: false }
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]
    if (arg === "--help" || arg === "-h") return { help: true }
    if (arg === "--keep") options.keep = true
    else if (arg === "--json") options.json = true
    else if (arg === "--harness") {
      const value = argv[index + 1]
      if (!value || value.startsWith("--")) throw new Error("--harness requires a value")
      if (!["claude", "codex", "agy", "opencode", "dsh"].includes(value)) {
        throw new Error(`Unknown harness '${value}'. Choose from claude, codex, agy, opencode, dsh`)
      }
      options.harnesses.push(value)
      index += 1
    } else if (arg.startsWith("-")) throw new Error(`Unknown option: ${arg}`)
    else if (options.project) throw new Error("Only one project directory may be supplied")
    else options.project = arg
  }
  return options
}

async function readJson(path, problems) {
  try {
    return JSON.parse(await readFile(path, "utf8"))
  } catch (error) {
    problems.push(`${path}: invalid JSON (${error.message})`)
    return undefined
  }
}

function skillNames(skillsDirectory) {
  if (!existsSync(skillsDirectory)) return []
  return readdirSync(skillsDirectory)
    .filter((entry) => existsSync(join(skillsDirectory, entry, "SKILL.md")))
}

export async function runInstallCheck(argv) {
  let options
  try {
    options = parseArgs(argv)
  } catch (error) {
    console.error(error.message)
    console.error(usage())
    return 2
  }
  if (options.help) {
    console.log(usage())
    return 0
  }

  const scriptDirectory = dirname(fileURLToPath(import.meta.url))
  const root = options.project
    ? resolve(options.project)
    : findProjectRoot(process.cwd()) ?? findProjectRoot(scriptDirectory)
  if (!root) {
    console.error("Could not find a universal plugin project. Pass its directory explicitly.")
    return 2
  }

  const problems = []
  const layoutPath = join(root, "alchemy.json")
  const layout = existsSync(layoutPath) ? (await readJson(layoutPath, problems)) ?? {} : {}
  const pluginRoot = resolve(root, layout.pluginRoot ?? ".")
  const runtime = layout.runtime ?? "npm"

  const claudePlugin = await readJson(join(pluginRoot, ".claude-plugin/plugin.json"), problems)
  const claudeMarketplace = await readJson(join(root, ".claude-plugin/marketplace.json"), problems)
  if (problems.length > 0) {
    for (const problem of problems) console.error(problem)
    return 1
  }

  const pluginName = claudePlugin.name
  const marketplaceName = claudeMarketplace.name
  const skills = skillNames(join(pluginRoot, "skills"))
  const selected = options.harnesses.length > 0
    ? options.harnesses
    : ["claude", "codex", "agy", "opencode", "dsh"]

  const results = []
  const record = (harness, status, details) => {
    results.push({ harness, status, details })
    if (options.json) return
    const marker = status === "pass" ? "✔" : status === "skip" ? "○" : "✖"
    console.log(`${marker} ${harness}: ${status}`)
    for (const detail of details) console.log(`    ${detail}`)
  }

  const adapterPath = join(pluginRoot, "dist/opencode.js")

  for (const harness of selected) {
    if (harness === "claude") {
      const details = []
      let status = "pass"
      const steps = [
        ["validate", pluginRoot, "--strict"],
        ["marketplace", "add", root],
        ["install", `${pluginName}@${marketplaceName}`],
        ["details", pluginName],
      ]
      for (const args of steps) {
        const result = run("claude", ["plugin", ...args])
        if (result.missing) { status = "skip"; details.push("claude CLI not found"); break }
        if (!result.ok) {
          status = "fail"
          details.push(`\`${command} ${args.join(" ")}\` failed: ${firstLine(result.stderr || result.stdout)}`)
          break
        }
        if (args[0] === "details") {
          for (const skill of skills) {
            if (!result.stdout.includes(skill)) {
              status = "fail"
              details.push(`skill '${skill}' missing from plugin details`)
            }
          }
          details.push(`${skills.length} skills registered`)
        }
      }
      if (!options.keep) {
        run("claude", ["plugin", "uninstall", `${pluginName}@${marketplaceName}`])
        run("claude", ["plugin", "marketplace", "remove", marketplaceName])
      }
      record(harness, status, details)
    } else if (harness === "codex") {
      const details = []
      let status = "pass"
      const add = run("codex", ["plugin", "marketplace", "add", root])
      if (add.missing) {
        record(harness, "skip", ["codex CLI not found"])
        continue
      }
      if (!add.ok) {
        record(harness, "fail", [`marketplace add failed: ${firstLine(add.stderr || add.stdout)}`])
        continue
      }
      const install = run("codex", ["plugin", "add", `${pluginName}@${marketplaceName}`])
      if (!install.ok) {
        status = "fail"
        details.push(`plugin add failed: ${firstLine(install.stderr || install.stdout)}`)
      } else {
        const list = run("codex", ["plugin", "list"])
        if (!list.ok || !list.stdout.includes(`${pluginName}@${marketplaceName}`)) {
          status = "fail"
          details.push("plugin not listed as installed")
        } else if (!list.stdout.includes("installed, enabled")) {
          status = "fail"
          details.push("plugin installed but not enabled")
        } else {
          details.push("installed and enabled")
        }
      }
      if (!options.keep) {
        run("codex", ["plugin", "remove", `${pluginName}@${marketplaceName}`])
        run("codex", ["plugin", "marketplace", "remove", marketplaceName])
      }
      record(harness, status, details)
    } else if (harness === "agy") {
      const details = []
      const validate = run("agy", ["plugin", "validate", pluginRoot])
      if (validate.missing) {
        record(harness, "skip", ["agy CLI not found"])
        continue
      }
      if (!validate.ok) {
        record(harness, "fail", [`validate failed: ${firstLine(validate.stderr || validate.stdout)}`])
        continue
      }
      const install = run("agy", ["plugin", "install", pluginRoot])
      if (!install.ok) {
        record(harness, "fail", [`install failed: ${firstLine(install.stderr || install.stdout)}`])
        continue
      }
      const list = run("agy", ["plugin", "list"])
      if (!list.ok || !list.stdout.includes(pluginName)) {
        record(harness, "fail", ["plugin missing from agy plugin list"])
        continue
      }
      details.push("validated, installed, and listed")
      if (!options.keep) run("agy", ["plugin", "uninstall", pluginName])
      record(harness, "pass", details)
    } else if (harness === "opencode") {
      if (runtime === "npm" && !existsSync(adapterPath)) {
        record(harness, "fail", [
          `npm runtime requires a built adapter; missing ${adapterPath}`,
          "run `npm run build` in the plugin package first",
        ])
        continue
      }
      const isolation = await mkdtemp(join(tmpdir(), "ha-opencode-"))
      const configDirectory = join(isolation, "opencode")
      const skillsDirectory = join(configDirectory, "skills")
      await cp(join(pluginRoot, "skills"), skillsDirectory, { recursive: true })
      const config = { $schema: "https://opencode.ai/config.json" }
      if (runtime === "npm") {
        config.plugin = [`file://${adapterPath}`]
      }
      await writeFile(join(configDirectory, "opencode.json"), `${JSON.stringify(config, null, 2)}\n`)
      const environment = {
        ...process.env,
        XDG_CONFIG_HOME: isolation,
        HOME: isolation,
      }
      const skillCheck = run("opencode", ["debug", "skill"], { env: environment })
      if (skillCheck.missing) {
        await rm(isolation, { recursive: true, force: true })
        record(harness, "skip", ["opencode CLI not found"])
        continue
      }
      const details = []
      let status = "pass"
      const missing = skills.filter((skill) => !skillCheck.stdout.includes(`"name": "${skill}"`))
      if (missing.length > 0) {
        status = "fail"
        details.push(`skills not discovered: ${missing.join(", ")}`)
      } else {
        details.push(`${skills.length} skills discovered`)
      }
      if (runtime === "npm" && status === "pass") {
        const startup = run("opencode", ["debug", "startup"], { env: environment })
        if (!startup.ok) {
          status = "fail"
          details.push(`startup failed: ${firstLine(startup.stderr || startup.stdout)}`)
        } else {
          details.push("plugin loaded at startup")
        }
      }
      await rm(isolation, { recursive: true, force: true })
      record(harness, status, details)
    } else if (harness === "dsh") {
      if (runtime !== "npm") {
        record(harness, "skip", ["skills runtime has no Cordis bundle; install skills via filesystem roots instead"])
        continue
      }
      if (!existsSync(adapterPath)) {
        record(harness, "fail", [
          `npm runtime requires built adapters; missing ${join(pluginRoot, "dist/deepseek.js")}`,
          "run `npm run build` in the plugin package first",
        ])
        continue
      }
      const dshHome = await mkdtemp(join(tmpdir(), "ha-dsh-"))
      const environment = { ...process.env, DSH_HOME: dshHome }
      let dshCommand = "dsh"
      let dshPrefix = []
      if (spawnSync("dsh", ["--version"], { encoding: "utf8" }).error?.code === "ENOENT") {
        dshCommand = "npx"
        dshPrefix = ["-y", "@deepseek-ai/dsh"]
      }
      const add = run(dshCommand, [...dshPrefix, "plugin", "--profile", "install-check", "add", pluginRoot], { env: environment })
      if (add.missing) {
        await rm(dshHome, { recursive: true, force: true })
        record(harness, "skip", ["dsh CLI not found and npx unavailable"])
        continue
      }
      if (!add.ok) {
        await rm(dshHome, { recursive: true, force: true })
        record(harness, "fail", [`profile add failed: ${firstLine(add.stderr || add.stdout)}`])
        continue
      }
      const dump = run(dshCommand, [...dshPrefix, "--profile", "install-check", "--dump-config"], { env: environment })
      const details = []
      let status = "pass"
      if (!dump.ok || !dump.stdout.includes(`- id: ${pluginName}`)) {
        status = "fail"
        details.push("cordis insert entry missing from composed profile")
      } else {
        details.push("cordis bundle composed into isolated profile")
      }
      await rm(dshHome, { recursive: true, force: true })
      record(harness, status, details)
    }
  }

  const failed = results.filter((entry) => entry.status === "fail")
  const skipped = results.filter((entry) => entry.status === "skip")
  if (options.json) {
    console.log(JSON.stringify({
      project: root,
      plugin: pluginName,
      runtime,
      results,
      summary: {
        pass: results.length - failed.length - skipped.length,
        fail: failed.length,
        skip: skipped.length,
      },
    }, null, 2))
  } else {
    console.log(
      failed.length === 0
        ? `Install check passed for ${pluginName} (${results.length - skipped.length} verified, ${skipped.length} skipped)`
        : `Install check failed for ${pluginName}: ${failed.length} of ${results.length} harnesses failed`,
    )
  }

  return failed.length > 0 ? 1 : 0
}
