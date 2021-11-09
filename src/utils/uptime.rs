use time::{self, OffsetDateTime};

use serenity::prelude::*;

pub struct Uptime {
    start_time: OffsetDateTime,
}

impl Uptime {
    pub fn new() -> Self {
        Uptime {
            start_time: OffsetDateTime::now_utc(),
        }
    }

    pub fn to_str(&self) -> String {
        // Get duration
        let now = OffsetDateTime::now_utc();

        // Calculate difference
        let duration = (now - self.start_time).whole_seconds();

        // Divide and get the parts
        let days = duration / (60 * 60 * 24);
        let hours = (duration % (60 * 60 * 24)) / (60 * 60);
        let minutes = (duration % (60 * 60)) / 60;
        let seconds = duration % 60;

        format!(
            "{} days, {} hours, {} minutes, {} seconds",
            days, hours, minutes, seconds
        )
    }
}

// Type Map Key
impl TypeMapKey for Uptime {
    type Value = Uptime;
}
