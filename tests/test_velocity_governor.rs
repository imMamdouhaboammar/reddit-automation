use chrono::NaiveTime;
use reddit_agent::agents::velocity::{InteractionVerdict, VelocityGovernor};

#[test]
fn test_velocity_governor_enforces_sleep_and_daily_cap() {
    let gov = VelocityGovernor::new(8 /* max_daily */, 45 /* min_interval_mins */);

    // 1. Simulate 03:00 AM local time (Sleep mode between 23:00 and 07:00)
    let late_night = NaiveTime::from_hms_opt(3, 0, 0).unwrap();
    assert_eq!(gov.evaluate_time(late_night), InteractionVerdict::SleepWindow);

    // 2. Simulate 14:00 PM local time (Active hours)
    let daytime = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
    assert_eq!(gov.evaluate_time(daytime), InteractionVerdict::Allowed);

    // 3. Test daily cap
    assert_eq!(
        gov.evaluate_daily_count(8),
        InteractionVerdict::DailyLimitExceeded
    );
    assert_eq!(gov.evaluate_daily_count(4), InteractionVerdict::Allowed);
}

#[test]
fn test_velocity_governor_enforces_randomized_interval() {
    let gov = VelocityGovernor::new(8, 45);

    let current_ts = 1700000000;
    // 20 minutes ago (45 min interval not satisfied)
    let too_recent_ts = current_ts - (20 * 60);

    match gov.evaluate_interval(too_recent_ts, current_ts) {
        InteractionVerdict::IntervalTooClose { remaining_mins } => {
            assert!(remaining_mins > 0);
        }
        _ => panic!("Expected IntervalTooClose"),
    }

    // 50 minutes ago (Satisfied)
    let valid_ts = current_ts - (50 * 60);
    assert_eq!(
        gov.evaluate_interval(valid_ts, current_ts),
        InteractionVerdict::Allowed
    );
}
