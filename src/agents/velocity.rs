use chrono::{NaiveTime, Timelike};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionVerdict {
    Allowed,
    SleepWindow,
    DailyLimitExceeded,
    IntervalTooClose { remaining_mins: u32 },
}

/// Anti-Ban Velocity Governor enforcing human circadian cycles and randomized interaction intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityGovernor {
    pub max_daily: u32,
    pub min_interval_mins: u32,
    pub sleep_start_hour: u32,
    pub sleep_end_hour: u32,
}

impl Default for VelocityGovernor {
    fn default() -> Self {
        Self::new(8, 45)
    }
}

impl VelocityGovernor {
    pub fn new(max_daily: u32, min_interval_mins: u32) -> Self {
        Self {
            max_daily,
            min_interval_mins,
            sleep_start_hour: 23,
            sleep_end_hour: 7,
        }
    }

    /// Evaluates if the current local time falls into human circadian sleep window (e.g. 23:00 - 07:00)
    pub fn evaluate_time(&self, time: NaiveTime) -> InteractionVerdict {
        let hour = time.hour();
        if hour >= self.sleep_start_hour || hour < self.sleep_end_hour {
            InteractionVerdict::SleepWindow
        } else {
            InteractionVerdict::Allowed
        }
    }

    /// Checks whether the account has reached its safe daily interaction limit
    pub fn evaluate_daily_count(&self, current_count: u32) -> InteractionVerdict {
        if current_count >= self.max_daily {
            InteractionVerdict::DailyLimitExceeded
        } else {
            InteractionVerdict::Allowed
        }
    }

    /// Enforces randomized human interval between consecutive operations
    pub fn evaluate_interval(&self, last_ts: i64, current_ts: i64) -> InteractionVerdict {
        let elapsed_secs = current_ts.saturating_sub(last_ts);
        let min_secs = (self.min_interval_mins as i64) * 60;

        if elapsed_secs < min_secs {
            let remaining_secs = min_secs - elapsed_secs;
            let remaining_mins = ((remaining_secs + 59) / 60) as u32;
            InteractionVerdict::IntervalTooClose { remaining_mins }
        } else {
            InteractionVerdict::Allowed
        }
    }

    /// Holistic verdict combining Circadian clock, daily cap, and interval spacing
    pub fn can_interact_now(
        &self,
        last_ts: Option<i64>,
        current_ts: i64,
        daily_count: u32,
        local_time: NaiveTime,
    ) -> InteractionVerdict {
        let time_check = self.evaluate_time(local_time);
        if time_check != InteractionVerdict::Allowed {
            return time_check;
        }

        let count_check = self.evaluate_daily_count(daily_count);
        if count_check != InteractionVerdict::Allowed {
            return count_check;
        }

        if let Some(prev) = last_ts {
            let interval_check = self.evaluate_interval(prev, current_ts);
            if interval_check != InteractionVerdict::Allowed {
                return interval_check;
            }
        }

        InteractionVerdict::Allowed
    }
}
