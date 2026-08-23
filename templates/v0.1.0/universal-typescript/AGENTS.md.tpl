# Repository Guidance

This generated repository packages one shared Agent Skill with separate runtime
entrypoints for several coding-agent harnesses.

- `skills/{{NAME}}/SKILL.md` is the plugin's end-user workflow; keep its
  instructions portable and domain-specific.
- `.agents/skills/develop-{{NAME}}/` is the project-maintenance workflow. Read
  it before changing manifests, metadata, or runtime entrypoints.
- Keep OpenCode behavior in `src/opencode.ts`; keep Cordis behavior in
  `src/deepseek.ts` with named exports and no default export.
- Treat `package.json` as canonical metadata and run `npm run sync` after
  metadata changes.
- Keep manifest paths inside the repository. Do not add invented connector IDs,
  credentials, legal URLs, or presentation assets.
- Run `npm run verify` before reporting a change complete.
