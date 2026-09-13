# Gemini Provider

- `mod.rs` exposes Gemini provider operations.
- `account.rs` manages Google OAuth accounts.
- `buckets.rs` parses quota buckets and usage windows.
- `code_assist.rs` calls Google Code Assist endpoints and resolves a fallback project through Cloud Resource Manager.
- `host_session.rs` reads the active gemini-cli session.
- `id_token.rs` parses Google identity claims.
- `login.rs` implements browser OAuth login.
- `oauth.rs` configures shared Google OAuth helpers for authorization, code exchange, and token refresh.
- `plan_label.rs` formats Gemini plan labels.
