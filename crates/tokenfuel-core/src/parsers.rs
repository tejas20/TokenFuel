use crate::{Source, UsageLimit};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;

fn timestamp(v: Option<&Value>) -> Option<DateTime<Utc>> {
    let v = v?;
    v.as_i64()
        .and_then(|x| DateTime::from_timestamp(x, 0))
        .or_else(|| {
            v.as_str()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|x| x.with_timezone(&Utc))
        })
}
fn decimal(v: Option<&Value>) -> Option<Decimal> {
    let v = v?;
    let s = match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    Decimal::from_str(&s).ok()
}
fn period_name(minutes: i64) -> String {
    match minutes {
        1440 => "Daily".into(),
        10080 => "Weekly".into(),
        x if x >= 60 && x % 60 == 0 => format!("{} hours", x / 60),
        x => format!("{x} minutes"),
    }
}

/// The documented app-server multi-bucket view takes precedence over its legacy alias.
pub fn codex(value: &Value) -> Vec<UsageLimit> {
    let mut out = vec![];
    let buckets: Vec<(String, &Value)> = if let Some(map) = value
        .get("rateLimitsByLimitId")
        .and_then(Value::as_object)
        .filter(|m| !m.is_empty())
    {
        map.iter().map(|(k, v)| (k.clone(), v)).collect()
    } else if let Some(v) = value.get("rateLimits").filter(|v| v.is_object()) {
        vec![(
            v.get("limitId")
                .and_then(Value::as_str)
                .unwrap_or("codex")
                .into(),
            v,
        )]
    } else {
        vec![]
    };
    for (id, bucket) in buckets {
        let product = bucket
            .get("limitName")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or("Codex");
        for slot in ["primary", "secondary"] {
            let Some(window) = bucket.get(slot).filter(|x| x.is_object()) else {
                continue;
            };
            let Some(p) = window.get("usedPercent").and_then(Value::as_f64) else {
                continue;
            };
            let minutes = window
                .get("windowDurationMins")
                .and_then(Value::as_i64)
                .filter(|m| *m > 0);
            let quota_id = minutes
                .map(|minutes| {
                    let repeated = ["primary", "secondary"]
                        .iter()
                        .filter(|slot| {
                            bucket
                                .get(**slot)
                                .and_then(|w| w.get("windowDurationMins"))
                                .and_then(Value::as_i64)
                                == Some(minutes)
                        })
                        .count()
                        > 1;
                    if repeated {
                        format!("{id}:window:{minutes}:{slot}")
                    } else {
                        format!("{id}:window:{minutes}")
                    }
                })
                .unwrap_or_else(|| format!("{id}:unknown:{slot}"));
            let label = minutes
                .map(period_name)
                .unwrap_or_else(|| "Reported window".into());
            if let Some(mut q) =
                UsageLimit::percentage(&quota_id, &label, product, "rolling", p, Source::Documented)
            {
                q.resets_at = timestamp(window.get("resetsAt"));
                q.period = match window.get("windowDurationMins").and_then(Value::as_i64) {
                    Some(10080) => "weekly",
                    Some(1440) => "daily",
                    _ => "rolling",
                }
                .into();
                out.push(q);
            }
        }
        if let Some(credits) = bucket.get("credits") {
            let unlimited = credits.get("unlimited").and_then(Value::as_bool) == Some(true);
            if (unlimited || credits.get("hasCredits").and_then(Value::as_bool) == Some(true))
                && let Ok(mut q) = UsageLimit::amounts(
                    &format!("{id}:credits"),
                    "Credit balance",
                    product,
                    "credits",
                    "balance",
                    Decimal::ZERO,
                    None,
                    Source::Documented,
                )
            {
                q.unlimited = unlimited;
                q.used = None;
                q.remaining = decimal(credits.get("balance")).map(|x| x.to_string());
                out.push(q);
            }
        }
    }
    out
}

