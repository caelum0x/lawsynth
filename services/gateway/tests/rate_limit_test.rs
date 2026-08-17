mod support;

use lawsynth_gateway::{HttpRequest, RateDecision, RateLimiter};

#[test]
fn fixed_window_allows_quota_then_limits() {
    let limiter = RateLimiter::new(3, 60);
    assert!(limiter.check("client", 0).is_allowed());
    assert!(limiter.check("client", 10).is_allowed());
    assert!(limiter.check("client", 20).is_allowed());
    assert!(matches!(limiter.check("client", 30), RateDecision::Limited { .. }));
}

#[test]
fn a_new_window_restores_the_quota() {
    let limiter = RateLimiter::new(1, 60);
    assert!(limiter.check("client", 5).is_allowed());
    assert!(!limiter.check("client", 40).is_allowed());
    assert!(limiter.check("client", 61).is_allowed());
}

#[test]
fn distinct_clients_have_independent_budgets() {
    let limiter = RateLimiter::new(1, 60);
    assert!(limiter.check("a", 0).is_allowed());
    assert!(!limiter.check("a", 1).is_allowed());
    assert!(limiter.check("b", 1).is_allowed());
}

#[test]
fn gateway_returns_429_with_retry_after_when_exhausted() {
    // A gateway whose quota is 1: the second request in the window is limited.
    // /healthz is local and never proxied, so no upstream is needed.
    let mut config = lawsynth_gateway::GatewayConfig::new("127.0.0.1:0", "127.0.0.1:9");
    config.rate_limit_quota = 1;
    let gateway = support::gateway_with(config, 100);

    let request = || HttpRequest::new("GET", "/v1/health", Vec::new(), Vec::new());
    // First request passes the limiter and then fails at the (absent) upstream,
    // which is fine: it still consumed the single-token budget.
    let first = gateway.handle(&request(), "10.0.0.1");
    assert_ne!(first.status, 429);

    let second = gateway.handle(&request(), "10.0.0.1");
    assert_eq!(second.status, 429);
    assert!(second.header("retry-after").is_some());
}
