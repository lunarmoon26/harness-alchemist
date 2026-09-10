# Canonical Templates

Each published versioned directory is a complete, immutable scaffold contract.

- `v0.1.0/universal-typescript/` is the unreleased canonical universal plugin layout and may be updated in place.
- `v0.1.0/licenses/` contains license source text used during rendering.
- The native Rust generator maps template directory names, renders metadata tokens, injects `generated/validate.mjs` and the selected license, validates in staging, and atomically installs the result.

After a version is published, make incompatible changes in a new `v<version>/` directory and require callers to select that version explicitly. Do not silently redirect old template identifiers.
