// c-menus visual harness — canvas renderer that mirrors the Rust module's
// animation math (easing, enter/idle/hover/exit states, hud anchoring) and
// draws the menu the way the Minecraft client would (display entities with
// client-side interpolation, billboard panels, legacy-colored text).
//
// Query params (used by CI screenshots):
//   ?t=30          fixed tick (default: live 20 tps playback)
//   ?hover=jar     simulate hovering an element id
//   ?phase=exit    render the exit animation (t counts from close)
//   ?page=0        current page

"use strict";

const EASE = {
  linear: (t) => t,
  in_quad: (t) => t * t,
  out_quad: (t) => 1 - (1 - t) * (1 - t),
  in_out_quad: (t) => (t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2),
  in_cubic: (t) => t * t * t,
  out_cubic: (t) => 1 - Math.pow(1 - t, 3),
  in_out_cubic: (t) => (t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2),
  out_quart: (t) => 1 - Math.pow(1 - t, 4),
  out_quint: (t) => 1 - Math.pow(1 - t, 5),
  in_out_sine: (t) => -(Math.cos(Math.PI * t) - 1) / 2,
  out_back: (t) => { const c1 = 1.70158, c3 = c1 + 1; return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2); },
  out_elastic: (t) => {
    if (t <= 0) return 0; if (t >= 1) return 1;
    const c4 = (2 * Math.PI) / 3;
    return Math.pow(2, -10 * t) * Math.sin((t * 10 - 0.75) * c4) + 1;
  },
  out_bounce: (t) => {
    const n1 = 7.5625, d1 = 2.75;
    if (t < 1 / d1) return n1 * t * t;
    if (t < 2 / d1) { t -= 1.5 / d1; return n1 * t * t + 0.75; }
    if (t < 2.5 / d1) { t -= 2.25 / d1; return n1 * t * t + 0.9375; }
    t -= 2.625 / d1; return n1 * t * t + 0.984375;
  },
};

const clamp = (v, a, b) => Math.min(b, Math.max(a, v));
const loopWave = (ticks, period) => (period <= 0 ? 0 : Math.sin(2 * Math.PI * ticks / period));
const loopWave01 = (ticks, period) => (loopWave(ticks, period) + 1) / 2;
const segment = (elapsed, delay, dur) => (dur <= 0 ? 1 : (elapsed - delay < 0 ? null : Math.min(1, (elapsed - delay) / dur)));

// legacy "&" codes + #RRGGBB -> styled spans
function parseLegacy(s) {
  const out = [];
  let color = "FFFFFF", bold = false, italic = false, underline = false;
  const codes = { "0": "000000", "1": "0000AA", "2": "00AA00", "3": "00AAAA", "4": "AA0000", "5": "AA00AA", "6": "FFAA00", "7": "AAAAAA", "8": "555555", "9": "5555FF", a: "55FF55", b: "55FFFF", c: "FF5555", d: "FF55FF", e: "FFFF55", f: "FFFFFF" };
  const chars = [...s];
  let buf = "";
  const flush = () => { if (buf) { out.push({ text: buf, color, bold, italic, underline }); buf = ""; } };
  for (let i = 0; i < chars.length; i++) {
    if (chars[i] === "\u00a7" || chars[i] === "&") {
      const n = (chars[i + 1] || "").toLowerCase();
      if (/[0-9a-f]/.test(n)) { flush(); color = codes[n]; bold = italic = underline = false; i++; continue; }
      if (n === "l") { flush(); bold = true; i++; continue; }
      if (n === "o") { flush(); italic = true; i++; continue; }
      if (n === "n") { flush(); underline = true; i++; continue; }
      if (n === "r") { flush(); color = "FFFFFF"; bold = italic = underline = false; i++; continue; }
      if (n === "x" && chars[i + 2] === "\u00a7") {
        let hx = "";
        for (let k = 0; k < 6; k++) hx += chars[i + 3 + k * 2];
        flush(); color = hx; bold = italic = underline = false; i += 13; continue;
      }
    }
    buf += chars[i];
  }
  flush();
  return out;
}

// menu-space -> world (anchor + right*x + up*y - forward*z), yaw in degrees
function basisFromYaw(yawDeg) {
  const y = (yawDeg * Math.PI) / 180;
  return { fx: -Math.sin(y), fz: Math.cos(y), rx: -Math.cos(y), rz: -Math.sin(y) };
}
function menuToWorld(anchor, b, mx, my, mz) {
  return {
    x: anchor.x + b.rx * mx - b.fx * mz,
    y: anchor.y + my,
    z: anchor.z + b.rz * mx - b.fz * mz,
  };
}

