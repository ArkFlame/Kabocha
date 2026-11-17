#!/usr/bin/env bash
# tracker-watch.sh -- keep TRACKER.md current against our commits. Polls every 30s; when master's HEAD or the set of running
# workers changed, regenerates TRACKER.md; commits and pushes it at most every KAB_TRACKER_COMMIT_SECS (default 300).
# Single instance (flock). Start: (setsid nohup scripts/tracker-watch.sh >/tmp/kabocha-runs/tracker-watch.log 2>&1 &)
source "$(dirname "$0")/lib.sh"; set +e; cd "$KAB_ROOT"
exec 5>"$KAB_RUNS/tracker-watch.lock"; flock -n 5 || { echo "tracker-watch already running"; exit 0; }
GAP="${KAB_TRACKER_COMMIT_SECS:-300}"; prev=""; last=0
while true; do
  cur="$(git rev-parse master) $(scripts/oc-status.sh 2>/dev/null | grep ' running$' | sort | md5sum | cut -c1-8)"
  if [ "$cur" != "$prev" ]; then
    now=$(date +%s)
    if [ $((now - last)) -ge "$GAP" ]; then scripts/tracker-md.sh --commit 5>&- ; last=$now
    else scripts/tracker-md.sh 5>&-; fi
    prev="$cur"
  fi
  sleep 30
done
