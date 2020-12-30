use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Like in Java with `System.currentTimeMillis()`.
///
/// It returns the difference in milliseconds between
/// the current time and midnight, January 1, 1970.
pub fn current_millis() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() * 1000 +
        since_the_epoch.subsec_nanos() as u64 / 1_000_000
}