/// Claude OAuth and web usage responses. An Enterprise account can return spend only.
pub fn claude(value: &Value) -> Vec<UsageLimit> {
    let mut out = vec![];
    for (key, label, period) in [
        ("five_hour", "5 hours", "rolling"),
        ("seven_day", "Weekly", "weekly"),
        ("seven_day_sonnet", "Sonnet weekly", "weekly"),
        ("seven_day_opus", "Opus weekly", "weekly"),
    ] {
        if let Some(v) = value.get(key)
            && let Some(p) = v.get("utilization").and_then(Value::as_f64)
            && let Some(mut q) =
                UsageLimit::percentage(key, label, "Claude", period, p, Source::Experimental)
        {
            q.resets_at = timestamp(v.get("resets_at"));
            out.push(q);
        }
    }
    if let Some(limits) = value.get("limits").and_then(Value::as_array) {
        for v in limits {
            let Some(p) = v
                .get("percent")
                .or_else(|| v.get("utilization"))
                .and_then(Value::as_f64)
            else {
                continue;
            };
            let kind = v.get("kind").and_then(Value::as_str).unwrap_or("reported");
            let model = v
                .pointer("/scope/model/display_name")
                .and_then(Value::as_str)
                .or_else(|| v.pointer("/scope/model/id").and_then(Value::as_str))
                .unwrap_or("");
            // Non-model scopes also identify separate pools. Never let two
            // workspace/feature quotas share a pin or alert identity.
            let scope_identity = v
                .get("scope")
                .filter(|scope| !scope.is_null())
                .map(|scope| {
                    let hash = scope
                        .to_string()
                        .bytes()
                        .fold(0xcbf29ce484222325u64, |hash, byte| {
                            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
                        });
                    format!("{hash:016x}")
                })
                .unwrap_or_default();
            let label = if model.is_empty() {
                kind.replace('_', " ")
            } else {
                format!("{model} · {}", kind.replace('_', " "))
            };
            if let Some(mut q) = UsageLimit::percentage(
                &format!(
                    "scoped:{kind}:{}",
                    v.pointer("/scope/model/id")
                        .and_then(Value::as_str)
                        .unwrap_or(if model.is_empty() {
                            &scope_identity
                        } else {
                            model
                        })
                ),
                &label,
                "Claude",
                kind,
                p,
                Source::Experimental,
            ) {
                q.scope = if model.is_empty() {
                    v.get("scope")
                        .filter(|scope| !scope.is_null())
                        .map(Value::to_string)
                        .unwrap_or_else(|| "account".into())
                } else {
                    model.into()
                };
                q.resets_at = timestamp(v.get("resets_at"));
                out.push(q);
            }
        }
    }
    if let Some(spend) = value
        .get("spend")
        .filter(|s| s.get("enabled").and_then(Value::as_bool) == Some(true))
    {
        let used = minor_amount(spend.get("used"));
        let used_currency = spend.pointer("/used/currency").and_then(Value::as_str);
        let limit_currency = spend.pointer("/limit/currency").and_then(Value::as_str);
        let total = if used_currency
            .zip(limit_currency)
            .is_some_and(|(a, b)| a != b)
        {
            None
        } else {
            minor_amount(spend.get("limit"))
        };
        let currency = spend
            .pointer("/used/currency")
            .or_else(|| spend.pointer("/limit/currency"))
            .and_then(Value::as_str)
            .unwrap_or("currency not reported");
        if let Some(used) = used
            && let Ok(mut q) = UsageLimit::amounts(
                "monthly-spend",
                "Monthly budget",
                "Claude",
                currency,
                "monthly",
                used,
                total,
                Source::Experimental,
            )
        {
            q.resets_at = timestamp(spend.get("resets_at"));
            out.push(q);
        }
    }
    if let Some(extra) = value
        .get("extra_usage")
        .filter(|s| s.get("is_enabled").and_then(Value::as_bool) == Some(true))
        && let Some(used) = decimal(extra.get("used_credits"))
    {
        // This legacy response uses cents, as does its monthly_limit field.
        if let Ok(q) = UsageLimit::amounts(
            "extra-usage",
            "Extra usage budget",
            "Claude",
            "USD",
            "monthly",
            used / Decimal::ONE_HUNDRED,
            decimal(extra.get("monthly_limit")).map(|v| v / Decimal::ONE_HUNDRED),
            Source::Experimental,
        ) {
            out.push(q);
        }
    }
    out
}

/// OpenCode Go subscription status. Uses micro-cents (1 USD = 100,000,000 micro-cents).
pub fn opencode(value: &Value) -> Vec<UsageLimit> {
    let mut out = Vec::new();
    let access = value.get("access").unwrap_or(value);
    let Some(meters) = access.get("meters").and_then(Value::as_object) else {
        return out;
    };
    let microcents_divisor = Decimal::from(100_000_000u64);
    let ends_at = timestamp(access.get("endsAt").or_else(|| access.get("ends_at")));

    let specs: [(&[&str], &str, &str, &str); 3] = [
        (&["fiveHour", "five_hour"], "opencode:five_hour", "5 hours", "rolling"),
        (&["week", "weekly"], "opencode:weekly", "Weekly", "weekly"),
        (&["month", "monthly"], "opencode:monthly", "Monthly", "monthly"),
    ];

    for (keys, id, name, period) in specs {
        let meter_obj = keys.iter().find_map(|k| meters.get(*k)).and_then(Value::as_object);
        let Some(meter) = meter_obj else {
            continue;
        };
        let used_dec = decimal(meter.get("usedMicroCents").or_else(|| meter.get("used_micro_cents")));
        let limit_dec = decimal(meter.get("limitMicroCents").or_else(|| meter.get("limit_micro_cents")));

        if let Some(used_raw) = used_dec {
            let used = used_raw / microcents_divisor;
            let total = limit_dec.map(|l| l / microcents_divisor);
            if let Ok(mut q) = UsageLimit::amounts(
                id,
                name,
                "OpenCode Go",
                "USD",
                period,
                used,
                total,
                Source::Experimental,
            ) {
                let res_time = timestamp(meter.get("resetsAt").or_else(|| meter.get("resets_at")));
                q.resets_at = res_time.or(if period == "monthly" { ends_at } else { None });
                out.push(q);
            }
        }
    }
    out
}

