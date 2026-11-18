#!/usr/bin/env bash
# own-gaps.sh -- record the gaps our own vanilla audits found (docs/tracker/*.md) as rows in docs/PARITY.tsv (category "own").
# Idempotent: rewrites every "own" row, keeps all other rows. One row per audit note; counts from its FIXED / VERIFIED / STILL OPEN / BLOCKED-DATA marks.
source "$(dirname "$0")/lib.sh"; set +e; cd "$KAB_ROOT"
python3 - <<'PY'
import csv, glob, json, os, re
P = 'docs/PARITY.tsv'
rows = list(csv.DictReader(open(P), delimiter='\t'))
cols = list(rows[0].keys())
rows = [r for r in rows if r['category'] != 'own']
for f in sorted(glob.glob('docs/tracker/*.md')):
    b = os.path.basename(f)[:-3]
    t = open(f).read()
    res = t.split('## Result')[-1] if '## Result' in t else ''
    lines = [l.strip() for l in t.splitlines() if re.search(r'STILL OPEN|FIXED|VERIFIED|BLOCKED-DATA', l)]
    n = lambda k: sum(1 for l in lines if k in l)
    op, fx, vf, bd = n('STILL OPEN'), n('FIXED'), n('VERIFIED'), n('BLOCKED-DATA')
    if not (op or fx or vf or bd): continue
    cov = 'OWN-OPEN' if op else ('OWN-BLOCKED-DATA' if bd else 'OWN-CLOSED')
    srcs = sorted(set(re.findall(r'crates/[A-Za-z0-9_./-]+\.rs', t)))[:6]
    open_items = [re.sub(r'\s+', ' ', l)[:200] for l in lines if 'STILL OPEN' in l][:40]
    r = dict.fromkeys(cols, '')
    r.update(category='own', name=b, status='partial' if op or bd else 'done', covered=cov, issues='-',
             note='own audit: %d open, %d fixed, %d verified, %d blocked-data (docs/tracker/%s.md)' % (op, fx, vf, bd, b),
             source=' '.join(srcs[:1]), summary='docs/tracker/%s.md' % b, checklist=json.dumps(open_items, ensure_ascii=False), ids='[]')
    rows.append(r)
with open(P, 'w', newline='') as fh:
    w = csv.DictWriter(fh, fieldnames=cols, delimiter='\t', lineterminator='\n'); w.writeheader(); w.writerows(rows)
own = [r for r in rows if r['category'] == 'own']
print('own rows: %d (open %d)' % (len(own), sum(r['covered'] == 'OWN-OPEN' for r in own)))
PY
