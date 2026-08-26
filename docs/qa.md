# YapCap QA Plan

Manual test plan for v0.5.2. Run against both Native (`just install`) and Flatpak (`just flatpak-install`) builds unless noted.

Paths used below:

**Native** (default XDG layout on typical Linux installs):

- Config: `~/.config/cosmic/io.github.TopiCsarno.YapCap/v502/`
- Former snapshot cache, no longer active runtime state: `~/.cache/yapcap/snapshots.json`
- Accounts + logs: `~/.local/state/yapcap/` (e.g. `…/logs/yapcap.log`)

**Flatpak** (app id `io.github.TopiCsarno.YapCap`; paths use passwd `pw_dir` as `~`):

- Config: same COSMIC config schema `v502` dir (manifest mounts `~/.config/cosmic`)
- Former snapshot cache, no longer active runtime state: `~/.var/app/io.github.TopiCsarno.YapCap/cache/yapcap/snapshots.json`
- Accounts + logs: `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/`

Do not expect the Flatpak build to use `~/.local/state/yapcap/` for YapCap data—that is native-only.

---

## 1. Fresh install

- `just clear-all-data` then install. All eight provider tabs visible with "Login required" state (not hidden).
- Existing `v501` COSMIC settings are not loaded after the `v502` schema boundary; users must re-add accounts.
- Existing account directories, old snapshot caches, and logs are not automatically deleted by the schema boundary and may remain orphaned.
- Settings → General → About shows correct version and dist label ("Native" or "Flatpak").
- Panel icon renders without clipping or overflow.

---

## 2. Panel icon styles

In Settings → General, cycle through all four panel icon styles and verify the panel updates immediately each time:

- `Logo and bars` — provider logo + two usage bars visible.
- `Bars only` — no logo, just bars.
- `Logo and percent` — logo + one percentage number.
- `Percent only` — only percentage, no logo. Tooltip in Settings explains it shows the first usage window.

---

## 3. General settings

- Autorefresh interval buttons — set each value, restart, confirm the interval persisted.
- Reset time format `relative` — usage windows show "Resets in Xd Xh".
- Reset time format `absolute` — windows show "Resets tomorrow at …" or day + time.
- Usage amount format `used` — bars and labels show consumed quota.
- Usage amount format `left` — bars and labels flip to remaining quota.
- Select a non-Codex provider tab, restart, and confirm YapCap opens on the same provider.
- Settings survive an app restart (kill and re-open).

---

## 4. Theme

- Flatpak permissions include `--talk-name=com.system76.CosmicSettingsDaemon.Config.*` so libcosmic can subscribe to per-config COSMIC theme watchers.
- Native: switch COSMIC to dark theme — provider icons switch to dark-panel variant without restart.
- Native: switch COSMIC to light theme — provider icons switch to reversed/light variant without restart.
- Native: change COSMIC accent colour — accent fill on selected tabs and rows updates without restart.
- Flatpak: switch COSMIC to dark theme — provider icons switch to dark-panel variant without restart.
- Flatpak: switch COSMIC to light theme — provider icons switch to reversed/light variant without restart.
- Flatpak: change COSMIC accent colour — accent fill on selected tabs and rows updates without restart.

---

## 5. Update checker

- About section shows "Checking for updates…" briefly on startup.
- If up to date, shows "Up to date".
- Simulate update available: `YAPCAP_DEBUG_UPDATE_AVAILABLE=1 cargo run` — red dot on Settings gear, General tab, and About title. Hovering dots shows "Update available".
- "Check again" appears and works when update check fails.

---

## 6. Codex

### 6.1 Add account

- Settings → Codex → Add account opens browser OAuth flow.
- Cancel during login returns to normal add-account state with no partial account stored.
- Successful login stores account under native `~/.local/state/yapcap/codex-accounts/` or Flatpak `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/codex-accounts/`.
- Stored directory contains `metadata.json` and `tokens.json`; `metadata.json` has `email` and `provider_account_id`; `tokens.json` has `access_token`, `refresh_token`, and `expires_at`.
- Duplicate login (same email) updates the existing account directory, not a second entry.
- New account is selected immediately in single-account mode.

### 6.2 Usage display

- Session window (5h) shows used/left percent and reset time.
- Weekly window (7d) shows used/left percent and reset time.
- If credits balance present, cost card is visible.
- Pace indicator marker visible on bars with both `reset_at` and `window_seconds`.

