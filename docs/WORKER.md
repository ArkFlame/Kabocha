# MODE (2026-11-16, user rule): the ORCHESTRATOR evaluates each issue/PR itself (is the gap real, skip or commit) and only for items that need edits runs `scripts/impl.sh <issue|pr> <n> <guidance-file>` with EVALUATION + RECOMMENDATION + TECHNICAL GUIDANCE. Autonomous triage workers (oc-refill.sh, next-issue.sh) are paused.

# Worker rules (prepended to every brief; lessons from sessions)
Mistakes seen so far and the rule that fixes each:
1. Private cold `CARGO_TARGET_DIR` per worker, many parallel builds -> machine load 98. **Do not run cargo at all.** If you truly must, use `scripts/cargo.sh` only.
2. Workers waited minutes on slow builds, re-armed monitors, and sent the same "still running" report dozens of times. **Never poll, sleep, or arm monitors. Finish in one pass; report once.**
3. Staged merges left uncommitted while waiting for a build. **Always commit before reporting.** An unverified commit is fine; the orchestrator batch-compiles.
4. `git add -A` swept unrelated dirty files into commits. **Stage only the files you changed** (`git add <paths>`), never `-A` in a shared or dirty tree.
5. OpenCode workers burned runs on PRs already fixed or closed. **Triage is mechanical** (`scripts/pr-triage.sh`): only CONFLICT PRs reach workers. If you still find the effect already present, say SKIPPED with file:line evidence.
6. Old PRs target pre-refactor paths (`crates/pumpkin/src/...` now `crates/pumpkin-core/src/...`). **Port the intent to the current file layout**; do not resurrect old files.
7. Two PRs can clash when merged together (new struct field vs new initializer, Arc vs Weak). **When you add a struct field or change a signature, grep every constructor/caller** and update them in the same commit.
8. Typos in PR text (e.g. a key like `shearing_sound_sound`): prefer the existing correct code.
12. **Integrate with judgement, never blindly.** Before editing, check that the issue/PR concern actually fills the gap required: reproduce the gap in the current code (file:line), confirm master does not already cover it, and confirm the proposed change fixes that exact gap (not a neighbouring one) and matches vanilla/Paper. If the gap is already filled, SKIP with file:line proof; if the PR only partly fits, port only the part that fills the gap and report partial naming what was left out.
13. **No runtime repro is not a failure reason by itself.** Compare the code path with vanilla/Paper/.vendor and with pumpkin-data; if it already matches, report SKIPPED with file:line. Report FAILED only for a concrete blocker (needs a seed, screenshot, client, or a multi-file redesign) and name it, since `scripts/oc-collect.sh` classifies it into docs/OUTCOMES.tsv and the item is not retried.
Report format (one message, no prose): `STATUS: committed|partial|skipped|failed | PR #N | commit <hash> | files <list> | compiled: no | note <one line>`. committed = on your branch only, never master.

## Brief structure (prompt engineering, implemented in scripts/brief-pr.sh)
XML-tagged sections in this order: `<role>` (one-line persona + "alone, one pass"), `<context>` (repo, PR number/title/files, known refactor pitfall, what to read), `<task>` (numbered, with the skip branch), `<constraints>` (explicit don'ts: no cargo, no polling, minimum change, no markers), `<output>` (exact one-line format plus a worked example). Facts the orchestrator already knows (files, title, layout move) are injected so the worker does not rediscover them. Negative instructions are specific and checkable. Keep briefs under ~2.5 KB.
9. Before committing, view your own diff (`git diff --cached`) and delete any stray fragments (lone `}`, `>`, orphan `#[test]`, markers): they break the batch build.

## Read budget and stop rule (user-mandated 2026-11-16)
Read only what the fix needs: grep -n first, then offset/limit around the hit, about 8 reads total, never whole large files. The moment the blocker is "needs a client/runtime repro, a video, a world sample, codegen, or already fixed", stop and report skipped/failed with one line of proof. Evidence: ~65% of issue workers ended skipped/failed after 40-70k tokens of reading.
10. (2026-11-16) Issue workers wasted slots on CLOSED issues, repeated `gh issue view`, `find /` over the whole disk and fetching upstream fix commits. **next-issue.sh now skips CLOSED issues; never `find /`; search only the worktree, `.vendor/` and `../`; fetch the issue once; if it is already fixed, report SKIPPED with file:line and stop.**
11. (2026-11-16) 15 of 20 workers hung on `ECONNRESET`, idle 28 min, repeating one command up to 20 times. **oc-agent.sh now runs each worker in its own process group, kills it after 300 s idle, retries once on ECONNRESET and always writes .exit. Briefs carry a 25-tool-call cap; the third identical command/read means stop.**

## Never search the parent directory
Never run grep/find/rg on `../` or `../..`: it contains every worker worktree (hundreds of GB) and saturates the disk for everyone. Search only the current worktree, `.vendor/`, or one named sibling repo (e.g. `../Pumpkin-upstream/`).

## Vanilla reference
`docs/VANILLA_REF.md` is a condensed, non-decompiled numeric digest for entities/blocks/items in the parity tracker. Use it to cross-check values; decompiled vanilla and `assets/` win on any disagreement.

## Shell is denied here
The plain shell tool returns "Permission denied: shell". Run every command (git, gh, grep, sed) through `ctx_execute` with language `shell`; never retry the plain shell tool. If `ctx_execute` is also denied, report BLOCKED once and stop.
