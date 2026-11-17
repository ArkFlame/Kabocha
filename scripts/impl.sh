#!/usr/bin/env bash
# impl.sh <issue|pr> <number> <guidance-file>  -- orchestrator-verified mode: launch ONE OpenCode implementer for an item the
# orchestrator already evaluated. The guidance file holds: EVALUATION (why it is a real gap), RECOMMENDATION (the exact change)
# and TECHNICAL GUIDANCE (files, functions, snippets, test). The worker implements it; it does not re-evaluate or skip.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
kind="$1"; n="$2"; g="$3"; [ -s "$g" ] || { echo "guidance file missing: $g" >&2; exit 1; }
name=$([ "$kind" = pr ] && echo "oc$n" || echo "i$n"); scripts/wt.sh new "$name" >/dev/null 2>&1 || true
f="$KAB_RUNS/brief-$name.md"; label=$([ "$kind" = pr ] && echo "upstream PR #$n" || echo "upstream issue #$n")
cat > "$f" <<B
<role>You are a Rust engineer implementing a change that was already evaluated and designed. Work alone, one pass.</role>
<context>Repo: Kabocha (Pumpkin Minecraft server fork, Rust). Your cwd is your own git worktree; never leave it. Read docs/WORKER.md once. This is $label.</context>
<guidance>
$(cat "$g")
</guidance>
<task>
1. Implement exactly the RECOMMENDATION following the TECHNICAL GUIDANCE. The evaluation is done: do not re-triage or skip. If the guidance is wrong (named symbol or file missing), make the closest correct change and say so in the note.
2. Update every caller of anything you change; add the test the guidance names (write it, do not run it).
3. Commit: git add <your files only>; git commit -m "<type>(<scope>): <subject> (#$n)" (subject only; the trailer is automatic).
</task>
<constraints>Do NOT run cargo/tests/clippy, no background jobs, no questions, no find /. About 25 tool calls max. Do not touch other worktrees.</constraints>
<report>One line: STATUS: committed|failed | $( [ "$kind" = pr ] && echo PR || echo ISSUE ) #$n | commit <hash> | files <list> | compiled: no | note <one sentence></report>
B
echo "$n" >> "$KAB_RUNS/$([ "$kind" = pr ] && echo tried || echo tried-issues)"
scripts/oc-agent.sh "$name" "$f" 9>&-