### 6.3 Token refresh

- Corrupt `tokens.json` → `access_token` only, remove `refresh_token`. Verify "Login required" state after one failed refresh.
- Set `expires_at` to one minute in the past with a valid `refresh_token`. On next refresh, YapCap should transparently renew the token and fetch usage without showing an error. Verify `tokens.json` `expires_at` is updated.
- Set `expires_at` far in the past and set `refresh_token` to a junk value. Verify `ActionRequired` state ("Login" badge) and re-auth prompt in Settings.

### 6.4 Remove account

- Remove account from Settings — account directory deleted, provider shows empty state.

### 6.5 Active account badge

- switch accounts through CLI, active badge should update

---

## 7. Claude

### 7.1 Add account

- Settings → Claude → Add account opens browser OAuth flow and prompts for authentication code paste.
- Pasting a wrong or malformed code shows an explicit plain-language error ("paste the authentication code"); existing accounts are untouched.
- Pasting a full callback URL or raw query string is rejected with the same authentication-code guidance.

- Successful add stores account under native `~/.local/state/yapcap/claude-accounts/` or Flatpak `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/claude-accounts/`.
- Stored directory contains `metadata.json` and `tokens.json`; `tokens.json` has `access_token`, `refresh_token`, and `expires_at`.
- Duplicate email upserts the existing account rather than creating a second entry.
- New account is selected immediately in single-account mode.

### 7.2 Usage display

- 5h session window and 7d weekly window visible.
- Max plan accounts: Sonnet, Opus, and Cowork model-specific windows visible.
- Pro plan accounts: model-specific windows absent.
- Extra usage / credits cost card visible when present.
- `utilization=0` + `resets_at=null` on the 5h window shows "Reset" label, not an error.

### 7.3 Token refresh

- Set `expires_at` to one minute in the past with a valid `refresh_token`. Verify silent refresh on next cycle. Verify `tokens.json` `expires_at` is updated.
- Replace `refresh_token` with junk. Verify `ActionRequired` badge and re-auth icon in Settings.
- Per-account re-auth: click re-auth icon → complete OAuth with the same email → usage refreshes immediately.
- Per-account re-auth with a different email → rejected with error, existing account unchanged.

### 7.4 Rate limiting

- Observe `RateLimited` behaviour: provider shows rate-limited message; if `Retry-After` header present, "(retry in Xm)" appended.
- After the backoff window passes, the next refresh clears `rate_limit_until`.

### 7.5 Change active account

- Native: switch accounts through `claude auth login`; Active badge updates from `~/.claude.json` without restart.
- Flatpak: switch accounts through `claude auth login`; Active badge updates from host `~/.claude.json` without restart. The Flatpak manifest grants read-only home access so the app can watch the home directory for `.claude.json` replacement events.
- Flatpak fallback: if the badge does not update automatically after `claude auth login`, click manual refresh. Active badge must reread host `~/.claude.json` and update.

### 7.6 Remove account

- Remove from Settings — account directory deleted, provider shows empty state.

---

## 8. Cursor

### 8.1 Add account (SQLite scan flow)

- Settings → Cursor → Add account triggers a scan of `~/.config/Cursor/User/globalStorage/state.vscdb`.
- If Cursor is not installed or the state DB is absent, YapCap reports that no Cursor account was detected and no account is stored.
- If Cursor IDE is installed but logged out, YapCap asks the user to log into Cursor IDE and does not expose internal `cursorAuth` key names.
- Successful scan stores account under native `~/.local/state/yapcap/cursor-accounts/<opaque-id>/` or Flatpak `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/cursor-accounts/<opaque-id>/`.
- Stored `tokens.json` contains `access_token`, `token_id`, `expires_at`, and `refresh_token`.
- Stored `metadata.json` contains `email` (non-empty), display name, and plan.
- Directory name is opaque (`cursor-<millis>-<pid>` format) and does not embed the email.
- Duplicate scan for the same email replaces the existing managed account directory rather than creating a second entry.
- New account is selected immediately in single-account mode.
- Config `cursor_managed_accounts` entry has `id`, `email`, and `managed_account_root`; no bearer tokens.

### 8.2 Usage display

- Total and API windows shown on the thin panel bars; Auto + Composer windows are skipped on the panel.
- Full popup shows all usage windows.
- Billing cycle end date drives reset time.
- Membership type shown in identity/plan badge.

### 8.3 Token refresh

