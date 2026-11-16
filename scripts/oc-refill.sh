#!/usr/bin/env bash
# oc-refill.sh  -- start OpenCode PR workers until 10 are running. Picks smallest non-draft PRs from docs/queue-prs.tsv
# not yet done (docs/DONE.md), not already attempted (worktree or $KAB_RUNS/tried). Run after every worker exit.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"; touch "$KAB_RUNS/tried"
running=$(scripts/oc-status.sh | grep -c ' running$' || true); need=$((10-running)); [ $need -le 0 ] && { echo "full (10 running)"; exit 0; }
python3 - "$need" <<'PY' | while read -r pr; do
import re,sys
need=int(sys.argv[1]); kr='/tmp/kabocha-runs'
done=set(re.findall(r'#(\d+)',open('docs/DONE.md').read()))
tried=set(open(kr+'/tried').read().split())|set('2858 2489'.split())
rows=[l.rstrip('\n').split('\t') for l in open('docs/queue-prs.tsv')]
m=[r for r in rows if r[4]=='false' and r[0] not in done|tried and int(r[1])>0]
m.sort(key=lambda r:int(r[1])); print('\n'.join(r[0] for r in m[:need]))
PY
  echo "$pr" >> "$KAB_RUNS/tried"; scripts/wt.sh new "oc$pr" >/dev/null 2>&1; scripts/oc-agent.sh "oc$pr" "$(scripts/brief-pr.sh "$pr")"
done
scripts/oc-status.sh | tail -1