/// Cursor usage summary. Maps component pools to Cursor Models and API models.
/// Avoids duplicating totalPercentUsed when component pools are present.
pub fn cursor(value: &Value) -> Vec<UsageLimit> {
    let mut out = Vec::new();
    let reset = timestamp(
        value
            .get("billingCycleEnd")
            .or_else(|| value.get("billing_cycle_end")),
    );
    let plan = value
        .pointer("/individualUsage/plan")
        .or_else(|| value.get("plan"))
        .or_else(|| value.get("individualUsage"))
        .unwrap_or(value);

    let auto = plan
        .get("autoPercentUsed")
        .or_else(|| plan.get("auto_percent_used"))
        .and_then(Value::as_f64);
    let api = plan
        .get("apiPercentUsed")
        .or_else(|| plan.get("api_percent_used"))
        .and_then(Value::as_f64);
    let total = plan
        .get("totalPercentUsed")
        .or_else(|| plan.get("total_percent_used"))
        .and_then(Value::as_f64);

    let has_components = auto.is_some() || api.is_some();

    if let Some(used) = auto
        && let Some(mut q) = UsageLimit::percentage(
            "cursor:plan:auto",
            "Cursor Models",
            "Cursor",
            "monthly",
            used,
            Source::Experimental,
        )
    {
        q.resets_at = reset;
        out.push(q);
    }

    if let Some(used) = api
        && let Some(mut q) = UsageLimit::percentage(
            "cursor:plan:api",
            "API models",
            "Cursor",
            "monthly",
            used,
            Source::Experimental,
        )
    {
        q.resets_at = reset;
        out.push(q);
    }

    if !has_components
        && let Some(used) = total
        && let Some(mut q) = UsageLimit::percentage(
            "cursor:plan:total",
            "Monthly plan",
            "Cursor",
            "monthly",
            used,
            Source::Experimental,
        )
    {
        q.resets_at = reset;
        out.push(q);
    }

    out
}

