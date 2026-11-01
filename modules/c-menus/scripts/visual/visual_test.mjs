// c-menus visual test: renders the harness in headless Chromium, captures
// animation frames + hover/exit states, and compares against the committed
// golden images (pixelmatch, tolerant threshold — this is a visual smoke
// test for layout/animation, not a pixel-perfect renderer).
//
// Usage: node visual_test.mjs [--update-golden] [--base-url URL]
// Exits 0 when every frame matches its golden (or golden is updated).

import { chromium } from "playwright-core";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const GOLDEN = path.join(HERE, "golden");
const UPDATE = process.argv.includes("--update-golden");
const baseArg = process.argv.indexOf("--base-url");
// Serve <module>/ as the web root (menus/ + pack/ + scripts/visual).
const BASE = baseArg > -1 ? process.argv[baseArg + 1] : null;

function chromeCandidates() {
  const cands = [process.env.CHROME_PATH];
  try {
    const cache = path.join(os.homedir(), ".cache/ms-playwright");
    for (const d of fs.readdirSync(cache)) {
      if (d.startsWith("chromium")) {
        for (const sub of ["chrome-linux64/chrome", "chrome-linux/chrome"]) {
          cands.push(path.join(cache, d, sub));
        }
      }
    }
  } catch {}
  cands.push("/usr/bin/chromium", "/usr/bin/chromium-browser", "/usr/bin/google-chrome");
  return cands.filter(Boolean);
}

const FRAMES = [
  { name: "enter_t6", q: "?t=6" },
  { name: "enter_t12", q: "?t=12" },
  { name: "open_t40", q: "?t=40" },
  { name: "hover_jar", q: "?t=40&hover=jar" },
  { name: "hover_button", q: "?t=40&hover=btn_back" },
  { name: "exit_t5", q: "?t=5&phase=exit" },
];

function findChrome() {
  for (const c of chromeCandidates()) if (fs.existsSync(c)) return c;
  return null;
}

async function startServer() {
  // tiny static server: map / -> module root
  const root = path.join(HERE, "..", "..");
  const http = await import("node:http");
  const mimes = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".png": "image/png", ".mcmeta": "application/json" };
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, "http://localhost");
    let p = path.join(root, decodeURIComponent(url.pathname));
    if (!p.startsWith(root)) { res.writeHead(403); res.end(); return; }
    if (url.pathname.endsWith("/")) p = path.join(p, "index.html");
    fs.readFile(p, (err, data) => {
      if (err) { res.writeHead(404); res.end("nf"); return; }
      res.writeHead(200, { "content-type": mimes[path.extname(p)] || "application/octet-stream" });
      res.end(data);
    });
  });
  await new Promise((r) => server.listen(0, "127.0.0.1", r));
  const port = server.address().port;
  return { server, url: `http://127.0.0.1:${port}/scripts/visual/index.html` };
}

async function main() {
  fs.mkdirSync(GOLDEN, { recursive: true });
  let base = BASE;
  let server = null;
  if (!base) {
    ({ server, url: base } = await startServer());
  }
  const exe = findChrome();
  if (!exe) {
    console.error("no chromium found; set CHROME_PATH");
    process.exit(2);
  }
  const browser = await chromium.launch({
    executablePath: exe,
    args: ["--no-sandbox", "--disable-dev-shm-usage", "--force-device-scale-factor=1", "--hide-scrollbars"],
  });
  const page = await browser.newPage({ viewport: { width: 1000, height: 660 } });

  const results = [];
  for (const f of FRAMES) {
    await page.goto(base + f.q, { waitUntil: "networkidle" });
    await page.waitForTimeout(250);
    await page.evaluate(() => document.fonts ? document.fonts.ready : null);
    await page.waitForTimeout(100);
    const buf = await page.screenshot({ clip: { x: 20, y: 18, width: 960, height: 600 } });
    const shot = PNG.sync.read(buf);
    const goldenPath = path.join(GOLDEN, `${f.name}.png`);
    if (UPDATE || !fs.existsSync(goldenPath)) {
      fs.writeFileSync(goldenPath, buf);
      results.push({ name: f.name, status: "golden-updated" });
      continue;
    }
    const golden = PNG.sync.read(fs.readFileSync(goldenPath));
    if (golden.width !== shot.width || golden.height !== shot.height) {
      results.push({ name: f.name, status: "SIZE-MISMATCH", diff: -1 });
      continue;
    }
    const diff = pixelmatch(golden.data, shot.data, null, shot.width, shot.height, { threshold: 0.12 });
    const total = shot.width * shot.height;
    const pct = (diff / total) * 100;
    results.push({ name: f.name, status: pct <= 3.0 ? "ok" : "DIFF", diffPct: Number(pct.toFixed(3)) });
  }
  await browser.close();
  if (server) server.close();

  let failed = 0;
  for (const r of results) {
    console.log(`${r.status.padEnd(14)} ${r.name}${r.diffPct !== undefined ? `  diff=${r.diffPct}%` : ""}`);
    if (r.status === "DIFF" || r.status === "SIZE-MISMATCH") failed++;
  }
  if (failed) {
    console.error(`\n${failed} visual frame(s) diverged from golden`);
    process.exit(1);
  }
  console.log("\nall visual frames match golden");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
