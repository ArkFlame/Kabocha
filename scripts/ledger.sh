#!/usr/bin/env bash
# Builds docs/LEDGER.tsv (+ .md): single source of truth for TODO vs DONE of
# every upstream issue and PR, from local files only. Idempotent.
set -euo pipefail
cd "$(dirname "$0")/.."
exec python3 - <<'PY'
import os, re, subprocess, datetime, collections

def readlines(p):
    try:
        with open(p, encoding="utf-8", errors="replace") as f:
            return f.read().splitlines()
    except FileNotFoundError:
        return []

today = datetime.date.today().isoformat()

prs, issues = {}, {}
for line in readlines("docs/queue-prs.tsv"):
    c = line.split("\t")
    if len(c) < 6 or not c[0].strip().isdigit():
        continue
    prs[int(c[0])] = {"title": c[5].strip(), "labels": c[6].strip() if len(c) > 6 else ""}
for line in readlines("docs/queue-issues.tsv"):
    c = line.split("\t")
    if len(c) < 2 or not c[0].strip().isdigit():
        continue
    issues[int(c[0])] = {"title": c[1].strip(), "labels": c[2].strip() if len(c) > 2 else ""}

done = {}      # n -> (state, evidence)
for i, line in enumerate(readlines("docs/DONE.md"), 1):
    nums = [int(m) for m in re.findall(r"#(\d+)", line)]
    if not nums:
        continue
    low = line.lower()
    if "skipped" in low or "already present" in low or "already handles" in low or "already" in low:
        state = "SKIPPED"
    else:
        state = "DONE"
    ev = "DONE.md:%d %s" % (i, line.strip())
    for n in nums:
        if n not in done or (done[n][0] == "SKIPPED" and state == "DONE"):
            done[n] = (state, ev)

queue_state = {}   # n -> (state, evidence)
for i, line in enumerate(readlines("docs/QUEUE.md"), 1):
    nums = [int(m) for m in re.findall(r"#(\d+)", line)]
    if not nums:
        continue
    low = line.lower()
    if "partial" in low:
        state = "BLOCKED"
    elif "blocked" in low:
        state = "BLOCKED"
    elif "failed" in low or "fail compile" in low or "dropped" in low:
        state = "FAILED"
    else:
        continue
    ev = "QUEUE.md:%d %s" % (i, line.strip())
    for n in nums:
        queue_state.setdefault(n, (state, ev))

outcome_state = {}  # (kind, n) -> (state, evidence) from docs/OUTCOMES.tsv (worker STATUS lines)
try:
    for line in readlines("docs/OUTCOMES.tsv"):
        f = line.rstrip("\n").split("\t")
        if len(f) >= 5 and f[1].isdigit() and f[3] in ("skipped", "failed"):
            outcome_state[(f[0], int(f[1]))] = (
                "SKIPPED" if f[3] == "skipped" else "FAILED",
                "OUTCOMES.tsv %s/%s: %s" % (f[3], f[2], f[4][:120]))
except OSError:
    pass

master = {}        # n -> (hash, subject)
try:
    log = subprocess.run(["git", "log", "master", "--format=%h %s"],
                         capture_output=True, text=True, timeout=60).stdout
except Exception:
    log = ""
for line in log.splitlines():
    h, _, subj = line.partition(" ")
    for m in re.findall(r"\(#(\d+)\)", subj):
        n = int(m)
        if n not in master:
            master[n] = (h, subj)

branches = collections.defaultdict(list)   # n -> [branch]
try:
    refs = subprocess.run(["git", "for-each-ref", "refs/heads/kab",
                           "--format=%(refname:short)"],
                          capture_output=True, text=True, timeout=60).stdout
except Exception:
    refs = ""
for b in refs.split():
    m = re.fullmatch(r"kab/(?:i|n|oc)(\d+)", b)
    if m:
        branches[int(m.group(1))].append(b)

try:
    unmerged = set(subprocess.run(["git", "for-each-ref", "--no-merged=master",
                                   "refs/heads/kab", "--format=%(refname:short)"],
                                  capture_output=True, text=True, timeout=60).stdout.split())
except Exception:
    unmerged = set()

