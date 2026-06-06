#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
LABEL_FOREIGN="native-gate-contamination-foreign-smoke-$$"
LABEL_FOREIGN_ALLOWANCE="native-gate-contamination-foreign-allowance-smoke-$$"
LABEL_CONTAMINATED="native-gate-contamination-claim-smoke-$$"
LABEL_ABORT_POLICY="native-gate-contamination-abort-policy-smoke-$$"
LABEL_NATIVE_GATE="native-gate-contamination-harness-smoke-$$"
foreign_pid=""

cleanup() {
  if [[ -n "$foreign_pid" ]]; then
    kill "$foreign_pid" 2>/dev/null || true
    wait "$foreign_pid" 2>/dev/null || true
  fi
  rm -rf "$TMP" \
    "$ROOT/runs/load-$LABEL_FOREIGN" \
    "$ROOT/runs/load-$LABEL_FOREIGN_ALLOWANCE" \
    "$ROOT/runs/load-$LABEL_CONTAMINATED" \
    "$ROOT/runs/load-$LABEL_ABORT_POLICY" \
    "$ROOT/runs/load-$LABEL_NATIVE_GATE" \
    "$ROOT/logs/load-$LABEL_NATIVE_GATE-bots" \
    "$ROOT/logs/load-$LABEL_NATIVE_GATE-jstacks"
  rm -f "$ROOT/logs/load-$LABEL_FOREIGN.log" \
    "$ROOT/logs/load-$LABEL_FOREIGN-bots.log" \
    "$ROOT/logs/load-$LABEL_FOREIGN_ALLOWANCE.log" \
    "$ROOT/logs/load-$LABEL_FOREIGN_ALLOWANCE-bots.log" \
    "$ROOT/logs/load-$LABEL_CONTAMINATED.log" \
    "$ROOT/logs/load-$LABEL_CONTAMINATED-bots.log" \
    "$ROOT/logs/load-$LABEL_ABORT_POLICY.log" \
    "$ROOT/logs/load-$LABEL_ABORT_POLICY-bots.log" \
    "$ROOT/logs/load-$LABEL_NATIVE_GATE.log" \
    "$ROOT/logs/load-$LABEL_NATIVE_GATE-bots.log" \
    "$ROOT/reports/load-$LABEL_FOREIGN-preflight.txt" \
    "$ROOT/reports/load-$LABEL_FOREIGN-resources.csv" \
    "$ROOT/reports/load-$LABEL_FOREIGN-summary.txt" \
    "$ROOT/reports/load-$LABEL_FOREIGN-status.json" \
    "$ROOT/reports/load-$LABEL_FOREIGN-gate.txt" \
    "$ROOT/reports/load-$LABEL_FOREIGN_ALLOWANCE-preflight.txt" \
    "$ROOT/reports/load-$LABEL_FOREIGN_ALLOWANCE-resources.csv" \
    "$ROOT/reports/load-$LABEL_FOREIGN_ALLOWANCE-summary.txt" \
    "$ROOT/reports/load-$LABEL_FOREIGN_ALLOWANCE-status.json" \
    "$ROOT/reports/load-$LABEL_FOREIGN_ALLOWANCE-gate.txt" \
    "$ROOT/reports/load-$LABEL_CONTAMINATED-preflight.txt" \
    "$ROOT/reports/load-$LABEL_CONTAMINATED-resources.csv" \
    "$ROOT/reports/load-$LABEL_CONTAMINATED-summary.txt" \
    "$ROOT/reports/load-$LABEL_CONTAMINATED-status.json" \
    "$ROOT/reports/load-$LABEL_CONTAMINATED-gate.txt" \
    "$ROOT/reports/load-$LABEL_ABORT_POLICY-preflight.txt" \
    "$ROOT/reports/load-$LABEL_ABORT_POLICY-resources.csv" \
    "$ROOT/reports/load-$LABEL_ABORT_POLICY-summary.txt" \
    "$ROOT/reports/load-$LABEL_ABORT_POLICY-status.json" \
    "$ROOT/reports/load-$LABEL_ABORT_POLICY-gate.txt" \
    "$ROOT/reports/load-$LABEL_NATIVE_GATE-preflight.txt" \
    "$ROOT/reports/load-$LABEL_NATIVE_GATE-resources.csv" \
    "$ROOT/reports/load-$LABEL_NATIVE_GATE-summary.txt" \
    "$ROOT/reports/load-$LABEL_NATIVE_GATE-status.json" \
    "$ROOT/reports/load-$LABEL_NATIVE_GATE-gate.txt" \
    "$ROOT/reports/load-$LABEL_NATIVE_GATE-harness-exit.txt"
}
trap cleanup EXIT

