---
name: {{NAME}}
description: {{SHARED_SKILL_DESCRIPTION_JSON}}
compatibility: Requires Node.js 22+ or Bun 1.2+ for scripts/main.mjs; optional Python 3.10+ for scripts/main.py.
metadata:
  plugin: {{NAME}}
---

# {{DISPLAY_NAME}}

Apply the {{DISPLAY_NAME}} workflow to the user's request.

## Workflow

1. Confirm the requested outcome and inspect the relevant project context.
2. Run the skill entrypoint to perform the smallest complete change:

   ```
   echo '{"request": "..."}' | scripts/main.mjs
   ```

   When Python 3.10+ is available, `scripts/main.py` behaves identically.
3. Interpret the JSON result; both entrypoints return `{"ok": true, ...}` on
   success and exit non-zero with a stderr diagnostic on failure.
4. Use harness-provided tools only when they are needed for the workflow.
5. Verify the result with the project's available checks.
6. Report the outcome and any unresolved external dependency.

See [the tool contract](references/tool-contract.md) before changing or adding
entrypoints, and keep the `.mjs` and `.py` twins behaviorally identical.

Replace this starter workflow with the plugin's domain-specific procedure as the capability develops.