- Set `expires_at` in `tokens.json` to one minute in the past with a valid `refresh_token`. On next usage cycle, YapCap calls the refresh endpoint, writes rotated tokens, and fetches usage without showing an error. Verify `expires_at` updated in `tokens.json`.
- Replace `refresh_token` with a junk value and set `expires_at` in the past. Verify the stale usage snapshot remains visible and the account shows `Re-auth needed`.
- Verify provider status tells the user to log into that Cursor account in Cursor IDE and rescan.
- Re-scan after logging into Cursor IDE. Verify YapCap updates `tokens.json`, clears `Re-auth needed`, and triggers a fresh usage fetch.
- HTTP 429 or network error during refresh → transient; stale snapshot stays visible with error status and no re-auth badge is shown.

### 8.4 Remove account

- Remove from Settings — YapCap-owned account directory deleted, Cursor's own `~/.config/Cursor` files are untouched, provider shows empty state.

---

## 9. Gemini

### 9.1 Fresh install / Login required

- With no Gemini accounts configured, the Gemini provider tab is visible and shows the **Login required** empty state pointing to Settings → Gemini → Add account.
- Pre-existing host `~/.gemini/oauth_creds.json` is **not** imported. YapCap does not read host tokens.

### 9.2 Add account (Native and Flatpak)

- Settings → Gemini → Add account opens the system browser (Native: directly; Flatpak: via `org.freedesktop.portal.OpenURI`) at Google's sign-in page.
- The browser redirects back to a loopback `127.0.0.1:<port>/?code=…&state=…` callback served by YapCap; the success page reads "Signed in to Gemini — you can close this tab and return to YapCap."
- Cancel during login aborts cleanly with no partial account stored.
- Successful login stores the account under native `~/.local/state/yapcap/gemini-accounts/<id>/` or Flatpak `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/gemini-accounts/<id>/`.
- Stored directory contains `metadata.json` (email, sub, optional `hd`, last tier id, last `cloudaicompanionProject`) and `tokens.json` (`access_token`, `refresh_token`, `expires_at`, `scope`).
- New account is selected immediately in single-account mode.

### 9.3 Multi-account dedupe

- Add a second Gemini account with a different Google identity — both accounts appear in Settings and the popup.
- Re-running Add account with an already-stored Google account updates the existing managed directory by normalized email; no second entry is created.

### 9.4 Usage display

- Free-tier account: popup shows two bars (**Flash**, **Lite**); the Pro bar is hidden.
- Standard-tier (AI Pro) account: popup shows three bars (**Pro**, **Flash**, **Lite**); panel bars show Pro + Flash.
- Workspace account (id_token `hd` present, `currentTier.id = standard-tier`): plan badge reads **Workspace**.
- Each bucket reset follows the YapCap-wide `reset_time_format` preference.

### 9.5 Tier transitions