running = set()
runs = "/tmp/kabocha-runs"
for fn in sorted(os.listdir(runs)) if os.path.isdir(runs) else []:
    m = re.fullmatch(r"(?:i|n|oc)(\d+)\.pid", fn)
    if not m:
        continue
    n = int(m.group(1))
    if os.path.exists(os.path.join(runs, fn[:-4] + ".exit")):
        continue
    try:
        pid = int((open(os.path.join(runs, fn)).read().strip() or "0"))
        os.kill(pid, 0)
    except (ValueError, ProcessLookupError, PermissionError, FileNotFoundError):
        continue
    running.add(n)

rows = []
for kind, table, url in (("pr", prs, "pull"), ("issue", issues, "issues")):
    for n in sorted(table):
        st = "TODO"
        branch = ""
        commit = ""
        ev = ""
        if n in running:
            st, ev = "IN-PROGRESS", "worker pid live in /tmp/kabocha-runs"
        elif n in master:
            st, commit, ev = "DONE", master[n][0], master[n][1]
        elif n in done and done[n][0] == "DONE":
            st, ev = "DONE", done[n][1]
        elif n in done and done[n][0] == "SKIPPED":
            st, ev = "SKIPPED", done[n][1]
        elif n in queue_state:
            st, ev = queue_state[n]
        elif (kind, n) in outcome_state and not [x for x in branches.get(n, []) if x in unmerged]:
            st, ev = outcome_state[(kind, n)]
        else:
            cands = [b for b in branches.get(n, []) if b in unmerged]
            if cands:
                st, branch = "COMMITTED-UNLANDED", " ".join(cands)
                ev = "branch ahead of master"
        if not commit and n in done and done[n][0] == "DONE":
            m = re.search(r"\[([0-9a-f]{7,40})\]", done[n][1])
            if m:
                commit = m.group(1)
        rows.append((kind, n, "https://github.com/Pumpkin-MC/Pumpkin/%s/%d" % (url, n),
                     table[n]["title"], st, branch, commit, ev, today))

hdr = ["kind", "number", "url", "title", "state", "branch", "commit", "evidence", "updated"]
with open("docs/LEDGER.tsv", "w", encoding="utf-8") as f:
    f.write("\t".join(hdr) + "\n")
    for r in rows:
        f.write("\t".join(str(x).replace("\t", " ") for x in r) + "\n")

states = ["IN-PROGRESS", "DONE", "SKIPPED", "FAILED", "BLOCKED",
          "COMMITTED-UNLANDED", "TODO"]
counts = {k: collections.Counter() for k in ("pr", "issue")}
for kind, n, *_ , st, _, _, _, _ in rows:
    counts[kind][st] += 1

def pct(kind):
    tot = sum(counts[kind].values())
    if not tot:
        return "0.0"
    return "%.1f" % (100.0 * (counts[kind]["DONE"] + counts[kind]["SKIPPED"]) / tot)

lines = ["| state | PR | issue |", "|---|---|---|"]
for s in states:
    lines.append("| %s | %d | %d |" % (s, counts["pr"][s], counts["issue"][s]))
lines.append("| total | %d | %d |" % (sum(counts["pr"].values()), sum(counts["issue"].values())))
lines.append("| DONE+SKIPPED %% | %s | %s |" % (pct("pr"), pct("issue")))

md = ["# Ledger", "",
      "Generated by `scripts/ledger.sh` on " + today + ". Do not edit by hand. "
      "Source of truth: `docs/LEDGER.tsv`.", "",
      "## Counts", ""] + lines + [""]
for want in ("COMMITTED-UNLANDED", "FAILED", "BLOCKED"):
    sel = [r for r in rows if r[4] == want]
    md.append("## %s (%d)" % (want, len(sel)))
    md.append("")
    if not sel:
        md.append("None.")
    for r in sel:
        md.append("- %s %s%s | %s" % (
            r[0].upper(), "#%d" % r[1],
            (" | " + r[5]) if r[5] else "",
            r[7] or r[3]))
    md.append("")
with open("docs/LEDGER.md", "w", encoding="utf-8") as f:
    f.write("\n".join(md))

print("\n".join(lines))
PY