// perspective projection; eye looks +Z, yaw 0
const FOV = 70;
let canvas, ctx, focal;

function project(w, eye) {
  const dx = w.x - eye.x, dy = w.y - eye.y, dz = w.z - eye.z;
  if (dz < 0.2) return null; // behind camera
  // MC view: facing +Z puts -X on the right (west is your right hand)
  return { sx: canvas.width / 2 - (dx * focal) / dz, sy: canvas.height / 2 - (dy * focal) / dz, depth: dz };
}

function enterState(el, tOpen) {
  let sm = 1, off = [0, 0, 0], rot = 0, bg = 1;
  const e = el.anim && el.anim.enter;
  if (e) {
    const p = segment(tOpen, e.delay || 0, e.dur || 10);
    if (true) {
      // during the delay window the element holds its "from" state (k=0)
      const k = p === null ? 0 : (EASE[e.ease] || EASE.out_cubic)(p);
      const f = e.from || {};
      off[0] += (f.offset ? f.offset[0] : 0) * (1 - k);
      off[1] += (f.offset ? f.offset[1] : 0) * (1 - k);
      off[2] += (f.offset ? f.offset[2] : 0) * (1 - k);
      const fscale = f.scale === undefined ? 1 : f.scale;
      const fbg = f.bg_alpha === undefined ? 1 : f.bg_alpha;
      sm *= fscale + (1 - fscale) * k;
      rot += (f.rot_y || 0) * (1 - k);
      bg *= fbg + (1 - fbg) * k;
    }
  }
  return { sm, off, rot, bg };
}

function exitState(el, tClose) {
  let sm = 1, off = [0, 0, 0], rot = 0, bg = 1, done = true;
  const e = el.anim && el.anim.exit;
  if (e) {
    const p = segment(tClose, e.delay || 0, e.dur || 10);
    done = tClose > (e.delay || 0) + (e.dur || 10) + 2;
    if (p !== null) {
      const k = (EASE[e.ease] || EASE.in_cubic)(p);
      const to = e.to || {};
      off[0] += (to.offset ? to.offset[0] : 0) * k;
      off[1] += (to.offset ? to.offset[1] : 0) * k;
      off[2] += (to.offset ? to.offset[2] : 0) * k;
      sm *= 1 - (1 - (to.scale === undefined ? 1 : to.scale)) * k;
      rot += (to.rot_y || 0) * k;
      bg *= 1 - (1 - (to.bg_alpha === undefined ? 1 : to.bg_alpha)) * k;
    }
  } else {
    done = tClose > 2;
  }
  return { sm, off, rot, bg, done };
}

function scaledBg(base, mult, add, hidden) {
  if (!base || hidden) return 0;
  let alpha = ((base >>> 24) & 0xff) / 255;
  alpha *= Math.max(0, mult);
  alpha += add;
  const a = Math.round(clamp(alpha, 0, 1) * 255);
  return ((base & 0x00ffffff) | (a << 24)) >>> 0;
}
const argbCss = (argb) => {
  if (!argb) return null;
  const a = ((argb >>> 24) & 0xff) / 255, r = (argb >>> 16) & 0xff, g = (argb >>> 8) & 0xff, b = argb & 0xff;
  return `rgba(${r},${g},${b},${a})`;
};

const state = {
  menu: null,
  anchor: { x: 0, y: 1.62, z: 3.8 },
  basis: basisFromYaw(0),
  page: 0,
  hovered: null,
  hoverT: {},
  bgCache: {},
  tOpen: 0,
  phase: "open",
};

