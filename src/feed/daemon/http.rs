use chrono::Duration;
use reqwest::header::HeaderValue;

pub fn parse_retry_after(header: &HeaderValue) -> Option<Duration> {
    let retry_after = header.to_str().ok()?;

    let from_seconds = retry_after.parse::<i64>().ok().map(Duration::seconds);
    let from_date = chrono::DateTime::parse_from_rfc2822(retry_after)
        .ok()
        .map(|d| d.signed_duration_since(chrono::Utc::now()));

    from_seconds.or(from_date)
}

pub fn parse_cache_control_max_age(header: &HeaderValue) -> Option<Duration> {
    let cache_control = header.to_str().ok()?;

    // Split the header value by commas in case there are multiple directives.
    for directive in cache_control.split(',') {
        // Remove any leading/trailing whitespace.
        let directive = directive.trim();

        // Check if this directive starts with "max-age=".
        if directive.to_lowercase().starts_with("max-age=") {
            // Split on '=' and get the second part.
            if let Some(value) = directive.split('=').nth(1) {
                // Try to parse the value as a u64.
                if let Ok(age) = value.parse::<u32>() {
                    return Some(Duration::seconds(age as i64));
                }
            }
        }
    }
    // Return None if max-age is not found or parsing fails.
    None
}

pub fn parse_etag(header: &HeaderValue) -> Option<String> {
    Some(header.to_str().ok()?.to_string())
}
