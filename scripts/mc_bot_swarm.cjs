#!/usr/bin/env node
"use strict";

const fs = require("fs");
const { performance, monitorEventLoopDelay } = require("perf_hooks");
const v8 = require("v8");
const minecraft = require("minecraft-protocol");
const mcDataLoader = require("minecraft-data");

function readOption(name, fallback) {
  const prefix = `--${name}=`;
  const found = process.argv.find(arg => arg.startsWith(prefix));
  return found ? found.slice(prefix.length) : fallback;
}

function readNumberOption(name, fallback) {
  const value = Number(readOption(name, String(fallback)));
  return Number.isFinite(value) ? value : fallback;
}

function readIntegerOption(name, fallback) {
  return Math.trunc(readNumberOption(name, fallback));
}

function readBooleanOption(name, fallback) {
  const value = String(readOption(name, fallback ? "true" : "false")).toLowerCase();
  return !["false", "0", "no", "off"].includes(value);
}

function clampInteger(value, minimum, maximum) {
  return Math.min(maximum, Math.max(minimum, Math.trunc(value)));
}

function usernameFor(baseName, index) {
  return `${baseName}${String(index).padStart(3, "0")}`;
}

function gridPosition(index, count, columns, spacing, centerX, centerZ) {
  const rows = Math.max(1, Math.ceil(count / columns));
  const startX = centerX - Math.floor(((columns - 1) * spacing) / 2);
  const startZ = centerZ - Math.floor(((rows - 1) * spacing) / 2);
  const col = index % columns;
  const row = Math.floor(index / columns);
  return {
    x: startX + col * spacing,
    z: startZ + row * spacing,
    rows,
    cols: columns,
    row,
    col,
  };
}

function approach(current, target, maxDelta) {
  const delta = target - current;
  if (Math.abs(delta) <= maxDelta) {
    return target;
  }
  return current + Math.sign(delta) * maxDelta;
}

function makeCreativeSlot(itemId, count) {
  return {
    itemCount: count,
    itemId,
    addedComponentCount: 0,
    removedComponentCount: 0,
    components: [],
    removeComponents: [],
  };
}

const host = readOption("host", "127.0.0.1");
const port = Number(readOption("port", "25565"));
const count = Number(readOption("count", "1"));
const startIndex = Math.max(0, readIntegerOption("start-index", 0));
const totalCount = Math.max(count, readIntegerOption("total-count", count));
const durationMs = Number(readOption("duration-ms", "60000"));
const minRunAfterActionOpenMs = Math.max(0, readIntegerOption("min-run-after-action-open-ms", 0));
const maxDurationFallbackMs = minRunAfterActionOpenMs > 0 ? durationMs + minRunAfterActionOpenMs : durationMs;
const maxDurationMs = Math.max(durationMs, readIntegerOption("max-duration-ms", maxDurationFallbackMs));
const rampMs = Number(readOption("ramp-ms", String(Math.max(1000, count * 100))));
const rampShardCount = Math.max(1, readIntegerOption("ramp-shard-count", 1));
const rampShardIndex = clampInteger(readIntegerOption("ramp-shard-index", 0), 0, rampShardCount - 1);
const selfTestMode = process.argv.some(arg => arg === "--self-test" || arg.startsWith("--self-test="));
const moveIntervalMs = Number(readOption("move-interval-ms", "100"));
const blockActionIntervalMs = Math.max(1, Number(readOption("block-action-interval-ms", String(moveIntervalMs))));
const globalActionStartAfterMs = Math.max(0, Number(readOption("global-action-start-after-ms", "0")));
const sharedActionGateFile = readOption("shared-action-gate-file", "");
const mode = readOption("mode", "move");
const actionStartAfterMs = Number(
  readOption(
    "action-start-after-ms",
    mode === "block" ? "5000" : "30000"
  )
);
const positionSettleMs = Math.max(0, readIntegerOption("position-settle-ms", 1500));
const actionStartModeInput = readOption("action-start-mode", "timer").toLowerCase().replace(/_/g, "-");
const actionStartMode = ({
  timer: "timer",
  "all-ready": "all-ready",
  allready: "all-ready",
  "swarm-ready": "all-ready",
  "ready-count": "ready-count",
  "ready-fraction": "ready-fraction",
})[actionStartModeInput] || actionStartModeInput;
if (!["timer", "all-ready", "ready-count", "ready-fraction"].includes(actionStartMode)) {
  throw new Error(`Unknown action-start-mode: ${actionStartModeInput}`);
}
const actionReadySettleMs = Math.max(0, readIntegerOption("action-ready-settle-ms", positionSettleMs));
const actionReadyMinCount = Math.max(0, readIntegerOption("action-ready-min-count", count));
const actionReadyMinFraction = Math.max(0, Math.min(1, readNumberOption("action-ready-min-fraction", 1)));
const checkTimeoutIntervalMs = Math.max(1000, readIntegerOption("check-timeout-interval-ms", 60000));
const strictFailures = readBooleanOption("strict-failures", false);
const speedBlocksPerSecond = Number(readOption("speed", "48"));
const sendStationaryPositions = readOption("send-stationary-positions", "true") !== "false";
const stationaryPositionIntervalMs = Math.max(0, readIntegerOption("stationary-position-interval-ms", "1000"));
const dropChatPlayerInfo = readBooleanOption("drop-chat-player-info", true);
const baseName = readOption("name", "LoadBot");
const version = readOption("version", "1.21.10");
const hideProtocolErrors = readBooleanOption("hide-protocol-errors", true);
const altitude = Number(readOption("altitude", "160"));
const spread = Number(readOption("spread", "8"));
const waveAmplitude = Number(readOption("wave-amplitude", "0"));
const blockScenario = mode === "block";
const mixedGameplayScenario = mode === "mixed-gameplay";
const blockActionScenario = blockScenario || mixedGameplayScenario;
const blockItemName = readOption("block-item", "stone");
const blockArenaCenterX = readIntegerOption("block-arena-center-x", 0);
const blockArenaCenterZ = readIntegerOption("block-arena-center-z", 0);
const blockArenaTargetY = readIntegerOption("block-arena-target-y", altitude);
const blockArenaSpacing = Math.max(1, readIntegerOption("block-arena-spacing", 4));
const blockArenaColumns = Math.max(1, readIntegerOption("block-arena-columns", Math.max(1, Math.ceil(Math.sqrt(count)))));
const mixedActionIntervalMs = Math.max(1, readIntegerOption("mixed-action-interval-ms", String(Math.max(250, moveIntervalMs))));
const mixedCommandIntervalMs = Math.max(
  mixedActionIntervalMs,
  readIntegerOption("mixed-command-interval-ms", "10000")
);
const mixedUseEntityAttacks = readOption("mixed-use-entity-attacks", "false") === "true";
const dephaseActions = readBooleanOption("dephase-actions", true);
function defaultSchedulerTickMs(intervalMs) {
  const interval = Math.max(1, Math.trunc(intervalMs));
  if (!dephaseActions) {
    return interval;
  }
  return Math.max(5, Math.min(25, Math.max(1, Math.floor(interval / 10))));
}
const moveSchedulerTickMs = Math.max(1, readIntegerOption("move-scheduler-tick-ms", String(defaultSchedulerTickMs(moveIntervalMs))));
const blockSchedulerTickMs = Math.max(1, readIntegerOption("block-scheduler-tick-ms", String(defaultSchedulerTickMs(blockActionIntervalMs))));
const mixedSchedulerTickMs = Math.max(1, readIntegerOption("mixed-scheduler-tick-ms", String(defaultSchedulerTickMs(mixedActionIntervalMs))));
const blockMovementMode = readOption(
  "block-movement-mode",
  blockActionScenario ? "wait-for-teleport" : "walk"
).toLowerCase().replace(/_/g, "-");
if (!["walk", "wait-for-teleport"].includes(blockMovementMode)) {
  throw new Error(`Unknown block-movement-mode: ${blockMovementMode}`);
}
const actionReadyRequiresBlockArmed = readOption(
  "action-ready-requires-block-armed",
  mixedGameplayScenario ? "true" : "false"
) === "true";

