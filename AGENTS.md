# Crussty repo conventions

## Releases (semver)

Versions follow SemVer with a `v` prefix. Two paths:

### Automatic (default)

Pushing to `master` with changes in `runtime/`, `launcher/`, `cplug-abi/`,
`cplug-sdk/`, `cplug-sdk-c/`, `cli/`, `modules/` or `scripts/` triggers
`.github/workflows/auto-release.yml`, which computes the next version from
the commits since the last `v[0-9]*` tag and publishes the release itself
(launcher.jar + libcrussty_runtime.so + run.sh + run.bat). The commit
message decides the bump:

- contains `BREAKING` or `major:` → **major** (v2.0.1 → v3.0.0)
- contains `feat:` (or `minor:`) → **minor** (v2.0.1 → v2.1.0)
- anything else (fix, docs, refactor, …) → **patch** (v2.0.1 → v2.0.2)

So a `fix:`/`docs:` commit automatically releases the next patch version.
Doc-only changes (paths outside the list above) do NOT trigger a release.

### Manual (for pinned tags like `v1.0.0-ce`)

```bash
git tag v2.0.1 && git push origin v2.0.1
```

The `v[0-9]*` tag triggers `.github/workflows/release.yml`; tags like
`foo`, `wip-*` or `cli-*` do NOT create a release (the latter only publishes
CLI binaries to npm). The automatic path can double-publish if a commit
contains both `BREAKING` and `feat:` — bump the version instead of
overwriting an existing tag.

## Other notes

- CLI binaries are published as npm packages (`publish-npm.yml`), not as
  GitHub release assets.
- Docs live in `book/` on `master`; the published site is built from the
  `gh-pages` branch (structure may differ from `book/src`).