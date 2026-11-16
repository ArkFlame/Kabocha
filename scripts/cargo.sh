#!/usr/bin/env bash
# cargo.sh <cargo args>  -- the ONLY way agents run cargo: shared warm target, 2 slots, half the cores, waits for free RAM.
source "$(dirname "$0")/lib.sh"; cargo_slot command cargo "$@"
