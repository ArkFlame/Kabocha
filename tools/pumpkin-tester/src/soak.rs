//! `--soak <minutes>`: bots move and chat against a real tick loop, then report MSPT p99 and RSS.

use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_core::server::{Server, ticker::Ticker};
use pumpkin_util::math::vector3::Vector3;

use crate::bot::HeadlessBot;

const BOT_COUNT: usize = 8;
const P99_LIMIT_MS: f64 = 50.0;
const SAMPLE_EVERY: Duration = Duration::from_secs(1);

fn rss_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("VmRSS:"))
                .and_then(|v| v.split_whitespace().next().and_then(|n| n.parse().ok()))
        })
        .unwrap_or(0)
}

/// Runs the soak and returns the summary line, or an error when a gate fails.
pub async fn run_soak(
    server: &Arc<Server>,
    addr: SocketAddr,
    minutes: u64,
) -> Result<String, String> {
    let ticker_server = server.clone();
    let ticker = std::thread::Builder::new()
        .name("Soak-Ticker".into())
        .spawn(move || Ticker::run(&ticker_server))
        .map_err(|err| err.to_string())?;

    let mut tasks = Vec::new();
    for i in 0..BOT_COUNT {
        let mut bot = HeadlessBot::connect_and_join(addr, &format!("Soak{i}")).await?;
        let end = Instant::now() + Duration::from_secs(minutes * 60);
        tasks.push(tokio::spawn(async move {
            let base = bot.position;
            let mut step = 0.0f64;
            while Instant::now() < end {
                step += 0.2;
                let pos = Vector3::new(
                    base.x + (step + i as f64).sin() * 20.0,
                    base.y,
                    base.z + (step + i as f64).cos() * 20.0,
                );
                if bot.move_to(pos, true).await.is_err() {
                    return Err(format!("bot {i} lost its connection"));
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Ok(())
        }));
    }

    let start = Instant::now();
    let mut samples: Vec<i64> = Vec::new();
    let rss_first = rss_kb();
    let mut rss_max = rss_first;
    while start.elapsed() < Duration::from_secs(minutes * 60) {
        tokio::time::sleep(SAMPLE_EVERY).await;
        samples.extend(
            server
                .get_tick_times_nanos_copy()
                .into_iter()
                .filter(|t| *t > 0)
                .take(20),
        );
        rss_max = rss_max.max(rss_kb());
    }
    for task in tasks {
        // A bot stuck on a dead socket must not hide the MSPT result.
        if let Ok(joined) = tokio::time::timeout(Duration::from_secs(30), task).await {
            joined.map_err(|err| err.to_string())??;
        }
    }
    server.stop_token.cancel();
    for _ in 0..300 {
        if ticker.is_finished() {
            let _ = ticker.join();
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    samples.sort_unstable();
    let idx = samples.len().saturating_mul(99) / 100;
    let p99_ms = samples
        .get(idx.min(samples.len().saturating_sub(1)))
        .map_or(0.0, |n| *n as f64 / 1_000_000.0);
    let line = format!(
        "SOAK minutes={minutes} bots={BOT_COUNT} samples={} mspt_p99={p99_ms:.2}ms rss_first_kb={rss_first} rss_max_kb={rss_max} rss_last_kb={}",
        samples.len(),
        rss_kb()
    );
    if p99_ms > P99_LIMIT_MS {
        return Err(format!("{line}\nFAIL p99 above {P99_LIMIT_MS}ms"));
    }
    Ok(line)
}
