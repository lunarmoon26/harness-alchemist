#!/usr/bin/env python3
"""Harness Alchemist shared-skill entrypoint (Python twin of main.mjs).

Reads one JSON payload from stdin and writes one JSON result to stdout.
See references/compatibility.md for the full contract.
"""

import json
import sys


def main() -> int:
    try:
        request = json.load(sys.stdin)
    except json.JSONDecodeError as error:
        print(f"Invalid JSON payload: {error}", file=sys.stderr)
        return 1

    if not isinstance(request, dict):
        print("Payload must be a JSON object", file=sys.stderr)
        return 1

    print(json.dumps({"ok": True, "plugin": "harness-alchemist", "echo": request}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
