{
  "$schema": "https://raw.githubusercontent.com/lunarmoon26/agent-skill-runtime/main/skill-runtime.schema.json",
  "manifestVersion": 1,
  "name": {{NAME_JSON}},
  "tools": [
    {
      "title": "Run {{DISPLAY_NAME}}",
      "name": {{TOOL_NAME_JSON}},
      "description": "Run the {{DISPLAY_NAME}} shared-skill workflow.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "request": {
            "type": "string",
            "description": "Requested {{DISPLAY_NAME}} workflow"
          }
        },
        "required": ["request"],
        "additionalProperties": false
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "ok": { "type": "boolean", "const": true },
          "plugin": { "type": "string", "const": {{NAME_JSON}} },
          "echo": {
            "type": "object",
            "properties": {
              "request": { "type": "string" }
            },
            "required": ["request"],
            "additionalProperties": false
          }
        },
        "required": ["ok", "plugin", "echo"],
        "additionalProperties": false
      },
      "entrypoint": {
        "engine": "node",
        "path": "scripts/main.mjs"
      },
      "annotations": {
        "readOnlyHint": true,
        "destructiveHint": false,
        "idempotentHint": true,
        "openWorldHint": false
      }
    }
  ]
}
