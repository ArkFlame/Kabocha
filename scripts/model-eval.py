#!/usr/bin/env python3
"""model-eval.py -- model comparison tracker for the gap pool (scripts/gap-pool.py).

Ingests finished runs from /tmp/kabocha-runs/pool/<unit>.{exit,log} and the lane lines of
/tmp/kabocha-runs/pool.log into /tmp/kabocha-runs/eval.jsonl (append-only, deduped by unit|lane|ts),
then writes docs/MODEL_EVAL.md from that history. Stdlib only, no network, idempotent.
Refresh: python3 scripts/model-eval.py   (env KAB_RUNS overrides /tmp/kabocha-runs)."""
import collections
import datetime
import glob
import json
import math
import os
import re
import statistics
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
RUNS = os.environ.get('KAB_RUNS', '/tmp/kabocha-runs')
POOL_DIR = os.path.join(RUNS, 'pool')
POOL_LOG = os.path.join(RUNS, 'pool.log')
EVAL = os.path.join(RUNS, 'eval.jsonl')
TRACKER = os.path.join(ROOT, 'docs', 'tracker')
OUT = os.path.join(ROOT, 'docs', 'MODEL_EVAL.md')
LANE_MODEL = {'hy4': 'hy4-flash-free', 'ds': 'deepseek-v4.1-flash', 'ic': 'iceman-contributor-free'}
MODEL_LANE = {v: k for k, v in LANE_MODEL.items()}
MIN_RUNS = 5  # models with fewer finished runs are not ranked in the verdict
MAX_SECS = 1300  # gap-pool kills a run at 1200s; longer spans come from relaunched logs or host sleep
ANSI = re.compile(r'\x1b\[[0-9;?]*[A-Za-z]')
TOOL_GLYPHS = set('→✱⚙✗←✓◈')
POOL_EXIT = re.compile(r'^(\d{2}:\d{2}:\d{2}) (\S+) (hy4|ds|ic) exit (\d+) (\d+)s')
POOL_LIMIT = re.compile(r'^(\d{2}:\d{2}:\d{2}) (\S+) (hy4|ds|ic) LIMIT/TIMEOUT')
HEADER = re.compile(r'^> build · (\S+)')
ERR_RULES = [
    ('ECONNRESET', re.compile(r'ECONNRESET')),
    ('network', re.compile(r"Can't reach|getaddrinfo|EAI_AGAIN|ENOTFOUND|ECONNREFUSED|fetch failed", re.I)),
    ('sqlite', re.compile(r'Failed to execute statement')),
    ('5xx', re.compile(r'(UnexpectedStatus|Status): 5\d\d')),
    ('rate limit', re.compile(r'Rate limit exceeded|rate.?limit|usage limit|\b429\b|Insufficient balance|'
                              r'tier_limit_exceeded|concurrency limit|rpm limit|Provider blocked', re.I)),
    ('shell denied', re.compile(r'Permission denied: shell')),
    ('timeout', re.compile(r'(?<!Search )timed out|ETIMEDOUT|\bTimeout\b', re.I)),
    ('missing tool', re.compile(r'No tool named')),
]
ERR_CATS = [c for c, _ in ERR_RULES] + ['other']
WRITE_CODE = re.compile(r"write_text|writeFileSync|\.write\(|open\([^)]*['\"][wa]|sed -i|\btee\b|Path\(.*\)\.write")
PATH_IN = re.compile(r'(?:crates|docs|scripts)/[\w./-]+')
NARR_NO_CHANGE = re.compile(r'no (code |source )?(change|edit)s? (was |were )?(needed|required|necessary)|nothing to (change|fix)', re.I)
REVISION = re.compile(r'\b(wait|actually|hmm|on second thought|let me reconsider|I was wrong|correction)\b', re.I)
REASON_MARK = re.compile(r'^\s*(_?thinking:?_?|<think>|reasoning:)', re.I)
RESULT_TOKENS = [('FIXED', 'FIXED'), ('VERIFIED', 'VERIFIED'), ('STILL OPEN', 'STILL_OPEN'), ('BLOCKED-DATA', 'BLOCKED_DATA')]


def read_text(path):
    try:
        with open(path, 'rb') as fh:
            return fh.read().decode('utf8', 'replace')
    except OSError:
        return ''


