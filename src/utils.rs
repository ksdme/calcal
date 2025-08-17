// Returns a short human formatted duration.
pub fn human_short_duration(delta: chrono::TimeDelta) -> String {
    let mut seconds = delta.num_seconds().max(0) as u64;
    seconds -= seconds % 60;

    let duration = std::time::Duration::new(seconds, 0);
    humantime::format_duration(duration).to_string()
}
