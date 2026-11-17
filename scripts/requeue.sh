#!/usr/bin/env bash
# requeue.sh -- make unfinished work eligible again so worker pools never run dry.
# Candidates: ledger rows TODO/FAILED (max KAB_MAX_ATTEMPTS tries) with no running worker. Stale exit/pid files and dead branches are cleared.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
R="$KAB_RUNS"; MAX="${KAB_MAX_ATTEMPTS:-4}"; touch "$R/tried" "$R/tried-issues" "$R/attempts"
exec 7>"$R/requeue.lock"; flock -n 7 || exit 0
running=$(scripts/oc-status.sh | sed -n 's/ running$//p')
awk -F'\t' 'NR>1&&($4=="TODO"||$4=="FAILED"){print $1" "$2}' docs/LEDGER.tsv | while read k n; do
  p=i; f="$R/tried-issues"; [ "$k" = pr ] && { p=oc; f="$R/tried"; }
  grep -qx "$p$n" <<<"$running" && continue
  grep -qx "$n" "$f" || continue                      # already eligible
  a=$(grep -cx "$p$n" "$R/attempts"); [ "$a" -ge "$MAX" ] && continue
  echo "$p$n" >> "$R/attempts"
  sed -i "/^$n\$/d" "$f"; rm -f "$R/$p$n.exit" "$R/$p$n.pid"
  git branch -D "kab/$p$n" >/dev/null 2>&1; echo "requeued $p$n"
done | tail -200 | wc -l
