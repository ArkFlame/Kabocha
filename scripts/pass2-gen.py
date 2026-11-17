#!/usr/bin/env python3
"""pass2-gen.py -- turn the rows workers left OPEN (in each doc's "## Result" section) into one small unit doc per row under
docs/tracker/pass2/, skipping rows blocked on generated data, WIT/plugin API or large refactors (those go to docs/tracker/deferred-pass2.md)."""
import glob, os, re
DEFER = re.compile(r'generated|extractor|needs data|\bwit\b|plugin api|cross-crate|refactor|brain|engine', re.I)
rows, deferred = 0, []
for f in sorted(glob.glob('docs/tracker/t*.md')):
    t = open(f).read()
    if '## Result' not in t: continue
    for l in t.split('## Result', 1)[1].splitlines():
        if not re.search(r'\bOPEN\b', l) or re.search(r'RESOLVED|FIXED|VERIFIED', l.split('OPEN')[0][-40:]): continue
        if DEFER.search(l): deferred.append(f'- {os.path.basename(f)}: {l.strip()[:300]}'); continue
        rows += 1; name = f'docs/tracker/pass2/{rows:04d}.md'
        open(name, 'w').write(f'# Pass 2 row {rows}\nSource doc: {f}\n\nPASS2-OPEN {l.strip()}\n\nYou may edit ANY .rs file needed for this single row (not only the files named in the source doc). Append a "## Result" line here: FIXED/VERIFIED/STILL OPEN + reason.\n')
open('docs/tracker/deferred-pass2.md', 'w').write('# Deferred after pass 1 (data, WIT, large refactors)\n\n' + '\n'.join(deferred) + '\n')
print(rows, 'row units;', len(deferred), 'deferred')
