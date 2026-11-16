#!/usr/bin/env bash
# next-pr.sh [N]  -- pick the next N (default 1) untried PRs (smallest first), create worktree n<PR>, fetch the PR ref,
# write the brief, and print "<PR> <worktree> <brief>" per ready PR. Launch one native `general` worker per line.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"; touch "$KAB_RUNS/tried"; n="${1:-1}"
python3 - "$n" <<'PY' | while read -r pr; do
import re,sys
done=set(re.findall(r'#(\d+)',open('docs/DONE.md').read()))
tried=set(open('/tmp/kabocha-runs/tried').read().split())|set('2858 2489'.split())
rows=[l.rstrip('\n').split('\t') for l in open('docs/queue-prs.tsv')]
m=[r for r in rows if r[4]=='false' and r[0] not in done|tried and int(r[1])>0]
m.sort(key=lambda r:int(r[1])); print('\n'.join(r[0] for r in m[:int(sys.argv[1])]))
PY
  echo "$pr" >> "$KAB_RUNS/tried"
  git fetch -q upstream "pull/$pr/head:refs/kab/pr-$pr" 2>/dev/null
  scripts/wt.sh new "n$pr" >/dev/null 2>&1
  scripts/brief-pr.sh "$pr" >/dev/null 2>&1 || true
  [ -s "$KAB_RUNS/brief-pr$pr.md" ] && echo "$pr $KAB_WT/n$pr $KAB_RUNS/brief-pr$pr.md" || echo "FAILED $pr" >&2
done