def load_pool_log():
    """Return unit -> {lane, secs, exit, limit} from the latest pool.log lines (pool.log is truncated on restart)."""
    info = {}
    for line in read_text(POOL_LOG).split('\n'):
        m = POOL_EXIT.match(line)
        if m:
            d = info.setdefault(m.group(2), {'limit': 0})
            d.update(lane=m.group(3), exit=int(m.group(4)), secs=int(m.group(5)), hhmmss=m.group(1))
            continue
        m = POOL_LIMIT.match(line)
        if m:
            d = info.setdefault(m.group(2), {'limit': 0})
            d['lane'] = m.group(3)
            d['limit'] += 1
    return info


def birth_times(paths):
    """Creation time (epoch) of each file via stat %W; 0 when the filesystem does not report it."""
    out = {}
    if not paths:
        return out
    res = subprocess.run(['stat', '-c', '%W %n'] + paths, capture_output=True, text=True)
    for line in res.stdout.splitlines():
        parts = line.split(' ', 1)
        if len(parts) == 2 and parts[0].isdigit():
            out[parts[1]] = int(parts[0])
    return out


def find_doc(unit, log_text, walk):
    top = os.path.join(TRACKER, unit + '.md')
    if os.path.exists(top):
        return top
    for p in re.findall(r'docs/tracker/[\w./-]+\.md', log_text):
        if p.endswith('/' + unit + '.md') or os.path.basename(p) == unit + '.md':
            full = os.path.join(ROOT, p)
            if os.path.exists(full):
                return full
    return walk.get(unit + '.md')


def walk_tracker():
    found = {}
    for dp, _, files in os.walk(TRACKER):
        for f in sorted(files):
            found.setdefault(f, os.path.join(dp, f))
    return found


def result_of(doc):
    """Status counts of the latest '## Result' section in a unit doc, or None when there is no doc."""
    if not doc:
        return None
    lines = read_text(doc).split('\n')
    heads = [i for i, ln in enumerate(lines) if re.match(r'^## Result\b', ln)]
    if not heads:
        return {'present': False, 'empty': False, 'counts': {}}
    body = []
    for ln in lines[heads[-1] + 1:]:
        if ln.startswith('## '):
            break
        body.append(ln)
    counts = {key: 0 for _, key in RESULT_TOKENS}
    for ln in body:
        for token, key in RESULT_TOKENS:
            if re.search(r'\b' + re.escape(token) + r'\b', ln):
                counts[key] += 1
                break
    return {'present': True, 'empty': not any(ln.strip() for ln in body), 'counts': counts}


def parse_log(text):
    """Derive run-level signals from one opencode transcript."""
    model = None
    tool_calls, edits, edit_fail, files = 0, 0, 0, set()
    errors = collections.Counter()
    loop = False
    prev, streak = None, 0
    narr, narr_chars, revisions, prose_repeat = [], 0, 0, False
    reasoning = False
    contradiction = False
    seen_narr = collections.Counter()
    for raw in text.split('\n'):
        s = ANSI.sub('', raw).strip()
        if not s:
            continue
        m = HEADER.match(s)
        if m:
            model = model or m.group(1)
            continue
        if REASON_MARK.match(s):
            reasoning = True
        if s[0] in TOOL_GLYPHS and len(s) > 2 and s[1] == ' ':
            tool_calls += 1
            if s == prev:
                streak += 1
                if streak == 3:
                    loop = True
            else:
                prev, streak = s, 1
            if s.startswith(('← Edit ', '← Write ')):
                edits += 1
                files.add(s.split(' ', 2)[2].split(' [')[0].strip())
            elif s.startswith(('✗ Edit ', '✗ Write ')):
                edit_fail += 1
            elif s.startswith(('⚙ ', '⚙')) and WRITE_CODE.search(s):
                edits += 1
                files.update(PATH_IN.findall(s))
            continue
        if s.startswith(('Error', '✗', '!', 'UnexpectedStatus', 'Status')):
            cat = 'other'
            for name, rx in ERR_RULES:
                if rx.search(s):
                    cat = name
                    break
            if s.startswith('Error') or cat != 'other':
                errors[cat] += 1
            continue
        if (s[0].isalpha() or re.match(r'^- \S', s)) and len(s) >= 25 and not s.startswith('Read '):
            narr.append(s)
            narr_chars += len(s)
            seen_narr[s] += 1
            if REVISION.search(s):
                revisions += 1
            if NARR_NO_CHANGE.search(s):
                contradiction = True
    prose_repeat = any(c >= 3 for c in seen_narr.values())
    return dict(model=model, tool_calls=tool_calls, edits=edits, edit_failed=edit_fail,
                files=sorted(files), errors=dict(errors), loop=loop, narr_lines=len(narr),
                narr_chars=narr_chars, revisions=revisions, prose_repeat=prose_repeat,
                reasoning=reasoning, contradiction=contradiction and edits > 0)


