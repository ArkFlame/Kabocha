#!/usr/bin/env python3
"""Name-aware id audit: crate id per (state, direction, packet) vs minecraft-data.
usage: audit-ids-by-name.py <minecraft-data-dir>  (needs docs/multiversion/audit/era-*.tsv name map)"""
import csv, glob, json, re, collections, os, sys
R = os.path.dirname(os.path.dirname(os.path.abspath(__file__))); MD = sys.argv[1] + "/data"
nm = collections.defaultdict(set)
for f in glob.glob(R + "/docs/multiversion/audit/era-*.tsv"):
    for r in csv.DictReader(open(f), delimiter="\t"):
        m = re.match(r"([a-z0-9_]+) \[([^\]]+)\]", r.get("packet") or "")
        if m:
            for c in re.split(r"[/ ,]+", m.group(2)):
                if re.fullmatch(r"[A-Z0-9_]+", c): nm[(r["state"], r["direction"], m.group(1))].add(c)
src = open(R + "/crates/pumpkin-multiversion/src/legacy/generated_ids.rs").read()
T = {}; d = st = None
for m in re.finditer(r"pub mod (\w+) \{|pub const (\w+): super::super::PacketId = (?:super::super::PacketId\(\[([^\]]*)\]\)|(\w+));", src):
    if m.group(1):
        if m.group(1) in ("serverbound", "clientbound"): d = m.group(1)
        else: st = m.group(1)
    elif m.group(3) is not None: T[(st, d, m.group(2))] = [int(x) for x in m.group(3).replace("\n", " ").split(",") if x.strip()]
    else: T[(st, d, m.group(2))] = T.get((st, d, m.group(4)))
tsv = [l.split("\t") for l in open(R + "/docs/VERSION_SUPPORT.tsv")][1:]
dp = json.load(open(MD + "/dataPaths.json"))["pc"]; byp = {}
for v in dp:
    try: byp.setdefault(json.load(open(MD + "/" + dp[v]["version"] + "/version.json"))["version"], v)
    except Exception: pass
# minecraft-data quirks: vanilla order differs (1.21.5 spectate/test_instance_block_action; split player_info since 1.19.3)
SKIP = {("play", "serverbound", "spectate"), ("play", "clientbound", "player_info")}
checked = 0; bad = []
for i, r in enumerate(tsv):
    ref = byp.get(int(r[1]))
    if not ref: continue
    j = json.load(open(MD + "/" + dp[ref]["protocol"] + "/protocol.json"))
    for stt, sd in j.items():
        if not isinstance(sd, dict) or stt == "types": continue
        s2 = {"handshaking": "handshake", "configuration": "config"}.get(stt, stt)
        for dr, k in (("clientbound", "toClient"), ("serverbound", "toServer")):
            pk = sd.get(k, {}).get("types", {}).get("packet")
            for t in (pk[1] if pk else []):
                if t["name"] != "name": continue
                for h, n in t["type"][1]["mappings"].items():
                    key = (s2, dr, n)
                    if key in SKIP or key not in nm: continue
                    vals = {T[(s2, dr, c)][i] for c in nm[key] if T.get((s2, dr, c))}
                    if not vals: continue
                    checked += 1
                    if int(h, 16) not in vals: bad.append((r[0], key, int(h, 16), sorted(vals)))
print("checked", checked, "mismatches", len(bad))
for b in bad: print(b)
sys.exit(1 if bad else 0)
