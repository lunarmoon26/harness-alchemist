{
  "name": {{PACKAGE_NAME_JSON}},
  "version": "0.1.0",
  "description": {{DESCRIPTION_JSON}},
  "type": "module",
  "main": "./dist/opencode.js",
  "types": "./dist/opencode.d.ts",
  "exports": {
    ".": {
      "types": "./dist/opencode.d.ts",
      "import": "./dist/opencode.js"
    },
    "./deepseek": {
      "types": "./dist/deepseek.d.ts",
      "import": "./dist/deepseek.js"
    },
    "./cordis.patch.yml": "./cordis.patch.yml"
  },
  "files": [
    "dist",
    "skills",
    "alchemy.json",
    "cordis.patch.yml",
    ".claude-plugin/plugin.json",
    ".codex-plugin/plugin.json",
    "plugin.json",
    "README.md",
    "LICENSE"
  ],
  "scripts": {
    "build": "tsc -p tsconfig.json",
    "check": "tsc -p tsconfig.json --noEmit",
    "test": "npm run build && node --test tests/*.test.mjs",
    "sync": "node .agents/skills/develop-{{NAME}}/scripts/sync-metadata.mjs",
    "validate": "node .agents/skills/develop-{{NAME}}/scripts/validate.mjs",
    "pack:check": "node .agents/skills/develop-{{NAME}}/scripts/check-package.mjs",
    "verify": "npm run check && npm test && npm run validate && npm run pack:check",
    "prepack": "npm run verify"
  },
  "engines": {
    "node": ">=22.20.0"
  },
  "peerDependencies": {
    "@deepseek-ai/cordis": "^4.0.1",
    "@opencode-ai/plugin": "^1.18.21"
  },
  "peerDependenciesMeta": {
    "@deepseek-ai/cordis": {
      "optional": true
    },
    "@opencode-ai/plugin": {
      "optional": true
    }
  },
  "devDependencies": {
    "@deepseek-ai/cordis": "^4.0.1",
    "@opencode-ai/plugin": "^1.18.21",
    "@types/node": "^26.2.0",
    "typescript": "^5.9.3"
  },
  "author": {{AUTHOR_JSON}},
  "license": {{LICENSE_JSON}},
  "repository": {
    "type": "git",
    "url": {{REPOSITORY_URL_JSON}}
  },
  "dsh": {
    "bundle": {
      "patch": "./cordis.patch.yml"
    }
  }
}