def ingest():
    pool = load_pool_log()
    exits = sorted(glob.glob(os.path.join(POOL_DIR, '*.exit')))
    units = [os.path.basename(p)[:-5] for p in exits]
    logs = {u: os.path.join(POOL_DIR, u + '.log') for u in units}
    births = birth_times([logs[u] for u in units if os.path.exists(logs[u])])
    walk = walk_tracker()
    have = set()
    if os.path.exists(EVAL):
        for line in read_text(EVAL).split('\n'):
            if line.strip():
                try:
                    r = json.loads(line)
                except ValueError:
                    continue
                have.add(r.get('key'))
    new = []
    for u in units:
        lp = logs[u]
        if not os.path.exists(lp):
            continue
        exit_path = os.path.join(POOL_DIR, u + '.exit')
        try:
            rc = int(read_text(exit_path).strip() or '-1')
            mtime = int(os.stat(exit_path).st_mtime)
        except (OSError, ValueError):
            continue
        text = read_text(lp)
        sig = parse_log(text)
        pl = pool.get(u, {})
        model = sig['model'] or LANE_MODEL.get(pl.get('lane', ''), 'unknown')
        lane = pl.get('lane') or MODEL_LANE.get(model, '?')
        birth = births.get(lp, 0)
        span = mtime - birth if birth > 0 and mtime >= birth else None
        if span is not None and span <= MAX_SECS:
            secs = span
        elif pl.get('secs') is not None and pl['secs'] <= MAX_SECS:
            secs = pl['secs']
        else:
            secs = None  # log re-used by a relaunch or the host slept: span is not a run time
        doc = find_doc(u, text, walk)
        res = result_of(doc)
        ts = datetime.datetime.fromtimestamp(mtime).isoformat(timespec='seconds')
        key = f'{u}|{lane}|{mtime}'
        if key in have:
            continue
        rec = dict(key=key, unit=u, lane=lane, model=model, ts=ts, mtime=mtime, exit=rc, secs=secs,
                   lane_mismatch=bool(pl.get('lane') and MODEL_LANE.get(model) not in (None, pl['lane'])),
                   limit_hits=pl.get('limit', 0), result=res, **{k: v for k, v in sig.items() if k != 'model'})
        new.append(rec)
    if new:
        with open(EVAL, 'a', encoding='utf8') as fh:
            for rec in new:
                fh.write(json.dumps(rec, ensure_ascii=False) + '\n')
    recs = []
    for line in read_text(EVAL).split('\n'):
        if line.strip():
            try:
                recs.append(json.loads(line))
            except ValueError:
                continue
    return recs, len(new)


def pct(a, b):
    return f'{100.0 * a / b:.0f}%' if b else '-'


def fmt(x, nd=0):
    return '-' if x is None else (f'{x:.{nd}f}')


def p90(vals):
    v = sorted(vals)
    return v[max(0, math.ceil(0.9 * len(v)) - 1)] if v else None


