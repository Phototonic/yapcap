// SPDX-License-Identifier: MPL-2.0

use base64::Engine as _;
use sha2::Digest as _;

use super::oauth::{
    PkceCodes, authorization_url, decode_jwt_claims, exchange_code, new_pkce, new_state,
    parse_token_response, refresh_token,
};
use super::usage::parse_billing_snapshot;
use crate::error::GrokError;
use crate::model::{ExtraUsageState, ProviderCost, ProviderId};

#[test]
fn parses_billing_fixture_into_snapshot() {
    let fixture = include_str!("../../../fixtures/grok/billing_response.json");
    let snapshot = parse_billing_snapshot(
        fixture,
        Some("user@x.ai"),
        Some("Grok User"),
        Some("sub-123"),
    )
    .unwrap();
    assert_eq!(snapshot.provider, ProviderId::Grok);
    assert_eq!(snapshot.windows.len(), 1);
    let window = &snapshot.windows[0];
    assert_eq!(window.label, "Weekly");
    assert!((window.used_percent - 46.0).abs() < f32::EPSILON);
    assert_eq!(window.window_seconds, Some(604_800));
    assert!(window.reset_at.is_some());
    assert_eq!(snapshot.identity.plan.as_deref(), Some("SuperGrok"));
    assert_eq!(snapshot.identity.email.as_deref(), Some("user@x.ai"));
    assert_eq!(snapshot.identity.display_name.as_deref(), Some("Grok User"));
    assert_eq!(snapshot.identity.account_id.as_deref(), Some("sub-123"));
    assert_eq!(
        snapshot.provider_cost,
        Some(ProviderCost {
            used: 1500.0,
            limit: None,
            units: "credits".to_string(),
        })
    );
    assert_eq!(snapshot.extra_usage, None);
}

#[test]
fn falls_back_to_grok_build_product_usage() {
    let json = r#"{
        "config": {
            "creditUsagePercent": null,
            "productUsage": [
                { "product": "Other", "usagePercent": 10.0 },
                { "product": "GrokBuild", "usagePercent": 75.5 }
            ]
        },
        "subscriptionTier": "GrokBasic"
    }"#;
    let snapshot = parse_billing_snapshot(json, None, None, None).unwrap();
    assert_eq!(snapshot.windows.len(), 1);
    assert!((snapshot.windows[0].used_percent - 75.5).abs() < f32::EPSILON);
    assert_eq!(snapshot.identity.plan.as_deref(), Some("GrokBasic"));
}

#[test]
fn clamps_used_percentage_to_range() {
    let json_high = r#"{ "config": { "creditUsagePercent": 140.0 } }"#;
    let snapshot_high = parse_billing_snapshot(json_high, None, None, None).unwrap();
    assert!((snapshot_high.windows[0].used_percent - 100.0).abs() < f32::EPSILON);

    let json_low = r#"{ "config": { "creditUsagePercent": -25.0 } }"#;
    let snapshot_low = parse_billing_snapshot(json_low, None, None, None).unwrap();
    assert!((snapshot_low.windows[0].used_percent - 0.0).abs() < f32::EPSILON);
}

#[test]
fn maps_positive_on_demand_cap_to_extra_usage() {
    let json = r#"{
        "config": {
            "creditUsagePercent": 20.0,
            "onDemandCap": { "val": 100.0 },
            "onDemandUsed": { "val": 25.0 }
        }
    }"#;
    let snapshot = parse_billing_snapshot(json, None, None, None).unwrap();
    assert_eq!(
        snapshot.extra_usage,
        Some(ExtraUsageState::Active {
            used_percent: 25.0,
            cost: ProviderCost {
                used: 25.0,
                limit: Some(100.0),
                units: "credits".to_string(),
            },
        })
    );
}

#[test]
fn returns_decode_usage_on_invalid_json() {
    let result = parse_billing_snapshot("not json", None, None, None);
    assert!(matches!(result, Err(GrokError::DecodeUsage(_))));
}

#[test]
fn returns_no_usage_data_when_config_or_percent_missing() {
    let json_no_config = r#"{ "subscriptionTier": "SuperGrok" }"#;
    assert!(matches!(
        parse_billing_snapshot(json_no_config, None, None, None),
        Err(GrokError::NoUsageData)
    ));

    let json_no_percent = r#"{ "config": {} }"#;
    assert!(matches!(
        parse_billing_snapshot(json_no_percent, None, None, None),
        Err(GrokError::NoUsageData)
    ));
}

#[test]
fn returns_invalid_reset_timestamp_on_malformed_end_date() {
    let json = r#"{
        "config": {
            "creditUsagePercent": 10.0,
            "currentPeriod": {
                "end": "invalid-date"
            }
        }
    }"#;
    let result = parse_billing_snapshot(json, None, None, None);
    assert!(matches!(
        result,
        Err(GrokError::InvalidResetTimestamp { .. })
    ));
}