const mcData = mcDataLoader(version);
const resolvedMinecraftVersion = mcData.version.minecraftVersion;
const resolvedProtocolVersion = mcData.version.version;
const nodeHeapSizeLimitMb = Math.round(v8.getHeapStatistics().heap_size_limit / 1048576);
const creativeItem = mcData.itemsByName[blockItemName] || mcData.itemsByName.stone;
if (!creativeItem) {
  throw new Error(`Unknown block item: ${blockItemName}`);
}
const creativeSlotItem = makeCreativeSlot(creativeItem.id, Math.max(1, creativeItem.stackSize || 64));

const flags = { onGround: false, hasHorizontalCollision: false };
const loadBotDroppedEvents = new Set([
  "player_info",
  "player_remove",
  "player_chat",
  "profileless_chat",
  "system_chat",
  "chat_message",
  "message_header",
  "hide_message",
]);
const clients = new Map();
let created = 0;
let connected = 0;
let playReady = 0;
let ended = 0;
let kicked = 0;
let errors = 0;
let sentPositions = 0;
let receivedChunks = 0;
let creativeSlotPackets = 0;
let blockPlacePackets = 0;
let blockDigPackets = 0;
let blockActionErrors = 0;
let actionGateReadySince = 0;
let actionGateOpenedAt = 0;
let actionGateSoftened = false;
let actionGateSoftenedReason = "";
let actionGateSoftenedRequiredCount = 0;
let actionGateSoftenedMissingCount = 0;
let actionSchedulesInitialized = false;
let sharedActionGateWaitLoggedAt = 0;
let sharedActionGateOpenLogged = false;
let sharedActionGateCache = {
  checkedAt: 0,
  open: !sharedActionGateFile,
  reason: sharedActionGateFile ? "shared-gate-pending" : "",
  snapshot: null,
};
let strictFailureLogged = false;
const mixedCounters = {
  actionTicks: 0,
  heldItemPackets: 0,
  armAnimationPackets: 0,
  playerInputPackets: 0,
  useItemPackets: 0,
  commandPackets: 0,
  blockPlacePackets: 0,
  blockDigPackets: 0,
  attackPackets: 0,
  actionErrors: 0,
};
let lastReport = Date.now();
let shuttingDown = false;

function log(line) {
  process.stdout.write(`${new Date().toISOString()} ${line}\n`);
}

function formatError(error) {
  return error && error.message ? error.message : error;
}

function describeDetail(detail) {
  const raw = typeof detail === "string" ? detail : JSON.stringify(detail);
  return String(raw ?? "").replace(/\s+/g, "_");
}

function noteStrictFailure(kind, username, detail) {
  if (!strictFailures || strictFailureLogged) {
    return;
  }
  strictFailureLogged = true;
  log(`swarm_strict_failure kind=${kind} username=${username} detail=${describeDetail(detail)}`);
}

function disposeBotState(state, closeReason = "") {
  if (state.disposed) {
    return;
  }
  state.disposed = true;
  clients.delete(state.username);
  if (state.client) {
    if (closeReason) {
      try {
        state.client.end(closeReason);
      } catch {
        // ignored
      }
    }
    try {
      state.client.removeAllListeners();
    } catch {
      // ignored
    }
  }
  state.client = null;
  state.lastTargetEntityId = null;
}

function markBotEnded(state, reason, closeReason = "") {
  if (!state.ended) {
    state.ended = true;
    ended++;
    log(`bot_end username=${state.username} reason=${reason ?? ""}`);
  }
  disposeBotState(state, closeReason);
}

function exitCodeForCompletion() {
  if (strictFailures) {
    return errors > 0 || kicked > 0 ? 1 : 0;
  }
  return errors > count * 0.25 ? 1 : 0;
}

function sharedActionGateStatus(now = Date.now()) {
  if (!sharedActionGateFile) {
    return { open: true, reason: "", snapshot: null };
  }
  if (now - sharedActionGateCache.checkedAt < 250) {
    return sharedActionGateCache;
  }

  try {
    const raw = fs.readFileSync(sharedActionGateFile, "utf8").trim();
    const snapshot = raw ? JSON.parse(raw) : {};
    if (snapshot && snapshot.open === true) {
      sharedActionGateCache = { checkedAt: now, open: true, reason: "", snapshot };
    } else {
      sharedActionGateCache = { checkedAt: now, open: false, reason: "shared-gate-pending", snapshot };
    }
  } catch (error) {
    const reason = error && error.code === "ENOENT" ? "shared-gate-pending" : "shared-gate-read-error";
    sharedActionGateCache = { checkedAt: now, open: false, reason, snapshot: null };
  }
  return sharedActionGateCache;
}

function writePosition(state) {
  try {
    state.client.write("position_look", {
      x: state.x,
      y: state.y,
      z: state.z,
      yaw: state.yaw,
      pitch: state.pitch,
      flags,
    });
    sentPositions++;
    return true;
  } catch (error) {
    errors++;
    const message = formatError(error);
    log(`bot_move_error username=${state.username} error=${message}`);
    noteStrictFailure("move", state.username, message);
    return false;
  }
}

function writeStationaryPosition(state, now) {
  if (!sendStationaryPositions) {
    return false;
  }
  if (stationaryPositionIntervalMs > 0 && now < state.nextStationaryPositionAt) {
    return false;
  }
  const written = writePosition(state);
  state.nextStationaryPositionAt = now + stationaryPositionIntervalMs;
  return written;
}

function installLoadBotPacketFilters(client) {
  if (!dropChatPlayerInfo) {
    return;
  }

  const originalEmit = client.emit;
  client.emit = function filteredEmit(eventName, ...args) {
    if (loadBotDroppedEvents.has(eventName)) {
      return false;
    }
    return originalEmit.call(this, eventName, ...args);
  };
}

function spreadOffsetMs(index, periodMs, multiplier) {
  const period = Math.max(1, Math.trunc(periodMs));
  if (!dephaseActions || period <= 1 || totalCount <= 1) {
    return 0;
  }
  const slot = ((index * multiplier) % totalCount + totalCount) % totalCount;
  return Math.floor((slot * period) / totalCount);
}

function computeRampLaunchDelayMs(localIndex, localCount, rampDurationMs, totalSlots, shardIndex, shardCount) {
  const effectiveLocalCount = Math.max(1, localCount);
  const effectiveShardCount = Math.max(1, shardCount);
  if (effectiveShardCount <= 1) {
    return Math.floor((rampDurationMs * localIndex) / effectiveLocalCount);
  }
  const effectiveShardIndex = ((shardIndex % effectiveShardCount) + effectiveShardCount) % effectiveShardCount;
  const rampTotal = Math.max(1, Math.max(totalSlots, effectiveLocalCount * effectiveShardCount));
  const globalSlot = effectiveShardIndex + localIndex * effectiveShardCount;
  return Math.floor((rampDurationMs * globalSlot) / rampTotal);
}

