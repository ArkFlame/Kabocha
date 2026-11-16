#!/usr/bin/env bash
# int-batch.sh <start> <count> -- integrate lines start..start+count-1 of $KAB_RUNS/unlanded.txt in ONE integrate.sh run.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
mapfile -t B < <(sed -n "$1,$(($1+$2-1))p" "$KAB_RUNS/unlanded.txt")
exec scripts/integrate.sh "${B[@]}"
