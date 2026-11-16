#!/usr/bin/env bash
# next-issue.sh [N] -- pick N untried issues (bugs first), create worktree i<ISSUE>, write brief; prints "<ISSUE> <worktree> <brief>".
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"; touch "$KAB_RUNS/tried-issues"; n="${1:-1}"
prep_issue() { # claim, check state, make worktree + brief; prints "<id> <worktree> <brief>"
  local id="$1" st t f
  echo "$id" >> "$KAB_RUNS/tried-issues"
  # skip issues already CLOSED upstream: log to DONE.md-style list and take no slot
  st=$(timeout 20 gh issue view "$id" -R Pumpkin-MC/Pumpkin --json state -q .state 2>/dev/null)
  [ "$st" = "CLOSED" ] && { echo "- #$id (issue, skipped: closed upstream)" >> docs/DONE.md; continue; }
  flock "$KAB_RUNS/wt.lock" scripts/wt.sh new "i$id" >/dev/null 2>&1
  t=$(awk -F'\t' -v n="$id" '$1==n{print $2}' docs/queue-issues.tsv)
  f="$KAB_RUNS/brief-issue$id.md"
  cat > "$f" <<B
<role>You are a senior Rust engineer fixing one upstream issue in a fork. Work alone, one pass, no chat context beyond this brief.</role>
<context>Repo: Kabocha (fork of Pumpkin Minecraft server, Rust). Current dir is your own git worktree; never leave it. Read docs/WORKER.md first (short).
Upstream issue #$id: "$t". Read it once: gh issue view $id -R Pumpkin-MC/Pumpkin --comments (state is OPEN; do not re-fetch it)</context>
<task>
1. Read the issue; locate the root cause in the current code (grep) and confirm the fix you plan fills exactly the gap the issue describes (WORKER.md rule 12). Match vanilla/Paper behaviour.
2. If already fixed in the code: report SKIPPED with file:line proof, change nothing.
3. If unreproducible without runtime/client or too large for one pass: report FAILED with the reason.
4. Else make the smallest fix, update all callers, add the smallest automated test only if trivial.
5. Commit: git add <your files>; git commit -m "fix(<scope>): <subject> (#$id)" (subject only; trailer is automatic).
</task>
<constraints>TOOL CAP: stop and report after about 25 tool calls total, committed or not; a second identical command or read means change approach, a third means stop. READ BUDGET: at most ~8 file reads/greps total; read only the function(s) the issue names (use offset/limit or grep -n, never whole large files). STOP RULE: the moment you know it needs a runtime/client/video repro, extra data, a 1.13 world sample, codegen, or is already fixed, STOP and report skipped/failed at once with one line of proof; do not keep investigating. Never run find / or search outside this worktree and .vendor/; reference sources are in .vendor/ and ../ only. Do not chase upstream fix commits. No cargo/tests/builds, no sleeping or background jobs, no GitHub writes, no other worktrees, no conflict markers.</constraints>
<report>One line: STATUS: committed|partial|skipped|failed | ISSUE #$id | commit <hash> | files <list> | compiled: no | note <one line></report>
B
  [ -s "$f" ] && echo "$id $KAB_WT/i$id $f"
}
export -f prep_issue; export KAB_RUNS KAB_WT
python3 - "$n" <<'PY' | xargs -r -P "${KAB_PREP_JOBS:-10}" -I{} bash -c 'prep_issue {}'
import re,sys
done=set(re.findall(r'#(\d+)',open('docs/DONE.md').read()))|{l.split('\t')[1] for l in open('docs/OUTCOMES.tsv') if l.startswith('issue')}
tried=set(open('/tmp/kabocha-runs/tried-issues').read().split())
rows=[l.rstrip('\n').split('\t') for l in open('docs/queue-issues.tsv')]
c=[r for r in rows if r[0] not in done|tried]
# runtime-only areas (worldgen/protocol) need a seeded world or client; workers failed on them 7/7, so they go last
c.sort(key=lambda r:(1 if ('worldgen' in r[2] or 'Bedrock' in r[1]) else 0, 0 if 'bug' in r[2] else 1, -int(r[0])))
print('\n'.join(r[0] for r in c[:int(sys.argv[1])]))
PY
