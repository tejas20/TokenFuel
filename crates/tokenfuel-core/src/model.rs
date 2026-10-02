use chrono::{DateTime, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Claude,
    Openai,
    Gemini,
    Grok,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Documented,
    Experimental,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Disconnected,
    Available,
    Unavailable,
    LoginRequired,
    Offline,
    RateLimited,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageLimit {
    pub id: String,
    pub name: String,
    pub product: String,
    pub scope: String,
    pub unit: String,
    pub period: String,
    pub resets_at: Option<DateTime<Utc>>,
    /// Decimal strings preserve monetary precision across Rust/JavaScript.
    pub used: Option<String>,
    pub total: Option<String>,
    pub remaining: Option<String>,
    pub remaining_percent: Option<f64>,
    pub unlimited: bool,
    pub source: Source,
    pub observed_at: DateTime<Utc>,
}

impl UsageLimit {
    pub fn percentage(
        id: &str,
        name: &str,
        product: &str,
        period: &str,
        used_percent: f64,
        source: Source,
    ) -> Option<Self> {
        if !used_percent.is_finite() || used_percent < 0.0 {
            return None;
        }
        Some(Self {
            id: id.into(),
            name: name.into(),
            product: product.into(),
            scope: "account".into(),
            unit: "percent".into(),
            period: period.into(),
            resets_at: None,
            used: None,
            total: None,
            remaining: None,
            remaining_percent: Some((100.0 - used_percent).clamp(0.0, 100.0)),
            unlimited: false,
            source,
            observed_at: Utc::now(),
        })
    }

    // Named identity, units and source stay explicit at the adapter boundary.
    #[allow(clippy::too_many_arguments)]
    pub fn amounts(
        id: &str,
        name: &str,
        product: &str,
        unit: &str,
        period: &str,
        used: Decimal,
        total: Option<Decimal>,
        source: Source,
    ) -> Result<Self, &'static str> {
        if used.is_sign_negative() || total.is_some_and(|x| x.is_sign_negative()) {
            return Err("Amounts cannot be negative.");
        }
        let remaining = total.map(|t| (t - used).max(Decimal::ZERO));
        let percent = total.and_then(|t| {
            if t.is_zero() {
                Some(0.0)
            } else {
                ((remaining? / t) * Decimal::ONE_HUNDRED).to_f64()
            }
        });
        Ok(Self {
            id: id.into(),
            name: name.into(),
            product: product.into(),
            scope: "account".into(),
            unit: unit.into(),
            period: period.into(),
            resets_at: None,
            used: Some(used.normalize().to_string()),
            total: total.map(|x| x.normalize().to_string()),
            remaining: remaining.map(|x| x.normalize().to_string()),
            remaining_percent: percent,
            unlimited: false,
            source,
            observed_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountIdentity {
    pub display_name: String,
    pub plan: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub account_id: String,
    pub provider: Provider,
    #[serde(default)]
    pub identity: Option<AccountIdentity>,
    pub status: Status,
    pub message: String,
    pub limits: Vec<UsageLimit>,
    pub fetched_at: Option<DateTime<Utc>>,
    pub retry_at: Option<DateTime<Utc>>,
}

impl Snapshot {
    pub fn empty(
        account_id: &str,
        provider: Provider,
        status: Status,
        message: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            provider,
            identity: None,
            status,
            message: message.into(),
            limits: vec![],
            fetched_at: None,
            retry_at: None,
        }
    }
    pub fn ready(account_id: &str, provider: Provider, limits: Vec<UsageLimit>) -> Self {
        let status = if limits.is_empty() {
            Status::Unavailable
        } else {
            Status::Available
        };
        Self {
            account_id: account_id.into(),
            provider,
            identity: None,
            status,
            message: if limits.is_empty() {
                "This source did not expose a supported usage counter.".into()
            } else {
                String::new()
            },
            limits,
            fetched_at: Some(Utc::now()),
            retry_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }
    #[test]
    fn monthly_money_is_exact_and_overages_are_preserved() {
        let q = UsageLimit::amounts(
            "month",
            "Monthly budget",
            "Claude",
            "USD",
            "monthly",
            d("80.125"),
            Some(d("200")),
            Source::Experimental,
        )
        .unwrap();
        assert_eq!(q.remaining.as_deref(), Some("119.875"));
        let q = UsageLimit::amounts(
            "month",
            "Monthly budget",
            "Claude",
            "USD",
            "monthly",
            d("201.25"),
            Some(d("200")),
            Source::Experimental,
        )
        .unwrap();
        assert_eq!(q.used.as_deref(), Some("201.25"));
        assert_eq!(q.remaining_percent, Some(0.0));
    }
    #[test]
    fn missing_denominator_is_not_a_percentage_or_unlimited() {
        let q = UsageLimit::amounts(
            "x",
            "Budget",
            "Claude",
            "USD",
            "monthly",
            d("10"),
            None,
            Source::Manual,
        )
        .unwrap();
        assert_eq!(q.remaining_percent, None);
        assert!(!q.unlimited);
    }
    #[test]
    fn zero_limit_means_zero_remaining_and_negative_values_are_rejected() {
        let q = UsageLimit::amounts(
            "x",
            "Budget",
            "Claude",
            "USD",
            "monthly",
            d("0"),
            Some(d("0")),
            Source::Manual,
        )
        .unwrap();
        assert_eq!(q.remaining_percent, Some(0.0));
        assert!(
            UsageLimit::amounts(
                "x",
                "Budget",
                "Claude",
                "USD",
                "monthly",
                d("0"),
                Some(d("-1")),
                Source::Manual
            )
            .is_err()
        );
    }
    #[test]
    fn malformed_and_overage_percentages() {
        assert!(
            UsageLimit::percentage("x", "X", "X", "rolling", f64::NAN, Source::Manual).is_none()
        );
        assert!(UsageLimit::percentage("x", "X", "X", "rolling", -1.0, Source::Manual).is_none());
        assert_eq!(
            UsageLimit::percentage("x", "X", "X", "rolling", 112.0, Source::Manual)
                .unwrap()
                .remaining_percent,
            Some(0.0)
        );
    }
}
