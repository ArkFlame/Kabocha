#!/usr/bin/env bash
# tracker-refill.sh -- keep KAB_TRACKER_SLOTS (default 10) OpenCode workers on parity-tracker slices that no upstream issue/PR covers.
# Source: docs/PARITY.tsv (covered = UNCOVERED -> implement, COVERED-DONE -> verify, RE-REVIEW* -> audit). Worker names t<slug>.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
exec 8>"$KAB_RUNS/tracker.lock"; flock -n 8 || { echo "tracker refill already running"; exit 0; }
SLOTS="${KAB_TRACKER_SLOTS:-10}"; tried="$KAB_RUNS/tracker-tried"; touch "$tried"
need=$(( SLOTS - $(scripts/oc-status.sh | grep -c '^t.* running$' || true) )); [ "$need" -le 0 ] && { echo "tracker full"; exit 0; }
python3 - "$need" "$tried" <<'PY' | while IFS=$'\t' read -r slug cat name cov note; do
import csv,re,sys
need=int(sys.argv[1]); tried=set(open(sys.argv[2]).read().split())
rank={'UNCOVERED':0,'COVERED-DONE':1,'RE-REVIEW':2,'RE-REVIEW-UNTRACKED':3}
rows=[r for r in csv.DictReader(open('docs/PARITY.tsv'),delimiter='\t') if r['covered'] in rank]
rows.sort(key=lambda r:rank[r['covered']])
n=0
for r in rows:
    slug='t'+re.sub(r'[^a-z0-9]+','-',(r['category']+'-'+r['name']).lower()).strip('-')[:40]
    if slug in tried: continue
    print('\t'.join([slug,r['category'],r['name'],r['covered'],r['note'].replace('\t',' ')[:600]])); n+=1
    if n>=need: break
PY
  echo "$slug" >> "$tried"
  flock "$KAB_RUNS/wt.lock" scripts/wt.sh new "$slug" >/dev/null 2>&1
  f="$KAB_RUNS/brief-$slug.md"
  case "$cov" in
   UNCOVERED) goal="IMPLEMENT the missing behaviour described in the tracker note, matching vanilla (see docs/TODO_2.md section 2 row for this item for file:line and vanilla class). Smallest correct change; update callers; add the smallest automated test only if trivial." ;;
   COVERED-DONE) goal="VERIFY: an upstream issue marked DONE exists but the tracker still lists a gap. Check master at file:line; if the gap is real, fix it with the smallest change; if not, report SKIPPED with file:line proof." ;;
   *) goal="AUDIT: the tracker says the reviewed source changed. Compare the Rust source with vanilla behaviour (decompiled references under .vendor/ if present, otherwise your knowledge of the vanilla class) and list concrete gaps. Fix gaps that are small and certain (one commit each). Write remaining gaps to docs/tracker/$slug.md (file:line, vanilla class, suggested fix, size S/M/L) and commit that file." ;;
  esac
  cat > "$f" <<B
<role>You are a senior Rust engineer working on vanilla-parity for one tracker row. Work alone, one pass.</role>
<context>Repo: Kabocha (Pumpkin Minecraft server fork). Your cwd is your own worktree on branch kab/$slug; never leave it. Read docs/WORKER.md first. Tracker row: category=$cat, name="$name", coverage=$cov.
Tracker note: $note
Locate the source via grep (e.g. the entity/block/item name under crates/pumpkin-core/src).</context>
<task>$goal
Check that each change fills a real gap (WORKER.md rule 12). Commit with: git add <files>; git commit -m "feat(<scope>): <subject>" (subject only, conventional, no body; trailer is automatic).</task>
<constraints>Do NOT run cargo or tests. TOOL CAP about 30 tool calls. No questions, no background jobs. A second identical command means change approach, a third means stop.</constraints>
<report>One line: STATUS: committed|partial|skipped|failed | TRACKER $cat/$name | commit <hash or none> | files <list> | compiled: no | note <one line></report>
B
  scripts/oc-agent.sh "$slug" "$f" 9>&- 8>&- 2>/dev/null | cut -c1-80 &
done
wait
