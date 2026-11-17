#!/usr/bin/env python3
"""gap-pool.py [N=40] -- run a pool of N OpenCode workers in the MAIN checkout (no worktrees), one per tracker doc
with STILL OPEN rows. A freed slot is refilled at once. Units that mention the same .rs basename never run together.
Workers only edit (no cargo, no git); the orchestrator compiles and commits once when the queue is empty.
State: /tmp/kabocha-runs/pool/<unit>.{log,exit}; summary in pool.tsv. Exits when the queue is empty."""
import glob, os, re, signal, subprocess, sys, time
ROOT = os.environ.get('KAB_ROOT', os.getcwd()); RUNS = '/tmp/kabocha-runs/pool'; os.makedirs(RUNS, exist_ok=True)
N = int(sys.argv[1]) if len(sys.argv) > 1 else 40
TIMEOUT = int(os.environ.get('KAB_OC_TIMEOUT', '1200'))
MODEL = os.environ.get('KAB_OC_MODEL', 'opencode-go/hy4-flash-free')
cool = 0  # no launches until this time (quota/rate-limit backoff)
limit_hits = 0
HDR = open(f'{ROOT}/docs/POOL_BRIEF.md').read()
units = []
for f in sorted(glob.glob(f'{ROOT}/docs/tracker/*.md')):
    b = os.path.basename(f)
    if b.startswith(('ss', 'tsmall', 'tverify', 'tdeferred', 'pool-')): continue
    t = open(f).read()
    if 'STILL OPEN' not in t: continue
    units.append((b[:-3], f, set(re.findall(r'([A-Za-z0-9_]+\.rs)', t))))
done = {os.path.basename(p)[:-5] for p in glob.glob(f'{RUNS}/*.exit')}
units_by_name = {u[0]: u for u in units}
queue = [u for u in units if u[0] not in done]
print(f'{len(units)} units, {len(queue)} queued, pool {N}', flush=True)
running = {}  # name -> (proc, files, start)
def launch(u):
    name, path, files = u
    brief = HDR + f'\n\nYOUR UNIT: {path}\nFiles this unit may edit: only the source files named in it (basenames: {", ".join(sorted(files)) or "none"}). Record each row result by appending a section "## Result" to that doc.\n'
    log = open(f'{RUNS}/{name}.log', 'w')
    p = subprocess.Popen(['opencode', 'run', '-m', MODEL, '--auto', '--title', 'pool:' + name, brief], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    running[name] = (p, files, time.time())
def busy(files): return any(files & r[1] for r in running.values())
while queue or running:
    if limit_hits >= 6: print('QUOTA EXHAUSTED, stopping', flush=True); break
    for name, (p, files, t0) in list(running.items()):
        rc = p.poll()
        if rc is None and time.time() - t0 > TIMEOUT:
            os.killpg(p.pid, signal.SIGKILL); rc = 124
        if rc not in (None, 0) and re.search(r'usage limit|Insufficient balance|rate limit|429', open(f'{RUNS}/{name}.log').read()[-600:], re.I):
            queue.insert(0, units_by_name[name]); del running[name]; limit_hits += 1; cool = time.time() + 120 * limit_hits
            print(time.strftime('%H:%M:%S'), name, 'LIMIT, requeued, cooling', 120 * limit_hits, 's', flush=True); continue
        if rc is not None:
            open(f'{RUNS}/{name}.exit', 'w').write(str(rc)); del running[name]
            print(time.strftime('%H:%M:%S'), name, 'exit', rc, '| running', len(running), 'queued', len(queue), flush=True)
    for u in list(queue):
        if len(running) >= (N if limit_hits == 0 else max(2, N // (2 * limit_hits))) or time.time() < cool: break
        if not busy(u[2]): queue.remove(u); launch(u)
    time.sleep(5)
print('POOL DONE', flush=True)
