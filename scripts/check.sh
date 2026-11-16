#!/usr/bin/env bash
# check.sh [-f] <crate>...  -- default: fmt + cargo check -p (fast). -f: also clippy -D warnings + nextest (before landing).
# With no crates: derive from `git diff master` via .github/scripts/ci_packages.py-like path mapping.
source "$(dirname "$0")/lib.sh"; cd "${KAB_DIR:-$PWD}"
quick=1; [ "${1:-}" = "-f" ] && { quick=0; shift; }; [ "${1:-}" = "-q" ] && shift
crates=("$@")
if [ ${#crates[@]} -eq 0 ]; then
  mapfile -t crates < <(git diff --name-only master... | sed -n 's#^crates/\([^/]*\)/.*#\1#p;s#^tools/\([^/]*\)/.*#\1#p' | sort -u)
fi
[ ${#crates[@]} -eq 0 ] && { echo "no crates touched"; exit 0; }
pf=(); for c in "${crates[@]}"; do pf+=(-p "$c"); done
cargo fmt --check
cargo() { cargo_slot command cargo "$@"; }
if [ $quick = 1 ]; then cargo check "${pf[@]}" --all-targets --locked; exit; fi
cargo clippy "${pf[@]}" --all-targets --all-features --locked -- -D warnings
if command -v cargo-nextest >/dev/null; then cargo nextest run "${pf[@]}" --no-tests=pass; else cargo test "${pf[@]}"; fi
echo "CHECK OK: ${crates[*]}"
