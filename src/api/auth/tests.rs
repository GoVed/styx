use super::rate_limit::*;
use axum::http::HeaderMap;

#[tokio::test]
async fn test_rate_limiter_lockout_and_reset() {
    let test_ip = "192.168.1.100";

    // Initial state: not rate limited
    assert!(check_rate_limit(test_ip).await.is_ok());

    // Record 4 failed attempts: still not locked out
    for _ in 0..4 {
        record_failed_attempt(test_ip).await;
    }
    assert!(check_rate_limit(test_ip).await.is_ok());

    // 5th failed attempt: triggers lockout
    record_failed_attempt(test_ip).await;
    let limit_res = check_rate_limit(test_ip).await;
    assert!(limit_res.is_err());
    assert!(limit_res.unwrap_err() > 0);

    // Successful login resets the limiter
    record_successful_attempt(test_ip).await;
    assert!(check_rate_limit(test_ip).await.is_ok());
}

#[test]
fn test_client_ip_extraction() {
    let mut headers = HeaderMap::new();
    assert_eq!(get_client_ip(&headers), "unknown_client");

    headers.insert("x-real-ip", "203.0.113.195".parse().unwrap());
    assert_eq!(get_client_ip(&headers), "203.0.113.195");

    headers.insert(
        "x-forwarded-for",
        "198.51.100.1, 10.0.0.1".parse().unwrap(),
    );
    assert_eq!(get_client_ip(&headers), "198.51.100.1");

    headers.insert("cf-connecting-ip", "192.0.2.1".parse().unwrap());
    assert_eq!(get_client_ip(&headers), "192.0.2.1");
}
