#!/usr/bin/env bash
# sync-upstream.sh  -- fetch upstream and report ahead/behind; merges nothing.
source "$(dirname "$0")/lib.sh"; git -C "$KAB_ROOT" fetch upstream --quiet
echo "master...upstream/master (ours theirs): $(git -C "$KAB_ROOT" rev-list --left-right --count master...upstream/master)"
