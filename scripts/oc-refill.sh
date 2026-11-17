#!/usr/bin/env bash
# oc-refill.sh -- fill every free OpenCode slot (KAB_OC_SLOTS, default 20) in PARALLEL: PR workers first (smallest non-draft
# PRs from docs/queue-prs.tsv), then issue workers. Prep (fetch, triage, worktree, brief) runs KAB_PREP_JOBS (default 10) at a time;
# agents are backgrounded by oc-agent.sh. One refill at a time (flock), so the Monitor and a manual run never double-launch.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
exec 9>"$KAB_RUNS/refill.lock"; flock -n 9 || { echo "refill already running"; exit 0; }
scripts/reap-mcp.sh >/dev/null 2>&1 || true; scripts/oc-collect.sh >/dev/null 2>&1 || true
touch "$KAB_RUNS/tried" "$KAB_RUNS/tried-issues"
SLOTS="${KAB_OC_SLOTS:-20}"; JOBS="${KAB_PREP_JOBS:-10}"
free_slots() { echo $(( SLOTS - $(scripts/oc-status.sh | grep -v '^t' | grep -c ' running$' || true) )); }

launch_pr() { # prep one PR and start its worker; never fails the batch
  local pr="$1"; echo "$pr" >> "$KAB_RUNS/tried"   # claim first so a crash never loops on one PR
  case "$(scripts/pr-triage.sh "$pr")" in
    *EMPTY) flock "$KAB_RUNS/done.lock" -c "echo '- #$pr (PR, skipped: already present on master)' >> docs/DONE.md"; return 0 ;;
    *CLEAN) echo "$pr" >> "$KAB_RUNS/clean.txt"; return 0 ;;
    *CLOSED) return 0 ;;
  esac
  flock "$KAB_RUNS/wt.lock" scripts/wt.sh new "oc$pr" >/dev/null 2>&1 || true
  local brief; brief="$(scripts/brief-pr.sh "$pr" 2>/dev/null | tail -1)"
  [ -s "$brief" ] && scripts/oc-agent.sh "oc$pr" "$brief" 9>&- || echo "no brief for PR $pr" >&2
}
export -f launch_pr; export KAB_ROOT KAB_RUNS

need=$(free_slots); [ "$need" -le 0 ] && { echo "full"; exit 0; }
scripts/requeue.sh >/dev/null 2>&1 || true   # never run dry: recycle TODO/FAILED work
# 1) PR candidates
python3 - "$need" <<'PY' | xargs -r -P "$JOBS" -I{} bash -c 'launch_pr {}'
import re,sys,os
need=int(sys.argv[1]); kr=os.environ.get('KAB_RUNS','/tmp/kabocha-runs')
import csv
done={r[1] for r in csv.reader(open('docs/LEDGER.tsv'),delimiter='\t') if r[0]=='pr' and r[3] in ('DONE','SKIPPED','COMMITTED-UNLANDED')}
tried=set(open(kr+'/tried').read().split())
rows=[l.rstrip('\n').split('\t') for l in open('docs/queue-prs.tsv')]
m=[r for r in rows if r[4]=='false' and r[0] not in done|tried and int(r[1])>0]
m.sort(key=lambda r:int(r[1])); print('\n'.join(r[0] for r in m[:need]))
PY
# 2) issue workers fill whatever PR work did not use (next-issue.sh prints "<id> <worktree> <brief>" as each is ready)
need=$(free_slots)
[ "$need" -gt 0 ] && scripts/next-issue.sh "$((need*${KAB_TASKS:-3}))" | scripts/oc-group.sh "${KAB_TASKS:-3}" | xargs -r -P "$JOBS" -L1 bash -c 'scripts/oc-agent.sh "i$0" "$2" 9>&-'
scripts/oc-status.sh | tail -1
