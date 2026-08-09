#!/usr/bin/env node
"use strict";
// Crussty CLI installer: downloads the platform binary from GitHub releases.

const https = require("https");
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const VERSION = process.env.CRUSSTY_VERSION || "v2.0.0";
const BASE = `https://github.com/PLANETA9091/CRUSSTY/releases/download/${VERSION}`;

const platforms = {
  linux: { x64: "linux-x64", arm64: "linux-arm64" },
  darwin: { x64: "macos-x64", arm64: "macos-arm64" },
  win32: { x64: "win-x64" },
};

function tag() {
  const os = process.platform;
  const arch = process.arch === "x64" ? "x64" : process.arch === "arm64" ? "arm64" : process.arch;
  const p = platforms[os] && platforms[os][arch];
  if (!p) {
    console.error(`crussty: unsupported platform ${os}/${arch}`);
    process.exit(1);
  }
  return p;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const req = https.get(url, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        res.resume();
        download(res.headers.location, dest).then(resolve, reject);
        return;
      }
      if (res.statusCode !== 200) {
        reject(new Error(`HTTP ${res.statusCode} for ${url}`));
        return;
      }
      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on("finish", () => file.close(() => resolve()));
      file.on("error", reject);
    });
    req.on("error", reject);
  });
}

(async () => {
  const binDir = path.join(__dirname, "bin");
  fs.mkdirSync(binDir, { recursive: true });
  const dest = path.join(binDir, process.platform === "win32" ? "crussty.exe" : "crussty");
  if (fs.existsSync(dest)) {
    console.log("crussty: already installed");
    return;
  }
  const url = `${BASE}/crussty-${tag()}`;
  console.log(`crussty: downloading ${url}`);
  await download(url, dest);
  if (process.platform !== "win32") {
    fs.chmodSync(dest, 0o755);
  }
  console.log(`crussty: installed ${dest}`);
})();