#[test]
fn authorization_url_contains_required_params() {
    let pkce = PkceCodes {
        code_verifier: "verifier123".to_string(),
        code_challenge: "challenge456".to_string(),
    };
    let url = authorization_url("http://127.0.0.1:12345/callback", &pkce, "state789");
    assert!(url.starts_with("https://auth.x.ai/oauth2/authorize"));
    assert!(url.contains("client_id=b1a00492-073a-47ea-816f-4c329264a828"));
    assert!(url.contains("code_challenge=challenge456"));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("state=state789"));
}

#[test]
fn decodes_grok_jwt_claims() {
    let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let payload = "eyJzdWIiOiJ1c3ItMSIsImVtYWlsIjoidGVzdGVyQHguYWkiLCJuYW1lIjoiVGVzdGVyIFgiLCJ0ZWFtX2lkIjoidGVhbS0xIn0";
    let token = format!("{header}.{payload}.signature");
    let claims = decode_jwt_claims(&token).unwrap();
    assert_eq!(claims.sub.as_deref(), Some("usr-1"));
    assert_eq!(claims.email.as_deref(), Some("tester@x.ai"));
    assert_eq!(claims.name.as_deref(), Some("Tester X"));
    assert_eq!(claims.team_id.as_deref(), Some("team-1"));
}

#[test]
fn pkce_generates_valid_codes() {
    let pkce = new_pkce();
    assert!(!pkce.code_verifier.is_empty());
    assert!(!pkce.code_challenge.is_empty());
    let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(sha2::Sha256::digest(pkce.code_verifier.as_bytes()));
    assert_eq!(pkce.code_challenge, expected);
}

#[test]
fn new_state_generates_32_alphanumeric_chars() {
    let state1 = new_state();
    let state2 = new_state();
    assert_eq!(state1.len(), 32);
    assert!(state1.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_ne!(state1, state2);
}

#[test]
fn decodes_grok_jwt_claims_with_tier() {
    let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let payload = "eyJzdWIiOiJ1c3ItMiIsImVtYWlsIjoidGVzdDJAeC5haSIsIm5hbWUiOiJUZXN0ZXIgMiIsInRlYW1faWQiOiJ0ZWFtLTIiLCJ0aWVyIjoiU3VwZXJHcm9rIn0";
    let token = format!("{header}.{payload}.signature");
    let claims = decode_jwt_claims(&token).unwrap();
    assert_eq!(claims.sub.as_deref(), Some("usr-2"));
    assert_eq!(claims.email.as_deref(), Some("test2@x.ai"));
    assert_eq!(claims.name.as_deref(), Some("Tester 2"));
    assert_eq!(claims.team_id.as_deref(), Some("team-2"));
    assert_eq!(claims.tier.as_deref(), Some("SuperGrok"));
}

#[test]
fn decode_jwt_claims_invalid_returns_none() {
    assert_eq!(decode_jwt_claims("not-a-jwt"), None);
    assert_eq!(decode_jwt_claims("a.invalid-base64!@#.c"), None);
    assert_eq!(decode_jwt_claims("a.bm90LWpzb24.c"), None);
}

#[test]
fn parses_valid_token_response() {
    let raw = r#"{
        "access_token": "grok-acc-1",
        "refresh_token": "grok-ref-1",
        "expires_in": 7200,
        "token_type": "Bearer",
        "scope": "openid email"
    }"#;
    let resp = parse_token_response(raw).unwrap();
    assert_eq!(resp.access_token, "grok-acc-1");
    assert_eq!(resp.refresh_token, "grok-ref-1");
    assert_eq!(resp.token_type.as_deref(), Some("Bearer"));
    assert_eq!(resp.scope.as_deref(), Some("openid email"));
    assert!(resp.expires_at > chrono::Utc::now());
}

#[test]
fn parses_token_response_with_timestamp() {
    let raw = r#"{
        "access_token": "grok-acc-2",
        "refresh_token": "grok-ref-2",
        "expires_at": 1800000000
    }"#;
    let resp = parse_token_response(raw).unwrap();
    assert_eq!(resp.access_token, "grok-acc-2");
    assert_eq!(resp.refresh_token, "grok-ref-2");
    assert_eq!(resp.expires_at.timestamp(), 1800000000);
}

