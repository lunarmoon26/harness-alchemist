#!/usr/bin/env node

import { readFile } from "node:fs/promises"

import { listTemplates, runCreate } from "../lib/create.mjs"
import { runValidate } from "../lib/validate.mjs"

const packageJson = JSON.parse(
  await readFile(new URL("../package.json", import.meta.url), "utf8"),
)

function usage() {
  return `Harness Alchemist ${packageJson.version}

Usage: harness-alchemist <command> [options]

Commands:
  create <directory>    Create a universal coding-agent plugin repository.
  validate [directory] Validate a generated repository.
  templates            List bundled canonical templates.
  version              Print the CLI version.
  help                 Show this help.

Aliases: init and new are aliases for create.`
}

const [command, ...args] = process.argv.slice(2)

if (!command || command === "help" || command === "--help" || command === "-h") {
  console.log(usage())
} else if (command === "version" || command === "--version" || command === "-v") {
  console.log(packageJson.version)
} else if (command === "templates") {
  console.log(listTemplates())
} else if (["create", "init", "new"].includes(command)) {
  process.exitCode = await runCreate(args)
} else if (command === "validate") {
  process.exitCode = await runValidate(args.length > 0 ? args : [process.cwd()])
} else {
  console.error(`Unknown command: ${command}\n`)
  console.error(usage())
  process.exitCode = 2
}