fn grok_cents(v: Option<&Value>) -> Option<Decimal> {
    let v = v?;
    let inner = if let Some(obj) = v.as_object() {
        obj.get("val")?
    } else {
        v
    };
    let s = match inner {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    let cents = Decimal::from_str(&s).ok()?;
    Some(cents / Decimal::ONE_HUNDRED)
}

/// Grok billing and subscription status. Monetary values are in cents.
/// Shared pools and product breakdowns (Build, Chat, API) are kept distinct.
pub fn grok(value: &Value) -> Vec<UsageLimit> {
    let mut out = Vec::new();
    let config = value.get("config").unwrap_or(value);

    let period_type = config
        .pointer("/currentPeriod/type")
        .or_else(|| config.pointer("/currentPeriod/periodType"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let period = if period_type.to_uppercase().contains("WEEKLY") {
        "weekly"
    } else if period_type.to_uppercase().contains("MONTHLY") {
        "monthly"
    } else {
        "rolling"
    };

    let reset = timestamp(config.pointer("/currentPeriod/end"));

    // 1. Shared credit usage percent
    if let Some(used) = config
        .get("creditUsagePercent")
        .or_else(|| config.get("credit_usage_percent"))
        .and_then(Value::as_f64)
        && let Some(mut q) = UsageLimit::percentage(
            "grok:credits",
            "Grok Credits",
            "Grok",
            period,
            used,
            Source::Experimental,
        )
    {
        q.resets_at = reset;
        out.push(q);
    }

    // 2. On-demand usage / spending
    let on_demand_used = grok_cents(config.get("onDemandUsed").or_else(|| config.get("on_demand_used")));
    if let Some(used) = on_demand_used {
        let total = grok_cents(config.get("onDemandCap").or_else(|| config.get("on_demand_cap")));
        if let Ok(mut q) = UsageLimit::amounts(
            "grok:on_demand",
            "On-demand spending",
            "Grok",
            "USD",
            period,
            used,
            total,
            Source::Experimental,
        ) {
            q.resets_at = reset;
            out.push(q);
        }
    }

    // 3. Prepaid balance
    if let Some(balance) = grok_cents(config.get("prepaidBalance").or_else(|| config.get("prepaid_balance")))
        && balance > Decimal::ZERO
        && let Ok(mut q) = UsageLimit::amounts(
            "grok:prepaid_balance",
            "Prepaid balance",
            "Grok",
            "USD",
            "balance",
            Decimal::ZERO,
            Some(balance),
            Source::Experimental,
        )
    {
        q.used = None;
        q.remaining = Some(balance.normalize().to_string());
        q.total = Some(balance.normalize().to_string());
        q.remaining_percent = None;
        out.push(q);
    }

    out
}

/// GitHub Copilot quota and entitlement status.
/// Handles documented quota_snapshots as well as limited_user_quotas.
pub fn copilot(value: &Value) -> Vec<UsageLimit> {
    let mut out = Vec::new();
    let reset = timestamp(
        value
            .get("quota_reset_date_utc")
            .or_else(|| value.get("quotaResetDateUtc"))
            .or_else(|| value.get("quota_reset_date"))
            .or_else(|| value.get("limited_user_reset_date"))
            .or_else(|| value.get("resetDate")),
    );

    if let Some(snapshots) = value
        .get("quota_snapshots")
        .or_else(|| value.get("quotaSnapshots"))
        .and_then(Value::as_object)
    {
        for (key, snap) in snapshots {
            let name = match key.as_str() {
                "premium_interactions" => "Premium requests",
                "chat" => "Chat",
                "completions" => "Code completions",
                other => other,
            };
            let unit = match key.as_str() {
                "completions" => "completions",
                _ => "requests",
            };
            let unlimited = snap.get("unlimited").and_then(Value::as_bool).unwrap_or(false)
                || snap.get("entitlement").and_then(Value::as_i64) == Some(-1);

            if unlimited {
                if let Ok(mut q) = UsageLimit::amounts(
                    &format!("copilot:{key}"),
                    name,
                    "Copilot",
                    unit,
                    "monthly",
                    Decimal::ZERO,
                    None,
                    Source::Documented,
                ) {
                    q.unlimited = true;
                    q.used = None;
                    q.total = None;
                    q.remaining = None;
                    q.remaining_percent = None;
                    q.resets_at = reset;
                    out.push(q);
                }
            } else {
                let ent = decimal(snap.get("entitlement"));
                let rem = decimal(snap.get("remaining"));
                let pct_rem = snap
                    .get("percent_remaining")
                    .or_else(|| snap.get("percentRemaining"))
                    .and_then(Value::as_f64);

                if let (Some(ent), Some(rem)) = (ent, rem) {
                    let used = (ent - rem).max(Decimal::ZERO);
                    if let Ok(mut q) = UsageLimit::amounts(
                        &format!("copilot:{key}"),
                        name,
                        "Copilot",
                        unit,
                        "monthly",
                        used,
                        Some(ent),
                        Source::Documented,
                    ) {
                        if let Some(p) = pct_rem {
                            q.remaining_percent = Some(p.clamp(0.0, 100.0));
                        }
                        q.resets_at = reset;
                        out.push(q);
                    }
                } else if let Some(p) = pct_rem
                    && let Some(mut q) = UsageLimit::percentage(
                        &format!("copilot:{key}"),
                        name,
                        "Copilot",
                        "monthly",
                        100.0 - p,
                        Source::Documented,
                    )
                {
                    q.resets_at = reset;
                    out.push(q);
                }
            }
        }
    } else if let Some(limited) = value.get("limited_user_quotas").and_then(Value::as_object) {
        let monthly = value.get("monthly_quotas").and_then(Value::as_object);
        for (key, rem_val) in limited {
            let name = match key.as_str() {
                "premium_interactions" => "Premium requests",
                "chat" => "Chat",
                "completions" => "Code completions",
                other => other,
            };
            let unit = match key.as_str() {
                "completions" => "completions",
                _ => "requests",
            };
            let rem = decimal(Some(rem_val));
            let total = monthly.and_then(|m| m.get(key)).and_then(|v| decimal(Some(v)));

            if let (Some(rem), Some(tot)) = (rem, total) {
                let used = (tot - rem).max(Decimal::ZERO);
                if let Ok(mut q) = UsageLimit::amounts(
                    &format!("copilot:{key}"),
                    name,
                    "Copilot",
                    unit,
                    "monthly",
                    used,
                    Some(tot),
                    Source::Documented,
                ) {
                    q.resets_at = reset;
                    out.push(q);
                }
            }
        }
    }

    out
}

/// Google Antigravity quota and model limits.
/// Supports both retrieveUserQuotaSummary groups/buckets and fetchAvailableModels models.
pub fn antigravity(value: &Value) -> Vec<UsageLimit> {
    let mut out = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // 1. Quota Summary Groups
    let groups = value
        .get("groups")
        .or_else(|| value.pointer("/summary/groups"))
        .and_then(Value::as_array);

    if let Some(groups) = groups {
        for group in groups {
            let group_display = group
                .get("displayName")
                .or_else(|| group.get("display_name"))
                .and_then(Value::as_str)
                .unwrap_or("Antigravity");

            if let Some(buckets) = group.get("buckets").and_then(Value::as_array) {
                for bucket in buckets {
                    let bucket_id = bucket
                        .get("bucketId")
                        .or_else(|| bucket.get("bucket_id"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let window = bucket
                        .get("window")
                        .and_then(Value::as_str)
                        .unwrap_or("rolling");
                    let period = if window.eq_ignore_ascii_case("5h") {
                        "rolling"
                    } else if window.eq_ignore_ascii_case("weekly") {
                        "weekly"
                    } else if window.eq_ignore_ascii_case("monthly") {
                        "monthly"
                    } else {
                        "rolling"
                    };
                    let bucket_display = bucket
                        .get("displayName")
                        .or_else(|| bucket.get("display_name"))
                        .and_then(Value::as_str);

                    let name = if let Some(bd) = bucket_display {
                        bd.to_string()
                    } else if !bucket_id.is_empty() {
                        bucket_id.replace('-', " ")
                    } else {
                        format!("{group_display} {window}")
                    };

                    let pool_id = if !bucket_id.is_empty() {
                        format!("antigravity:{bucket_id}")
                    } else {
                        format!(
                            "antigravity:{}:{}",
                            group_display.to_lowercase().replace(' ', "_"),
                            window
                        )
                    };

                    if !seen_ids.insert(pool_id.clone()) {
                        continue;
                    }

                    let remaining_fraction = bucket
                        .get("remainingFraction")
                        .or_else(|| bucket.get("remaining_fraction"))
                        .and_then(Value::as_f64);

                    if let Some(rf) = remaining_fraction
                        && let Some(mut q) = UsageLimit::percentage(
                            &pool_id,
                            &name,
                            "Antigravity",
                            period,
                            100.0 - (rf.clamp(0.0, 1.0) * 100.0).clamp(0.0, 100.0),
                            Source::Experimental,
                        )
                    {
                        q.remaining_percent = Some((rf.clamp(0.0, 1.0) * 100.0).clamp(0.0, 100.0));
                        q.resets_at = timestamp(
                            bucket
                                .get("resetTime")
                                .or_else(|| bucket.get("reset_time")),
                        );
                        out.push(q);
                    }
                }
            }
        }
    }

    // 2. Models fallback if no quota summary buckets were found
    if out.is_empty() {
        let models = value
            .get("models")
            .and_then(|m| m.get("models").or(Some(m)))
            .or_else(|| value.pointer("/models/models"))
            .and_then(Value::as_object);

        if let Some(models) = models {
            for (model_name, info) in models {
                let is_display_model = model_name.starts_with("gemini")
                    || model_name.starts_with("claude")
                    || model_name.starts_with("gpt")
                    || model_name.starts_with("image")
                    || model_name.starts_with("imagen");
                if !is_display_model {
                    continue;
                }

                let quota_info = info
                    .get("quotaInfo")
                    .or_else(|| info.get("quota_info"));
                let remaining_fraction = quota_info
                    .and_then(|q| q.get("remainingFraction").or_else(|| q.get("remaining_fraction")))
                    .and_then(Value::as_f64);

                if let Some(rf) = remaining_fraction {
                    let pool_id = format!("antigravity:model:{model_name}");
                    if !seen_ids.insert(pool_id.clone()) {
                        continue;
                    }

                    let remaining_pct = (rf.clamp(0.0, 1.0) * 100.0).clamp(0.0, 100.0);
                    let used_pct = 100.0 - remaining_pct;

                    if let Some(mut q) = UsageLimit::percentage(
                        &pool_id,
                        model_name,
                        "Antigravity",
                        "rolling",
                        used_pct,
                        Source::Experimental,
                    ) {
                        q.remaining_percent = Some(remaining_pct);
                        q.resets_at = timestamp(
                            quota_info
                                .and_then(|q| q.get("resetTime").or_else(|| q.get("reset_time"))),
                        );
                        out.push(q);
                    }
                }
            }
        }
    }

    out
}

fn minor_amount(v: Option<&Value>) -> Option<Decimal> {
    let v = v?;
    let amount = decimal(v.get("amount_minor"))?;
    let exponent = v.get("exponent").and_then(Value::as_u64)?;
    if exponent > 9 {
        return None;
    }
    Some(amount / Decimal::from(10u64.pow(exponent as u32)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn codex_multi_bucket_is_not_double_counted() {
        let v = json!({"rateLimits":{"primary":{"usedPercent":99}},"rateLimitsByLimitId":{"codex":{"primary":{"usedPercent":6,"windowDurationMins":300,"resetsAt":1790951817},"secondary":{"usedPercent":5,"windowDurationMins":10080}},"model":{"primary":{"usedPercent":30,"windowDurationMins":60}}}});
        let q = codex(&v);
        assert_eq!(q.len(), 3);
        assert!(
            q.iter()
                .any(|x| x.remaining_percent == Some(94.0) && x.name == "5 hours")
        );
        assert!(q.iter().any(|x| x.name == "Weekly"));
    }
    #[test]
    fn enterprise_monthly_only_creates_no_fictional_windows() {
        let v = json!({"spend":{"enabled":true,"used":{"amount_minor":"8012.5","exponent":2},"limit":{"amount_minor":"20000","exponent":2,"currency":"USD"},"resets_at":"2026-10-17T00:00:00Z"}});
        let q = claude(&v);
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].remaining.as_deref(), Some("119.875"));
        assert_eq!(
            q[0].resets_at.unwrap().to_rfc3339(),
            "2026-10-17T00:00:00+00:00"
        );
    }
    #[test]
    fn codex_pin_tracks_window_duration_when_provider_moves_weekly_to_primary() {
        let both = codex(
            &json!({"rateLimits":{"primary":{"usedPercent":10,"windowDurationMins":300},"secondary":{"usedPercent":20,"windowDurationMins":10080}}}),
        );
        let weekly_only =
            codex(&json!({"rateLimits":{"primary":{"usedPercent":21,"windowDurationMins":10080}}}));
        assert_eq!(both[1].id, weekly_only[0].id);
        assert_ne!(both[0].id, weekly_only[0].id);
        let duplicate_duration = codex(
            &json!({"rateLimits":{"primary":{"usedPercent":10,"windowDurationMins":300},"secondary":{"usedPercent":20,"windowDurationMins":300}}}),
        );
        assert_ne!(duplicate_duration[0].id, duplicate_duration[1].id);
    }
    #[test]
    fn disabled_missing_changed_fields_do_not_become_zero_or_unlimited() {
        assert!(claude(&json!({"five_hour":null,"spend":{"enabled":false}})).is_empty());
        assert!(codex(&json!({"rateLimits":{"primary":{"usedPercent":"unknown"}}})).is_empty());
    }
    #[test]
    fn weekly_and_monthly_scoped_pools_remain_separate() {
        let q = claude(
            &json!({"limits":[{"kind":"weekly_all","percent":25},{"kind":"monthly_scoped","percent":90,"scope":{"model":{"display_name":"Model A"}}}]}),
        );
        assert_eq!(q.len(), 2);
        assert_eq!(q[1].remaining_percent, Some(10.0));
    }

    #[test]
    fn scoped_utilization_alias_and_non_model_scopes_remain_distinct() {
        let a = json!({"kind":"monthly_scoped","utilization":43,"scope":{"workspace":"alpha"}});
        let b = json!({"kind":"monthly_scoped","percent":80,"scope":{"workspace":"beta"}});
        let first = claude(&json!({"limits":[a.clone(),b.clone()]}));
        let reordered = claude(&json!({"limits":[b,a]}));
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].remaining_percent, Some(57.0));
        assert!(first[0].scope.contains("alpha"));
        assert_ne!(first[0].id, first[1].id);
        assert_eq!(first[0].id, reordered[1].id);
        assert_eq!(first[1].id, reordered[0].id);
    }
    #[test]
    fn currency_mismatch_does_not_create_a_percentage() {
        let q = claude(
            &json!({"spend":{"enabled":true,"used":{"amount_minor":100,"exponent":2,"currency":"EUR"},"limit":{"amount_minor":1000,"exponent":2,"currency":"USD"}}}),
        );
        assert_eq!(q[0].remaining_percent, None);
        assert_eq!(q[0].unit, "EUR");
        assert!(claude(&json!({"spend":{"enabled":true,"used":{"amount_minor":100}}})).is_empty());
    }
    #[test]
    fn quota_pin_identity_survives_reordering() {
        let a =
            json!({"kind":"weekly","percent":25,"scope":{"model":{"id":"a","display_name":"A"}}});
        let b =
            json!({"kind":"weekly","percent":50,"scope":{"model":{"id":"b","display_name":"B"}}});
        let first = claude(&json!({"limits":[a.clone(),b.clone()]}));
        let second = claude(&json!({"limits":[b,a]}));
        assert_eq!(first[0].id, second[1].id);
    }

    #[test]
    fn opencode_parses_microcents_and_windows() {
        let text = include_str!("../tests/fixtures/opencode.json");
        let v: Value = serde_json::from_str(text).unwrap();
        let q = opencode(&v);
        assert_eq!(q.len(), 3);

        let five_hour = q.iter().find(|x| x.id == "opencode:five_hour").unwrap();
        assert_eq!(five_hour.name, "5 hours");
        assert_eq!(five_hour.used.as_deref(), Some("1.25"));
        assert_eq!(five_hour.total.as_deref(), Some("5"));
        assert_eq!(five_hour.remaining.as_deref(), Some("3.75"));
        assert_eq!(five_hour.remaining_percent, Some(75.0));
        assert_eq!(five_hour.unit, "USD");
        assert_eq!(
            five_hour.resets_at.unwrap().to_rfc3339(),
            "2026-10-02T19:30:00+00:00"
        );

        let weekly = q.iter().find(|x| x.id == "opencode:weekly").unwrap();
        assert_eq!(weekly.used.as_deref(), Some("14"));
        assert_eq!(weekly.total.as_deref(), Some("35"));
        assert_eq!(weekly.remaining.as_deref(), Some("21"));
        assert_eq!(weekly.remaining_percent, Some(60.0));

        let monthly = q.iter().find(|x| x.id == "opencode:monthly").unwrap();
        assert_eq!(monthly.used.as_deref(), Some("45"));
        assert_eq!(monthly.total.as_deref(), Some("150"));
        assert_eq!(monthly.remaining.as_deref(), Some("105"));
        assert_eq!(monthly.remaining_percent, Some(70.0));
        assert_eq!(
            monthly.resets_at.unwrap().to_rfc3339(),
            "2026-11-01T00:00:00+00:00"
        );
    }

    #[test]
    fn cursor_parses_component_pools_without_duplicating_total() {
        let text = include_str!("../tests/fixtures/cursor.json");
        let v: Value = serde_json::from_str(text).unwrap();
        let q = cursor(&v);
        assert_eq!(q.len(), 2);

        let auto = q.iter().find(|x| x.id == "cursor:plan:auto").unwrap();
        assert_eq!(auto.name, "Cursor Models");
        assert_eq!(auto.remaining_percent, Some(57.5));
        assert_eq!(
            auto.resets_at.unwrap().to_rfc3339(),
            "2026-10-31T23:59:59+00:00"
        );

        let api = q.iter().find(|x| x.id == "cursor:plan:api").unwrap();
        assert_eq!(api.name, "API models");
        assert_eq!(api.remaining_percent, Some(85.0));

        // totalPercentUsed is not duplicated
        assert!(q.iter().all(|x| x.id != "cursor:plan:total"));

        // Fallback test: legacy or unified plan with only totalPercentUsed
        let legacy = json!({
            "billingCycleEnd": "2026-10-31T23:59:59.000Z",
            "individualUsage": {
                "plan": {
                    "totalPercentUsed": 40.0
                }
            }
        });
        let q_legacy = cursor(&legacy);
        assert_eq!(q_legacy.len(), 1);
        assert_eq!(q_legacy[0].id, "cursor:plan:total");
        assert_eq!(q_legacy[0].name, "Monthly plan");
        assert_eq!(q_legacy[0].remaining_percent, Some(60.0));
    }

    #[test]
    fn grok_parses_shared_credits_on_demand_and_prepaid_balance() {
        let text = include_str!("../tests/fixtures/grok.json");
        let v: Value = serde_json::from_str(text).unwrap();
        let q = grok(&v);
        assert_eq!(q.len(), 3);

        // Shared credits pool (64.5% remaining)
        let credits = q.iter().find(|x| x.id == "grok:credits").unwrap();
        assert_eq!(credits.name, "Grok Credits");
        assert_eq!(credits.remaining_percent, Some(64.5));
        assert_eq!(credits.period, "monthly");
        assert_eq!(
            credits.resets_at.unwrap().to_rfc3339(),
            "2026-10-31T23:59:59+00:00"
        );

        // On-demand spending ($12.50 of $50.00 => 75% remaining)
        let on_demand = q.iter().find(|x| x.id == "grok:on_demand").unwrap();
        assert_eq!(on_demand.name, "On-demand spending");
        assert_eq!(on_demand.used.as_deref(), Some("12.5"));
        assert_eq!(on_demand.total.as_deref(), Some("50"));
        assert_eq!(on_demand.remaining.as_deref(), Some("37.5"));
        assert_eq!(on_demand.remaining_percent, Some(75.0));

        // Prepaid balance ($25.00 remaining, no depleting percent)
        let prepaid = q.iter().find(|x| x.id == "grok:prepaid_balance").unwrap();
        assert_eq!(prepaid.name, "Prepaid balance");
        assert_eq!(prepaid.used, None);
        assert_eq!(prepaid.remaining.as_deref(), Some("25"));
        assert_eq!(prepaid.remaining_percent, None);

        // Product breakdowns (Build, Chat, API) must not become separate depleting meters
        assert!(q.iter().all(|x| x.name != "Build" && x.name != "Chat" && x.name != "API"));
    }

    #[test]
    fn copilot_parses_premium_unlimited_and_completions() {
        let text = include_str!("../tests/fixtures/copilot.json");
        let v: Value = serde_json::from_str(text).unwrap();
        let q = copilot(&v);
        assert_eq!(q.len(), 3);

        // Premium interactions: entitlement 300, remaining 210, percent 70%
        let premium = q.iter().find(|x| x.id == "copilot:premium_interactions").unwrap();
        assert_eq!(premium.name, "Premium requests");
        assert_eq!(premium.used.as_deref(), Some("90"));
        assert_eq!(premium.total.as_deref(), Some("300"));
        assert_eq!(premium.remaining.as_deref(), Some("210"));
        assert_eq!(premium.remaining_percent, Some(70.0));
        assert_eq!(premium.unit, "requests");
        assert!(!premium.unlimited);
        assert_eq!(
            premium.resets_at.unwrap().to_rfc3339(),
            "2026-11-01T00:00:00+00:00"
        );

        // Chat: unlimited (-1 entitlement)
        let chat = q.iter().find(|x| x.id == "copilot:chat").unwrap();
        assert_eq!(chat.name, "Chat");
        assert!(chat.unlimited);
        assert_eq!(chat.remaining_percent, None);
        assert_eq!(chat.unit, "requests");

        // Completions: entitlement 2000, remaining 1500, percent 75%
        let completions = q.iter().find(|x| x.id == "copilot:completions").unwrap();
        assert_eq!(completions.name, "Code completions");
        assert_eq!(completions.used.as_deref(), Some("500"));
        assert_eq!(completions.total.as_deref(), Some("2000"));
        assert_eq!(completions.remaining.as_deref(), Some("1500"));
        assert_eq!(completions.remaining_percent, Some(75.0));
        assert_eq!(completions.unit, "completions");
        assert!(!completions.unlimited);
    }

    #[test]
    fn antigravity_parses_groups_and_fallback_models() {
        let text = include_str!("../tests/fixtures/antigravity.json");
        let v: Value = serde_json::from_str(text).unwrap();
        let q = antigravity(&v);
        assert_eq!(q.len(), 2);

        let session = q.iter().find(|x| x.id == "antigravity:gemini-5h").unwrap();
        assert_eq!(session.name, "Session 5h");
        assert_eq!(session.period, "rolling");
        assert_eq!(session.remaining_percent, Some(85.0));
        assert_eq!(
            session.resets_at.unwrap().to_rfc3339(),
            "2026-10-02T20:15:00+00:00"
        );

        let weekly = q.iter().find(|x| x.id == "antigravity:gemini-weekly").unwrap();
        assert_eq!(weekly.name, "Weekly Quota");
        assert_eq!(weekly.period, "weekly");
        assert_eq!(weekly.remaining_percent, Some(60.0));
        assert_eq!(
            weekly.resets_at.unwrap().to_rfc3339(),
            "2026-10-08T00:00:00+00:00"
        );

        // Standalone models fallback test
        let models_text = r#"{
            "models": {
                "gemini-2.5-pro": {
                    "quotaInfo": {
                        "remainingFraction": 0.72,
                        "resetTime": "2026-10-02T21:00:00Z"
                    }
                }
            }
        }"#;
        let mv: Value = serde_json::from_str(models_text).unwrap();
        let mq = antigravity(&mv);
        assert_eq!(mq.len(), 1);
        assert_eq!(mq[0].id, "antigravity:model:gemini-2.5-pro");
        assert_eq!(mq[0].remaining_percent, Some(72.0));
    }
}
