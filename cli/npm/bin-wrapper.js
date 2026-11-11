#!/usr/bin/env node
"use strict";
// Shims the platform-specific binary package (crussty-<platform>), installed
// via optionalDependencies. No scripts are used — npm picks the right package
// based on the os/cpu fields in each platform package.

const { spawnSync } = require("child_process");
const path = require("path");

const platformPkgs = {
  "linux-x64": "crussty-linux-x64",
  "linux-arm64": "crussty-linux-arm64",
  "darwin-arm64": "crussty-darwin-arm64",
  "win32-x64": "crussty-windows-x64",
};
const pkgName = platformPkgs[`${process.platform}-${process.arch}`];
const binName = process.platform === "win32" ? "crussty.exe" : "crussty";

let bin;
if (pkgName) {
  try {
    bin = require.resolve(`${pkgName}/bin/${binName}`, { paths: [__dirname] });
  } catch {}
}
if (!bin || !require("fs").existsSync(bin)) {
  console.error(
    `crussty: no binary found for ${process.platform}-${process.arch}` +
      (pkgName ? ` (expected from ${pkgName})` : " — unsupported platform")
  );
  console.error("reinstall crussty or install the matching platform package manually");
  process.exit(1);
}

const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
process.exit(r.status === null ? 1 : r.status);