#[test]
fn parses_token_response_falls_back_to_jwt_exp() {
    let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let payload = "eyJzdWIiOiJ1c3ItMSIsImV4cCI6MTgwMDAwMDAwMH0";
    let access_token = format!("{header}.{payload}.sig");
    let raw = format!(r#"{{"access_token":"{access_token}","refresh_token":"r1"}}"#);
    let resp = parse_token_response(&raw).unwrap();
    assert_eq!(resp.expires_at.timestamp(), 1800000000);
}

#[test]
fn parse_token_response_missing_fields_fail() {
    let missing_access = r#"{"refresh_token":"r1","expires_in":3600}"#;
    assert!(matches!(
        parse_token_response(missing_access),
        Err(GrokError::TokenRefreshParse(_))
    ));

    let missing_refresh = r#"{"access_token":"a1","expires_in":3600}"#;
    assert!(matches!(
        parse_token_response(missing_refresh),
        Err(GrokError::TokenRefreshParse(_))
    ));

    let invalid_expires = r#"{"access_token":"a1","refresh_token":"r1","expires_in":-10}"#;
    assert!(matches!(
        parse_token_response(invalid_expires),
        Err(GrokError::TokenRefreshParse(_))
    ));
}

#[tokio::test]
async fn exchange_code_sends_form_data_and_parses_response() {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let token_url = format!("http://{addr}/oauth2/token");

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buffer = [0u8; 4096];
        let bytes = tokio::io::AsyncReadExt::read(&mut stream, &mut buffer)
            .await
            .unwrap();
        let request = String::from_utf8_lossy(&buffer[..bytes]).to_string();

        assert!(request.contains("grant_type=authorization_code"));
        assert!(request.contains("code=auth-code-123"));
        assert!(request.contains("code_verifier=pkce-verifier-456"));
        assert!(request.contains("client_id=b1a00492-073a-47ea-816f-4c329264a828"));

        let body = r#"{"access_token":"acc-new","refresh_token":"ref-new","expires_in":3600}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
            .await
            .unwrap();
    });

    let client = reqwest::Client::new();
    let resp = exchange_code(
        &client,
        "auth-code-123",
        "pkce-verifier-456",
        "http://127.0.0.1/cb",
        &token_url,
    )
    .await
    .unwrap();

    assert_eq!(resp.access_token, "acc-new");
    assert_eq!(resp.refresh_token, "ref-new");
    server.await.unwrap();
}

#[tokio::test]
async fn exchange_code_handles_429_retry_after() {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let token_url = format!("http://{addr}/oauth2/token");

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let response = "HTTP/1.1 429 Too Many Requests\r\nretry-after: 45\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
        tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
            .await
            .unwrap();
    });

    let client = reqwest::Client::new();
    let err = exchange_code(&client, "code", "ver", "http://127.0.0.1/cb", &token_url)
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        GrokError::RateLimited {
            retry_after_secs: Some(45)
        }
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn refresh_token_handles_rotation_and_fallback() {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let token_url = format!("http://{addr}/oauth2/token");

    let server = tokio::spawn(async move {
        let (mut stream1, _) = listener.accept().await.unwrap();
        let mut buffer = [0u8; 4096];
        let bytes1 = tokio::io::AsyncReadExt::read(&mut stream1, &mut buffer)
            .await
            .unwrap();
        let request1 = String::from_utf8_lossy(&buffer[..bytes1]).to_string();
        assert!(request1.contains("grant_type=refresh_token"));
        assert!(request1.contains("refresh_token=old-refresh-1"));

        let body1 =
            r#"{"access_token":"rotated-acc","refresh_token":"rotated-ref","expires_in":3600}"#;
        let response1 = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body1}",
            body1.len()
        );
        tokio::io::AsyncWriteExt::write_all(&mut stream1, response1.as_bytes())
            .await
            .unwrap();

        let (mut stream2, _) = listener.accept().await.unwrap();
        let body2 = r#"{"access_token":"reused-acc","expires_in":3600}"#;
        let response2 = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body2}",
            body2.len()
        );
        tokio::io::AsyncWriteExt::write_all(&mut stream2, response2.as_bytes())
            .await
            .unwrap();
    });

    let client = reqwest::Client::new();
    let res1 = refresh_token(&client, "old-refresh-1", &token_url)
        .await
        .unwrap();
    assert_eq!(res1.access_token, "rotated-acc");
    assert_eq!(res1.refresh_token, "rotated-ref");

    let res2 = refresh_token(&client, "fallback-refresh-2", &token_url)
        .await
        .unwrap();
    assert_eq!(res2.access_token, "reused-acc");
    assert_eq!(res2.refresh_token, "fallback-refresh-2");

    server.await.unwrap();
}

#[tokio::test]
async fn refresh_token_handles_http_errors() {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let token_url = format!("http://{addr}/oauth2/token");

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let response =
            "HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
        tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
            .await
            .unwrap();
    });

    let client = reqwest::Client::new();
    let err = refresh_token(&client, "ref", &token_url).await.unwrap_err();
    assert!(matches!(err, GrokError::TokenRefreshHttp { status: 500 }));
    server.await.unwrap();
}
