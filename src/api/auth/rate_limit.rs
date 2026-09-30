use axum::http::HeaderMap;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::warn;

pub struct FailedAttemptRecord {
    pub count: u32,
    pub window_start: Instant,
    pub locked_until: Option<Instant>,
}

pub static RATE_LIMITER: LazyLock<Mutex<HashMap<String, FailedAttemptRecord>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_client_ip(headers: &HeaderMap) -> String {
    // 1. Check CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = headers.get("cf-connecting-ip").and_then(|v| v.to_str().ok()) {
        let trimmed = cf_ip.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // 2. Check X-Forwarded-For (first entry)
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
        && let Some(first) = xff.split(',').next() {
            let trimmed = first.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

    // 3. Check X-Real-IP
    if let Some(rip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        let trimmed = rip.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    "unknown_client".to_string()
}

pub async fn check_rate_limit(ip: &str) -> Result<(), u64> {
    let mut limiter = RATE_LIMITER.lock().await;
    let now = Instant::now();

    if let Some(record) = limiter.get_mut(ip) {
        if let Some(locked_until) = record.locked_until {
            if now < locked_until {
                let remaining = (locked_until - now).as_secs();
                return Err(remaining.max(1));
            } else {
                // Lockout period expired: reset
                record.locked_until = None;
                record.count = 0;
                record.window_start = now;
            }
        }

        // Window of 5 minutes (300 seconds)
        if now.duration_since(record.window_start) > Duration::from_secs(300) {
            record.count = 0;
            record.window_start = now;
        }
    }

    Ok(())
}

pub async fn record_failed_attempt(ip: &str) {
    let mut limiter = RATE_LIMITER.lock().await;
    let now = Instant::now();

    let record = limiter.entry(ip.to_string()).or_insert_with(|| FailedAttemptRecord {
        count: 0,
        window_start: now,
        locked_until: None,
    });

    if now.duration_since(record.window_start) > Duration::from_secs(300) {
        record.count = 0;
        record.window_start = now;
    }

    record.count += 1;
    if record.count >= 5 {
        // Lockout for 5 minutes (300 seconds)
        record.locked_until = Some(now + Duration::from_secs(300));
        warn!(
            "Rate limit triggered: Client IP {} locked out for 5 minutes after {} failed attempts",
            ip, record.count
        );
    }
}

pub async fn record_successful_attempt(ip: &str) {
    let mut limiter = RATE_LIMITER.lock().await;
    limiter.remove(ip);
}