function drawElement(el, st, eye, texs, tick) {
  const ent = st.enter, ex = st.exit;
  let scale = ent.sm * ex.sm;
  const offAll = [ent.off[0] + ex.off[0], ent.off[1] + ex.off[1], ent.off[2] + ex.off[2]];
  let rotY = ent.rot + ex.rot;
  let bgMult = ent.bg * ex.bg;

  const anchor = st.anchor, b = st.basis;
  const hidden = (el.page || 0) !== st.page;
  if (hidden) { scale *= 1e-4; bgMult = 0; }

  const idle = el.anim && el.anim.idle;
  const tOpen = st.tOpen;
  if (idle && !hidden) {
    if (idle.float) offAll[1] += idle.float.amp * loopWave(tOpen, idle.float.period || 80);
    if (idle.spin && idle.spin.period > 0) rotY += (tOpen / idle.spin.period) * 360;
    if (idle.pulse) scale *= 1 + idle.pulse.amp * loopWave01(tOpen, idle.pulse.period || 80);
  }

  let hoverAdd = 0;
  const h = el.anim && el.anim.hover;
  if (h) {
    const target = st.hovered === el.id ? 1 : 0;
    st.hoverT[el.id] = clamp((st.hoverT[el.id] || 0) + (target > (st.hoverT[el.id] || 0) ? 1 : -1) / (h.dur || 5), 0, 1);
    const k = (EASE[h.ease] || EASE.out_quint)(st.hoverT[el.id]);
    scale *= 1 - (1 - (h.scale === undefined ? 1 : h.scale)) * k;
    offAll[1] += (h.lift || 0) * k;
    hoverAdd = (h.bg_alpha_add || 0) * k;
  }

  const p = menuToWorld(anchor, b, el.pos[0] + offAll[0], el.pos[1] + offAll[1], el.pos[2] + offAll[2]);
  const sc = el.scale || [1, 1, 1];

  if (el.type === "text") {
    const argb = scaledBg(st.bgCache[el.id] || 0, bgMult, hoverAdd, hidden);
    drawText(el, p, sc, scale, rotY, argb, eye, st);
  } else {
    drawModel(el, p, sc, scale, rotY, eye, texs, tick);
  }
}

function drawText(el, p, sc, scale, rotY, argb, eye, st) {
  const c = project(p, eye);
  if (!c) return;
  const lines = el.text.split("\n");
  const lh = 0.115 * (sc[1] || 1) * scale; // line height in meters
  const fs = Math.max(4, ((lh * focal) / Math.max(0.4, c.depth)) * 0.9);
  ctx.save();
  const bgCss = argbCss(argb);
  if (bgCss) {
    const w = (el.line_width || 220) / 340;
    const h = lh * lines.length + 0.05;
    const tl = project(menuToWorld(st.anchor, st.basis, el.pos[0] - w / 2, el.pos[1] - 0.02, el.pos[2]), eye);
    const br = project(menuToWorld(st.anchor, st.basis, el.pos[0] + w / 2, el.pos[1] + h - 0.02, el.pos[2]), eye);
    if (tl && br) {
      ctx.fillStyle = bgCss;
      ctx.fillRect(tl.sx, tl.sy, br.sx - tl.sx, br.sy - tl.sy);
      ctx.fillStyle = "rgba(140,180,255,0.10)";
      ctx.fillRect(tl.sx, tl.sy, br.sx - tl.sx, Math.max(1, (br.sy - tl.sy) * 0.06));
    }
  }
  ctx.textBaseline = "top";
  const spans = parseLegacy(el.text.replace(/\n/g, "\u0000"));
  let y = c.sy;
  let lineBuf = [];
  const drawLine = (spans, y) => {
    const totalW = spans.reduce((a, s) => a + ctx.measureText(s.text).width, 0);
    let x = c.sx - (el.align === "center" ? totalW / 2 : el.align === "right" ? totalW : 0);
    for (const s of spans) {
      ctx.font = `${s.bold ? "bold " : ""}${s.italic ? "italic " : ""}${fs}px "Noto Sans", "DejaVu Sans", sans-serif`;
      ctx.fillStyle = "rgba(0,0,0,0.85)";
      ctx.fillText(s.text, x + fs * 0.06, y + fs * 0.06);
      ctx.fillStyle = "#" + s.color;
      ctx.fillText(s.text, x, y);
      if (s.underline) {
        const w2 = ctx.measureText(s.text).width;
        ctx.fillRect(x, y + fs * 0.95, w2, Math.max(1, fs * 0.07));
      }
      x += ctx.measureText(s.text).width;
    }
  };
  for (const s of spans) {
    const parts = s.text.split("\u0000");
    for (let i = 0; i < parts.length; i++) {
      if (i > 0) { drawLine(lineBuf, y); lineBuf = []; y += fs * 1.25; }
      if (parts[i]) lineBuf.push({ ...s, text: parts[i] });
    }
  }
  drawLine(lineBuf, y);
  ctx.restore();
}

