//! Per-IP hit-rate limiter with an in-process ban list.
//!
//! Modelled on iptables `hashlimit` (sliding windows, dropped hits do not count) and an
//! `ipset` with timeout (repeated violations ban an IP for a fixed time). This is pure state:
//! every decision takes the current `Instant`, so the logic is deterministic under test.
//! It never touches the system `iptables` or `ipset`.

use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use pumpkin_config::RateLimitConfig;
use tracing::{info, warn};

const MIN_HITS: u32 = 1;
const MAX_HITS: u32 = 100_000;
const MIN_WINDOW_SECS: u64 = 1;
const MAX_WINDOW_SECS: u64 = 3600;
const MAX_BAN_SECS: u64 = 365 * 24 * 60 * 60;
/// Expired state is swept once every this many checks, which keeps the cost amortised.
const PRUNE_INTERVAL: u64 = 256;
const KINDS: usize = 3;

/// Which kind of hit is being checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Any new TCP connection, before any work is done.
    Connection,
    /// A server-list status ping.
    Status,
    /// A join attempt (login or transfer intent).
    Login,
}

impl Kind {
    const fn index(self) -> usize {
        match self {
            Self::Connection => 0,
            Self::Status => 1,
            Self::Login => 2,
        }
    }
}

/// Outcome of a rate-limit check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The hit is allowed and recorded.
    Allow,
    /// The hit is rejected. The caller closes the connection without a response.
    Drop,
}

/// One `hits/seconds` rule. A hit passes only when fewer than `hits` allowed hits fall in `window`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    /// Maximum allowed hits within `window`.
    pub hits: u32,
    /// Sliding window length.
    pub window: Duration,
}

impl Rule {
    /// Parses `"hits/seconds"`. Hits must be `1..=100000` and seconds `1..=3600`.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let (hits, secs) = raw.trim().split_once('/')?;
        let hits: u32 = hits.trim().parse().ok()?;
        let secs: u64 = secs.trim().parse().ok()?;
        ((MIN_HITS..=MAX_HITS).contains(&hits)
            && (MIN_WINDOW_SECS..=MAX_WINDOW_SECS).contains(&secs))
        .then(|| Self {
            hits,
            window: Duration::from_secs(secs),
        })
    }
}

/// An IP address or `ip/prefix` range, used for exemptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cidr {
    addr: IpAddr,
    prefix: u8,
}

