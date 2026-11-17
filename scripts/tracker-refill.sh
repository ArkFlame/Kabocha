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
rank={'UNCOVERED':0,'COVERED-DONE':1,'RE-REVIEW':2,'RE-REVIEW-UNTRACKED':3,'AUDIT':4}
rows=[r for r in csv.DictReader(open('docs/PARITY.tsv'),delimiter='\t') if r['covered'] in rank]
rows.sort(key=lambda r:rank[r['covered']])
import subprocess
# endless supply: once tracker rows are exhausted, audit source files carrying vanilla-parity TODOs
out=subprocess.run(['git','grep','-c','-E','TODO|FIXME|vanilla',  '--','crates/pumpkin-core/src'],capture_output=True,text=True).stdout.split('\n')
files=sorted(((int(l.rsplit(':',1)[1]),l.rsplit(':',1)[0]) for l in out if ':' in l),reverse=True)
rows+= [{'category':'audit','name':p,'covered':'AUDIT','note':'%d TODO/FIXME/vanilla markers in this file'%c} for c,p in files if c>=2]
import hashlib
def slug_of(r):  # hash suffix keeps long names unique after truncation
    full=r['category']+'-'+r['name']
    return 't'+re.sub(r'[^a-z0-9]+','-',full.lower()).strip('-')[:32]+'-'+hashlib.md5(full.encode()).hexdigest()[:6]
def pick(tried):
    n=0
    for r in rows:
        slug=slug_of(r)
        if slug in tried: continue
        print('\t'.join([slug,r['category'],r['name'],r['covered'],r['note'].replace('\t',' ')[:600]])); n+=1; tried.add(slug)
        if n>=need: break
    return n
if pick(set(tried))==0:  # supply exhausted: audits repeat against the latest master, so recycle the tried list
    open(sys.argv[2],'w').close(); pick(set())
PY
  echo "$slug" >> "$tried"
  scripts/wt.sh new "$slug" >/dev/null 2>&1
  f="$KAB_RUNS/brief-$slug.md"
  case "$cov" in
   UNCOVERED) goal="IMPLEMENT the missing behaviour described in the tracker note, matching vanilla (see docs/TODO_2.md section 2 row for this item for file:line and vanilla class). Smallest correct change; update callers; add the smallest automated test only if trivial." ;;
   COVERED-DONE) goal="VERIFY: an upstream issue marked DONE exists but the tracker still lists a gap. Check master at file:line; if the gap is real, fix it with the smallest change; if not, report SKIPPED with file:line proof." ;;
   AUDIT) goal="DEEP AUDIT of the source file named above. List every TODO/FIXME and every place where behaviour differs from vanilla (read the matching vanilla class under .vendor/ or your knowledge of it; also read how Paper/Spigot handle it). Fix every gap that is small and certain, one commit per fix, with the smallest automated test when trivial. Write remaining larger gaps to docs/tracker/$slug.md (file:line, vanilla class, suggested fix, size). You may use up to 60 tool calls." ;;
   *) goal="AUDIT: the tracker says the reviewed source changed. Compare the Rust source with vanilla behaviour (decompiled references under .vendor/ if present, otherwise your knowledge of the vanilla class) and list concrete gaps. Fix gaps that are small and certain (one commit each). Write remaining gaps to docs/tracker/$slug.md (file:line, vanilla class, suggested fix, size S/M/L) and commit that file." ;;
  esac
  cat > "$f" <<B
<role>You are a senior Rust engineer working on vanilla-parity for one tracker row. Work alone, one pass.</role>
<context>Repo: Kabocha (Pumpkin Minecraft server fork). Your cwd is your own worktree on branch kab/$slug; never leave it. Read docs/WORKER.md first. Tracker row: category=$cat, name="$name", coverage=$cov.
Tracker note: $note
Locate the source via grep (e.g. the entity/block/item name under crates/pumpkin-core/src).</context>
<task>$goal
Check that each change fills a real gap (WORKER.md rule 12). Commit with: git add <files>; git commit -m "feat(<scope>): <subject>" (subject only, conventional, no body; trailer is automatic).</task>
<constraints>Do NOT run cargo or tests. TOOL CAP about 30 tool calls (60 for a DEEP AUDIT). No questions, no background jobs. A second identical command means change approach, a third means stop.</constraints>
<report>One line: STATUS: committed|partial|skipped|failed | TRACKER $cat/$name | commit <hash or none> | files <list> | compiled: no | note <one line></report>
B
  scripts/oc-agent.sh "$slug" "$f" 9>&- 8>&- 2>/dev/null | cut -c1-80 8>&- 9>&- &
done
flock -u 8
wait
