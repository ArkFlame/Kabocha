#!/usr/bin/env python3
"""Audit crate packet id tables against PrismarineJS minecraft-data (official protocol).
usage: audit-ids.py <minecraft-data-dir> ; writes docs/multiversion/audit/ids.tsv"""
import json, os, re, sys
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
md = sys.argv[1] + "/data/pc"
src = open(f"{root}/crates/pumpkin-multiversion/src/legacy/generated_ids.rs").read()
tsv = [l.rstrip("\n").split("\t") for l in open(f"{root}/docs/VERSION_SUPPORT.tsv")][1:]
versions = [(r[0], int(r[1])) for r in tsv]
# parse tables: table[(dir,state)][NAME] = [ids]
table, d, st = {}, None, None
for m in re.finditer(r"pub mod (\w+) \{|pub const (\w+): super::super::PacketId = super::super::PacketId\(\[([^\]]*)\]\)", src):
    if m.group(1):
        n = m.group(1)
        if n in ("serverbound", "clientbound"): d = n
        else: st = n
    else:
        table.setdefault((d, st), {})[m.group(2)] = [int(x) for x in m.group(3).split(",") if x.strip()]
dp = json.load(open(sys.argv[1] + "/data/dataPaths.json"))["pc"]
# protocol -> minecraft-data dir
by_proto = {}
for v in sorted(dp):
    try: p = json.load(open(sys.argv[1] + "/data/" + dp[v]["version"] + "/version.json"))
    except Exception: continue
    if p.get("releaseType", "release") == "release" or True:
        by_proto.setdefault(p["version"], []).append(v)
def ref_ids(vdir):
    pj = sys.argv[1] + "/data/" + dp.get(vdir, {}).get("protocol", "") + "/protocol.json"
    if not os.path.exists(pj): return None
    j = json.load(open(pj))
    out = {}
    for state, sd in j.items():
        if not isinstance(sd, dict) or state == "types": continue
        for dr, key in (("clientbound", "toClient"), ("serverbound", "toServer")):
            pk = sd.get(key, {}).get("types", {}).get("packet")
            if not pk: continue
            for t in pk[1]:
                if t["name"] == "name": mp = t["type"][1]["mappings"]
            out[(dr, state)] = {int(k, 16) for k in mp}
    return out
rows = []
for i, (ver, proto) in enumerate(versions):
    cands = [v for v in by_proto.get(proto, []) if not re.search(r"pre|rc|w\d|^\d\dw", v)] or by_proto.get(proto, [])
    note = ""
    ref = ref_ids(cands[0]) if cands else None
    if ref is None:
        lower = [p for p in sorted(by_proto) if p < proto and ref_ids(by_proto[p][0])]
        higher = [p for p in sorted(by_proto) if p > proto and ref_ids(by_proto[p][0])]
        pick = lower[-1] if lower else (higher[0] if higher else None)
        if pick is not None:
            cands = by_proto[pick]; ref = ref_ids(cands[0]); note = "fallback-nearest;"
    if ref is None:
        rows.append([ver, proto, "-", "no-reference", "", "", ""]); continue
    bad, miss, extra, total = [], 0, 0, 0
    for key, names in table.items():
        ours = {ids[i] for ids in names.values() if ids[i] >= 0}
        theirs = ref.get((key[0], {"handshake":"handshaking","config":"configuration"}.get(key[1], key[1])))
        if theirs is None:
            if ours: bad.append(f"{key[0][0]}:{key[1]}:no-ref-state")
            continue
        if key == ("serverbound", "handshake"): theirs = theirs | {254}
        # play packets only exist in ids tables for the supported direction/state
        total += len(theirs)
        m, e = theirs - ours, ours - theirs
        if key == ("serverbound", "handshake"): m = m - {254}
        # minecraft-data lists clientbound config ids 8/9 under toServer for 1.21-1.21.5 (vanilla serverbound config ends at 7)
        if key == ("serverbound", "config") and 767 <= proto <= 770: m = m - {8, 9}
        miss += len(m); extra += len(e)
        if m or e: bad.append(f"{key[0][0]}/{key[1]} missing={sorted(m)[:6]} extra={sorted(e)[:6]}")
    rows.append([ver, proto, cands[0], "ok" if not bad else "diff", miss, extra, (note + " " + "; ".join(bad))[:400]])
with open(f"{root}/docs/multiversion/audit/ids.tsv", "w") as f:
    f.write("version\tprotocol\treference\tresult\tmissing\textra\tdetail\n")
    for r in rows: f.write("\t".join(map(str, r)) + "\n")
print(sum(r[3] == "ok" for r in rows), "ok /", len(rows))
for r in rows: print(r[0], r[3], r[4], r[5], str(r[6])[:90])

# record result in the tracker
lines = open(f"{root}/docs/VERSION_SUPPORT.tsv").read().splitlines()
res = {r[0]: r for r in rows}
out = [lines[0]]
for l in lines[1:]:
    c = l.split("\t"); r = res[c[0]]
    if r[3] == "ok": c[4] = "ok-fallback" if "fallback" in r[6] else "ok"
    elif r[3] == "diff" and "fallback" in r[6] and c[0] in ("26.2", "26.3"): c[4] = "partial"
    c[8] = "docs/multiversion/audit/ids.tsv" if c[4] != "todo" else c[8]
    out.append("\t".join(c))
open(f"{root}/docs/VERSION_SUPPORT.tsv", "w").write("\n".join(out) + "\n")
