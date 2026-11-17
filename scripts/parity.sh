#!/usr/bin/env bash
# parity.sh [--no-fetch] -- diff the Pumpkin parity tracker (Pumpkin-Website data.json) against docs/PARITY.tsv.
# Prints the per-category scoreboard, rows whose upstream status changed or that are missing from PARITY.tsv,
# and our gap-closing progress (TRACKER.md). Cache: /tmp/kabocha-runs/tracker.json. Exit 1 when PARITY.tsv is stale.
source "$(dirname "$0")/lib.sh"; set +e; cd "$KAB_ROOT"
J="$KAB_RUNS/tracker.json"
if [ "${1:-}" != "--no-fetch" ] || [ ! -s "$J" ]; then
  gh api -H "Accept: application/vnd.github.raw" repos/Pumpkin-MC/Pumpkin-Website/contents/src/pages/tracker/data.json > "$J.new" 2>/dev/null \
    && [ -s "$J.new" ] && mv "$J.new" "$J" || rm -f "$J.new"
fi
[ -s "$J" ] || { echo "parity: no tracker data (gh api failed and no cache)"; exit 2; }
python3 - "$J" <<'PY'
import csv, json, sys, collections
d = json.load(open(sys.argv[1]))
ours = {(r['category'], r['name']): r for r in csv.DictReader(open('docs/PARITY.tsv'), delimiter='\t')}
print('tracker %s commit %s updated %s' % (d['version'], d['commit'][:9], d['updated']))
print('%-10s %6s %5s %7s %7s' % ('category', 'rows', 'done', 'partial', 'planned'))
tot = collections.Counter(); changed = []; missing = []
for c in d['categories']:
    cnt = collections.Counter(e['status'] for e in c['entries'])
    tot.update(cnt)
    print('%-10s %6d %5d %7d %7d' % (c['id'], len(c['entries']), cnt['done'], cnt['partial'], cnt['planned']))
    for e in c['entries']:
        r = ours.get((c['id'], e['name']))
        if e['status'] == 'done':
            if r and r['status'] != 'done': changed.append((c['id'], e['name'], r['status'], 'done'))
        elif r is None: missing.append((c['id'], e['name'], e['status']))
        elif r['status'] != e['status']: changed.append((c['id'], e['name'], r['status'], e['status']))
print('%-10s %6d %5d %7d %7d' % ('all', sum(tot.values()), tot['done'], tot['partial'], tot['planned']))
pct = 100.0 * tot['done'] / max(1, sum(tot.values()))
print('upstream parity: %.1f%% done' % pct)
for c, n, a, b in changed: print('CHANGED %s/%s: %s -> %s' % (c, n, a, b))
for c, n, s in missing: print('MISSING %s/%s (%s): not in docs/PARITY.tsv' % (c, n, s))
sys.exit(1 if changed or missing else 0)
PY
rc=$?
grep -E '^\| \*\*all\*\*' TRACKER.md 2>/dev/null | sed 's/^/our gaps (TRACKER.md): /'
exit $rc
