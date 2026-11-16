---
name: kabocha-triage
description: Classify an upstream Pumpkin issue or PR into a sized, deduped queue entry with a plan.
---
# Triage
- Dedupe: same symptom/PR already in QUEUE.md? Link, do not duplicate. Check `git log upstream/master` for an existing fix.
- Size: S (<100 lines, one crate), M (<500, <=3 crates), L (more; split into ordered stages first).
- Type: crash/build > protocol/data > parity > feature > perf. Mark invalid/won't-fix with a one-line reason.
- For parity items, name the vanilla class to decompile (see kabocha-vanilla-port). For PRs: conflicts? (`pr-branch.sh`), CI scope (`.github/scripts/ci_packages.py`).
- Output a QUEUE.md row: `id | source | size | files | plan | status`.
