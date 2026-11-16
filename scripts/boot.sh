#!/usr/bin/env bash
# boot.sh [secs=90]  -- boot debug server in scratch dir, wait for "Done", send `stop`, grep panics/errors. Exit 0 if clean.
source "$(dirname "$0")/lib.sh"; secs="${1:-90}"
d=$(mktemp -d "$KAB_RUNS/boot.XXXX"); cd "$d"; log="$d/server.log"
mkfifo in; cargo run --manifest-path "$KAB_ROOT/Cargo.toml" -p pumpkin --locked <in >"$log" 2>&1 & pid=$!
exec 3>in
for _ in $(seq "$secs"); do grep -qi "done" "$log" && break; sleep 1; done
grep -qi "done" "$log" || { echo "BOOT TIMEOUT"; tail -20 "$log"; kill $pid 2>/dev/null; exit 1; }
echo stop >&3; wait $pid 2>/dev/null || true
if grep -Ei "panic|ERROR" "$log"; then echo "BOOT DIRTY ($log)"; exit 1; fi; echo "BOOT OK ($log)"
