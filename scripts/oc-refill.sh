#!/usr/bin/env bash
# load guard: do not start workers while the machine is saturated (load > KAB_MAX_LOAD or RAM available < KAB_MIN_AVAIL_GB)
read -r l1 _ < /proc/loadavg; avail=$(awk '/MemAvailable/{printf "%d",$2/1048576}' /proc/meminfo)
if awk -v l="$l1" -v m="${KAB_MAX_LOAD:-18}" 'BEGIN{exit !(l>m)}' || [ "$avail" -lt "${KAB_MIN_AVAIL_GB:-7}" ]; then echo "refill paused: load=$l1 avail=${avail}GB"; exit 0; fi
# oc-refill.sh  -- start OpenCode PR workers until 20 are running. Picks smallest non-draft PRs from docs/queue-prs.tsv
# not yet done (docs/DONE.md), not already attempted (worktree or $KAB_RUNS/tried). Run after every worker exit.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"; touch "$KAB_RUNS/tried"
running=$(scripts/oc-status.sh | grep -c ' running$' || true); need=$((${KAB_OC_SLOTS:-20}-running)); [ $need -le 0 ] && { echo "full"; exit 0; }
python3 - "$need" <<'PY' | while read -r pr; do
import re,sys
need=int(sys.argv[1]); kr='/tmp/kabocha-runs'
done=set(re.findall(r'#(\d+)',open('docs/DONE.md').read()))
tried=set(open(kr+'/tried').read().split())|set('2858 2489'.split())
rows=[l.rstrip('\n').split('\t') for l in open('docs/queue-prs.tsv')]
m=[r for r in rows if r[4]=='false' and r[0] not in done|tried and int(r[1])>0]
m.sort(key=lambda r:int(r[1])); print('\n'.join(r[0] for r in m[:need]))
PY
  echo "$pr" >> "$KAB_RUNS/tried"
  case "$(scripts/pr-triage.sh "$pr")" in
    *EMPTY) echo "- #$pr (PR, skipped: already present) $(awk -F'\t' -v n=$pr '$1==n{print $6}' docs/queue-prs.tsv)" >> docs/DONE.md; continue ;;
    *CLEAN) echo "$pr" >> "$KAB_RUNS/clean.txt"; continue ;;
    *CLOSED) continue ;;
  esac
  scripts/wt.sh new "oc$pr" >/dev/null 2>&1; scripts/oc-agent.sh "oc$pr" "$(scripts/brief-pr.sh "$pr")"
done
# PR queue drained -> fill the rest with issue workers
running=$(scripts/oc-status.sh | grep -c ' running$' || true); need=$((${KAB_OC_SLOTS:-20}-running))
[ $need -gt 0 ] && scripts/next-issue.sh "$need" | while read -r id wt brief; do scripts/oc-agent.sh "i$id" "$brief"; done
scripts/oc-status.sh | tail -1
