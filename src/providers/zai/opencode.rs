// SPDX-License-Identifier: MPL-2.0

use std::path::Path;

const PROVIDER_IDS: [&str; 2] = ["zai-coding-plan", "zai"];

pub fn discover_api_key() -> Option<String> {
    crate::providers::opencode_auth::discover_api_key(&PROVIDER_IDS)
}

pub fn has_usable_api_key_at(path: &Path) -> bool {
    crate::providers::opencode_auth::has_api_key_at_path(path, &PROVIDER_IDS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discovers_primary_key_before_alias() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(
            &path,
            r#"{"zai-coding-plan":{"type":"api","key":"primary"},"zai":{"type":"api","key":"alias"}}"#,
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.remove(crate::providers::opencode_auth::OPENCODE_AUTH_CONTENT_ENV);
        env.set(
            crate::providers::opencode_auth::OPENCODE_AUTH_PATH_ENV,
            &path,
        );

        assert_eq!(discover_api_key().as_deref(), Some("primary"));
    }

    #[test]
    fn falls_back_to_alias_when_primary_is_not_an_api_key() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(
            &path,
            r#"{"zai-coding-plan":{"type":"oauth","refresh":"r","access":"a","expires":1},"zai":{"type":"api","key":" alias "}}"#,
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.remove(crate::providers::opencode_auth::OPENCODE_AUTH_CONTENT_ENV);
        env.set(
            crate::providers::opencode_auth::OPENCODE_AUTH_PATH_ENV,
            &path,
        );

        assert_eq!(discover_api_key().as_deref(), Some("alias"));
    }

    #[test]
    fn path_predicate_does_not_expose_the_key() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, r#"{"zai":{"type":"api","key":"key"}}"#).unwrap();

        assert!(has_usable_api_key_at(&path));
    }
}
