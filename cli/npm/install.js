#!/usr/bin/env node
"use strict";
// Crussty CLI installer: downloads the platform binary from GitHub releases.

const https = require("https");
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const VERSION = process.env.CRUSSTY_VERSION || "v2.0.0";
const BASE = `https://github.com/PLANETA9091/CRUSSTY/releases/download/${VERSION}`;

const assets = {
  "linux:x64": "crussty-x64",
  "linux:arm64": "crussty-arm64",
  "darwin:x64": "crussty-x64",
  "darwin:arm64": "crussty-arm64",
  "win32:x64": "crussty-x64.exe",
  "win32:arm64": "crussty-arm64.exe",
};

function asset() {
  const key = `${process.platform}:${process.arch}`;
  const a = assets[key];
  if (!a) {
    console.error(`crussty: unsupported platform ${key}`);
    process.exit(1);
  }
  return a;
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
  const url = `${BASE}/${asset()}`;
  console.log(`crussty: downloading ${url}`);
  await download(url, dest);
  if (process.platform !== "win32") {
    fs.chmodSync(dest, 0o755);
  }
  console.log(`crussty: installed ${dest}`);
})();