function drawModel(el, p, sc, scale, rotY, eye, texs, tick) {
  if (el.model === "jar") { drawJar(el, p, sc, scale, eye, texs); return; }
  const w = (sc[0] || 1) * scale, h = (sc[1] || 1) * scale;
  const corners = [
    menuToWorld(state.anchor, state.basis, el.pos[0] - w / 2, el.pos[1] - h / 2, el.pos[2]),
    menuToWorld(state.anchor, state.basis, el.pos[0] + w / 2, el.pos[1] - h / 2, el.pos[2]),
    menuToWorld(state.anchor, state.basis, el.pos[0] + w / 2, el.pos[1] + h / 2, el.pos[2]),
    menuToWorld(state.anchor, state.basis, el.pos[0] - w / 2, el.pos[1] + h / 2, el.pos[2]),
  ].map((c) => project(c, eye));
  if (corners.some((c) => !c)) return;
  if (w < 0.005) return;

  const [a, bb, c2, d] = corners;
  const img = texs[el.model] || (el.type === "item" ? texs["item"] : null);
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(a.sx, a.sy); ctx.lineTo(bb.sx, bb.sy); ctx.lineTo(c2.sx, c2.sy); ctx.lineTo(d.sx, d.sy);
  ctx.closePath();
  ctx.clip();
  const minX = Math.min(a.sx, bb.sx, c2.sx, d.sx), minY = Math.min(a.sy, bb.sy, c2.sy, d.sy);
  const maxX = Math.max(a.sx, bb.sx, c2.sx, d.sx), maxY = Math.max(a.sy, bb.sy, c2.sy, d.sy);
  if (img && img.complete && img.naturalWidth) {
    let sy = 0, sh = img.naturalHeight;
    if (el.model === "panel_main" || el.model === "panel_dark" || el.model === "panel_soft") {
      const FR = 8, fh = img.naturalHeight / FR;
      const frame = Math.floor(tick / 3) % FR;
      sy = frame * fh; sh = fh;
    }
    ctx.drawImage(img, 0, sy, img.naturalWidth, sh, minX, minY, maxX - minX, maxY - minY);
  } else {
    ctx.fillStyle = "rgba(16,19,26,0.92)";
    ctx.fillRect(minX, minY, maxX - minX, maxY - minY);
  }
  ctx.restore();
  if (el.model === "cell_hover") {
    ctx.strokeStyle = "rgba(133,200,255,0.9)";
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(a.sx, a.sy); ctx.lineTo(bb.sx, bb.sy); ctx.lineTo(c2.sx, c2.sy); ctx.lineTo(d.sx, d.sy);
    ctx.closePath(); ctx.stroke();
  }
}

function drawJar(el, p, sc, scale, eye, texs) {
  const w = (sc[0] || 1) * scale;
  const hM = (sc[1] || 1) * scale;
  const cx = project(p, eye);
  if (!cx) return;
  const s = focal / Math.max(0.4, p.z - eye.z); // px per meter at this depth
  const glass = texs["jar_glass"], powder = texs["jar_powder"], lid = texs["jar_lid"];
  // model px: 0 = bottom, 16 = top; item anchor = center (px 8)
  const py = (m) => cx.sy + ((8 - m) / 16) * hM * s;
  const pw = (px16) => (px16 / 16) * w * s;
  const ph = (px16) => (px16 / 16) * hM * s;
  const drawTex = (img, cxp, yTop, wpx, hpx, alpha) => {
    if (img && img.complete && img.naturalWidth) {
      ctx.globalAlpha = alpha === undefined ? 1 : alpha;
      ctx.drawImage(img, cxp - wpx / 2, yTop, wpx, hpx);
      ctx.globalAlpha = 1;
    } else {
      ctx.fillStyle = "rgba(80,80,90,0.6)";
      ctx.fillRect(cxp - wpx / 2, yTop, wpx, hpx);
    }
  };
  const X = cx.sx;
  // glass body: px 0..12 (front face), powder 0.5..6.5, lid 12..15.5
  drawTex(glass, X, py(12), pw(12), ph(12), 0.62);
  drawTex(powder, X, py(6.5), pw(10), ph(6));
  drawTex(lid, X, py(15.5), pw(14), ph(3.5));
  ctx.strokeStyle = "rgba(210,235,255,0.5)";
  ctx.lineWidth = Math.max(1, s * 0.012);
  ctx.strokeRect(X - pw(12) / 2, py(12), pw(12), ph(12));
}

// ring/powder/cobalt_dust textures map by model name
const MODEL_TEX = {
  ring: "../../pack/assets/cmenus/textures/item/ring.png",
  powder: "../../pack/assets/cmenus/textures/item/powder.png",
  cobalt_dust: "../../pack/assets/cmenus/textures/item/cobalt_dust.png",
};

