/// Get the current timestamp (time since the UNIX epoch) in milliseconds
pub fn timestamp_ms() -> u64 {
    get_timestamp().as_millis() as _
}

/// Get the current timestamp (time since the UNIX epoch) in seconds
pub fn timestamp() -> u64 {
    get_timestamp().as_secs() as _
}

fn get_timestamp() -> std::time::Duration {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}
