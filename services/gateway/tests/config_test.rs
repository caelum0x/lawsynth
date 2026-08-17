use lawsynth_gateway::GatewayConfig;
use std::time::Duration;

#[test]
fn default_configuration_is_valid() {
    let config = GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000");
    assert!(config.validate().is_ok());
    assert_eq!(config.api_prefix, "/v1");
    assert_eq!(config.rate_limit_window, Duration::from_secs(60));
}

#[test]
fn empty_endpoints_are_rejected() {
    assert!(GatewayConfig::new("", "127.0.0.1:9000").validate().is_err());
    assert!(GatewayConfig::new("127.0.0.1:8080", "").validate().is_err());
}

#[test]
fn zero_quota_and_zero_window_are_rejected() {
    let mut config = GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000");
    config.rate_limit_quota = 0;
    assert!(config.validate().is_err());

    let mut config = GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000");
    config.rate_limit_window = Duration::from_secs(0);
    assert!(config.validate().is_err());
}

#[test]
fn relative_api_prefix_is_rejected() {
    let mut config = GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000");
    config.api_prefix = "v1".into();
    assert!(config.validate().is_err());
}

#[test]
fn non_absolute_origin_is_rejected() {
    let config = GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000")
        .with_allowed_origins(vec!["app.example".into()]);
    assert!(config.validate().is_err());
}

#[test]
fn absolute_origins_are_accepted() {
    let config = GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000")
        .with_allowed_origins(vec!["https://app.example".into()]);
    assert!(config.validate().is_ok());
}