function rampLaunchDelayMs(localIndex) {
  return computeRampLaunchDelayMs(localIndex, count, rampMs, totalCount, rampShardIndex, rampShardCount);
}

function scheduleStateActions(state, openedAt) {
  state.nextMoveAt = openedAt + state.moveOffsetMs;
  state.nextBlockActionAt = openedAt + state.blockActionOffsetMs;
  state.nextMixedActionAt = openedAt + state.mixedActionOffsetMs;
  state.mixedNextCommandAt = openedAt + state.mixedCommandOffsetMs;
}

function ensureActionSchedules(now = Date.now()) {
  if (!actionGateOpenedAt || actionSchedulesInitialized) {
    return;
  }
  for (const state of clients.values()) {
    scheduleStateActions(state, actionGateOpenedAt);
  }
  actionSchedulesInitialized = true;
  log(
    `swarm_action_schedules_initialized openedAfterMs=${actionGateOpenedAt - startTime} ` +
    `dephaseActions=${dephaseActions} moveSchedulerTickMs=${moveSchedulerTickMs} ` +
    `blockSchedulerTickMs=${blockSchedulerTickMs} mixedSchedulerTickMs=${mixedSchedulerTickMs}`
  );
}

function blockArenaPositionReady(state) {
  if (!state.blockTarget) {
    return false;
  }
  const targetX = state.blockTarget.x + 0.5;
  const targetY = blockArenaTargetY + 1;
  const targetZ = state.blockTarget.z + 0.5;
  return Math.abs(state.x - targetX) <= 1.5
    && Math.abs(state.y - targetY) <= 2.0
    && Math.abs(state.z - targetZ) <= 1.5;
}

function blockArenaActionReady(state) {
  return Boolean(state.blockArmedAt) && blockArenaPositionReady(state);
}

function moveTowardBlockTarget(state, now, step, options = {}) {
  if (!state.blockTarget) {
    return false;
  }
  const allowPreOpenMove = options.allowPreOpenMove === true;
  if (blockMovementMode === "wait-for-teleport" && !state.blockArmedAt && !allowPreOpenMove) {
    if (blockArenaPositionReady(state)) {
      state.blockArmedAt = now;
    }
    writeStationaryPosition(state, now);
    return Boolean(state.blockArmedAt);
  }
  const targetX = state.blockTarget.x + 0.5;
  const targetY = blockArenaTargetY + 1;
  const targetZ = state.blockTarget.z + 0.5;
  const dx = targetX - state.x;
  const dz = targetZ - state.z;
  const distance = Math.hypot(dx, dz);
  const horizontalStep = Math.max(0.05, Math.abs(step));
  if (distance <= 1.5) {
    if (!state.blockArmedAt) {
      state.blockArmedAt = now;
    }
  } else {
    const move = Math.min(horizontalStep, distance);
    state.x += (dx / distance) * move;
    state.z += (dz / distance) * move;
  }
  const verticalStep = Math.max(0.25, Math.min(0.75, horizontalStep * 0.1));
  state.y = approach(state.y, targetY, verticalStep);
  state.yaw = Math.atan2(targetZ - state.z, targetX - state.x) * 180 / Math.PI - 90;
  state.pitch = state.blockArmedAt ? 35 : 20;
  if (blockArenaPositionReady(state)) {
    if (!state.blockArmedAt) {
      state.blockArmedAt = now;
    }
  }
  writePosition(state);
  return Boolean(state.blockArmedAt);
}

function advanceBlockPreOpenArming(now, step) {
  if (!blockActionScenario) {
    return 0;
  }
  let advanced = 0;
  for (const state of clients.values()) {
    if (!state.ready || state.ended || !state.blockTarget || state.blockArmedAt) {
      continue;
    }
    if (now < state.nextMoveAt) {
      continue;
    }
    if (now < state.motionHoldUntil) {
      writeStationaryPosition(state, now);
      state.nextMoveAt = now + moveIntervalMs;
      continue;
    }
    if (moveTowardBlockTarget(state, now, step, { allowPreOpenMove: true })) {
      advanced++;
    }
    state.nextMoveAt = now + moveIntervalMs;
  }
  return advanced;
}

function currentSwarmCounts(now = Date.now(), includeBlockReportCounts = false) {
  let ready = 0;
  let active = 0;
  let settled = 0;
  let blockArmed = 0;
  let motionHoldUntilMax = 0;
  let blockPrimed = 0;
  let blockActionReady = 0;
  for (const state of clients.values()) {
    if (state.blockArmedAt) {
      blockArmed++;
    }
    if (includeBlockReportCounts) {
      if (state.blockReady && !state.ended) {
        blockPrimed++;
      }
      if (!state.ended && blockArenaActionReady(state)) {
        blockActionReady++;
      }
    }
    if (!state.ready) {
      continue;
    }
    ready++;
    if (state.ended) {
      continue;
    }
    active++;
    if (state.motionHoldUntil <= now) {
      settled++;
    }
    if (state.motionHoldUntil > motionHoldUntilMax) {
      motionHoldUntilMax = state.motionHoldUntil;
    }
  }
  return {
    created,
    connected,
    ready,
    active,
    settled,
    blockArmed,
    motionHoldUntilMax,
    ...(includeBlockReportCounts ? { blockPrimed, blockActionReady } : {}),
  };
}

function requiredActionReadyCount() {
  if (count <= 0) {
    return 0;
  }
  switch (actionStartMode) {
    case "all-ready":
      return actionGateSoftened ? actionGateSoftenedRequiredCount : count;
    case "ready-count":
      return clampInteger(actionReadyMinCount || count, 1, count);
    case "ready-fraction":
      return clampInteger(Math.ceil(count * actionReadyMinFraction), 1, count);
    default:
      return 0;
  }
}

function maybeSoftenActionGate(counts) {
  if (actionGateOpenedAt || actionStartMode !== "all-ready" || !mixedGameplayScenario) {
    return;
  }
  if (counts.created < count || counts.active <= 0 || counts.active >= count || counts.ended + counts.kicked <= 0) {
    return;
  }

  const liveRequired = clampInteger(counts.active, 1, count);
  if (actionGateSoftened && liveRequired >= actionGateSoftenedRequiredCount) {
    return;
  }

  actionGateSoftened = true;
  actionGateSoftenedReason = counts.kicked > 0 ? "early_disconnect_or_kick" : "early_disconnect";
  actionGateSoftenedRequiredCount = liveRequired;
  actionGateSoftenedMissingCount = count - liveRequired;
  log(
    `swarm_action_gate_softened mode=${actionStartMode} reason=${actionGateSoftenedReason} ` +
    `originalRequired=${count} liveRequired=${liveRequired} missing=${actionGateSoftenedMissingCount} ` +
    `created=${counts.created} connected=${counts.connected} ready=${counts.ready} active=${counts.active} ` +
    `settled=${counts.settled} blockArmed=${counts.blockArmed} ended=${ended} kicked=${kicked} errors=${errors}`
  );
}

