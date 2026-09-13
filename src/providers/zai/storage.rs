// SPDX-License-Identifier: MPL-2.0

use crate::account_storage::ProviderAccountStorage;
use std::path::Path;

pub const API_KEY_FILE: &str = "api_key.txt";

pub fn write_api_key(account_id: &str, api_key: &str) -> Result<(), String> {
    write_api_key_at(
        &crate::config::paths().zai_accounts_dir,
        account_id,
        api_key,
    )
}

pub fn load_api_key(account_id: &str) -> Result<String, String> {
    load_api_key_at(&crate::config::paths().zai_accounts_dir, account_id)
}

pub(crate) fn delete_account(account_id: &str) -> Result<(), String> {
    ProviderAccountStorage::new(&crate::config::paths().zai_accounts_dir)
        .delete_account(account_id)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(crate) fn write_api_key_at(root: &Path, account_id: &str, api_key: &str) -> Result<(), String> {
    let api_key = normalize_api_key(api_key)?;
    ProviderAccountStorage::new(root)
        .write_text_file(account_id, API_KEY_FILE, &api_key)
        .map_err(|error| error.to_string())
}

pub(crate) fn load_api_key_at(root: &Path, account_id: &str) -> Result<String, String> {
    ProviderAccountStorage::new(root)
        .read_text_file(account_id, API_KEY_FILE)
        .map_err(|error| error.to_string())
}

pub(crate) fn normalize_api_key(api_key: &str) -> Result<String, String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err("API key is required".to_string());
    }
    if api_key.chars().any(char::is_control) {
        return Err("API key contains invalid characters".to_string());
    }
    Ok(api_key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn writes_normalized_key_under_managed_root() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("zai-accounts");

        write_api_key_at(&root, "zai-1", " test-key ").unwrap();

        assert_eq!(load_api_key_at(&root, "zai-1").unwrap(), "test-key");
        assert!(root.join("zai-1").join(API_KEY_FILE).exists());
    }

    #[test]
    fn rejects_empty_and_control_character_keys() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("zai-accounts");

        assert!(write_api_key_at(&root, "zai-1", " \t ").is_err());
        assert!(write_api_key_at(&root, "zai-1", "bad\nkey").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_account_and_api_key_paths() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let root = temp.path().join("zai-accounts");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        let target = outside.join(API_KEY_FILE);
        fs::write(&target, "unchanged").unwrap();
        fs::create_dir_all(&root).unwrap();
        symlink(&outside, root.join("zai-1")).unwrap();

        assert!(write_api_key_at(&root, "zai-1", "replacement").is_err());
        assert!(load_api_key_at(&root, "zai-1").is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "unchanged");

        fs::remove_file(root.join("zai-1")).unwrap();
        fs::create_dir(root.join("zai-1")).unwrap();
        symlink(&target, root.join("zai-1").join(API_KEY_FILE)).unwrap();

        assert!(write_api_key_at(&root, "zai-1", "replacement").is_err());
        assert!(load_api_key_at(&root, "zai-1").is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "unchanged");
    }
}
