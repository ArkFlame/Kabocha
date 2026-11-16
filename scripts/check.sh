#!/usr/bin/env bash
# check.sh [-q] <crate>...  -- fmt + clippy + tests on touched crates. -q = check only (fast loop).
# With no crates: derive from `git diff master` via .github/scripts/ci_packages.py-like path mapping.
source "$(dirname "$0")/lib.sh"; cd "${KAB_DIR:-$PWD}"
quick=0; [ "${1:-}" = "-q" ] && { quick=1; shift; }
crates=("$@")
if [ ${#crates[@]} -eq 0 ]; then
  mapfile -t crates < <(git diff --name-only master... | sed -n 's#^crates/\([^/]*\)/.*#\1#p;s#^tools/\([^/]*\)/.*#\1#p' | sort -u)
fi
[ ${#crates[@]} -eq 0 ] && { echo "no crates touched"; exit 0; }
pf=(); for c in "${crates[@]}"; do pf+=(-p "$c"); done
cargo fmt --check
if [ $quick = 1 ]; then cargo check "${pf[@]}" --all-targets --locked; exit; fi
cargo clippy "${pf[@]}" --all-targets --all-features --locked -- -D warnings
if command -v cargo-nextest >/dev/null; then cargo nextest run "${pf[@]}" --no-tests=pass; else cargo test "${pf[@]}"; fi
echo "CHECK OK: ${crates[*]}"