fake_launcher="$TMP/fake-launcher.sh"
cat > "$fake_launcher" <<'SH'
#!/usr/bin/env bash
printf 'fake launcher executed\n' > "$FAKE_LAUNCHER_MARKER"
exit 99
SH
chmod +x "$fake_launcher"

native_gate_server="$TMP/native-gate-server.py"
native_gate_launcher="$TMP/native-gate-launcher.sh"
cat > "$native_gate_server" <<'PY'
#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import socket
import sys
import threading


def read_varint(sock: socket.socket) -> int:
    value = 0
    shift = 0
    for _ in range(5):
        chunk = sock.recv(1)
        if not chunk:
            raise EOFError("socket closed while reading varint")
        byte = chunk[0]
        value |= (byte & 0x7F) << shift
        if not (byte & 0x80):
            return value
        shift += 7
    raise ValueError("varint too long")


def read_exact(sock: socket.socket, length: int) -> bytes:
    data = bytearray()
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk:
            raise EOFError("socket closed while reading packet payload")
        data.extend(chunk)
    return bytes(data)


def read_varint_bytes(data: bytes, offset: int) -> tuple[int, int]:
    value = 0
    shift = 0
    cursor = offset
    for _ in range(5):
        byte = data[cursor]
        cursor += 1
        value |= (byte & 0x7F) << shift
        if not (byte & 0x80):
            return value, cursor
        shift += 7
    raise ValueError("varint too long")


def write_varint(value: int) -> bytes:
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            byte |= 0x80
        out.append(byte)
        if not value:
            return bytes(out)


def packet(packet_id: int, payload: bytes = b"") -> bytes:
    body = write_varint(packet_id) + payload
    return write_varint(len(body)) + body


def read_port() -> int:
    for line in pathlib.Path("server.properties").read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("server-port="):
            return int(line.split("=", 1)[1])
    raise RuntimeError("server-port is missing")


def handle_client(conn: socket.socket) -> None:
    with conn:
        conn.settimeout(2.0)
        try:
            length = read_varint(conn)
            payload = read_exact(conn, length)
            packet_id, cursor = read_varint_bytes(payload, 0)
            if packet_id != 0:
                return
            _protocol, cursor = read_varint_bytes(payload, cursor)
            host_len, cursor = read_varint_bytes(payload, cursor)
            cursor += host_len + 2
            next_state, cursor = read_varint_bytes(payload, cursor)
            if next_state == 1:
                try:
                    status_length = read_varint(conn)
                    _ = read_exact(conn, status_length)
                except Exception:
                    pass
                status = {
                    "version": {"name": "fake-native-gate", "protocol": 773},
                    "players": {"max": 20, "online": 0, "sample": []},
                    "description": {"text": "native gate smoke"},
                }
                encoded = json.dumps(status, separators=(",", ":")).encode("utf-8")
                conn.sendall(packet(0, write_varint(len(encoded)) + encoded))
            elif next_state == 2:
                try:
                    login_length = read_varint(conn)
                    _ = read_exact(conn, login_length)
                except Exception:
                    pass
        except Exception:
            return


