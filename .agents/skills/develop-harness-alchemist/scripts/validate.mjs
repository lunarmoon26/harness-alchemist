#!/usr/bin/env node

import { join } from "node:path"
import { fileURLToPath } from "node:url"
import { spawnSync } from "node:child_process"

const root = fileURLToPath(new URL("../../../../", import.meta.url))
const result = spawnSync(process.execPath, [
  join(root, "bin/harness-alchemist.mjs"),
  "validate",
  ...process.argv.slice(2),
], { stdio: "inherit" })

if (result.error) throw result.error
process.exitCode = result.status ?? 1
