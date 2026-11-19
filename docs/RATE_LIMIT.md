# Java connection rate limiting

Pumpkin limits how fast a single source IP can open connections, query the status, or try to join. It replaces the old Spigot-style `connection_throttle` key, which is now ignored.

The limiter is in-process. It follows iptables `hashlimit` and `ipset` semantics, but it never touches the system `iptables` or `ipset`.

## Configuration

```toml
[networking.java.rate_limit]
enabled = true
# Each rule is "hits/seconds" per source IP. ALL rules in a list must pass (sliding window).
connections = ["20/1", "60/10"]   # every new TCP connection, any kind
status      = ["10/1"]            # server-list pings
logins      = ["2/1", "6/10"]     # join attempts (login and transfer intents)
exempt      = ["127.0.0.1", "::1"] # IPs or CIDR ranges that are never limited

[networking.java.rate_limit.ipset]
enabled = true            # ban an IP after repeated violations
ban_after = 10            # rejected hits within ban_window_seconds before the ban
ban_window_seconds = 10
ban_seconds = 60          # banned IPs are dropped at accept, before any work
max_entries = 65536       # bound on tracked IPs and bans
```

- A rule is `hits/seconds`. `hits` is `1..=100000` and `seconds` is `1..=3600`.
- An invalid rule or exempt entry logs one warning at startup and is skipped.
- Rejected hits are not recorded, so they never extend the window.

## Behaviour

- **Connections** are checked right after accept, before any other work.
- **Status and logins** are checked when the handshake declares its intent.
- A rejected hit closes the TCP connection with no response packet and logs at debug level.
- Banned IPs are dropped at accept. A ban is logged once at info level.
- Every rejected hit counts towards the ban. Hits from an IP that is already banned do not count.
- Expired state is pruned periodically. When `max_entries` is reached, the least recently seen IPs are evicted.

## Exemptions and NAT

Many players can share one public IP behind NAT or a proxy. If they hit the limit, either:

- raise the `logins` or `connections` rules, or
- add the proxy or NAT address to `exempt`, using an IP or a CIDR range such as `"10.0.0.0/8"`.

Set `enabled = false` to turn off all limiting and banning.

Bedrock connections are not covered by this limiter.