def main() -> int:
    port = read_port()
    stop = threading.Event()
    ready = threading.Event()

    def serve() -> None:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as server:
            server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            server.bind(("127.0.0.1", port))
            server.listen()
            server.settimeout(0.2)
            ready.set()
            while not stop.is_set():
                try:
                    conn, _ = server.accept()
                except socket.timeout:
                    continue
                except OSError:
                    break
                threading.Thread(target=handle_client, args=(conn,), daemon=True).start()

    threading.Thread(target=serve, daemon=True).start()
    if not ready.wait(5.0):
        print("server failed to bind", file=sys.stderr)
        return 1
    print('[optimized-runtime] native_lib_available=true native_dir=/tmp/native native_climate_rtree=true native_climate_rtree_hook_present=true native_area_map=true native_improved_noise=true native_normal_noise=true native_perlin_noise=true native_perlin_noise_generic=false native_perlin_noise_no_y_scale=true method_profiler_disabled=false', flush=True)
    print('[00:00:00 INFO]: Paper: Native Climate RTree unavailable', flush=True)
    print('[00:00:00 INFO]: Done (0.050s)! For help, type "help"', flush=True)
    for raw in sys.stdin:
        if raw.strip() == "stop":
            break
    stop.set()
    print('[00:00:01 INFO]: Stopping server', flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
PY
chmod +x "$native_gate_server"
cat > "$native_gate_launcher" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'fake native gate launcher executed\n' > "$FAKE_LAUNCHER_MARKER"
exec python3 "$FAKE_NATIVE_GATE_SERVER"
SH
chmod +x "$native_gate_launcher"

good_summary="$TMP/native-good-summary.txt"
bad_summary="$TMP/native-bad-summary.txt"
required_bad_summary="$TMP/native-required-bad-summary.txt"
cat > "$good_summary" <<'SUMMARY'
native_runtime_line=[optimized-runtime] native_lib_available=true native_dir=/tmp/native native_climate_rtree=true native_climate_rtree_hook_present=true native_area_map=true native_improved_noise=true native_normal_noise=true native_perlin_noise=true native_perlin_noise_generic=false native_perlin_noise_no_y_scale=true method_profiler_disabled=false
native_climate_rtree_loaded=true
native_area_map_loaded=true
native_improved_noise_loaded=true
native_normal_noise_loaded=true
native_perlin_noise_loaded=true
native_perlin_noise_generic_loaded=false
native_perlin_noise_no_y_scale_loaded=true
bot_log_tail:
SUMMARY

python3 "$ROOT/scripts/evaluate_load_native_gate.py" \
  --profile production-500 \
  "$good_summary"

python3 "$ROOT/scripts/evaluate_load_native_gate.py" \
  --profile production-500 \
  --require-native-modules climate_rtree,area_map,improved_noise,normal_noise,perlin_noise_no_y_scale \
  "$good_summary"

python3 "$ROOT/scripts/evaluate_load_native_gate.py" \
  --profile production-500 \
  --require-native-modules all \
  "$good_summary"

set +e
python3 "$ROOT/scripts/evaluate_load_native_gate.py" \
  --profile production-500 \
  --require-native-modules definitely_unknown_native_module \
  "$good_summary" > "$TMP/native-unknown.out" 2> "$TMP/native-unknown.err"
unknown_native_status=$?
set -e
if [[ "$unknown_native_status" -ne 1 ]]; then
  echo "expected unknown native module token to exit 1, got $unknown_native_status" >&2
  cat "$TMP/native-unknown.out" >&2
  cat "$TMP/native-unknown.err" >&2
  exit 1
fi
rg -q 'Unknown LOAD_TEST_REQUIRE_NATIVE_MODULES token: definitely_unknown_native_module' "$TMP/native-unknown.err"

sed \
  -e 's/^native_area_map_loaded=true$/native_area_map_loaded=false/' \
  "$good_summary" > "$required_bad_summary"
set +e
python3 "$ROOT/scripts/evaluate_load_native_gate.py" \
  --profile stress-mixed \
  --require-native-modules area_map \
  "$required_bad_summary" > "$TMP/native-required-bad.out" 2> "$TMP/native-required-bad.err"
required_native_status=$?
set -e
if [[ "$required_native_status" -ne 76 ]]; then
  echo "expected explicit native module gate to exit 76, got $required_native_status" >&2
  cat "$TMP/native-required-bad.out" >&2
  cat "$TMP/native-required-bad.err" >&2
  exit 1
fi
rg -q 'Native load gate failed; missing loaded modules: area_map' "$TMP/native-required-bad.err"

sed \
  -e 's/^native_climate_rtree_loaded=true$/native_climate_rtree_loaded=false/' \
  -e 's/^native_normal_noise_loaded=true$/native_normal_noise_loaded=false/' \
  "$good_summary" > "$bad_summary"
set +e
python3 "$ROOT/scripts/evaluate_load_native_gate.py" \
  --profile production-500 \
  "$bad_summary" > "$TMP/native-bad.out" 2> "$TMP/native-bad.err"
native_status=$?
set -e
if [[ "$native_status" -ne 76 ]]; then
  echo "expected native gate mismatch to exit 76, got $native_status" >&2
  cat "$TMP/native-bad.out" >&2
  cat "$TMP/native-bad.err" >&2
  exit 1
fi
rg -q 'Production native load gate failed; advertised native modules were not loaded: climate_rtree,normal_noise' "$TMP/native-bad.err"

native_gate_label="$LABEL_NATIVE_GATE"
native_gate_marker="$TMP/native-gate-launcher.executed"
set +e
MC_EULA_AGREE=true \
LOAD_TEST_GATE_PROFILE=strict-block \
LOAD_TEST_REQUIRE_NATIVE_MODULES=all \
LOAD_TEST_SCENARIO=movement \
LOAD_TEST_LABEL="$native_gate_label" \
LOAD_TEST_ALLOW_BUSY_HOST=true \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true \
LOAD_TEST_WAIT_FOR_HOST_READY=false \
LOAD_TEST_ABORT_ON_HOST_CONTENTION=false \
LOAD_TEST_MIN_HOST_IDLE_PERCENT=0 \
LOAD_TEST_MAX_LOAD_PER_CPU=999 \
LOAD_TEST_MAX_HOST_STEAL_PERCENT=100 \
LOAD_TEST_MAX_HOST_IOWAIT_PERCENT=100 \
LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS=5 \
BOT_COUNT=1 \
DURATION_SECONDS=1 \
BOT_RAMP_SECONDS=1 \
BOT_MAX_DURATION_MS=1500 \
LAUNCHER="$native_gate_launcher" \
FAKE_LAUNCHER_MARKER="$native_gate_marker" \
FAKE_NATIVE_GATE_SERVER="$native_gate_server" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/native-gate.out" 2> "$TMP/native-gate.err"
native_gate_status=$?
set -e
if [[ "$native_gate_status" -eq 0 || "$native_gate_status" -eq 76 ]]; then
  echo "expected run_load_test to reach the normal gate path and fail nonzero, got $native_gate_status" >&2
  cat "$TMP/native-gate.out" >&2
  cat "$TMP/native-gate.err" >&2
  exit 1
fi
test -e "$native_gate_marker"
test -f "$ROOT/reports/load-$native_gate_label-summary.txt"
test -f "$ROOT/reports/load-$native_gate_label-gate.txt"
rg -q '^native_runtime_line=\[optimized-runtime\] native_lib_available=true' "$ROOT/reports/load-$native_gate_label-summary.txt"
rg -q '^native_climate_rtree_loaded=false$' "$ROOT/reports/load-$native_gate_label-summary.txt"
rg -q '^native_gate_evaluation_exit=76$' "$ROOT/reports/load-$native_gate_label-summary.txt"
rg -q '^claim_eligible=false$' "$ROOT/reports/load-$native_gate_label-gate.txt"
rg -q '^gate_pass=false$' "$ROOT/reports/load-$native_gate_label-gate.txt"

bash -c 'exec -a server.jar sleep 120' &
foreign_pid=$!
for _ in $(seq 1 50); do
  if ps -p "$foreign_pid" -o args= | rg -q '^server\.jar\b'; then
    break
  fi
  sleep 0.1
done
if ! ps -p "$foreign_pid" -o args= | rg -q '^server\.jar\b'; then
  echo "synthetic foreign server.jar process did not start" >&2
  exit 1
fi

foreign_marker="$TMP/foreign-launcher.executed"
set +e
MC_EULA_AGREE=true \
LOAD_TEST_ALLOW_BUSY_HOST=false \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=false \
LOAD_TEST_MIN_HOST_IDLE_PERCENT=0 \
LOAD_TEST_MAX_LOAD_PER_CPU=999 \
LOAD_TEST_MAX_HOST_STEAL_PERCENT=100 \
LOAD_TEST_MAX_HOST_IOWAIT_PERCENT=100 \
LOAD_TEST_GATE_PROFILE=production-500 \
LOAD_TEST_LABEL="$LABEL_FOREIGN" \
LAUNCHER="$fake_launcher" \
FAKE_LAUNCHER_MARKER="$foreign_marker" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/foreign.out" 2> "$TMP/foreign.err"
foreign_status=$?
set -e
if [[ "$foreign_status" -ne 75 ]]; then
  echo "expected production foreign-process gate to exit 75, got $foreign_status" >&2
  cat "$TMP/foreign.out" >&2
  cat "$TMP/foreign.err" >&2
  exit 1
fi
test ! -e "$foreign_marker"
test -f "$ROOT/reports/load-$LABEL_FOREIGN-preflight.txt"
test -f "$ROOT/reports/load-$LABEL_FOREIGN-summary.txt"
test -f "$ROOT/reports/load-$LABEL_FOREIGN-gate.txt"
rg -q '^strict_foreign_process_gate_pass=false$' "$ROOT/reports/load-$LABEL_FOREIGN-preflight.txt"
rg -q '^strict_foreign_process_count=[1-9][0-9]*$' "$ROOT/reports/load-$LABEL_FOREIGN-preflight.txt"
rg -q 'foreign_process_pattern=server\.jar' "$ROOT/reports/load-$LABEL_FOREIGN-preflight.txt"
rg -q '^load_window_policy=prelaunch-abort$' "$ROOT/reports/load-$LABEL_FOREIGN-summary.txt"
rg -q '^early_abort_reason=host_contention_prelaunch_host_ready_foreign_process_elapsed_seconds=' "$ROOT/reports/load-$LABEL_FOREIGN-summary.txt"
rg -q '^claim_eligible=false$' "$ROOT/reports/load-$LABEL_FOREIGN-gate.txt"
rg -q '^gate_pass=false$' "$ROOT/reports/load-$LABEL_FOREIGN-gate.txt"
rg -q '^environment_invalid=true$' "$ROOT/reports/load-$LABEL_FOREIGN-gate.txt"
rg -q '^environment_invalid_kind=host_contention$' "$ROOT/reports/load-$LABEL_FOREIGN-gate.txt"
rg -q 'Foreign Minecraft/load-test process detected before benchmark start' "$TMP/foreign.err"

kill "$foreign_pid" 2>/dev/null || true
wait "$foreign_pid" 2>/dev/null || true
foreign_pid=""

foreign_allowance_marker="$TMP/foreign-allowance-launcher.executed"
set +e
MC_EULA_AGREE=true \
LOAD_TEST_ALLOW_BUSY_HOST=false \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true \
LOAD_TEST_GATE_PROFILE=production-500 \
LOAD_TEST_LABEL="$LABEL_FOREIGN_ALLOWANCE" \
LAUNCHER="$fake_launcher" \
FAKE_LAUNCHER_MARKER="$foreign_allowance_marker" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/foreign-allowance.out" 2> "$TMP/foreign-allowance.err"
foreign_allowance_status=$?
set -e
if [[ "$foreign_allowance_status" -ne 75 ]]; then
  echo "expected production foreign-process override to exit 75, got $foreign_allowance_status" >&2
  cat "$TMP/foreign-allowance.out" >&2
  cat "$TMP/foreign-allowance.err" >&2
  exit 1
fi
test ! -e "$foreign_allowance_marker"
test ! -e "$ROOT/reports/load-$LABEL_FOREIGN_ALLOWANCE-summary.txt"
test ! -e "$ROOT/reports/load-$LABEL_FOREIGN_ALLOWANCE-gate.txt"
rg -q 'LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true is not allowed with production claim profiles' "$TMP/foreign-allowance.err"
rg -q 'Use a non-production profile for explicitly contaminated/non-claim evidence.' "$TMP/foreign-allowance.err"
if rg -q 'claim_eligible=' "$TMP/foreign-allowance.out" "$TMP/foreign-allowance.err"; then
  echo "foreign-process override path reached claim gate output" >&2
  exit 1
fi

contaminated_marker="$TMP/contaminated-launcher.executed"
set +e
MC_EULA_AGREE=true \
LOAD_TEST_ALLOW_BUSY_HOST=true \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=false \
LOAD_TEST_GATE_PROFILE=production-500 \
LOAD_TEST_LABEL="$LABEL_CONTAMINATED" \
LAUNCHER="$fake_launcher" \
FAKE_LAUNCHER_MARKER="$contaminated_marker" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/contaminated.out" 2> "$TMP/contaminated.err"
contaminated_status=$?
set -e
if [[ "$contaminated_status" -ne 75 ]]; then
  echo "expected contaminated production profile to exit 75, got $contaminated_status" >&2
  cat "$TMP/contaminated.out" >&2
  cat "$TMP/contaminated.err" >&2
  exit 1
fi
test ! -e "$contaminated_marker"
test ! -e "$ROOT/reports/load-$LABEL_CONTAMINATED-summary.txt"
test ! -e "$ROOT/reports/load-$LABEL_CONTAMINATED-gate.txt"
rg -q 'LOAD_TEST_ALLOW_BUSY_HOST=true is not allowed with production claim profiles' "$TMP/contaminated.err"
rg -q 'Use a non-production profile for explicitly noisy/non-claim evidence.' "$TMP/contaminated.err"
if rg -q 'claim_eligible=' "$TMP/contaminated.out" "$TMP/contaminated.err"; then
  echo "contaminated production path reached claim gate output" >&2
  exit 1
fi

abort_policy_marker="$TMP/abort-policy-launcher.executed"
set +e
MC_EULA_AGREE=true \
LOAD_TEST_ALLOW_BUSY_HOST=false \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=false \
LOAD_TEST_ABORT_ON_HOST_CONTENTION=false \
LOAD_TEST_GATE_PROFILE=production-500 \
LOAD_TEST_LABEL="$LABEL_ABORT_POLICY" \
LAUNCHER="$fake_launcher" \
FAKE_LAUNCHER_MARKER="$abort_policy_marker" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/abort-policy.out" 2> "$TMP/abort-policy.err"
abort_policy_status=$?
set -e
if [[ "$abort_policy_status" -ne 75 ]]; then
  echo "expected production abort-policy override to exit 75, got $abort_policy_status" >&2
  cat "$TMP/abort-policy.out" >&2
  cat "$TMP/abort-policy.err" >&2
  exit 1
fi
test ! -e "$abort_policy_marker"
test ! -e "$ROOT/reports/load-$LABEL_ABORT_POLICY-summary.txt"
test ! -e "$ROOT/reports/load-$LABEL_ABORT_POLICY-gate.txt"
rg -q 'LOAD_TEST_ABORT_ON_HOST_CONTENTION=false is not allowed with production claim profiles' "$TMP/abort-policy.err"
rg -q 'Production claim profiles must abort contaminated runs on sustained host contention.' "$TMP/abort-policy.err"
if rg -q 'claim_eligible=' "$TMP/abort-policy.out" "$TMP/abort-policy.err"; then
  echo "abort-policy override path reached claim gate output" >&2
  exit 1
fi

echo "run_load_test_native_gate_contamination_smoke=PASS"
