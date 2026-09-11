name: Publish npm package

on:
  push:
    tags:
      - "v*"

permissions:
  contents: read

concurrency:
  group: npm-publish-${{ github.ref }}
  cancel-in-progress: false

jobs:
  publish:
    name: Publish package
    if: github.event.created && !github.event.deleted
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    permissions:
      contents: read
      id-token: write # Required for npm Trusted Publishing OIDC.
    steps:
      - uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6
        with:
          fetch-depth: 0
          persist-credentials: false
      - uses: actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38 # v6
        with:
          node-version: "24.19.0"
          registry-url: https://registry.npmjs.org
          package-manager-cache: false
      - uses: oven-sh/setup-bun@0c5077e51419868618aeaa5fe8019c62421857d6 # v2
        with:
          bun-version: 1.4.0
          no-cache: true

      - name: Verify release tag
        shell: bash
        run: |
          PACKAGE_VERSION="$(node -p "require('./package.json').version")"
          EXPECTED_TAG="v${PACKAGE_VERSION}"
          if [[ "$GITHUB_REF_NAME" != "$EXPECTED_TAG" ]]; then
            echo "::error::Tag '$GITHUB_REF_NAME' must match package version '$EXPECTED_TAG'"
            exit 1
          fi

          git fetch --no-tags origin main:refs/remotes/origin/main
          TAG_COMMIT="$(git rev-list -n 1 "$GITHUB_REF_NAME")"
          if ! git merge-base --is-ancestor "$TAG_COMMIT" refs/remotes/origin/main; then
            echo "::error::Release tag '$GITHUB_REF_NAME' must point to a commit contained in main"
            exit 1
          fi

      - run: npm ci
      - name: Verify synchronized metadata
        run: |
          npm run sync
          git diff --exit-code
      - run: npm run verify
      - name: Verify npm package contents
        run: npm pack --dry-run
      - name: Select npm dist-tag
        shell: bash
        run: |
          VERSION="$(node -p "require('./package.json').version")"
          if [[ "$VERSION" == *-* ]]; then
            echo "NPM_DIST_TAG=next" >> "$GITHUB_ENV"
          else
            echo "NPM_DIST_TAG=latest" >> "$GITHUB_ENV"
          fi
      - name: Publish npm package
        run: npm publish --access public --tag "$NPM_DIST_TAG"
