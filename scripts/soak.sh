#!/usr/bin/env bash
# soak.sh [minutes=10]  -- boot the debug server on a fresh world, sample RSS every 10s, stop it, fail on panic lines.
# Bots and MSPT p99 are not in this script yet; see docs/SOAK.md.
source "$(dirname "$0")/lib.sh"; mins="${1:-10}"
d=$(mktemp -d "$KAB_RUNS/soak.XXXX"); cd "$d"; log="$d/server.log"; rss_log="$d/rss.tsv"
mkfifo in; cargo run --manifest-path "$KAB_ROOT/Cargo.toml" -p pumpkin --locked <in >"$log" 2>&1 & runner=$!
exec 3>in
for _ in $(seq 120); do grep -qi "done" "$log" && break; sleep 1; done
grep -qi "done" "$log" || { echo "SOAK BOOT TIMEOUT ($log)"; kill $runner 2>/dev/null; exit 1; }
spid=$(pgrep -n -x pumpkin); [ -n "$spid" ] || { echo "SOAK: no pumpkin process"; exit 1; }
end=$((SECONDS + mins * 60))
while [ "$SECONDS" -lt "$end" ]; do
  rss=$(awk '/^VmRSS:/{print $2}' "/proc/$spid/status" 2>/dev/null) || break
  printf '%s\t%s\n' "$SECONDS" "${rss:-0}" >> "$rss_log"
  sleep 10
done
echo stop >&3; wait $runner 2>/dev/null || true
first=$(head -n1 "$rss_log" | cut -f2); last=$(tail -n1 "$rss_log" | cut -f2)
echo "SOAK RSS kB first=${first:-?} last=${last:-?} ($rss_log)"
if grep -Ei "panicked|thread '.*' panicked" "$log"; then echo "SOAK DIRTY: panic in $log"; exit 1; fi
echo "SOAK OK ($log)"
