// Normalizes a timezone string value.
pub fn normalize_timezone(tz: &str) -> &str {
    let tz = if tz.starts_with('/') {
        // EDS can report timezones that look like "/freeassociation.sourceforge.net/Asia/Kolkata".
        // The prefix there is supposed to be the vendor prefix. So, in this case, if a timezone
        // looks like a path, we assume the second fragment to be the actual timezone value.
        tz.splitn(3, '/').last().unwrap_or(tz)
    } else {
        tz
    };

    localzone::win_zone_to_iana(tz, None).unwrap_or(tz)
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
    dt.with_timezone(&chrono::Local)
        .format("%-l:%M%P")
        .to_string()
}

// Escapes text so it can be safely rendered as Pango markup (used by Waybar).
pub fn escape_markup(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}
