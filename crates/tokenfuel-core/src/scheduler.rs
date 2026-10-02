use chrono::{DateTime, Duration, Utc};

/// Provider Retry-After is a floor, not a hint. Backoff is per account.
pub fn next_attempt(
    now: DateTime<Utc>,
    interval_secs: u64,
    failures: u32,
    retry_after: Option<u64>,
) -> DateTime<Utc> {
    let interval = interval_secs.clamp(30, 3600);
    let backoff = interval
        .saturating_mul(2u64.saturating_pow(failures.min(5)))
        .min(3600);
    i64::try_from(backoff.max(retry_after.unwrap_or(0)))
        .ok()
        .and_then(Duration::try_seconds)
        .and_then(|delay| now.checked_add_signed(delay))
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retry_after_is_respected_and_accounts_can_back_off_independently() {
        let now = Utc::now();
        assert_eq!((next_attempt(now, 120, 0, None) - now).num_seconds(), 120);
        assert_eq!(
            (next_attempt(now, 120, 2, Some(900)) - now).num_seconds(),
            900
        );
        assert_eq!(
            (next_attempt(now, 120, 100, None) - now).num_seconds(),
            3600
        );
    }
    #[test]
    fn hostile_retry_after_cannot_panic() {
        assert_eq!(
            next_attempt(Utc::now(), 120, 1, Some(u64::MAX)),
            DateTime::<Utc>::MAX_UTC
        );
    }
}
