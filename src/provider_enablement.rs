// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, ProviderEnablement};
use crate::detection::DetectionSnapshot;
use crate::model::ProviderId;
use crate::providers::registry;

#[must_use]
pub fn provider_enabled(
    config: &Config,
    detection: &DetectionSnapshot,
    provider: ProviderId,
) -> bool {
    resolve(
        provider,
        config.provider_enablement(provider),
        detection.detected(provider),
        !registry::discover_accounts(provider, config).is_empty(),
    )
}

#[must_use]
pub fn resolve(
    provider: ProviderId,
    enablement: ProviderEnablement,
    detected: bool,
    has_account: bool,
) -> bool {
    if provider == ProviderId::Gemini {
        return enablement == ProviderEnablement::Enabled;
    }
    match enablement {
        ProviderEnablement::Auto => detected || has_account,
        ProviderEnablement::Enabled => true,
        ProviderEnablement::Disabled => false,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;
    use crate::config::ProviderEnablement;
    use crate::model::ProviderId;

    #[test]
    fn auto_enablement_uses_detection_or_account_presence() {
        assert!(resolve(
            ProviderId::Codex,
            ProviderEnablement::Auto,
            true,
            false
        ));
        assert!(resolve(
            ProviderId::Codex,
            ProviderEnablement::Auto,
            false,
            true
        ));
        assert!(!resolve(
            ProviderId::Codex,
            ProviderEnablement::Auto,
            false,
            false
        ));
    }

    #[test]
    fn explicit_enablement_overrides_detection_and_account_presence() {
        assert!(resolve(
            ProviderId::Codex,
            ProviderEnablement::Enabled,
            false,
            false
        ));
        assert!(!resolve(
            ProviderId::Codex,
            ProviderEnablement::Disabled,
            true,
            true
        ));
    }

    #[test]
    fn gemini_requires_explicit_enablement() {
        assert!(!resolve(
            ProviderId::Gemini,
            ProviderEnablement::Auto,
            true,
            true
        ));
        assert!(!resolve(
            ProviderId::Gemini,
            ProviderEnablement::Disabled,
            true,
            true
        ));
        assert!(resolve(
            ProviderId::Gemini,
            ProviderEnablement::Enabled,
            false,
            false
        ));
    }

    #[test]
    fn fresh_config_without_detection_enables_no_providers() {
        let config = crate::config::Config::default();
        let detection = crate::detection::DetectionSnapshot::default();
        for provider in crate::model::ProviderId::ALL {
            assert!(!super::provider_enabled(&config, &detection, provider));
        }
    }
}
