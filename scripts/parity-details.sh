#!/usr/bin/env bash
# parity-details.sh -- add optional detail columns (source, summary, checklist, ids) to docs/PARITY.tsv.
# Sources: upstream tracker JSON (cache /tmp/kabocha-runs/tracker.json). Keeps status/covered/issues/note untouched.
# source only when the path exists in this tree; checklist only items marked done; JSON cells are single-line arrays.
source "$(dirname "$0")/lib.sh"; set +e; cd "$KAB_ROOT"
python3 - "$KAB_RUNS/tracker.json" <<'PY'
import csv, json, os, re, sys, unicodedata
d = json.load(open(sys.argv[1]))
key = lambda c, n: (c, re.sub(r'[\s_-]+', ' ', unicodedata.normalize('NFKC', n).lower()).strip())
up = {key(c['id'], e['name']): e for c in d['categories'] for e in c['entries']}
rows = list(csv.DictReader(open('docs/PARITY.tsv'), delimiter='\t'))
base = ['category', 'name', 'status', 'covered', 'issues', 'note']
new = ['source', 'summary', 'checklist', 'ids']
clean = lambda s: re.sub(r'[\t\r\n]+', ' ', s or '').strip()
BAD = re.compile(r"\b(no|not|missing|lacks?|without|pending|still|unimplemented|unverified|review|todo|partial|incomplete|only|never|cannot|does not|doesn't|isn't|aren't|absent|differs?|should)\b", re.I)
def summarize(t):
    ss = re.split(r'(?<=[.!?])\s+', clean(t))
    return ' '.join(x for x in ss if x and not BAD.search(x))
seen = set(); n = {k: 0 for k in new}
for r in rows:
    k = key(r['category'], r['name'])
    assert k not in seen, 'duplicate key %s' % (k,)
    seen.add(k)
    e = up.get(k, {})
    src = e.get('source', '')
    ok = src and not src.startswith(('/', '.')) and '..' not in src.split('/') and os.path.isfile(src)
    r['source'] = src if ok else ''
    r['summary'] = summarize(e.get('note', '')) if ok else ''
    r['checklist'] = json.dumps([clean(i['text']) for i in e.get('items', []) if i.get('done') and clean(i.get('text'))], ensure_ascii=False) if ok else ''
    if r['checklist'] == '[]': r['checklist'] = ''
    ids = [i for i in e.get('ids', []) if isinstance(i, str) and i]
    r['ids'] = json.dumps(ids, ensure_ascii=False) if ids else ''
    for f in new: n[f] += bool(r[f])
for r in rows:
    for f in base: r[f] = clean(r.get(f))
with open('docs/PARITY.tsv', 'w', newline='') as f:
    w = csv.DictWriter(f, base + new, delimiter='\t', lineterminator='\n', quoting=csv.QUOTE_NONE, quotechar=None, escapechar=None)
    w.writeheader(); w.writerows(rows)
print('rows %d; with source %d, summary %d, checklist %d, ids %d' % (len(rows), n['source'], n['summary'], n['checklist'], n['ids']))
PY
