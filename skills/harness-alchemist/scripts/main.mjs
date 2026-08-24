#!/usr/bin/env node
// Harness Alchemist shared-skill entrypoint (Node/Bun twin of main.py).
// Reads one JSON payload from stdin and writes one JSON result to stdout.
// See references/compatibility.md for the full contract.

const chunks = []
for await (const chunk of process.stdin) chunks.push(chunk)

let request
try {
  request = JSON.parse(Buffer.concat(chunks).toString("utf8"))
} catch (error) {
  console.error(`Invalid JSON payload: ${error.message}`)
  process.exit(1)
}

if (typeof request !== "object" || request === null || Array.isArray(request)) {
  console.error("Payload must be a JSON object")
  process.exit(1)
}

process.stdout.write(`${JSON.stringify({ ok: true, plugin: "harness-alchemist", echo: request })}\n`)
