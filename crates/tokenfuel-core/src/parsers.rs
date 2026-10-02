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
}
