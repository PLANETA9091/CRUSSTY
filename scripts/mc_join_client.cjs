#!/usr/bin/env node
"use strict";

const minecraft = require("minecraft-protocol");

const host = process.argv[2] || "127.0.0.1";
const port = Number(process.argv[3] || "25565");
const username = process.argv[4] || "CodexJoinProbe";
const timeoutMs = Number(process.argv[5] || "30000");

let loginSeen = false;
let stableSeen = false;
let clientEndRequested = false;
let ended = false;

function finish(code, message) {
  if (ended) {
    return;
  }
  ended = true;
  if (message) {
    const stream = code === 0 ? process.stdout : process.stderr;
    stream.write(`${message}\n`);
  }
  process.exit(code);
}

const client = minecraft.createClient({
  host,
  port,
  username,
  auth: "offline",
  version: false,
  hideErrors: false,
});

const timer = setTimeout(() => {
  try {
    client.end("timeout");
  } catch {
    // ignored
  }
  finish(1, `join_client=timeout username=${username} host=${host} port=${port}`);
}, timeoutMs);

client.once("login", () => {
  loginSeen = true;
  process.stdout.write(`join_client=login username=${username} host=${host} port=${port}\n`);
  setTimeout(() => {
    stableSeen = true;
    try {
      clientEndRequested = true;
      client.end("join-check-complete");
    } catch {
      // ignored
    }
  }, 5000);
});

client.on("kick_disconnect", packet => {
  clearTimeout(timer);
  if (stableSeen && clientEndRequested) {
    finish(0, `join_client=kick_after_stable username=${username} reason=${JSON.stringify(packet.reason)}`);
    return;
  }
  finish(1, `join_client=kick username=${username} reason=${JSON.stringify(packet.reason)}`);
});

client.on("error", error => {
  clearTimeout(timer);
  finish(1, `join_client=error username=${username} error=${error && error.stack ? error.stack : error}`);
});

client.on("end", reason => {
  clearTimeout(timer);
  if (stableSeen && clientEndRequested) {
    finish(0, `join_client=end username=${username} reason=${reason ?? ""}`);
  } else if (loginSeen) {
    finish(1, `join_client=end_before_stable username=${username} reason=${reason ?? ""}`);
  } else {
    finish(1, `join_client=end_before_login username=${username} reason=${reason ?? ""}`);
  }
});