function actionGateStatus(now = Date.now()) {
  const counts = currentSwarmCounts(now);
  maybeSoftenActionGate(counts);
  const requiredCount = requiredActionReadyCount();
  let reason = "";
  if (counts.created < requiredCount) {
    reason = `created ${counts.created}/${requiredCount}`;
  } else if (counts.ready < requiredCount) {
    reason = `ready ${counts.ready}/${requiredCount}`;
  } else if (counts.active < requiredCount) {
    reason = `active ${counts.active}/${requiredCount}`;
  } else if (counts.settled < requiredCount) {
    reason = `settled ${counts.settled}/${requiredCount}`;
  } else if (actionReadyRequiresBlockArmed && counts.blockArmed < requiredCount) {
    reason = `blockArmed ${counts.blockArmed}/${requiredCount}`;
  } else if (now - startTime < actionStartAfterMs) {
    reason = `elapsed ${now - startTime}/${actionStartAfterMs}`;
  }
  return {
    open: reason === "",
    reason,
    requiredCount,
    ...counts,
  };
}

function updateActionGate(now = Date.now()) {
  if (actionGateOpenedAt) {
    return true;
  }
  if (actionStartMode === "timer") {
    if (!actionGateOpenedAt && now - startTime >= actionStartAfterMs) {
      const status = actionGateStatus(now);
      actionGateOpenedAt = now;
      log(
        `swarm_action_gate_open mode=timer openedAfterMs=${now - startTime} ` +
        `readySinceMs=${actionGateReadySince ? now - actionGateReadySince : -1} ` +
        `created=${status.created} connected=${status.connected} ready=${status.ready} ` +
        `active=${status.active} settled=${status.settled} required=${status.requiredCount} ` +
        `settleMs=0 minDelayMs=${actionStartAfterMs} blockArmed=${status.blockArmed}`
      );
      ensureActionSchedules(now);
    }
    return now - startTime >= actionStartAfterMs;
  }

  const status = actionGateStatus(now);
  if (!status.open) {
    if (actionGateReadySince) {
      log(
        `swarm_action_gate_reset mode=${actionStartMode} reason=${status.reason} ` +
        `readySinceMs=${now - actionGateReadySince}`
      );
      actionGateReadySince = 0;
      actionSchedulesInitialized = false;
      sharedActionGateWaitLoggedAt = 0;
    }
    return false;
  }

  if (!actionGateReadySince) {
    actionGateReadySince = now;
    log(
      `swarm_action_gate_ready mode=${actionStartMode} created=${status.created} ` +
      `connected=${status.connected} ready=${status.ready} active=${status.active} ` +
      `settled=${status.settled} required=${status.requiredCount} settleMs=${actionReadySettleMs} ` +
      `blockArmed=${status.blockArmed}`
    );
    return false;
  }

  if (now - actionGateReadySince < actionReadySettleMs) {
    return false;
  }

  const sharedGate = sharedActionGateStatus(now);
  if (!sharedGate.open) {
    if (now - sharedActionGateWaitLoggedAt >= 5000) {
      sharedActionGateWaitLoggedAt = now;
      log(
        `swarm_action_gate_wait mode=${actionStartMode} reason=${sharedGate.reason} ` +
        `created=${status.created} connected=${status.connected} ready=${status.ready} ` +
        `active=${status.active} settled=${status.settled} required=${status.requiredCount} ` +
        `settleMs=${actionReadySettleMs} blockArmed=${status.blockArmed}`
      );
    }
    return false;
  }

  if (sharedActionGateFile && !sharedActionGateOpenLogged) {
    sharedActionGateOpenLogged = true;
    const snapshot = sharedGate.snapshot && sharedGate.snapshot.counts ? sharedGate.snapshot.counts : {};
    log(
      `swarm_action_gate_shared_observed mode=${actionStartMode} ` +
      `globalCreated=${snapshot.created ?? ""} globalConnected=${snapshot.connected ?? ""} ` +
      `globalReady=${snapshot.ready ?? ""} globalActive=${snapshot.active ?? ""} ` +
      `globalSettled=${snapshot.settled ?? ""} globalRequired=${sharedGate.snapshot ? sharedGate.snapshot.required ?? "" : ""}`
    );
  }

  if (!actionGateOpenedAt) {
    actionGateOpenedAt = now;
    log(
      `swarm_action_gate_open mode=${actionStartMode} openedAfterMs=${now - startTime} ` +
      `readySinceMs=${now - actionGateReadySince} created=${status.created} connected=${status.connected} ` +
      `ready=${status.ready} active=${status.active} settled=${status.settled} required=${status.requiredCount} ` +
      `settleMs=${actionReadySettleMs} minDelayMs=${actionStartAfterMs} blockArmed=${status.blockArmed}`
    );
    ensureActionSchedules(now);
  }
  return true;
}

function createBot(index) {
  const globalIndex = startIndex + index;
  const username = usernameFor(baseName, globalIndex);
  const angle = (Math.PI * 2 * globalIndex) / Math.max(1, totalCount);
  const blockTarget = blockActionScenario ? gridPosition(globalIndex, totalCount, blockArenaColumns, blockArenaSpacing, blockArenaCenterX, blockArenaCenterZ) : null;
  const moveOffsetMs = spreadOffsetMs(globalIndex, moveIntervalMs, 1);
  const blockActionOffsetMs = spreadOffsetMs(globalIndex, blockActionIntervalMs, 5);
  const mixedActionOffsetMs = spreadOffsetMs(globalIndex, mixedActionIntervalMs, 13);
  const mixedCommandOffsetMs = spreadOffsetMs(globalIndex, mixedCommandIntervalMs, 37);
  const state = {
    index: globalIndex,
    username,
    angle,
    x: Math.cos(angle) * spread,
    y: altitude,
    z: Math.sin(angle) * spread,
    yaw: angle * 180 / Math.PI,
    pitch: 0,
    ready: false,
    login: false,
    connected: false,
    ended: false,
    blockTarget,
    blockReady: false,
    blockArmedAt: 0,
    motionHoldUntil: 0,
    blockPhase: 0,
    blockSequence: 0,
    mixedPhase: 0,
    mixedLastCommandAt: 0,
    mixedNextCommandAt: startTime + mixedCommandOffsetMs,
    mixedSlot: index % 9,
    moveOffsetMs,
    blockActionOffsetMs,
    mixedActionOffsetMs,
    mixedCommandOffsetMs,
    nextMoveAt: startTime + moveOffsetMs,
    nextBlockActionAt: startTime + blockActionOffsetMs,
    nextMixedActionAt: startTime + mixedActionOffsetMs,
    nextStationaryPositionAt: 0,
    lastTargetEntityId: null,
  };
  if (actionGateOpenedAt) {
    scheduleStateActions(state, actionGateOpenedAt);
  }

  const client = minecraft.createClient({
    host,
    port,
    username,
    auth: "offline",
    version,
    keepAlive: true,
    checkTimeoutInterval: checkTimeoutIntervalMs,
    hideErrors: hideProtocolErrors,
  });
  installLoadBotPacketFilters(client);

  state.client = client;
  clients.set(username, state);
  created++;
  log(
    `bot_create username=${username} index=${globalIndex} elapsedMs=${Date.now() - startTime}` +
    (blockTarget ? ` blockTargetX=${blockTarget.x} blockTargetZ=${blockTarget.z}` : "")
  );

  client.once("login", packet => {
    const now = Date.now();
    state.login = true;
    state.connected = true;
    connected++;
    if (typeof packet.x === "number") state.x = packet.x;
    if (typeof packet.y === "number") state.y = packet.y;
    if (typeof packet.z === "number") state.z = packet.z;
    state.motionHoldUntil = Math.max(state.motionHoldUntil, now + positionSettleMs);
    log(`bot_login username=${username} elapsedMs=${now - startTime} x=${state.x.toFixed(2)} y=${state.y.toFixed(2)} z=${state.z.toFixed(2)}`);
  });

  client.once("playerJoin", () => {
    const now = Date.now();
    state.ready = true;
    playReady++;
    state.motionHoldUntil = Math.max(state.motionHoldUntil, now + positionSettleMs);
    try {
      client.write("player_loaded", {});
    } catch {
      // Older/newer protocol variants may not require this packet.
    }
    log(`bot_player_join username=${username} elapsedMs=${now - startTime}`);
  });

  client.on("position", packet => {
    const now = Date.now();
    if (typeof packet.teleportId === "number") {
      client.write("teleport_confirm", { teleportId: packet.teleportId });
    }
    if (typeof packet.x === "number") state.x = packet.x;
    if (typeof packet.y === "number") state.y = packet.y;
    if (typeof packet.z === "number") state.z = packet.z;
    if (typeof packet.yaw === "number") state.yaw = packet.yaw;
    if (typeof packet.pitch === "number") state.pitch = packet.pitch;
    state.motionHoldUntil = Math.max(state.motionHoldUntil, now + positionSettleMs);
    if (blockActionScenario && state.blockTarget && !state.blockArmedAt && blockArenaPositionReady(state)) {
      state.blockArmedAt = now;
    }
    try {
      client.write("player_loaded", {});
    } catch {
      // ignored
    }
  });

  client.on("chunk_batch_finished", packet => {
    try {
      client.write("chunk_batch_received", { chunksPerTick: Math.max(64, Number(packet.batchSize || 64)) });
    } catch {
      // ignored
    }
  });

  client.on("map_chunk", () => {
    receivedChunks++;
  });

  if (mixedUseEntityAttacks) {
    client.on("spawn_entity", packet => {
      if (typeof packet.entityId === "number") {
        state.lastTargetEntityId = packet.entityId;
      }
    });
  }

  client.on("ping", packet => {
    try {
      client.write("pong", { id: packet.id });
    } catch {
      // ignored
    }
  });

  client.on("kick_disconnect", packet => {
    kicked++;
    const reason = JSON.stringify(packet.reason);
    log(`bot_kick username=${username} reason=${reason}`);
    noteStrictFailure("kick", username, reason);
    markBotEnded(state, "kick_disconnect", "swarm-kick");
  });

  client.on("error", error => {
    errors++;
    const message = formatError(error);
    log(`bot_error username=${username} error=${message}`);
    noteStrictFailure("error", username, message);
    markBotEnded(state, "error", "swarm-error");
  });

  client.on("end", reason => {
    markBotEnded(state, reason);
  });
}

