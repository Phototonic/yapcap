// SPDX-License-Identifier: MPL-2.0

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
