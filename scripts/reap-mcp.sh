#!/usr/bin/env bash
# reap-mcp.sh -- kill orphaned legal-ar MCP node processes (no live opencode ancestor). opencode global config starts
# ~12 per run and leaks them when a run is killed. Safe: only matches mcp-legal-ar; spares trees under a live opencode/claude.
n=0
for p in $(pgrep -f 'mcp-legal-ar/servers' 2>/dev/null); do
  a=$p; alive=0
  for _ in $(seq 1 25); do a=$(ps -o ppid= -p "$a" 2>/dev/null | tr -d ' '); [ -z "$a" ] || [ "$a" -le 1 ] && break
    case "$(ps -o comm= -p "$a" 2>/dev/null)" in opencode|claude) alive=1; break;; esac; done
  [ "$alive" = 0 ] && { kill -KILL "$p" 2>/dev/null; n=$((n+1)); }
done
echo "reaped $n orphan mcp procs; left $(pgrep -fc 'mcp-legal-ar/servers')"
