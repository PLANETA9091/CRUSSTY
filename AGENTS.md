# Crussty repo conventions

## Releases (semver)

Versions follow SemVer with a `v` prefix: **`v2.0.1`** (patch), **`v2.1.0`**
(minor), **`v3.0.0`** (major), and so on. `v1.0.0-ce` and `v2.0.0` already exist.

To publish a release:

1. Bump the version where relevant (kernel compat version, module versions).
2. Create and push the tag — CI builds `launcher.jar` + `libcrussty_runtime.so`
   and attaches them (plus `run.sh`/`run.bat`) to the release automatically:

```bash
git tag v2.0.1 && git push origin v2.0.1
```

The `v[0-9]*` tag pattern triggers `.github/workflows/release.yml`; tags like
`foo`, `wip-*` or `cli-*` do NOT create a release (the latter only publishes
CLI binaries to npm). Do not overwrite an existing tag — bump the version
instead.

## Other notes

- CLI binaries are published as npm packages (`publish-npm.yml`), not as
  GitHub release assets.
- Docs live in `book/` on `master`; the published site is built from the
  `gh-pages` branch (structure may differ from `book/src`).