impl Cidr {
    /// Parses `"203.0.113.7"`, `"10.0.0.0/8"` or `"::1"`.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let (ip_part, prefix_part) = match raw.split_once('/') {
            Some((ip, prefix)) => (ip, Some(prefix)),
            None => (raw, None),
        };
        let addr: IpAddr = ip_part.trim().parse().ok()?;
        let max_prefix = if addr.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix_part {
            Some(prefix) => prefix
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|p| *p <= max_prefix)?,
            None => max_prefix,
        };
        Some(Self { addr, prefix })
    }

    /// Whether `ip` lies inside this address or range.
    #[must_use]
    pub fn contains(self, ip: IpAddr) -> bool {
        match (self.addr, ip) {
            (IpAddr::V4(net), IpAddr::V4(ip)) => {
                let mask = u32::MAX
                    .checked_shl(u32::from(32 - self.prefix))
                    .unwrap_or(0);
                net.to_bits() & mask == ip.to_bits() & mask
            }
            (IpAddr::V6(net), IpAddr::V6(ip)) => {
                let mask = u128::MAX
                    .checked_shl(u32::from(128 - self.prefix))
                    .unwrap_or(0);
                net.to_bits() & mask == ip.to_bits() & mask
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BanPolicy {
    ban_after: u64,
    window: Duration,
    duration: Duration,
}

#[derive(Debug)]
struct IpState {
    /// Allowed hit times per kind, oldest first.
    hits: [VecDeque<Instant>; KINDS],
    /// Rejected hit times inside the ban window, oldest first.
    violations: VecDeque<Instant>,
    last_seen: Instant,
}

impl IpState {
    const fn new(now: Instant) -> Self {
        Self {
            hits: [VecDeque::new(), VecDeque::new(), VecDeque::new()],
            violations: VecDeque::new(),
            last_seen: now,
        }
    }
}

/// Per-IP rate limiter with an ipset-style ban list. Share it behind a `Mutex`.
#[derive(Debug)]
pub struct RateLimiter {
    enabled: bool,
    rules: [Vec<Rule>; KINDS],
    /// Longest window per kind. Hits older than this can no longer affect a decision.
    retention: [Duration; KINDS],
    exempt: Vec<Cidr>,
    ban: Option<BanPolicy>,
    max_entries: usize,
    states: HashMap<IpAddr, IpState>,
    /// Ban expiry per banned IP.
    bans: HashMap<IpAddr, Instant>,
    checks: u64,
}

impl RateLimiter {
    /// Builds the limiter from config. Invalid rules and exemptions are logged and skipped.
    #[must_use]
    pub fn from_config(config: &RateLimitConfig) -> Self {
        let rules = [
            parse_rules("connections", &config.connections),
            parse_rules("status", &config.status),
            parse_rules("logins", &config.logins),
        ];
        let retention = rules.each_ref().map(|list| {
            list.iter()
                .map(|rule| rule.window)
                .max()
                .unwrap_or(Duration::ZERO)
        });
        let exempt = config
            .exempt
            .iter()
            .filter_map(|raw| {
                let cidr = Cidr::parse(raw);
                if cidr.is_none() {
                    warn!("Rate limit: invalid exempt address `{raw}`, skipping it");
                }
                cidr
            })
            .collect();
        let ipset = &config.ipset;
        let ban = ipset.enabled.then(|| BanPolicy {
            ban_after: ipset.ban_after.max(1).into(),
            window: Duration::from_secs(
                ipset
                    .ban_window_seconds
                    .clamp(MIN_WINDOW_SECS, MAX_WINDOW_SECS),
            ),
            duration: Duration::from_secs(ipset.ban_seconds.clamp(1, MAX_BAN_SECS)),
        });
        Self {
            enabled: config.enabled,
            rules,
            retention,
            exempt,
            ban,
            max_entries: usize::try_from(ipset.max_entries)
                .unwrap_or(usize::MAX)
                .max(1),
            states: HashMap::new(),
            bans: HashMap::new(),
            checks: 0,
        }
    }

    /// Checks one hit from `ip` of `kind` at time `now`.
    ///
    /// Allowed hits are recorded. Dropped hits are not, so they never extend the window.
    /// Exempt IPs are always allowed. Banned IPs are always dropped and do not add violations.
    /// Every drop counts towards the ban.
    #[must_use]
    pub fn check(&mut self, ip: IpAddr, kind: Kind, now: Instant) -> Verdict {
        if !self.enabled || self.is_exempt(ip) {
            return Verdict::Allow;
        }
        self.checks = self.checks.wrapping_add(1);
        if self.checks.is_multiple_of(PRUNE_INTERVAL) {
            self.prune(now);
        }
        if self.is_banned(ip, now) {
            return Verdict::Drop;
        }
        self.touch(ip, now);
        if self.allow_hit(ip, kind, now) {
            Verdict::Allow
        } else {
            self.record_violation(ip, now);
            Verdict::Drop
        }
    }

    fn is_exempt(&self, ip: IpAddr) -> bool {
        self.exempt.iter().any(|cidr| cidr.contains(ip))
    }

    fn is_banned(&mut self, ip: IpAddr, now: Instant) -> bool {
        let Some(until) = self.bans.get(&ip).copied() else {
            return false;
        };
        if now < until {
            return true;
        }
        self.bans.remove(&ip);
        false
    }

    /// Creates state for `ip` if needed, making room first so `max_entries` holds.
    fn touch(&mut self, ip: IpAddr, now: Instant) {
        if !self.states.contains_key(&ip) {
            if self.states.len() >= self.max_entries {
                self.prune(now);
                if self.states.len() >= self.max_entries {
                    self.evict_oldest_states();
                }
            }
            self.states.insert(ip, IpState::new(now));
        }
        if let Some(state) = self.states.get_mut(&ip) {
            state.last_seen = now;
        }
    }

    /// Drops the least recently seen quarter of tracked IPs. Runs only when the table is full.
    fn evict_oldest_states(&mut self) {
        let mut by_age: Vec<(Instant, IpAddr)> = self
            .states
            .iter()
            .map(|(ip, state)| (state.last_seen, *ip))
            .collect();
        by_age.sort_unstable();
        let drop_count = (self.max_entries / 4).max(1);
        for (_, ip) in by_age.into_iter().take(drop_count) {
            self.states.remove(&ip);
        }
    }

    fn allow_hit(&mut self, ip: IpAddr, kind: Kind, now: Instant) -> bool {
        let k = kind.index();
        if self.rules[k].is_empty() {
            return true;
        }
        let retention = self.retention[k];
        // Split the borrows: rules are read while the IP's hit history is mutated.
        let Self { rules, states, .. } = self;
        let Some(state) = states.get_mut(&ip) else {
            return true;
        };
        let hits = &mut state.hits[k];
        while hits
            .front()
            .is_some_and(|t| now.saturating_duration_since(*t) >= retention)
        {
            hits.pop_front();
        }
        let within_limits = rules[k].iter().all(|rule| {
            let used = hits
                .iter()
                .rev()
                .take_while(|t| now.saturating_duration_since(**t) < rule.window)
                .count();
            (used as u64) < u64::from(rule.hits)
        });
        if within_limits {
            hits.push_back(now);
        }
        within_limits
    }

    fn record_violation(&mut self, ip: IpAddr, now: Instant) {
        let Some(policy) = self.ban else {
            return;
        };
        let Some(state) = self.states.get_mut(&ip) else {
            return;
        };
        let violations = &mut state.violations;
        while violations
            .front()
            .is_some_and(|t| now.saturating_duration_since(*t) >= policy.window)
        {
            violations.pop_front();
        }
        violations.push_back(now);
        if (violations.len() as u64) < policy.ban_after {
            return;
        }
        violations.clear();
        self.ban_ip(ip, now, policy.duration);
    }

    fn ban_ip(&mut self, ip: IpAddr, now: Instant, duration: Duration) {
        if !self.bans.contains_key(&ip) && self.bans.len() >= self.max_entries {
            self.bans.retain(|_, until| now < *until);
            if self.bans.len() >= self.max_entries {
                let soonest = self
                    .bans
                    .iter()
                    .min_by_key(|(_, until)| **until)
                    .map(|(banned, _)| *banned);
                if let Some(banned) = soonest {
                    self.bans.remove(&banned);
                }
            }
        }
        self.bans.insert(ip, now + duration);
        info!("Rate limit: banned {ip} for {}s", duration.as_secs());
    }

    /// Removes expired hits, empty histories and expired bans.
    fn prune(&mut self, now: Instant) {
        let retention = self.retention;
        let ban_window = self.ban.map(|policy| policy.window);
        self.states.retain(|_, state| {
            for (hits, keep) in state.hits.iter_mut().zip(retention) {
                hits.retain(|t| now.saturating_duration_since(*t) < keep);
            }
            if let Some(window) = ban_window {
                state
                    .violations
                    .retain(|t| now.saturating_duration_since(*t) < window);
            }
            state.hits.iter().any(|hits| !hits.is_empty()) || !state.violations.is_empty()
        });
        self.bans.retain(|_, until| now < *until);
    }

    /// Number of IPs with tracked state. Exposed for tests of the `max_entries` bound.
    #[cfg(test)]
    fn tracked_ips(&self) -> usize {
        self.states.len()
    }
}

fn parse_rules(kind: &str, raw: &[String]) -> Vec<Rule> {
    raw.iter()
        .filter_map(|text| {
            let rule = Rule::parse(text);
            if rule.is_none() {
                warn!(
                    "Rate limit: invalid {kind} rule `{text}` (expected hits/seconds), skipping it"
                );
            }
            rule
        })
        .collect()
}

/// Runs [`RateLimiter::check`] on a shared limiter. A poisoned lock still yields the state.
#[must_use]
pub fn check_shared(limiter: &Mutex<RateLimiter>, ip: IpAddr, kind: Kind, now: Instant) -> Verdict {
    limiter
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .check(ip, kind, now)
}

#[cfg(test)]
mod tests {
    use super::{Cidr, Kind, RateLimiter, Rule, Verdict};
    use pumpkin_config::{IpSetConfig, RateLimitConfig};
    use std::net::{IpAddr, Ipv6Addr};
    use std::time::{Duration, Instant};

    fn ip(last: u8) -> IpAddr {
        IpAddr::from([198, 51, 100, last])
    }

    fn config(connections: &[&str]) -> RateLimitConfig {
        RateLimitConfig {
            enabled: true,
            connections: connections.iter().map(|s| (*s).to_string()).collect(),
            status: Vec::new(),
            logins: Vec::new(),
            exempt: Vec::new(),
            ipset: IpSetConfig {
                enabled: false,
                ..IpSetConfig::default()
            },
        }
    }

    fn ms(base: Instant, millis: u64) -> Instant {
        base + Duration::from_millis(millis)
    }

    #[test]
    fn four_per_second_allows_four_then_drops_and_recovers() {
        let mut limiter = RateLimiter::from_config(&config(&["4/1"]));
        let t0 = Instant::now();
        for step in 0..4 {
            assert_eq!(
                limiter.check(ip(1), Kind::Connection, ms(t0, step * 100)),
                Verdict::Allow
            );
        }
        assert_eq!(
            limiter.check(ip(1), Kind::Connection, ms(t0, 400)),
            Verdict::Drop
        );
        // The window started at t0 has elapsed, so one hit is allowed again.
        assert_eq!(
            limiter.check(ip(1), Kind::Connection, ms(t0, 1000)),
            Verdict::Allow
        );
    }

    #[test]
    fn dropped_hits_do_not_extend_the_window() {
        let mut limiter = RateLimiter::from_config(&config(&["1/1"]));
        let t0 = Instant::now();
        assert_eq!(limiter.check(ip(1), Kind::Connection, t0), Verdict::Allow);
        assert_eq!(
            limiter.check(ip(1), Kind::Connection, ms(t0, 500)),
            Verdict::Drop
        );
        assert_eq!(
            limiter.check(ip(1), Kind::Connection, ms(t0, 900)),
            Verdict::Drop
        );
        assert_eq!(
            limiter.check(ip(1), Kind::Connection, ms(t0, 1000)),
            Verdict::Allow
        );
    }

    #[test]
    fn every_rule_must_pass() {
        let t0 = Instant::now();
        // The per-second rule blocks a burst even though the 10-second rule has room.
        let mut burst = RateLimiter::from_config(&config(&["1/1", "10/10"]));
        assert_eq!(burst.check(ip(1), Kind::Connection, t0), Verdict::Allow);
        assert_eq!(
            burst.check(ip(1), Kind::Connection, ms(t0, 100)),
            Verdict::Drop
        );

        // The 10-second rule blocks a slow drip even though the per-second rule has room.
        let mut drip = RateLimiter::from_config(&config(&["10/1", "4/10"]));
        for step in 0..4 {
            assert_eq!(
                drip.check(ip(1), Kind::Connection, ms(t0, step * 2000)),
                Verdict::Allow
            );
        }
        assert_eq!(
            drip.check(ip(1), Kind::Connection, ms(t0, 8000)),
            Verdict::Drop
        );
    }

    #[test]
    fn exempt_ip_is_never_limited() {
        let mut cfg = config(&["1/1"]);
        cfg.exempt = vec!["10.0.0.0/8".to_string(), "::1".to_string()];
        let mut limiter = RateLimiter::from_config(&cfg);
        let t0 = Instant::now();
        let inside = IpAddr::from([10, 9, 8, 7]);
        for step in 0..50 {
            assert_eq!(
                limiter.check(inside, Kind::Connection, ms(t0, step)),
                Verdict::Allow
            );
            assert_eq!(
                limiter.check(
                    IpAddr::V6(Ipv6Addr::LOCALHOST),
                    Kind::Connection,
                    ms(t0, step)
                ),
                Verdict::Allow
            );
        }
    }

    #[test]
    fn ipset_bans_after_violations_then_expires() -> Result<(), &'static str> {
        let mut cfg = config(&["1/1"]);
        cfg.ipset = IpSetConfig {
            enabled: true,
            ban_after: 3,
            ban_window_seconds: 10,
            ban_seconds: 60,
            max_entries: 16,
        };
        let mut limiter = RateLimiter::from_config(&cfg);
        let t0 = Instant::now();
        if limiter.check(ip(1), Kind::Connection, t0) != Verdict::Allow {
            return Err("first hit must be allowed");
        }
        for step in 1..=3 {
            if limiter.check(ip(1), Kind::Connection, ms(t0, step)) != Verdict::Drop {
                return Err("rejected hit must drop");
            }
        }
        // Banned: dropped even after the rate window has passed.
        if limiter.check(ip(1), Kind::Connection, t0 + Duration::from_secs(30)) != Verdict::Drop {
            return Err("banned ip must stay dropped");
        }
        // Ban over: the IP is allowed again.
        if limiter.check(ip(1), Kind::Connection, t0 + Duration::from_secs(61)) != Verdict::Allow {
            return Err("ban must expire");
        }
        Ok(())
    }

    #[test]
    fn limits_are_per_ip() {
        let mut limiter = RateLimiter::from_config(&config(&["1/1"]));
        let t0 = Instant::now();
        assert_eq!(limiter.check(ip(1), Kind::Connection, t0), Verdict::Allow);
        assert_eq!(
            limiter.check(ip(1), Kind::Connection, ms(t0, 1)),
            Verdict::Drop
        );
        assert_eq!(
            limiter.check(ip(2), Kind::Connection, ms(t0, 1)),
            Verdict::Allow
        );
    }

    #[test]
    fn kinds_have_separate_rules() {
        let mut cfg = config(&["100/1"]);
        cfg.logins = vec!["1/1".to_string()];
        let mut limiter = RateLimiter::from_config(&cfg);
        let t0 = Instant::now();
        assert_eq!(limiter.check(ip(1), Kind::Login, t0), Verdict::Allow);
        assert_eq!(limiter.check(ip(1), Kind::Login, ms(t0, 1)), Verdict::Drop);
        // Status has no rules, so it is never limited.
        assert_eq!(
            limiter.check(ip(1), Kind::Status, ms(t0, 1)),
            Verdict::Allow
        );
    }

    #[test]
    fn max_entries_bounds_tracked_ips() {
        let mut cfg = config(&["10/1"]);
        cfg.ipset.max_entries = 4;
        let mut limiter = RateLimiter::from_config(&cfg);
        let t0 = Instant::now();
        for last in 1..=32 {
            assert_eq!(
                limiter.check(ip(last), Kind::Connection, t0),
                Verdict::Allow
            );
            assert!(limiter.tracked_ips() <= 4);
        }
    }

    #[test]
    fn invalid_rules_are_skipped() {
        let limiter = RateLimiter::from_config(&config(&["bogus", "2/1"]));
        assert_eq!(limiter.rules[Kind::Connection.index()].len(), 1);
    }

    #[test]
    fn parse_accepts_valid_and_rejects_invalid_rules() {
        assert_eq!(
            Rule::parse("2/1"),
            Some(Rule {
                hits: 2,
                window: Duration::from_secs(1)
            })
        );
        assert_eq!(Rule::parse("0/1"), None);
        assert_eq!(Rule::parse("a/b"), None);
        assert_eq!(Rule::parse("5/0"), None);
        assert_eq!(Rule::parse("5"), None);
    }

    #[test]
    fn cidr_matches_range_and_single_address() {
        let range = Cidr::parse("192.168.0.0/16");
        assert!(range.is_some_and(|c| c.contains(IpAddr::from([192, 168, 44, 1]))));
        assert!(range.is_some_and(|c| !c.contains(IpAddr::from([192, 169, 0, 1]))));
        let single = Cidr::parse("203.0.113.7");
        assert!(single.is_some_and(|c| c.contains(IpAddr::from([203, 0, 113, 7]))));
        assert_eq!(Cidr::parse("10.0.0.0/33"), None);
    }
}
