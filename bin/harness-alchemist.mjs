#!/usr/bin/env node

import { existsSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"
import { createRequire } from "node:module"
import { spawnSync } from "node:child_process"

const require = createRequire(import.meta.url)
const packageRoot = dirname(dirname(fileURLToPath(import.meta.url)))
const extension = process.platform === "win32" ? ".exe" : ""

const platformPackages = {
  darwin: {
    arm64: "@lunarmoon26/harness-alchemist-darwin-arm64",
    x64: "@lunarmoon26/harness-alchemist-darwin-x64",
  },
  linux: {
    arm64: "@lunarmoon26/harness-alchemist-linux-arm64",
    x64: "@lunarmoon26/harness-alchemist-linux-x64",
  },
  win32: {
    arm64: "@lunarmoon26/harness-alchemist-win32-arm64",
    x64: "@lunarmoon26/harness-alchemist-win32-x64",
  },
}

function installedBinary() {
  const packageName = platformPackages[process.platform]?.[process.arch]
  if (!packageName) return undefined
  try {
    return join(dirname(require.resolve(`${packageName}/package.json`)), `harness-alchemist${extension}`)
  } catch {
    return undefined
  }
}

function localBinary() {
  for (const profile of ["release", "debug"]) {
    const candidate = join(packageRoot, "target", profile, `harness-alchemist${extension}`)
    if (existsSync(candidate)) return candidate
  }
  return undefined
}

const binary = process.env.HARNESS_ALCHEMIST_BINARY || installedBinary() || localBinary()

if (!binary) {
  const supported = platformPackages[process.platform]?.[process.arch]
  const reason = supported
    ? `The optional package '${supported}' is not installed.`
    : `The platform '${process.platform}-${process.arch}' is not supported.`
  console.error(`Harness Alchemist could not locate its native binary. ${reason}`)
  console.error("Reinstall without --no-optional, or install a supported binary from GitHub Releases.")
  process.exit(1)
}

const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  env: {
    ...process.env,
    HARNESS_ALCHEMIST_PACKAGE_ROOT: packageRoot,
  },
})

if (result.error) {
  console.error(`Failed to start Harness Alchemist: ${result.error.message}`)
  process.exit(1)
}
if (result.signal) {
  process.kill(process.pid, result.signal)
} else {
  process.exit(result.status ?? 1)
}
