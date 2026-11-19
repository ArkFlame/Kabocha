#!/usr/bin/env bash
# bench.sh [record|compare] [crate...]
#   record  : run criterion with --save-baseline main, write benches/baselines/<crate>.json
#   compare : run criterion with --baseline main, fail if any mean regresses over 5 percent
# Default crates: pumpkin-protocol pumpkin-data. See docs/BENCHMARKS.md.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MODE="${1:-compare}"
[[ $# -gt 0 ]] && shift
CRATES=("$@")
[[ ${#CRATES[@]} -eq 0 ]] && CRATES=(pumpkin-protocol pumpkin-data)
THRESHOLD="0.05"
CRIT="$ROOT/target/criterion"

# Print the mean point_estimate from a criterion estimates.json (first "point_estimate" after "mean").
mean_of() {
    tr -d '\n' < "$1" | grep -o '"mean":.*' | grep -o '"point_estimate":[^,}]*' | head -1 | cut -d: -f2
}

case "$MODE" in
record)
    mkdir -p "$ROOT/benches/baselines"
    stamp="$(mktemp)"
    trap 'rm -f "$stamp"' EXIT
    for crate in "${CRATES[@]}"; do
        touch "$stamp"
        scripts/cargo.sh bench -p "$crate" -- --save-baseline main
        out="$ROOT/benches/baselines/$crate.json"
        {
            printf '{\n  "crate": "%s",\n  "unit": "ns",\n  "benchmarks": {\n' "$crate"
            first=1
            while IFS= read -r est; do
                rel="${est#"$CRIT"/}"
                name="${rel%/main/estimates.json}"
                val="$(mean_of "$est")"
                [[ $first -eq 1 ]] || printf ',\n'
                first=0
                printf '    "%s": %s' "$name" "$val"
            done < <(find "$CRIT" -path "*/main/estimates.json" -newer "$stamp" | sort)
            printf '\n  }\n}\n'
        } > "$out"
        echo "wrote $out"
    done
    ;;
compare)
    stamp="$(mktemp)"
    trap 'rm -f "$stamp"' EXIT
    for crate in "${CRATES[@]}"; do
        touch "$stamp"
        scripts/cargo.sh bench -p "$crate" -- --baseline main
        # Only change files written by this run, so stale results never count.
        regressions=0
        while IFS= read -r ch; do
            val="$(mean_of "$ch")"
            name="${ch#"$CRIT"/}"; name="${name%/change/estimates.json}"
            if awk -v v="$val" -v t="$THRESHOLD" 'BEGIN { exit !(v + 0 > t + 0) }'; then
                echo "REGRESSION $name: mean change $val > $THRESHOLD"
                regressions=$((regressions + 1))
            else
                echo "ok $name: mean change $val"
            fi
        done < <(find "$CRIT" -path "*/change/estimates.json" -newer "$stamp" | sort)
        if [[ $regressions -gt 0 ]]; then
            echo "FAILED: $regressions benchmark(s) in $crate regressed over 5 percent" >&2
            exit 1
        fi
    done
    echo "PASS: no benchmark regressed over 5 percent"
    ;;
*)
    echo "usage: scripts/bench.sh [record|compare] [crate...]" >&2
    exit 2
    ;;
esac