function moveBots() {
  const now = Date.now();
  const step = speedBlocksPerSecond * (moveIntervalMs / 1000);
  if (!updateActionGate(now)) {
    advanceBlockPreOpenArming(now, step);
    return;
  }
  ensureActionSchedules(now);
  if (speedBlocksPerSecond === 0 && waveAmplitude === 0 && !sendStationaryPositions && !blockActionScenario) {
    return;
  }
  for (const state of clients.values()) {
    if (!state.ready || state.ended) {
      continue;
    }
    if (now < state.nextMoveAt) {
      continue;
    }
    if (now < state.motionHoldUntil) {
      writeStationaryPosition(state, now);
      state.nextMoveAt = now + moveIntervalMs;
      continue;
    }
    if (mixedGameplayScenario && state.blockTarget) {
      const targetX = state.blockTarget.x + 0.5;
      const targetZ = state.blockTarget.z + 0.5;
      if (!state.blockArmedAt) {
        moveTowardBlockTarget(state, now, step);
        state.nextMoveAt = now + moveIntervalMs;
        continue;
      }
      const orbitAngle = state.angle + (now / 2500) + state.index * 0.17;
      const orbitRadius = Math.min(0.45, Math.max(0.05, step * 0.02));
      state.x = targetX + Math.cos(orbitAngle) * orbitRadius;
      state.z = targetZ + Math.sin(orbitAngle) * orbitRadius;
      const targetY = blockArenaTargetY + 1 + Math.abs(Math.sin((now / 900) + state.index)) * 0.35;
      const maxVerticalStep = Math.max(0.25, Math.min(0.5, step * 0.1));
      state.y = approach(state.y, targetY, maxVerticalStep);
      state.yaw = (orbitAngle * 180 / Math.PI + 90) % 360;
      state.pitch = 35;
    } else {
      const wave = Math.sin((now / 5000) + state.index) * waveAmplitude;
      const targetY = altitude + wave;
      const maxVerticalStep = Math.max(0.25, Math.min(0.5, step * 0.1));
      state.x += Math.cos(state.angle) * step;
      state.z += Math.sin(state.angle) * step;
      state.y = approach(state.y, targetY, maxVerticalStep);
      state.yaw = (state.angle * 180 / Math.PI + 90) % 360;
      state.pitch = 0;
    }
    writePosition(state);
    state.nextMoveAt = now + moveIntervalMs;
  }
}

function writeMixedPacket(state, packetName, payload, counterName) {
  try {
    state.client.write(packetName, payload);
    mixedCounters[counterName]++;
    return true;
  } catch (error) {
    errors++;
    mixedCounters.actionErrors++;
    const message = formatError(error);
    log(`bot_mixed_error username=${state.username} packet=${packetName} error=${message}`);
    noteStrictFailure("mixed", state.username, `packet=${packetName} error=${message}`);
    return false;
  }
}

function sendMixedCommand(state, now = Date.now()) {
  state.mixedLastCommandAt = now;
  state.mixedNextCommandAt = now + mixedCommandIntervalMs;
  try {
    if (typeof state.client._signedChat !== "function") {
      throw new Error("signed chat helper unavailable");
    }
    state.client._signedChat("/compatprobe");
    mixedCounters.commandPackets++;
    return true;
  } catch (error) {
    errors++;
    mixedCounters.actionErrors++;
    const message = formatError(error);
    log(`bot_mixed_error username=${state.username} packet=chat_command error=${message}`);
    noteStrictFailure("mixed", state.username, `packet=chat_command error=${message}`);
    return false;
  }
}

function primeCreativeSlot(state) {
  if (!blockActionScenario || !state.ready || state.ended) {
    return false;
  }
  if (state.blockReady) {
    return true;
  }
  try {
    state.client.write("set_creative_slot", { slot: 36, item: creativeSlotItem });
    state.client.write("held_item_slot", { slotId: 0 });
    state.blockReady = true;
    creativeSlotPackets++;
    return true;
  } catch (error) {
    errors++;
    blockActionErrors++;
    const message = formatError(error);
    log(`bot_block_error username=${state.username} action=prime error=${message}`);
    noteStrictFailure("block", state.username, `action=prime error=${message}`);
    return false;
  }
}

