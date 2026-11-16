#!/usr/bin/env bash
# vendor.sh <git-url> [name]  -- shallow-clone a reference repo into .vendor/ (git-ignored, read-only reference).
source "$(dirname "$0")/lib.sh"; url="$1"; name="${2:-$(basename "$url" .git)}"
[ -d "$KAB_ROOT/.vendor/$name" ] && { echo "$KAB_ROOT/.vendor/$name"; exit 0; }
git clone --depth 1 "$url" "$KAB_ROOT/.vendor/$name" >&2 && echo "$KAB_ROOT/.vendor/$name"
