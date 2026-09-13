# Z.AI quota fixtures

These sanitized responses exercise the fixed `GET https://api.z.ai/api/monitor/usage/quota/limit`
endpoint without credentials or personal data.

The parser sends `Authorization: Bearer <managed-key>` first and permits one retry with the raw
authorization value only after HTTP 401. `CREDIT_LIMIT` and `TOKENS_LIMIT` rows with `(unit: 3,
number: 5)` are the five-hour slot; `(unit: 6, number: 1)` is the weekly slot. A fixture-backed
`TIME_LIMIT` `(unit: 5, number: 1)` is rendered as MCP. The MCP fixture intentionally omits a
fixed duration because a reset instant does not establish a calendar-month span.

`percentage` is preferred over counters. When it is unavailable, `currentValue / usage` or
`(usage - remaining) / usage` supplies the used percentage only when the values are finite and
the usage limit is positive. Reset timestamps are epoch milliseconds. The responses contain only
synthetic values and are not raw captures.