function blockBots() {
  const now = Date.now();
  const step = speedBlocksPerSecond * (blockActionIntervalMs / 1000);
  if (!updateActionGate(now)) {
    advanceBlockPreOpenArming(now, step);
    return;
  }
  ensureActionSchedules(now);
  for (const state of clients.values()) {
    if (!state.ready || state.ended || !state.blockTarget) {
      continue;
    }
    if (now < state.nextBlockActionAt) {
      continue;
    }
    if (!state.blockArmedAt || !blockArenaActionReady(state)) {
      moveTowardBlockTarget(state, now, step);
      state.nextBlockActionAt = now + blockActionIntervalMs;
      continue;
    }
    if (!primeCreativeSlot(state)) {
      state.nextBlockActionAt = now + blockActionIntervalMs;
      continue;
    }
    if (globalActionStartAfterMs > 0 && now - startTime < globalActionStartAfterMs) {
      state.nextBlockActionAt = now + blockActionIntervalMs;
      continue;
    }
    if (now - state.blockArmedAt < actionStartAfterMs) {
      state.nextBlockActionAt = now + blockActionIntervalMs;
      continue;
    }
    if (!blockArenaActionReady(state)) {
      state.nextBlockActionAt = now + blockActionIntervalMs;
      continue;
    }
    const support = {
      x: state.blockTarget.x,
      y: blockArenaTargetY - 1,
      z: state.blockTarget.z,
    };
    const target = {
      x: state.blockTarget.x,
      y: blockArenaTargetY,
      z: state.blockTarget.z,
    };
    const placeAction = state.blockPhase % 2 === 0;
    try {
      if (placeAction) {
        state.client.write("block_place", {
          hand: 0,
          location: support,
          direction: 1,
          cursorX: 0.5,
          cursorY: 0.99,
          cursorZ: 0.5,
          insideBlock: false,
          worldBorderHit: false,
          sequence: state.blockSequence++,
        });
        blockPlacePackets++;
      } else {
        state.client.write("block_dig", {
          status: 0,
          location: target,
          face: 1,
          sequence: state.blockSequence++,
        });
        blockDigPackets++;
      }
      state.blockPhase++;
    } catch (error) {
      errors++;
      blockActionErrors++;
      const message = formatError(error);
      const action = placeAction ? "place" : "dig";
      log(`bot_block_error username=${state.username} action=${action} error=${message}`);
      noteStrictFailure("block", state.username, `action=${action} error=${message}`);
    }
    state.nextBlockActionAt = now + blockActionIntervalMs;
  }
}

function mixedBlockAction(state) {
  if (!state.blockTarget || !state.blockArmedAt || !blockArenaActionReady(state)) {
    return false;
  }
  if (!primeCreativeSlot(state)) {
    return false;
  }
  const support = {
    x: state.blockTarget.x,
    y: blockArenaTargetY - 1,
    z: state.blockTarget.z,
  };
  const target = {
    x: state.blockTarget.x,
    y: blockArenaTargetY,
    z: state.blockTarget.z,
  };
  const placeAction = state.blockPhase % 2 === 0;
  try {
    if (placeAction) {
      state.client.write("block_place", {
        hand: 0,
        location: support,
        direction: 1,
        cursorX: 0.5,
        cursorY: 0.99,
        cursorZ: 0.5,
        insideBlock: false,
        worldBorderHit: false,
        sequence: state.blockSequence++,
      });
      blockPlacePackets++;
      mixedCounters.blockPlacePackets++;
    } else {
      state.client.write("block_dig", {
        status: 0,
        location: target,
        face: 1,
        sequence: state.blockSequence++,
      });
      blockDigPackets++;
      mixedCounters.blockDigPackets++;
    }
    state.blockPhase++;
    return true;
  } catch (error) {
    errors++;
    blockActionErrors++;
    mixedCounters.actionErrors++;
    const message = formatError(error);
    const packet = placeAction ? "block_place" : "block_dig";
    log(`bot_mixed_error username=${state.username} packet=${packet} error=${message}`);
    noteStrictFailure("mixed", state.username, `packet=${packet} error=${message}`);
    return false;
  }
}

function mixedGameplayBots() {
  const now = Date.now();
  if (!updateActionGate(now)) {
    return;
  }
  ensureActionSchedules(now);
  mixedCounters.actionTicks++;
  for (const state of clients.values()) {
    if (!state.ready || state.ended) {
      continue;
    }
    if (now < state.nextMixedActionAt) {
      continue;
    }
    const phase = state.mixedPhase++ % 8;
    switch (phase) {
      case 0:
        state.mixedSlot = (state.mixedSlot + 1) % 9;
        writeMixedPacket(state, "held_item_slot", { slotId: state.mixedSlot }, "heldItemPackets");
        break;
      case 1:
        writeMixedPacket(state, "arm_animation", { hand: 0 }, "armAnimationPackets");
        break;
      case 2:
        writeMixedPacket(state, "player_input", {
          inputs: {
            forward: true,
            backward: false,
            left: state.index % 2 === 0,
            right: state.index % 2 !== 0,
            jump: state.mixedPhase % 3 === 0,
            shift: false,
            sprint: true,
          },
        }, "playerInputPackets");
        break;
      case 3:
        writeMixedPacket(state, "use_item", {
          hand: 0,
          sequence: state.blockSequence++,
          rotation: { x: state.yaw || 0, y: state.pitch || 0 },
        }, "useItemPackets");
        break;
      case 4:
      case 5:
        mixedBlockAction(state);
        break;
      case 6: {
        const targetEntityId = state.lastTargetEntityId;
        if (mixedUseEntityAttacks && targetEntityId !== null) {
          writeMixedPacket(state, "use_entity", { target: targetEntityId, mouse: 1, sneaking: false }, "attackPackets");
        } else {
          writeMixedPacket(state, "arm_animation", { hand: 0 }, "armAnimationPackets");
        }
        break;
      }
      case 7:
        if (now >= state.mixedNextCommandAt) {
          sendMixedCommand(state, now);
        } else {
          writeMixedPacket(state, "arm_animation", { hand: 0 }, "armAnimationPackets");
        }
        break;
      default:
        break;
    }
    state.nextMixedActionAt = now + mixedActionIntervalMs;
  }
}