const TEX_FILES = {
  panel_main: "../../pack/assets/cmenus/textures/ui/panel_main.png",
  panel_dark: "../../pack/assets/cmenus/textures/ui/panel_dark.png",
  panel_soft: "../../pack/assets/cmenus/textures/ui/panel_soft.png",
  cell: "../../pack/assets/cmenus/textures/ui/cell.png",
  cell_hover: "../../pack/assets/cmenus/textures/ui/cell_hover.png",
  button: "../../pack/assets/cmenus/textures/ui/button.png",
  jar_glass: "../../pack/assets/cmenus/textures/item/jar_glass.png",
  jar_powder: "../../pack/assets/cmenus/textures/item/jar_powder.png",
  jar_lid: "../../pack/assets/cmenus/textures/item/jar_lid.png",
  ...MODEL_TEX,
};

function bgArgbFor(el) {
  const bg = el.bg || "";
  if (!bg) return 0;
  let h = bg.replace("#", "");
  if (h.length === 6) h = "ff" + h;
  return parseInt(h, 16) >>> 0;
}

async function boot() {
  canvas = document.getElementById("c");
  ctx = canvas.getContext("2d");
  focal = (canvas.height / 2) / Math.tan(((FOV / 2) * Math.PI) / 180);

  const q = new URLSearchParams(location.search);
  const fixedT = q.get("t") !== null ? parseInt(q.get("t"), 10) : null;
  state.hovered = q.get("hover");
  state.phase = q.get("phase") || "open";
  state.page = parseInt(q.get("page") || "0", 10);

  const resp = await fetch("../../menus/item_view.json");
  state.menu = await resp.json();
  for (const el of state.menu.elements) {
    state.bgCache[el.id] = bgArgbFor(el);
  }

  const texs = {};
  for (const [k, path] of Object.entries(TEX_FILES)) {
    const img = new Image(); img.src = path; texs[k] = img;
  }

  let tick = fixedT !== null ? fixedT : 0;
  const render = () => {
    drawScene(tick, texs);
    document.getElementById("meta").textContent =
      `tick ${tick}  hover=${state.hovered || "-"}  phase=${state.phase}`;
    if (fixedT === null) tick += 1;
    requestAnimationFrame(render);
  };
  requestAnimationFrame(render);
}

function drawScene(tick, texs) {
  const menu = state.menu;
  const eye = { x: 0, y: 1.62, z: 0 };

  const sky = ctx.createLinearGradient(0, 0, 0, canvas.height);
  sky.addColorStop(0, "#0a0e1c");
  sky.addColorStop(0.55, "#101726");
  sky.addColorStop(0.78, "#18202f");
  sky.addColorStop(1, "#0c1018");
  ctx.fillStyle = sky;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "rgba(255,255,255,0.55)";
  for (let i = 0; i < 40; i++) {
    const sx = ((i * 197) % 990) / 990 * canvas.width;
    const sy = ((i * 131) % 300) / 300 * canvas.height * 0.5;
    ctx.fillRect(sx, sy, 2, 2);
  }
  ctx.fillStyle = "#0d1410";
  ctx.fillRect(0, canvas.height * 0.72, canvas.width, canvas.height * 0.28);
  ctx.fillStyle = "#0a100c";
  for (let i = 0; i < 9; i++) {
    const tx = i * 120 + 30, th = 90 + ((i * 53) % 60);
    ctx.beginPath();
    ctx.moveTo(tx, canvas.height * 0.74);
    ctx.lineTo(tx + 34, canvas.height * 0.74 - th);
    ctx.lineTo(tx + 68, canvas.height * 0.74);
    ctx.closePath(); ctx.fill();
  }

  if (!menu) return;

  state.tOpen = tick;
  const tOpen = tick;
  const tClose = state.phase === "exit" ? tick : 0;

  // painter's algorithm: farther (smaller z) first
  const els = [...menu.elements].sort((a, b) => a.pos[2] - b.pos[2]);
  for (const el of els) {
    const ent = enterState(el, tOpen);
    const ex = exitState(el, tClose);
    drawElement(el, {
      enter: ent, exit: ex, anchor: state.anchor, basis: state.basis,
      page: state.page, hovered: state.hovered, hoverT: state.hoverT,
      bgCache: state.bgCache, tOpen,
    }, eye, texs, tick);
  }
}

window.addEventListener("DOMContentLoaded", boot);