- Upgrade a free-tier account to AI Pro (or downgrade). On the next refresh cycle the Pro bar appears or disappears and the plan badge updates from **Free** to **Pro**/**Workspace** (or back), without restarting YapCap.

### 9.6 Active account hint

- With YapCap running, `gemini auth login` to a Gemini account YapCap is tracking — the **Active** badge follows the new active email written to `~/.gemini/google_accounts.json`.
- Switching to a Google account that YapCap does not track removes the Active badge from all tracked accounts.
- Deleting `~/.gemini/google_accounts.json` clears the Active badge; recreating it (e.g. via another `gemini auth login`) restores it without a YapCap restart.
- Flatpak: same behaviour through the read-only home mount; click **Refresh now** as a fallback if file watching misses an atomic replace.

### 9.7 Token refresh and re-auth

- Set `expires_at` to one minute in the past with a valid `refresh_token`. Verify silent refresh on the next cycle and updated `expires_at` in `tokens.json`.
- Replace `refresh_token` with junk. Verify `ActionRequired` badge ("Login") on the account, plus a per-account re-auth icon in Settings.
- Per-account re-auth: click re-auth icon → complete OAuth in the browser with the same Google account → usage refreshes immediately.
- Per-account re-auth with a different Google account (different `id_token.email`) → rejected with error, existing account left unchanged.

### 9.8 Remove account

- Remove from Settings — only the YapCap-owned account directory is deleted. Host `~/.gemini/` files (`oauth_creds.json`, `google_accounts.json`, `settings.json`) are not touched.
- If it was the last Gemini account, the provider returns to the Login required empty state.

### 9.9 Host CLI configurations that don't interfere

- Pre-existing `~/.gemini/settings.json` with `selectedAuthType: gemini-api-key` or `vertex-ai`: YapCap still runs OAuth login and stores its own tokens; the absence of an Active badge for these accounts is **expected**, not a bug.
- A `GEMINI_API_KEY` environment variable on the host shell has no effect on YapCap.

### 9.10 `cloudresourcemanager` fallback

- For accounts where `loadCodeAssist` returns no `cloudaicompanionProject` (common when the user has a paid GCP project but no auto-assigned Code Assist project), YapCap calls `cloudresourcemanager.googleapis.com/v1/projects` and picks the first `ACTIVE` project whose id begins with `gen-lang-client-`. Verify the discovered project id is persisted to `metadata.json` (`gemini_last_cloudaicompanion_project`) and the next refresh re-uses it directly.
- For accounts where neither path yields a project, the provider surfaces the actionable `NoCloudaicompanionProject` error in the popup.

---

## 10. Copilot

### 10.1 Add account

- Settings -> Copilot -> Add account starts GitHub device flow.
- Browser opens `https://github.com/login/device`; entering the displayed user code completes successfully.
- Cancel during polling leaves account storage and selected accounts unchanged.
- Adding the same GitHub account a second time refreshes the existing entry; no duplicate row appears.

### 10.2 Login hint

- The shared "Sign in to your browser as the account you want to add" private-window hint is visible at the add-account point.

### 10.3 Multi-account add

- Add a second GitHub account using private browsing or a different browser session.
- The second account creates a separate `copilot-<github-user-id>/` directory.
- Both accounts are visible in Settings.

### 10.4 Free tier display

- Free account popup renders Chat and Completions windows.
- Completions is the headline percentage.
- Panel shows two bars.
- Bar fills reflect the entitlements in the API response; do not assert fixed
  Free entitlement numbers (GitHub adjusts them and the response is authoritative).
- Reset time follows `quota_reset_date_utc` (falling back to `quota_reset_date`).
- No cost card is shown for the Free account.

### 10.5 Paid tier display

- Paid account popup renders one **Credits** window (token-based accounts).
- A dollar cost card shows used and included credits, e.g. `$28.00 / $70.00`.
- Panel shows one bar vertically centered within the two-bar height.
- Plan badge reads **Pro** for the Pro credit entitlement.
- Plan badge reads **Pro+** for `plus_monthly_subscriber_quota` / the Pro+ entitlement.
- Plan badge reads **Max** for the Max credit entitlement.
- Plan badge reads **Business** for `copilot_standalone_seat_quota`.
- An unknown SKU with no recognizable entitlement range falls back to **Plan**.
- Reset time follows `quota_reset_date_utc` (falling back to `quota_reset_date`).

### 10.6 Mixed bar counts

- Select a Free account and a paid account side by side.
- Panel shows a two-bar Free group beside a one-bar paid group.
- The one-bar paid group remains vertically centered; the Free group keeps two bars.

### 10.7 Overage rendering

- Run with `YAPCAP_DEMO=1`.
- Verify the `morgan-pro` Copilot account shows `+42 over plan` under the Credits bar.

### 10.8 `YAPCAP_DEMO`

- Run with `YAPCAP_DEMO=1`.
- Verify `casey-free` and `morgan-pro` Copilot accounts are both present.
- Verify `casey-free` shows Chat and Completions windows in the new Free shape
  and no cost card.
- Verify `morgan-pro` shows a Credits window, a dollar cost card, a **Pro+** badge,
  and `+42 over plan`.
- Verify both accounts are selected and Copilot `Show all accounts` is on.

### 10.9 Re-auth flow

- Revoke the YapCap GitHub App token at `github.com/settings/applications`.
- Trigger refresh and verify account badges flip to `Re-auth needed`.
- Verify the re-auth icon appears in Settings.
- Re-auth with the same GitHub account and verify the account refreshes successfully.
- Re-auth with a different GitHub account and verify YapCap rejects it with a different-account error without replacing the stored account.

### 10.10 Transient errors

- Disable network during refresh.
- Verify stale snapshot remains visible with the "No internet connection" message.
- Reconnect and click **Refresh now**; fresh data should restore.

### 10.11 Account removal

- Remove a Copilot account from Settings.
- Verify only the matching `copilot-<github-user-id>/` directory is deleted.
- Verify no host GitHub config is touched.

### 10.12 Native + Flatpak parity

- Repeat add, refresh, re-auth, and remove in Native and Flatpak builds.
- Under Flatpak, verify device flow opens the browser via the OpenURI portal.

---

## 11. Kimi for Coding

### 11.1 Add account and API-key input

- Settings → Kimi → Add account opens the API-key form without a browser flow.
- The API-key field is masked by default. Enter a test-only key, use the reveal
  control to verify the value is visible, then mask it again before saving.
- For OpenCode prefill, create a temporary test-only auth file containing a
  `kimi-for-coding` entry with a `key`, launch with
  `YAPCAP_OPENCODE_AUTH_PATH=/path/to/auth.json`, and open the Kimi add-account
  form. Verify the field is prefilled and the imported-from-OpenCode hint is
  shown. Do not use a production key in the fixture.
- Cancel the form and verify no Kimi account or key file was created.
- Save a non-empty test key and optional label. Verify one account appears,
  the new account is selected in single-account mode, the key is stored under
  native `~/.local/state/yapcap/kimi-accounts/<id>/` or Flatpak
  `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/kimi-accounts/<id>/`, and
  the key is absent from COSMIC configuration and logs.

### 11.2 Usage display and errors

- A successful refresh shows **Weekly** and **Rate Limit (300m)** windows when
  both are present, with used percentages and reset times.
- Verify remaining-only usage is displayed as used percentage and a response
  without `limits` still shows the Weekly window.
- A 401 or 403 response shows Login required without discarding the account.
- A 429 response shows rate-limited state and honors a numeric `Retry-After`
  value. A network failure keeps the previous snapshot visible as stale.
- Change or remove the OpenCode auth file after saving. Refresh Kimi and verify
  usage continues from YapCap storage or `KIMI_API_KEY`; the OpenCode file is
  not read or synchronized during refresh.

### 11.3 Multi-account selection

- Add a second Kimi account with a different label. Verify both accounts appear
  in Settings and the popup.
- With **Show all accounts** off, saving a Kimi account selects only that
  account. With it on, verify a new account is added while fewer than four are
  selected.
- Select four accounts, add another, and verify the fifth account is stored
  but remains unselected until an existing selection is changed.

### 11.4 Reauthentication

- Click the re-authenticate action for an existing Kimi account. Verify the
  target account id and label remain in the form.
- Enter a replacement key and edit the label before saving. Verify the existing
  id, original label, and creation time are preserved, while the stored key and
  authentication timestamps are replaced.
- Verify reauthentication leaves exactly one managed account and does not
  create a second generated Kimi account or directory.

### 11.5 Account removal and demo coverage

- Remove a Kimi account from Settings. Verify only the YapCap-owned Kimi account
  directory is deleted and the provider returns to Login required when empty.
- Run with `YAPCAP_DEMO=1`. Verify the Kimi card shows one API-key account with
  Weekly and Rate Limit windows and no Active badge.
- Repeat add, refresh, re-auth, and remove in Native and Flatpak builds.

---

## 12. Multi-account

- Add a second account for any provider.
- `Show all accounts` toggle appears only when the provider has more than one account.
- `Show all accounts` off — single active account column in popup.
- `Show all accounts` on — one panel usage-bar group per selected account; the popup keeps its fixed width and pages through selected accounts using the full-width account pager.
- Panel bars expand horizontally: one two-bar group per selected account.
- Unloaded accounts show 0% fill in panel until their snapshot arrives.
- Switching the active account in single-account mode triggers a refresh for only that provider, not a global refresh.

---

## 13. Stale / error states

- Kill network (`nmcli networking off`). Trigger a refresh. Verify "No internet connection. Showing cached data; information is not up to date." message. Cached usage data still visible. Re-enable network, verify Live badge returns.
- Wait 11 minutes without refreshing (or set refresh interval to max and advance clock). Verify account badge switches from Live to Stale. Status line appends "(stale)".
- Cold start with shared runtime state older than 10 minutes. Verify Stale badge on startup, not "Live · Updated 21 hours ago".
- Corrupt or delete the old `~/.cache/yapcap/snapshots.json` file. Verify current builds ignore it and continue from shared runtime/config without crashing.

---

## 14. Provider enable/disable

- Disable a provider via its settings toggle — provider tab disappears from popup nav.
- All provider-specific settings below the toggle are dimmed and non-interactive when disabled.
- Re-enable — tab reappears and a refresh is triggered.
- Fresh install with `auto_init_pending`: all providers enabled even with no accounts.

---

## 15. Popup sizing

- Popup width is 420 px for every provider and route, regardless of selected account count.
- Multi-account provider: a pager sits above one full-width account detail; previous/next buttons cycle accounts (wrapping at both ends), the active account name (or `Account N`) and `current of total` position are centered, and paging does not change the config selection.
- Multi-account provider paging a taller account resizes the popup height without clipping; switching provider tabs resets the pager to the first account.
- Provider nav tabs stay compact with two balanced rows at eight providers; labels stay readable and the selected tab keeps its accent border.
- Settings category navigation wraps into two balanced rows with all nine categories visible and clickable.
- OpenCode Go's provider tab shows the compact **5 Hour** and **Weekly** summary bars; opening its detail view also shows **Monthly**.
- Switching between provider tabs or routes updates the popup height immediately.
- Content taller than 1080 px: body scrolls, header/nav/footer stay fixed.

---

## 16. Accounts removed from filesystem

- Manually delete a provider account directory from the YapCap data tree (`~/.local/state/yapcap/<provider>-accounts/` native, or `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/<provider>-accounts/` Flatpak). Trigger a refresh. Verify the provider surfaces "Login required" or empty state rather than showing a stale snapshot indefinitely.

---

## 17. Config state file manipulation

- Delete old cached snapshots (native `~/.cache/yapcap/snapshots.json`, Flatpak `~/.var/app/io.github.TopiCsarno.YapCap/cache/yapcap/snapshots.json`). Restart. Verify runtime comes from shared COSMIC runtime state, not the old file.
- Delete the COSMIC config dir (`just clear-config`). Restart. Verify defaults apply: all providers enabled, refresh interval 300s, relative reset time, used amount format.
- Leave an older `~/.config/cosmic/io.github.TopiCsarno.YapCap/v501/` config in place. Restart the current build and verify `v502` defaults are used instead.
- Manually edit config to add a non-existent account id to `selected_codex_account_ids`. Restart. Verify graceful fallback to first valid account or Login Required — no crash.
- Set `refresh_interval_seconds = 5` in config. Verify it is clamped to 10s at runtime (not 5s).

---

## 18. Multi-process runtime sync

Use a COSMIC panel configured on two displays so two YapCap applet processes run
at the same time. For native builds, watch
`~/.local/state/yapcap/logs/yapcap.log`; for Flatpak builds, watch
`~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/logs/yapcap.log`.

- Startup: launch YapCap on both displays. Verify one process logs `refresh ownership acquired at startup` and another logs `refresh ownership held by another process; waiting for takeover`. Startup entries should include `process_id`, `pid`, `panel_output`, `lock_path`, `flatpak`, config version, shared runtime version/generation, and shared control version/generation.
- Provider selection sync: select a different provider tab on one display. Verify the other display switches to the same selected provider without sharing popup route/open state. If that provider is enabled and stale or missing data, verify a `provider_selected` shared refresh request is written and the owner observes it.
- Refresh now from owner display: click **Refresh now** on the display whose process is owner. Verify `manual refresh requested` includes its process id and control generation, and `owner evaluated shared refresh requests` includes the same generation and requester. Verify provider/account refresh start logs, `provider refresh finished`, `shared runtime written`, and `shared refresh request consumed`.
- Refresh now from non-owner display: click **Refresh now** on the other display. Verify its process id appears as the requester in the owner's compact evaluation, the non-owner does not write shared runtime, and both displays observe the final runtime generation.
- Automatic refresh: set a short refresh interval and wait for a stale or missing enabled provider. Verify only the owner logs provider refresh start/finish and `shared runtime written`; the non-owner does not run timer refresh work.
- After a successful refresh, verify shared runtime generations settle. A short burst for refreshing/final state is expected; continuous `shared runtime written` lines without provider refresh, config, account, or host-session changes are a bug.
- Owner takeover: identify the owner process from logs and terminate it, or remove the output that owns it. Verify a waiting process logs `refresh ownership acquired after waiting`, clears shared refresh requests, and resumes owner refresh behavior.
- Login, re-auth, account deletion: from the non-owner display, add or re-authenticate an account, then delete it. Verify account rows/settings update across displays through config/account storage immediately, while shared runtime refresh or cleanup is written only by the owner.
- UI attribution: alternate popup, navigation, provider-tab, and settings actions between displays. Verify every user-action event includes the initiating `process_id` and no inference from adjacent watcher events is required.
- Disabled provider: disable a provider, then click **Refresh now** from either display. Verify the disabled provider is absent from the compact evaluation outcomes and no provider refresh starts for it.
- Missing shared runtime: clear the shared runtime COSMIC config entry or make it invalid, then restart both displays. Verify logs include `shared runtime missing; using empty runtime fallback` or `shared runtime invalid; using empty runtime fallback`, credentials/account config remain intact, and the owner repopulates shared runtime on the next successful refresh.
- Old snapshot cache: create or corrupt native/Flatpak `snapshots.json` files and restart. Verify no active runtime data is loaded from those files and existing files remain on disk.

Expected diagnostic log patterns for this section:

- Ownership: `refresh ownership acquired at startup`, `refresh ownership held by another process; waiting for takeover`, `refresh ownership acquired after waiting`, `failed to acquire refresh ownership lock`.
- Shared control: `manual refresh requested`, `shared control observed`, `owner evaluated shared refresh requests`, `shared refresh request consumed`.
- Shared runtime: `shared runtime loaded`, `shared runtime missing; using empty runtime fallback`, `shared runtime invalid; using empty runtime fallback`, `shared runtime written`, `shared runtime observed`.
- Refresh lifecycle: `provider account refresh started`, `provider refresh finished`, provider refresh error logs.

---

## 19. Logging

- Native: verify `~/.local/state/yapcap/logs/yapcap.log`. Flatpak: verify `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/logs/yapcap.log`. Each is written during a normal session for that build.
- Verify no bearer tokens, access tokens, cookie values, or refresh tokens appear in the log.
- `RUST_LOG=debug just run` — debug output in terminal, still no credentials in log file.

---

## 20. Flatpak-specific

- Install via `just flatpak-install`. YapCap appears in COSMIC applet list.
- Install from the COSMIC Store. YapCap appears in the COSMIC panel applet picker after installation, uses the `io.github.TopiCsarno.YapCap` Flatpak id, appears under the applet category/filter, and shows "Place on desktop" rather than "Open".
- COSMIC Store details page shows developer `Tamás Csarnó`, version `0.5.2`, description paragraphs without manual line-break wrapping, and screenshots in this order: detail popup, Codex zoom, Claude Code zoom, Cursor zoom, Gemini zoom, Copilot zoom.
- About section shows "Flatpak" dist label.
- OAuth flows (Codex, Claude, Gemini, Copilot) open the system browser correctly from inside the sandbox.
- COSMIC dark/light theme and accent colour updates are observed immediately through the settings config watcher.
- Cursor add-account: Flatpak sandbox can read `~/.config/Cursor/User/globalStorage/state.vscdb` through the read-only home permission. Scan succeeds and account is stored.
- Flatpak permissions include `--filesystem=home:ro`, not writable home or `--filesystem=host`.
- Account state for the Flatpak build lives under `~/.var/app/io.github.TopiCsarno.YapCap/data/yapcap/` (not `~/.local/state/yapcap/`).
- `just flatpak-run` launches the installed Flatpak version.
- Native install (`just install`) About section shows "Native".

---

## 21. OpenCode discovery and OpenCode Go

### 21.1 Discovery boundaries

- Test with no OpenCode installation and no `~/.local/share/opencode/auth.json`.
  YapCap starts normally, shows the normal manual/native login controls, and does
  not require or attempt to launch an `opencode` executable.
- With an OpenCode installation but no relevant provider entry, add and
  reauthenticate forms remain usable manually and no account is created at
  startup. Repeat with a malformed file and an unknown credential type.
- Confirm discovery occurs only when adding or reauthenticating. Starting YapCap,
  refreshing, or changing provider tabs does not create accounts or reread the
  file.
- For fixtures only, set `YAPCAP_OPENCODE_AUTH_PATH` to a temporary auth file;
  do not use a production key. The normal default remains
  `~/.local/share/opencode/auth.json`.

### 21.2 API-key prefill and form behavior

- Put test-only `api` credentials for `kimi-for-coding`, `minimax`, and
  `opencode-go` in a fixture. Open each provider's Add account form and verify
  the key is prefilled, masked by default, and marked as imported from OpenCode.
- Edit the prefilled key or clear it. Verify the imported hint/provenance clears,
  the edited value remains the value that would be saved, and reveal/hide toggles
  never expose the key unless explicitly requested.
- Cancel each form and verify no account, key file, config value, or external
  OpenCode file is created or changed. Save a test-only value and verify it is
  copied to YapCap private storage only.
- For Minimax and Kimi, add a second labeled account, switch selection with
  **Show all accounts** off and on, reauthenticate each existing account, and
  delete each account. Verify labels, selected ids, private key files, and the
  normal four-account selection cap remain correct.

### 21.3 OAuth compatibility and native login priority

- Codex: use the normal **Sign in with ChatGPT** browser flow and verify it
  remains the primary action. With a fixture containing a valid `openai` OAuth
  record, use the separate **Import from OpenCode** action and verify the
  confirmed credential creates or updates the managed account. An `openai` API
  key is not offered as Codex subscription authentication.
- Copilot: complete the native GitHub device flow and verify it remains primary.
  With a fixture containing a valid `github-copilot` OAuth record, use the
  explicit OpenCode import action. Verify the OAuth `refresh` value is validated
  with GitHub identity and becomes the stored Copilot API token; the OAuth
  `access` value is not used for the API request. A GitHub Enterprise credential
  is declined; YapCap supports github.com only.
- Cancel imports and submit invalid, incomplete, or wrong-provider credentials.
  Verify existing accounts and stored tokens remain unchanged. Claude, Gemini,
  and Cursor forms remain unchanged: Anthropic/Google API keys and unconfirmed
  Cursor sources are not imported from OpenCode.

### 21.4 OpenCode Go auth and usage

- Add an OpenCode Go account with a manually entered test key. Verify the API-key
  field is masked/revealable, the account is stored privately, and the account is
  selected in single-account mode. Add multiple labeled accounts, switch the
  selected account, enable **Show all accounts**, and verify the normal selection
  cap and per-account refresh behavior.
- Add or reauthenticate with an `opencode-go` API-key fixture. Verify the field is
  editable and clearing it prevents saving. Reauthentication preserves the
  target account identity and does not create a second account.
- Stub or fixture `GET https://opencode.ai/zen/go/v1/usage` with Bearer
  authentication and valid server percentages/reset timestamps. Verify the popup
  shows **5 Hour**, **Weekly**, and **Monthly** windows in that order.
- Return HTTP 401 and verify authentication-required state without deleting the
  account. Return HTTP 403 and verify entitlement-required messaging without
  recommending reauthentication as a cure. Return HTTP 429 with and without
  `Retry-After` and verify normal rate-limit backoff.
- Disable the network or return a transient failure after a successful refresh.
  Verify the last successful three-window snapshot remains visible as stale and
  the account is not deleted. Restore connectivity and verify fresh data returns.
- Reauthenticate and delete OpenCode Go accounts. Verify deletion removes only
  YapCap's managed directory, updates selection, and never touches OpenCode.

### 21.5 Copy isolation, paths, and security

- Save a Kimi, Minimax, Codex, Copilot, or OpenCode Go credential, then change or
  delete OpenCode `auth.json`. Refresh the saved YapCap account and verify usage
  still comes from YapCap storage. Delete the YapCap account and verify the
  OpenCode file is byte-for-byte unchanged.
- Repeat with native and Flatpak builds. Native discovery uses the host default
  path; Flatpak discovery uses the mounted host home even when sandbox `HOME`
  points into `~/.var/app/`. Verify no OpenCode executable is needed in either
  build.
- Inspect logs, COSMIC config, and managed account metadata. Verify they contain
  no API keys, OAuth access/refresh tokens, auth-file contents, bearer headers,
  pasted codes, or secret paths. Verify API keys exist only in private YapCap
  account storage with owner-only permissions.
- Attempt to replace a managed account directory or key file with a symlink to an
  external path. Verify the operation is rejected and the external file remains
  unchanged. Verify no OpenCode path is persisted as an account root.
- Restrict an existing managed account directory or credential file incorrectly,
  then refresh or save the account. On Unix, verify the directory is repaired to
  `0700` and each managed credential file to `0600` before its contents are written.

### 21.6 Zen limitation

- Verify product/help text does not show a Zen balance, confuse local usage
  percentages with remaining credits, recommend console/cookie/private-RPC
  scraping, or cite an unsupported balance endpoint. It must state that Zen
  balance support is deferred until upstream provides a supported public API.
