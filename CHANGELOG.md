# Changelog

## Unreleased

- Added shared, optional one-time OpenCode `auth.json` discovery: compatible API
  credentials prefill Kimi, Minimax, and OpenCode Go forms, while Codex and
  GitHub Copilot provide explicit OAuth import actions. Confirmed credentials are
  copied into YapCap storage and are never synchronized afterward.
- Added OpenCode Go usage tracking with API-key managed multi-account support and
  5 Hour, Weekly, and Monthly usage windows.
- Hardened YapCap-managed account storage with private credential permissions and
  symlink-safe account and credential access.
- Zen pay-as-you-go balance support is explicitly deferred until upstream
  provides a supported public API.