def model_stats(recs):
    by = collections.defaultdict(list)
    for r in recs:
        by[r['model']].append(r)
    stats = {}
    for m, rs in by.items():
        n = len(rs)
        ok = [r for r in rs if r['exit'] == 0]
        secs = [r['secs'] for r in rs if r['secs'] is not None]
        worker_h = sum(secs) / 3600.0
        res_ok = [r['result'] for r in ok if r.get('result')]
        counts = collections.Counter()
        for r in ok:
            c = (r.get('result') or {}).get('counts') or {}
            for k, v in c.items():
                counts[k] += v
        err_runs = collections.Counter()
        err_raw = collections.Counter()
        for r in rs:
            for c, v in r.get('errors', {}).items():
                err_runs[c] += 1
                err_raw[c] += v
        files = set()
        for r in rs:
            files.update(r.get('files', []))
        ok_n = len(ok)
        stats[m] = dict(
            n=n, ok=ok_n, rate=ok_n / n if n else 0.0, secs=secs,
            avg_s=statistics.mean(secs) if secs else None,
            med_s=statistics.median(secs) if secs else None,
            p90_s=p90(secs), per_wh=(ok_n / worker_h) if worker_h else None,
            err_runs=err_runs, err_raw=err_raw,
            tools=statistics.mean([r['tool_calls'] for r in rs]) if rs else 0,
            edits=sum(r['edits'] for r in rs), edit_avg=(sum(r['edits'] for r in rs) / n) if n else 0,
            edit_fail=sum(r['edit_failed'] for r in rs), files=len(files),
            noop=sum(1 for r in ok if r['edits'] == 0), noop_rate=(sum(1 for r in ok if r['edits'] == 0) / ok_n) if ok_n else 0,
            fixed=counts['FIXED'], verified=counts['VERIFIED'], open=counts['STILL_OPEN'], blocked=counts['BLOCKED_DATA'],
            res_present=sum(1 for x in res_ok if x.get('present')),
            res_empty=sum(1 for x in res_ok if x.get('present') and x.get('empty')),
            res_missing=sum(1 for r in ok if not (r.get('result') or {}).get('present')),
            loops=sum(1 for r in rs if r['loop']), loop_rate=(sum(1 for r in rs if r['loop']) / n) if n else 0,
            fixed_no_edit=sum(1 for r in ok if (r.get('result') or {}).get('counts', {}).get('FIXED', 0) > 0 and r['edits'] == 0),
            narr_avg=(statistics.mean([r['narr_chars'] for r in rs]) if rs else 0),
            narr_lines_avg=(statistics.mean([r['narr_lines'] for r in rs]) if rs else 0),
            reasoning=sum(1 for r in rs if r['reasoning']),
            revisions_avg=(statistics.mean([r['revisions'] for r in rs]) if rs else 0),
            prose_repeat=sum(1 for r in rs if r['prose_repeat']),
            contradict=sum(1 for r in rs if r['contradiction']),
            limit_hits=sum(r.get('limit_hits', 0) for r in rs),
            mismatch=sum(1 for r in rs if r.get('lane_mismatch')),
            lane=MODEL_LANE.get(m, '?') if m != 'unknown' else 'failed before header',
        )
    return stats


def resolved_per_run(s):
    return (s['fixed'] + s['verified']) / s['ok'] if s['ok'] else 0.0


def verdict(stats):
    ranked = {m: s for m, s in stats.items() if s['n'] >= MIN_RUNS}
    if not ranked:
        return [f'No model has {MIN_RUNS}+ finished runs yet, so no ranking is made.']
    lines = []
    tp = [(s['per_wh'], m) for m, s in ranked.items() if s['per_wh'] is not None]
    if tp:
        v, m = max(tp)
        lines.append(f'- **Best throughput:** `{m}`, {v:.1f} successful runs per worker-hour (median {fmt(ranked[m]["med_s"])}s).')
    rel = max(ranked.items(), key=lambda kv: (kv[1]['rate'], -kv[1]['loop_rate']))
    lines.append(f'- **Most reliable:** `{rel[0]}`, exit 0 on {pct(rel[1]["ok"], rel[1]["n"])} of {rel[1]["n"]} runs.')
    q = max(ranked.items(), key=lambda kv: (resolved_per_run(kv[1]), -kv[1]['loop_rate']))
    lines.append(f'- **Best quality per run:** `{q[0]}`, {resolved_per_run(q[1]):.2f} FIXED+VERIFIED rows per successful run '
                 f'(loops in {pct(q[1]["loops"], q[1]["n"])} of runs).')
    unranked = [m for m in stats if m not in ranked]
    if unranked:
        lines.append(f'- Not ranked (under {MIN_RUNS} runs): ' + ', '.join(f'`{m}`' for m in sorted(unranked)) + '.')
    return lines


