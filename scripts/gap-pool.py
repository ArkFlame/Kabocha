#!/usr/bin/env python3
"""gap-pool.py [HY4_N=40] [DS_N=10] -- pool of OpenCode workers in the MAIN checkout (no worktrees), one per tracker doc with
STILL OPEN rows. Two lanes: hy4 (opencode-go/hy4-flash-free) and ds (open-design/deepseek-v4.1-flash, experimental).
A freed slot is refilled at once; units naming the same .rs basename never run together. Rate-limit/timeout hits requeue the
unit and shrink that lane (to 0 after repeated hits). Adopts workers already running from a previous pool process.
Workers only edit (no cargo, no git). State: /tmp/kabocha-runs/pool/<unit>.{log,exit}, lane stats in pool.log. Exits when done."""
import glob, os, re, signal, subprocess, sys, time
ROOT = os.environ.get('KAB_ROOT', os.getcwd()); RUNS = '/tmp/kabocha-runs/pool'; os.makedirs(RUNS, exist_ok=True)
TIMEOUT = int(os.environ.get('KAB_OC_TIMEOUT', '1200'))
LANES = {'hy4': {'model': 'opencode-go/hy4-flash-free', 'cap': int(sys.argv[1]) if len(sys.argv) > 1 else 40},
         'ds': {'model': 'open-design/deepseek-v4.1-flash', 'cap': int(sys.argv[2]) if len(sys.argv) > 2 else 10}}
for L in LANES.values(): L.update(hits=0, cool=0, done=0, fail=0, secs=0.0)
HDR = open(f'{ROOT}/docs/POOL_BRIEF.md').read()
units = []
GLOB = os.environ.get('KAB_POOL_GLOB', 'docs/tracker/*.md'); MARK = os.environ.get('KAB_POOL_MARK', 'STILL OPEN')
for f in sorted(glob.glob(f'{ROOT}/{GLOB}')):
    b = os.path.basename(f)
    if b.startswith(('ss', 'tsmall', 'tverify', 'tdeferred', 'pool-', 'deferred')): continue
    t = open(f).read()
    if MARK not in t: continue
    units.append((b[:-3], f, set(re.findall(r'([A-Za-z0-9_]+\.rs)', t)) - {'mod.rs', 'lib.rs', 'main.rs', 'tests.rs', 'registry.rs'}))
import collections
hub = {f for f, n in collections.Counter(f for u in units for f in u[2]).items() if n >= 8}  # hub files named by many docs would serialize the whole queue
units = [(n, p, fs - hub) for n, p, fs in units]
by = {u[0]: u for u in units}
done = {os.path.basename(p)[:-5] for p in glob.glob(f'{RUNS}/*.exit')}
running = {}  # name -> dict(proc|pid, files, t0, lane)
for line in subprocess.run(['ps', '-eo', 'pid,args'], capture_output=True, text=True).stdout.splitlines():  # adopt live workers
    m = re.search(r'^\s*(\d+) opencode run .*--title pool:(\S+)', line)
    if m and m.group(2) in by: running[m.group(2)] = dict(pid=int(m.group(1)), files=by[m.group(2)][2], t0=time.time() - 300, lane='hy4')
queue = [u for u in units if u[0] not in done and u[0] not in running]
print(f'{len(units)} units, {len(queue)} queued, {len(running)} adopted, lanes {[(k, v["cap"]) for k, v in LANES.items()]}', flush=True)
def alive(r):
    if 'proc' in r: return r['proc'].poll() is None
    try: os.kill(r['pid'], 0); return True
    except OSError: return False
def launch(u, lane):
    name, path, files = u
    if MARK == 'PASS2-OPEN':
        brief = HDR + f'\n\nYOUR UNIT: {path} (ONE row). Read the row and its source doc, find the Pumpkin code it concerns (grep), verify vs vanilla, fix. You may edit any .rs file this row needs, keep it minimal. Other workers edit other files at the same time: re-read a file right before each edit. If the row is already fixed or is only a note, mark VERIFIED. Append "## Result" to your unit doc.\n'
    else: brief = HDR + f'\n\nYOUR UNIT: {path}\nFiles this unit may edit: only the source files named in it (basenames: {", ".join(sorted(files)) or "none"}). Record each row result by appending a section "## Result" to that doc.\n'
    log = open(f'{RUNS}/{name}.log', 'w')
    p = subprocess.Popen(['opencode', 'run', '-m', LANES[lane]['model'], '--auto', '--title', 'pool:' + name, brief], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    running[name] = dict(proc=p, files=files, t0=time.time(), lane=lane)
MAX_PER_FILE = int(os.environ.get('KAB_POOL_PER_FILE', '2'))  # edit tool re-reads the file per edit, so 2 units per file rarely collide
def busy(files): return any(sum(1 for r in running.values() if f in r['files']) >= MAX_PER_FILE for f in files)
def lane_n(l): return sum(1 for r in running.values() if r['lane'] == l)
last = 0
while queue or running:
    for name, r in list(running.items()):
        L = LANES[r['lane']]; el = time.time() - r['t0']
        if alive(r) and el > TIMEOUT:
            try: os.killpg(r.get('proc', None).pid if 'proc' in r else r['pid'], signal.SIGKILL)
            except OSError: pass
            rc, timed = 124, True
        elif alive(r): continue
        else: rc, timed = (r['proc'].returncode if 'proc' in r else 0), False
        tail = open(f'{RUNS}/{name}.log').read()[-600:]
        if timed or (rc != 0 and re.search(r'usage limit|Insufficient balance|rate.?limit|429|timed out|timeout|tier_limit_exceeded|concurrency limit|rpm limit|Provider blocked', tail, re.I)):
            queue.insert(0, by[name]); del running[name]; L['hits'] += 1; L['cool'] = time.time() + 120 * L['hits']
            L['cap'] = max(0, L['cap'] // 2) if r['lane'] == 'ds' else max(2, L['cap'] * 3 // 4)
            print(time.strftime('%H:%M:%S'), name, r['lane'], 'LIMIT/TIMEOUT -> requeued, lane cap now', L['cap'], flush=True); continue
        open(f'{RUNS}/{name}.exit', 'w').write(str(rc)); del running[name]
        L['done' if rc == 0 else 'fail'] += 1; L['secs'] += el
        print(time.strftime('%H:%M:%S'), name, r['lane'], 'exit', rc, f'{el:.0f}s | running', len(running), 'queued', len(queue), flush=True)
    for u in list(queue):
        ok = [l for l in LANES if lane_n(l) < LANES[l]['cap'] and time.time() >= LANES[l]['cool']]
        lane = min(ok, key=lambda l: lane_n(l) / max(1, LANES[l]['cap'])) if ok else None  # fill lanes proportionally
        if lane is None: break
        if not busy(u[2]): queue.remove(u); launch(u, lane)
    if time.time() - last > 300:
        last = time.time()
        print('STATS', {k: dict(cap=v['cap'], running=lane_n(k), done=v['done'], fail=v['fail'], hits=v['hits'], avg_s=round(v['secs'] / max(1, v['done']))) for k, v in LANES.items()}, flush=True)
    if sum(v['hits'] for v in LANES.values()) >= 8 and LANES['hy4']['cap'] <= 2: print('QUOTA EXHAUSTED', flush=True); break
    time.sleep(5)
print('POOL DONE', flush=True)
