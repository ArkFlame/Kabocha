#!/usr/bin/env bash
# auto-commit.sh [interval=120] -- realtime patch organization while workers edit the shared checkout: every cycle, commit pending
# changes in one commit per area (also files other workers touched) and push master. Stops when /tmp/kabocha-runs/auto-commit.stop exists.
cd "${KAB_ROOT:-$PWD}" || exit 1; N=${1:-120}; export KAB_AGENT=opencode
commit() { local msg="$1"; shift; git add -- "$@" 2>/dev/null; git diff --cached --quiet || git commit -q -m "$msg" && echo "$(date +%T) $(git log --oneline -1)"; }
while [ ! -e /tmp/kabocha-runs/auto-commit.stop ]; do
  if [ -n "$(git status --porcelain)" ]; then
    for a in entity block item command net world data server plugin enchantment; do commit "fix($a): close $a parity gaps from pass 2 workers" "crates/pumpkin-core/src/$a"; done
    commit "fix(net): close protocol parity gaps from pass 2 workers" crates/pumpkin-protocol crates/pumpkin-config
    commit "fix(world): close world and command parity gaps from pass 2 workers" crates/pumpkin-world crates/pumpkin-command crates/pumpkin-gametest crates/pumpkin-plugin-api crates/pumpkin-plugin-wit crates/pumpkin-wasm-host
    commit "fix(core): close remaining core parity gaps from pass 2 workers" crates
    commit "docs(tracker): record pass 2 worker results" docs scripts TRACKER.md
    git add -A; git diff --cached --quiet || git commit -q -m "chore: record pass 2 worker output"
    timeout 100 git push -q origin master 2>&1 | tail -1
  fi
  sleep "$N"
done
