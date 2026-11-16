#!/usr/bin/env bash
# ledger.sh -- write docs/LEDGER.tsv (one row per open upstream issue/PR) and print DONE/total per kind.
# state: DONE (subject ending (#N) on master, or logged DONE.md) | SKIPPED (OUTCOMES skipped) | FAILED | COMMITTED-UNLANDED
# (kab/<w> has commits master lacks, per git cherry) | IN-PROGRESS (worker running) | TODO
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
python3 - <<'PY'
import re,subprocess,os,glob
sh=lambda *a:subprocess.run(a,capture_output=True,text=True).stdout
land={}
for l in sh('git','log','--format=%h %s','master').splitlines():
    h,_,s=l.partition(' ')
    for n in re.findall(r'\(#(\d+)\)\s*$',s): land.setdefault(int(n),h)
done={int(n) for n in re.findall(r'#(\d+)',open('docs/DONE.md').read())}
out={}
for l in open('docs/OUTCOMES.tsv'):
    f=l.rstrip('\n').split('\t')
    if len(f)>5 and f[1].isdigit(): out[(f[0],int(f[1]))]=(f[3],f[4])
running=set()
for p in glob.glob('/tmp/kabocha-runs/*.pid'):
    w=os.path.basename(p)[:-4]
    try: os.kill(int(open(p).read().split()[0]),0); running.add(w)
    except Exception: pass
br={}
for b in sh('git','for-each-ref','--format=%(refname:short)','refs/heads/kab/').split():
    m=re.match(r'kab/[a-z]*?(\d+)$',b)
    if m: br.setdefault(int(m.group(1)),b)
rows=[]; cnt={}
for kind,p in (('issue','docs/queue-issues.tsv'),('pr','docs/queue-prs.tsv')):
    for l in open(p):
        f=l.rstrip('\n').split('\t')
        if not f[0].isdigit(): continue
        n=int(f[0]); b=br.get(n,''); h=land.get(n,''); ev=''
        o=out.get((kind,n))
        if h: st='DONE'
        elif n in done: st='DONE'; ev='DONE.md'
        elif b and any(w.endswith(str(n)) for w in running if w.startswith(('i','oc','n'))): st='IN-PROGRESS'
        elif b and '+' in sh('git','cherry','master',b): st='COMMITTED-UNLANDED'
        elif o: st=o[0].upper(); ev=o[1][:160]
        else: st='TODO'
        rows.append((kind,n,f[1][:80],st,b,h,ev)); cnt.setdefault(kind,{}).setdefault(st,0); cnt[kind][st]+=1
with open('docs/LEDGER.tsv','w') as o:
    o.write('kind\tnumber\ttitle\tstate\tbranch\tmaster\tevidence\n')
    for r in rows: o.write('\t'.join(map(str,r))+'\n')
for k,c in cnt.items():
    t=sum(c.values()); print(k,t,'DONE %.1f%%'%(100*c.get('DONE',0)/t),dict(sorted(c.items())))
PY
