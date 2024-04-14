use crate::drive_structure::meta::TIMESTAMP;
use crate::prelude;
use chrono::offset::Utc;
use chrono::DateTime;
use chrono::TimeZone;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn system_time_to_timestamp(time: SystemTime) -> prelude::Result<TIMESTAMP> {
    let secs = time.duration_since(UNIX_EPOCH)?.as_secs() as i64;
    let nsecs = time.duration_since(UNIX_EPOCH)?.subsec_nanos();
    Ok((secs, nsecs))
}

pub fn datetime_to_timestamp(time: DateTime<Utc>) -> prelude::Result<TIMESTAMP> {
    let unix_epoch: DateTime<Utc> = Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
    let timestamp = time.signed_duration_since(unix_epoch);
    let secs = timestamp.num_seconds();
    let nsecs = timestamp.subsec_nanos() as u32;
    Ok((secs, nsecs))
}

pub fn system_time_from_timestamp(secs: i64, nsecs: u32) -> SystemTime {
    if secs >= 0 {
        UNIX_EPOCH + Duration::new(secs as u64, nsecs)
    } else {
        UNIX_EPOCH - Duration::new((-secs) as u64, nsecs)
    }
}

pub fn time_from_system_time(system_time: &SystemTime) -> (i64, u32) {
    // Convert to signed 64-bit time with epoch at 0
    match system_time.duration_since(UNIX_EPOCH) {
        Ok(duration) => (duration.as_secs() as i64, duration.subsec_nanos()),
        Err(before_epoch_error) => (
            -(before_epoch_error.duration().as_secs() as i64),
            before_epoch_error.duration().subsec_nanos(),
        ),
    }
}
