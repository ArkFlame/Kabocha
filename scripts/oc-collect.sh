#!/usr/bin/env bash
# oc-collect.sh -- idempotent: turn every worker STATUS line (failed|skipped) into a classified row in docs/OUTCOMES.tsv
# (kind number class status reason worker), so failures are tracked, never retried blindly, and feed the ledger.
# class: needs-runtime (seed/screenshot/client/sample needed) | already-fixed | too-large | other.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"; o=docs/OUTCOMES.tsv; touch "$o"
for f in "$KAB_RUNS"/*.out; do
  w=$(basename "$f" .out)
  line=$(sed 's/\x1b\[[0-9;]*m//g' "$f" | grep -E '^STATUS: *(failed|skipped)' | tail -1) || continue
  [ -n "$line" ] || continue
  st=$(echo "$line" | sed -E 's/^STATUS: *([a-z]+).*/\1/'); k=issue; echo "$line" | grep -q 'PR #' && k=pr
  n=$(echo "$line" | grep -oE '(ISSUE|PR) #[0-9]+' | head -1 | tr -dc 0-9); [ -n "$n" ] || continue
  grep -qP "^$k\t$n\t" "$o" && continue
  r=$(echo "$line" | sed -E 's/.*note *//' | tr '\t' ' ' | cut -c1-300)
  case "$r" in
    *seed*|*runtime*|*repro*|*screenshot*|*client*|*sample*|*coordinates*|*in-game*) c=needs-runtime;;
    *already*|*present*|*exists*|*on\ master*|*in\ HEAD*) c=already-fixed;;
    *multi*|*large*|*redesign*|*"one pass"*) c=too-large;;
    *) c=other;;
  esac
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$k" "$n" "$c" "$st" "$r" "$w" >> "$o"
done
echo "outcomes: $(wc -l <"$o") rows; $(cut -f3 "$o" | sort | uniq -c | tr '\n' ' ')"
