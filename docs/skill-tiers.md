# Skill Tiers

Status: Implemented

Harness Alchemist separates guidance by the reader's ownership boundary. A skill
loads only the instructions needed to act safely at that boundary.

| Tier | Location | Reader and responsibility |
| --- | --- | --- |
| 1: repository development | `.agents/skills/develop-harness-alchemist/` | Maintainers of this scaffolding CLI, its versioned templates, package, manifests, and runtime entrypoints. |
| 2: CLI use | `skills/harness-alchemist/` | People or agents using the published CLI to create and validate a universal plugin repository. |
| 3: generated-repository maintenance | `templates/v0.1.0/universal-typescript/.agents/skills/develop-template/` | Maintainers of a generated repository, who need the destination for shared skills, harness manifests, runtime code, and metadata. |

Tier 3 is copied to `.agents/skills/develop-<plugin-name>/` when a project is
created. It guides maintenance of that project; it does not describe how to
change Harness Alchemist or its canonical templates.

The generated `skills/<plugin-name>/SKILL.md` is a separate product surface: it
is a starter for the plugin's end-user workflow. Replace it with the
plugin-specific procedure rather than treating it as project-maintenance
guidance.

`templates/v0.1.0/` is the unreleased canonical scaffold and is the only
supported template version. It is updated in place until release.

## Acceptance

- The repository-maintainer skill retains the full self-hosting and
  compatibility workflow.
- The public skill teaches use of the CLI and routes generated-project changes
  to the generated repository's local maintenance skill.
- A default `v0.1.0` project contains a concise local maintenance skill that
  identifies where shared skills, harness manifests, runtime code, and metadata
  belong.
- Product skills ship `.mjs`/`.py` script twins under `skills/<name>/scripts/`
  with the tool-contract reference, and both runtime entrypoints delegate to
  them.