def render(recs, stats, new_count):
    out = []
    w = out.append
    w('# Model evaluation: gap pool')
    w('')
    w('Generated by `scripts/model-eval.py` from `/tmp/kabocha-runs/eval.jsonl` (history of finished pool runs, '
      'appended on every refresh, so restarts of `scripts/gap-pool.py` do not lose data). '
      'Refresh with `python3 scripts/model-eval.py`.')
    w('')
    if recs:
        w(f'Records: {len(recs)} finished runs, latest {max(r["ts"] for r in recs)}. Models: '
          + ', '.join(f'`{m}`' for m in sorted(stats)) + '.')
    w('')
    w('## Verdict')
    w('')
    for ln in verdict(stats):
        w(ln)
    w('')
    w('Ranking needs ' + str(MIN_RUNS) + '+ finished runs per model. Throughput is successful runs per worker-hour '
      '(sum of run wall time), so lanes with different concurrency compare fairly. Quality is FIXED+VERIFIED rows '
      'per successful run, taken from each run\'s own `## Result` section.')
    w('')
    w('## Head to head')
    w('')
    w('`unknown` is a run that died before opencode printed its model header; it cannot be attributed to a model.')
    w('')
    w('| model | lane | runs | exit 0 | median s | avg s | p90 s | ok/worker-h | tool calls/run | edits/run | no-op ok runs | FIXED | VERIFIED | STILL OPEN | loops | fixed w/o edit |')
    w('|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|')
    for m in sorted(stats):
        s = stats[m]
        w(f'| `{m}` | {s["lane"]} | {s["n"]} | {pct(s["ok"], s["n"])} | {fmt(s["med_s"])} | {fmt(s["avg_s"])} | '
          f'{fmt(s["p90_s"])} | {fmt(s["per_wh"], 1)} | {s["tools"]:.1f} | {s["edit_avg"]:.2f} | {pct(s["noop"], s["ok"])} | '
          f'{s["fixed"]} | {s["verified"]} | {s["open"]} | {pct(s["loops"], s["n"])} | {s["fixed_no_edit"]} |')
    w('')
    w('## Run outcomes and error taxonomy')
    w('')
    w('Error counts are runs affected (raw line count in brackets). Matched only on status/error lines '
      '(`Error`, `✗`, `!` retry lines), never on source code shown in tool output.')
    w('')
    w('| model | ' + ' | '.join(ERR_CATS) + ' | limit/timeout requeues |')
    w('|---|' + '---:|' * (len(ERR_CATS) + 1))
    for m in sorted(stats):
        s = stats[m]
        cells = [f'{s["err_runs"].get(c, 0)} ({s["err_raw"].get(c, 0)})' for c in ERR_CATS]
        w(f'| `{m}` | ' + ' | '.join(cells) + f' | {s["limit_hits"]} |')
    w('')
    w('## Quality and reasoning')
    w('')
    w('| model | empty Result rate | missing Result | edit tool failures | files touched | prose loops | self-contradicting runs | revision markers/run | narration chars/run | reasoning blocks in log |')
    w('|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|')
    for m in sorted(stats):
        s = stats[m]
        w(f'| `{m}` | {pct(s["res_empty"], s["res_present"])} | {pct(s["res_missing"], s["ok"])} | {s["edit_fail"]} | '
          f'{s["files"]} | {pct(s["prose_repeat"], s["n"])} | {pct(s["contradict"], s["n"])} | '
          f'{s["revisions_avg"]:.2f} | {s["narr_avg"]:.0f} | {s["reasoning"]} |')
    w('')
    w('Heuristics: a run is a "no-op" when it exited 0 with no edit (`← Edit`/`← Write`, or a shell step that writes files). '
      '"Loop" is the same tool call three times in a row. "Fixed w/o edit" is a run with FIXED rows and no edit. '
      '"Self-contradicting" is a run that says no change was needed and then edited. "Reasoning blocks" looks for explicit '
      'thinking markers; opencode logs show only the visible narration, so narration length is the usable trace.')
    w('')
    w('## Last 30 runs')
    w('')
    w('| unit | model | secs | exit | FIXED | VERIFIED | STILL OPEN |')
    w('|---|---|---:|---:|---:|---:|---:|')
    for r in sorted(recs, key=lambda r: (r['mtime'], r['key']), reverse=True)[:30]:
        c = (r.get('result') or {}).get('counts') or {}
        w(f'| {r["unit"]} | `{r["model"]}` | {fmt(r["secs"])} | {r["exit"]} | {c.get("FIXED", 0)} | '
          f'{c.get("VERIFIED", 0)} | {c.get("STILL_OPEN", 0)} |')
    w('')
    return '\n'.join(out)


def main():
    recs, new_count = ingest()
    stats = model_stats(recs)
    text = render(recs, stats, new_count)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    prev = read_text(OUT)
    if prev != text:
        with open(OUT, 'w', encoding='utf8') as fh:
            fh.write(text)
    print(f'{len(recs)} runs in {EVAL} (+{new_count} new); wrote {OUT}')


if __name__ == '__main__':
    main()