function report() {
  const now = Date.now();
  const elapsed = Math.max(1, (now - lastReport) / 1000);
  const swarmCounts = currentSwarmCounts(now, blockActionScenario);
  const loadGenTelemetry = sampleLoadGeneratorTelemetry();
  const active = swarmCounts.active;
  const actionGateIsOpen = Boolean(actionGateOpenedAt || (actionStartMode === "timer" && now - startTime >= actionStartAfterMs));
  const actionGateOpenedMs = actionGateOpenedAt
    ? actionGateOpenedAt - startTime
    : actionGateIsOpen ? actionStartAfterMs : -1;
  const baseFields = [
    "swarm_metrics",
    `mode=${mode}`,
    `created=${created}`,
    `connected=${connected}`,
    `ready=${playReady}`,
    `active=${active}`,
    `ended=${ended}`,
    `kicked=${kicked}`,
    `errors=${errors}`,
    `strictFailures=${strictFailures}`,
    `positions=${sentPositions}`,
    `positionsPerSec=${(sentPositions / Math.max(1, (now - startTime) / 1000)).toFixed(1)}`,
    `chunks=${receivedChunks}`,
    `chunksPerSec=${(receivedChunks / elapsed).toFixed(1)}`,
    `actionGateMode=${actionStartMode}`,
    `actionGate=${actionGateIsOpen ? "open" : "waiting"}`,
    `actionGateRequired=${requiredActionReadyCount()}`,
    `actionGateReady=${swarmCounts.ready}`,
    `actionGateActive=${swarmCounts.active}`,
    `actionGateSettled=${swarmCounts.settled}`,
    `actionGateOpenedMs=${actionGateOpenedMs}`,
    `actionGateReadySinceMs=${actionGateReadySince ? now - actionGateReadySince : -1}`,
    `loadGenLoopDelayP95Ms=${loadGenTelemetry.loopDelayP95Ms.toFixed(2)}`,
    `loadGenLoopDelayMaxMs=${loadGenTelemetry.loopDelayMaxMs.toFixed(2)}`,
    `loadGenLoopDelayMeanMs=${loadGenTelemetry.loopDelayMeanMs.toFixed(2)}`,
    `loadGenTimerDriftMaxMs=${loadGenTelemetry.timerDriftMaxMs.toFixed(2)}`,
    `loadGenTimerDriftAvgMs=${loadGenTelemetry.timerDriftAvgMs.toFixed(2)}`,
    `loadGenEluPct=${loadGenTelemetry.eluPct.toFixed(2)}`,
  ];

  if (blockActionScenario) {
    baseFields.push(
      `blockArmed=${swarmCounts.blockArmed}`,
      `blockPrimed=${swarmCounts.blockPrimed}`,
      `blockCreativeSlotPackets=${creativeSlotPackets}`,
      `blockPlacePackets=${blockPlacePackets}`,
      `blockDigPackets=${blockDigPackets}`,
      `blockActionErrors=${blockActionErrors}`,
      `blockActionsPerSec=${((blockPlacePackets + blockDigPackets) / Math.max(1, (now - startTime) / 1000)).toFixed(1)}`,
      `blockActionReady=${swarmCounts.blockActionReady}`
    );
  }

  if (mixedGameplayScenario) {
    const mixedActions = mixedCounters.heldItemPackets
      + mixedCounters.armAnimationPackets
      + mixedCounters.playerInputPackets
      + mixedCounters.useItemPackets
      + mixedCounters.commandPackets
      + mixedCounters.blockPlacePackets
      + mixedCounters.blockDigPackets
      + mixedCounters.attackPackets;
    baseFields.push(
      `mixedActionTicks=${mixedCounters.actionTicks}`,
      `mixedHeldItemPackets=${mixedCounters.heldItemPackets}`,
      `mixedArmAnimationPackets=${mixedCounters.armAnimationPackets}`,
      `mixedPlayerInputPackets=${mixedCounters.playerInputPackets}`,
      `mixedUseItemPackets=${mixedCounters.useItemPackets}`,
      `mixedCommandPackets=${mixedCounters.commandPackets}`,
      `mixedBlockPlacePackets=${mixedCounters.blockPlacePackets}`,
      `mixedBlockDigPackets=${mixedCounters.blockDigPackets}`,
      `mixedAttackPackets=${mixedCounters.attackPackets}`,
      `mixedActionErrors=${mixedCounters.actionErrors}`,
      `mixedActionsPerSec=${(mixedActions / Math.max(1, (now - startTime) / 1000)).toFixed(1)}`
    );
  }

  log(baseFields.join(" "));
  receivedChunks = 0;
  lastReport = now;
}

function shutdown(exitCode) {
  if (shuttingDown) {
    return;
  }
  shuttingDown = true;
  log(`swarm_shutdown exit=${exitCode} strictFailures=${strictFailures} errors=${errors} kicked=${kicked}`);
  for (const state of clients.values()) {
    try {
      state.client.end("swarm-shutdown");
    } catch {
      // ignored
    }
  }
  setTimeout(() => process.exit(exitCode), 3000).unref();
}

process.on("SIGINT", () => shutdown(130));
process.on("SIGTERM", () => shutdown(143));

const startTime = Date.now();
const loadGenLoopDelay = monitorEventLoopDelay({ resolution: 20 });
loadGenLoopDelay.enable();
let loadGenLastElu = performance.eventLoopUtilization();
const loadGenTimerDriftProbeMs = 1000;
let loadGenTimerDriftExpectedAt = startTime + loadGenTimerDriftProbeMs;
let loadGenTimerDriftMaxMs = 0;
let loadGenTimerDriftSumMs = 0;
let loadGenTimerDriftSamples = 0;

function loadGenMsFromNs(value) {
  const converted = value / 1_000_000;
  return Number.isFinite(converted) && converted > 0 ? converted : 0;
}

function sampleLoadGeneratorTelemetry() {
  const elu = performance.eventLoopUtilization(loadGenLastElu);
  loadGenLastElu = performance.eventLoopUtilization();
  const sample = {
    loopDelayP95Ms: loadGenMsFromNs(loadGenLoopDelay.percentile(95)),
    loopDelayMaxMs: loadGenMsFromNs(loadGenLoopDelay.max),
    loopDelayMeanMs: loadGenMsFromNs(loadGenLoopDelay.mean),
    timerDriftMaxMs: loadGenTimerDriftMaxMs,
    timerDriftAvgMs: loadGenTimerDriftSamples > 0
      ? loadGenTimerDriftSumMs / loadGenTimerDriftSamples
      : 0,
    eluPct: Number.isFinite(elu.utilization) ? Math.max(0, elu.utilization * 100) : 0,
  };

  loadGenLoopDelay.reset();
  loadGenTimerDriftMaxMs = 0;
  loadGenTimerDriftSumMs = 0;
  loadGenTimerDriftSamples = 0;
  return sample;
}

function runSelfTest() {
  const assert = (condition, message) => {
    if (!condition) {
      throw new Error(message);
    }
  };

  assert(
    computeRampLaunchDelayMs(2, 4, 1200, 4, 0, 1) === 600,
    "single-shard ramp delay should keep the original local i/count spacing"
  );
  assert(
    computeRampLaunchDelayMs(0, 50, 600000, 500, 3, 10) === 3600,
    "sharded ramp delay should include the shard slot offset"
  );
  assert(
    computeRampLaunchDelayMs(1, 50, 600000, 500, 3, 10) === 15600,
    "sharded ramp delay should advance by the shard-count global slot stride"
  );

  const fakeClient = {
    write() {
      // self-test only
    },
  };
  const now = Date.now();
  const blockedState = {
    blockTarget: { x: 0, z: 0 },
    blockArmedAt: 0,
    x: 10,
    y: altitude,
    z: 10,
    yaw: 0,
    pitch: 0,
    client: fakeClient,
  };
  const blockedX = blockedState.x;
  const blockedZ = blockedState.z;
  const blockedResult = moveTowardBlockTarget(blockedState, now, 1, { allowPreOpenMove: false });
  if (blockMovementMode === "wait-for-teleport") {
    assert(blockedResult === false, "wait-for-teleport should still block normal pre-armed movement");
    assert(blockedState.x === blockedX && blockedState.z === blockedZ, "wait-for-teleport moved without the pre-open override");
  }

  const movingState = {
    blockTarget: { x: 0, z: 0 },
    blockArmedAt: 0,
    x: 10,
    y: altitude,
    z: 10,
    yaw: 0,
    pitch: 0,
    client: fakeClient,
  };
  moveTowardBlockTarget(movingState, now, 1, { allowPreOpenMove: true });
  assert(movingState.x !== 10 || movingState.z !== 10, "pre-open override should move toward the block target");

  const stationaryState = {
    blockTarget: null,
    nextStationaryPositionAt: 0,
    x: 0,
    y: altitude,
    z: 0,
    yaw: 0,
    pitch: 0,
    client: fakeClient,
  };
  const firstStationaryWrite = writeStationaryPosition(stationaryState, now);
  const secondStationaryWrite = writeStationaryPosition(stationaryState, now + Math.max(1, stationaryPositionIntervalMs - 1));
  assert(firstStationaryWrite === true, "stationary position writes should pass through once");
  assert(
    stationaryPositionIntervalMs === 0 || secondStationaryWrite === false,
    "stationary position writes should be throttled"
  );

  const filterProbe = {
    emit(eventName) {
      return eventName;
    },
  };
  installLoadBotPacketFilters(filterProbe);
  assert(
    !dropChatPlayerInfo || filterProbe.emit("player_info") === false,
    "chat player info events should be filtered"
  );
  assert(filterProbe.emit("position") === "position", "non-chat events should keep flowing");

  const armedState = {
    blockTarget: { x: 0, z: 0 },
    blockArmedAt: 0,
    x: 0.6,
    y: blockArenaTargetY + 1,
    z: 0.6,
    yaw: 0,
    pitch: 0,
    client: fakeClient,
  };
  moveTowardBlockTarget(armedState, now, 1, { allowPreOpenMove: true });
  assert(Boolean(armedState.blockArmedAt), "pre-open override should be able to set blockArmedAt");

  log("mc_bot_swarm_self_test_ok");
}

