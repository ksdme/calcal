// Normalizes a timezone string value.
pub fn normalize_timezone(tz: &str) -> &str {
    if tz.starts_with('/') {
        // EDS can report timezones that look like "/freeassociation.sourceforge.net/Asia/Kolkata".
        // The prefix there is supposed to be the vendor prefix. So, in this case, if a timezone
        // looks like a path, we assume the second fragment to be the actual timezone value.
        tz.splitn(3, '/').last().unwrap_or(tz)
    } else {
        tz
    }
}

// Returns a short human formatted duration.
pub fn human_short_duration(delta: chrono::TimeDelta) -> String {
    let mut seconds = delta.num_seconds().max(0) as u64;
    seconds -= seconds % 60;

    let duration = std::time::Duration::new(seconds, 0);
    humantime::format_duration(duration).to_string()
}

// Returns a short human formatted time.
pub fn human_short_time(dt: chrono::DateTime<rrule::Tz>) -> String {
    dt.format("%-l:%M%P").to_string()
}
