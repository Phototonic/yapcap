# Changelog

## Unreleased

- Added Google Antigravity usage tracking with Google OAuth accounts, grouped
  Gemini and Claude/GPT quota windows, reauthentication, and multi-account
  support.
- Added Kimi for Coding usage tracking with API-key managed accounts, weekly and
  rate-limit windows, account reauthentication, and multi-account support.
- Re-added OpenCode Go usage tracking with API-key managed accounts and 5-hour,
  weekly, and monthly windows.
- Re-added optional one-time OpenCode `auth.json` credential discovery: compatible
  keys prefill Minimax, Kimi, and OpenCode Go forms, while Codex and GitHub
  Copilot offer explicit OAuth imports. OpenCode is not used as a live refresh
  or synchronization source.
- Added automatic provider detection with live re-detection, tri-state provider
  enablement, and first-run empty states that guide users to account setup.
- Added the popup provider picker, adaptive provider tabs, and detected-provider
  setup actions.
- Improved provider usage meters, account matching, popup navigation, and usage
  amount handling across provider tabs.