if (selfTestMode) {
  runSelfTest();
  process.exit(0);
}

log([
  "swarm_start",
  `host=${host}`,
  `port=${port}`,
  `requestedVersion=${version}`,
  `resolvedMinecraftVersion=${resolvedMinecraftVersion}`,
  `resolvedProtocolVersion=${resolvedProtocolVersion}`,
  `nodeHeapSizeLimitMb=${nodeHeapSizeLimitMb}`,
  `nodeExecArgv=${process.execArgv.length ? process.execArgv.join(",") : "none"}`,
  `hideProtocolErrors=${hideProtocolErrors}`,
  `count=${count}`,
  `startIndex=${startIndex}`,
  `totalCount=${totalCount}`,
  `durationMs=${durationMs}`,
  `minRunAfterActionOpenMs=${minRunAfterActionOpenMs}`,
  `maxDurationMs=${maxDurationMs}`,
  `rampMs=${rampMs}`,
  `rampShardIndex=${rampShardIndex}`,
  `rampShardCount=${rampShardCount}`,
  `mode=${mode}`,
  `speed=${speedBlocksPerSecond}`,
  `moveIntervalMs=${moveIntervalMs}`,
  `dephaseActions=${dephaseActions}`,
  "loadGenTelemetry=true",
  "loadGenLoopDelayResolutionMs=20",
  `loadGenTimerDriftProbeMs=${loadGenTimerDriftProbeMs}`,
  `moveSchedulerTickMs=${moveSchedulerTickMs}`,
  blockActionScenario ? `blockSchedulerTickMs=${blockSchedulerTickMs}` : "",
  mixedGameplayScenario ? `mixedSchedulerTickMs=${mixedSchedulerTickMs}` : "",
  `sendStationaryPositions=${sendStationaryPositions}`,
  `stationaryPositionIntervalMs=${stationaryPositionIntervalMs}`,
  `dropChatPlayerInfo=${dropChatPlayerInfo}`,
  `positionSettleMs=${positionSettleMs}`,
  `actionStartMode=${actionStartMode}`,
  `actionReadyRequiresBlockArmed=${actionReadyRequiresBlockArmed}`,
  `actionReadySettleMs=${actionReadySettleMs}`,
  `actionReadyMinCount=${actionReadyMinCount}`,
  `actionReadyMinFraction=${actionReadyMinFraction}`,
  `checkTimeoutIntervalMs=${checkTimeoutIntervalMs}`,
  `strictFailures=${strictFailures}`,
  blockActionScenario ? `blockActionIntervalMs=${blockActionIntervalMs}` : "",
  mixedGameplayScenario ? `mixedActionIntervalMs=${mixedActionIntervalMs}` : "",
  mixedGameplayScenario ? `mixedCommandIntervalMs=${mixedCommandIntervalMs}` : "",
  mixedGameplayScenario ? `mixedUseEntityAttacks=${mixedUseEntityAttacks}` : "",
  blockActionScenario ? `globalActionStartAfterMs=${globalActionStartAfterMs}` : "",
  sharedActionGateFile ? `sharedActionGateFile=${sharedActionGateFile}` : "",
  `actionStartAfterMs=${actionStartAfterMs}`,
  `altitude=${altitude}`,
  `waveAmplitude=${waveAmplitude}`,
  `version=${version}`,
  blockActionScenario ? `blockItem=${blockItemName}` : "",
  blockActionScenario ? `blockMovementMode=${blockMovementMode}` : "",
  blockActionScenario ? `blockArenaCenterX=${blockArenaCenterX}` : "",
  blockActionScenario ? `blockArenaCenterZ=${blockArenaCenterZ}` : "",
  blockActionScenario ? `blockArenaTargetY=${blockArenaTargetY}` : "",
  blockActionScenario ? `blockArenaSpacing=${blockArenaSpacing}` : "",
  blockActionScenario ? `blockArenaColumns=${blockArenaColumns}` : "",
].filter(Boolean).join(" "));

for (let i = 0; i < count; i++) {
  const delay = rampLaunchDelayMs(i);
  setTimeout(() => createBot(i), delay).unref();
}

const timers = [];
timers.push(setInterval(blockScenario ? blockBots : moveBots, blockScenario ? blockSchedulerTickMs : moveSchedulerTickMs));
if (mixedGameplayScenario) {
  timers.push(setInterval(mixedGameplayBots, mixedSchedulerTickMs));
}
const loadGenLagProbeTimer = setInterval(() => {
  const now = Date.now();
  const driftMs = Math.max(0, now - loadGenTimerDriftExpectedAt);
  loadGenTimerDriftMaxMs = Math.max(loadGenTimerDriftMaxMs, driftMs);
  loadGenTimerDriftSumMs += driftMs;
  loadGenTimerDriftSamples++;
  loadGenTimerDriftExpectedAt += loadGenTimerDriftProbeMs;
  if (now - loadGenTimerDriftExpectedAt > loadGenTimerDriftProbeMs * 4) {
    loadGenTimerDriftExpectedAt = now + loadGenTimerDriftProbeMs;
  }
}, loadGenTimerDriftProbeMs);
loadGenLagProbeTimer.unref();
const reportTimer = setInterval(report, 5000);
function stopForDuration(reason) {
  for (const timer of timers) {
    clearInterval(timer);
  }
  clearInterval(loadGenLagProbeTimer);
  clearInterval(reportTimer);
  clearInterval(durationTimer);
  log(
    `swarm_duration_complete reason=${reason} elapsedMs=${Date.now() - startTime}` +
    ` actionGateOpenedMs=${actionGateOpenedAt ? actionGateOpenedAt - startTime : -1}`
  );
  report();
  shutdown(exitCodeForCompletion());
}
const durationTimer = setInterval(() => {
  const now = Date.now();
  const elapsedMs = now - startTime;
  if (minRunAfterActionOpenMs > 0) {
    if (actionGateOpenedAt && now - actionGateOpenedAt >= minRunAfterActionOpenMs) {
      stopForDuration("post-action-window");
    } else if (elapsedMs >= maxDurationMs) {
      stopForDuration("max-duration");
    }
    return;
  }
  if (elapsedMs >= durationMs) {
    stopForDuration("duration");
  }
}, 500);
durationTimer.unref();
