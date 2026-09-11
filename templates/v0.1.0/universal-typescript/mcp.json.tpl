{
  "$schema": "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json",
  "mcpServers": {
    {{NAME_JSON}}: {
      "type": "stdio",
      "command": "npx",
      "args": [
        "-y",
        "@lunarmoon26/agent-skill-runtime@0.1.1",
        "mcp",
        "--root",
        "${PLUGIN_ROOT}",
        "--manifest",
        "skills/{{NAME}}/skill-runtime.json"
      ],
      "cwd": "${PLUGIN_ROOT}"
    }
  }
}
