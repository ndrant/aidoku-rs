use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec, format};

/// Strips a prefix from a string, returning the string unchanged if it
/// does not start with the prefix.
pub fn strip_prefix_or_self<'a>(value: &'a str, prefix: &str) -> &'a str {
    value.strip_prefix(prefix).unwrap_or(value)
}

/// Trims leading and trailing whitespace.
pub fn trim(text: &str) -> String {
    text.trim().to_string()
}

/// Collapses runs of whitespace into single spaces and trims the result.
pub fn normalize_whitespace(text: &str) -> String {
    let mut result = String::new();
    let mut pending_space = false;
    for c in text.trim().chars() {
        if c.is_whitespace() {
            pending_space = true;
        } else {
            if pending_space && !result.is_empty() {
                result.push(' ');
            }
            result.push(c);
            pending_space = false;
        }
    }
    result
}

/// URL-encodes a string for use in an `application/x-www-form-urlencoded`
/// body: unreserved characters are kept, spaces become `+`, and everything
/// else is percent-encoded.
pub fn form_encode(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char)
            }
            b' ' => result.push('+'),
            _ => {
                result.push('%');
                result.push_str(&format!("{byte:02X}"));
            }
        }
    }
    result
}

/// Strips a WordPress-style `-WxH` size suffix from an image URL, leaving
/// the original full-size file. Returns the input unchanged when the suffix
/// pattern is not present.
pub fn remove_size_suffix(url: &str) -> String {
    let Some((stem, ext)) = url.rsplit_once('.') else {
        return String::from(url);
    };
    let Some((head, size)) = stem.rsplit_once('-') else {
        return String::from(url);
    };
    let digits = size.split('x').collect::<Vec<_>>();
    if digits.len() == 2
        && !digits[0].is_empty()
        && !digits[1].is_empty()
        && digits[0].bytes().all(|b| b.is_ascii_digit())
        && digits[1].bytes().all(|b| b.is_ascii_digit())
    {
        format!("{head}.{ext}")
    } else {
        String::from(url)
    }
}

/// Parses an ISO-8601 timestamp into Unix time in seconds.
///
/// Accepts `YYYY-MM-DDTHH:MM:SS[.fff]` with a trailing `Z` or `±HH:MM`
/// offset. Returns `None` when the text cannot be parsed.
pub fn iso8601_seconds(text: &str) -> Option<i64> {
    let text = text.trim();
    let (year, rest) = take_digits(text, 4)?;
    let rest = rest.strip_prefix('-')?;
    let (month, rest) = take_digits(rest, 2)?;
    let rest = rest.strip_prefix('-')?;
    let (day, rest) = take_digits(rest, 2)?;
    let rest = rest.strip_prefix('T')?;
    let (hour, rest) = take_digits(rest, 2)?;
    let rest = rest.strip_prefix(':')?;
    let (minute, rest) = take_digits(rest, 2)?;
    let rest = rest.strip_prefix(':')?;
    let (second, rest) = take_digits(rest, 2)?;

    let mut rest = rest;
    if let Some(after_dot) = rest.strip_prefix('.') {
        let fraction_len = after_dot.chars().take_while(|c| c.is_ascii_digit()).count();
        rest = &after_dot[fraction_len..];
    }

    let offset = if rest.starts_with('Z') {
        0
    } else if rest.starts_with('+') || rest.starts_with('-') {
        let sign = if rest.starts_with('-') { -1 } else { 1 };
        let (offset_hour, rest) = take_digits(&rest[1..], 2)?;
        let rest = rest.strip_prefix(':')?;
        let (offset_minute, _rest) = take_digits(rest, 2)?;
        sign * (offset_hour * 3600 + offset_minute * 60) as i64
    } else {
        0
    };

    let days = days_from_civil(year as i64, month, day);
    let seconds = days * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64;
    Some(seconds - offset)
}

/// Reads `count` ASCII digits from the start of `text`.
fn take_digits(text: &str, count: usize) -> Option<(u32, &str)> {
    let bytes = text.as_bytes();
    if bytes.len() < count {
        return None;
    }
    let mut value: u32 = 0;
    for &byte in &bytes[..count] {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value * 10 + u32::from(byte - b'0');
    }
    Some((value, &text[count..]))
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Hinnant's algorithm).
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * if month > 2 { month - 3 } else { month + 9 } + 2) / 5 + day - 1;
    let day_of_era =
        year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + i64::from(day_of_year);
    era * 146097 + day_of_era - 719468
}

/// Parses an Indonesian relative time phrase ("3 jam lalu") into a duration
/// in seconds. Returns `None` when the text cannot be parsed.
pub fn relative_time_seconds(text: &str) -> Option<i64> {
    let text = text.trim().to_lowercase();
    for (suffix, unit_seconds) in [
        ("detik", 1),
        ("menit", 60),
        ("jam", 3600),
        ("hari", 86400),
        ("minggu", 604800),
        ("bulan", 2629800),
        ("tahun", 31557600),
    ] {
        if text.contains(suffix) {
            let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
            let value: i64 = digits.parse().ok()?;
            return Some(value * unit_seconds);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn strip_prefix_present() {
        assert_eq!(
            strip_prefix_or_self("https://natsu.one/x", "https://"),
            "natsu.one/x"
        );
    }

    #[aidoku_test]
    fn strip_prefix_absent() {
        assert_eq!(
            strip_prefix_or_self("natsu.one/x", "https://"),
            "natsu.one/x"
        );
    }

    #[aidoku_test]
    fn normalize_spaces() {
        assert_eq!(
            normalize_whitespace("  a\n\tb   c  "),
            String::from("a b c")
        );
    }

    #[aidoku_test]
    fn relative_hours() {
        assert_eq!(relative_time_seconds("3 jam lalu"), Some(10800));
    }

    #[aidoku_test]
    fn relative_days() {
        assert_eq!(relative_time_seconds("2 hari yang lalu"), Some(172800));
    }

    #[aidoku_test]
    fn relative_invalid() {
        assert_eq!(relative_time_seconds("kemarin"), None);
    }

    #[aidoku_test]
    fn iso8601_zulu() {
        assert_eq!(
            iso8601_seconds("2026-08-02T03:19:22.546Z"),
            Some(1785640762)
        );
    }

    #[aidoku_test]
    fn iso8601_plus_offset() {
        assert_eq!(
            iso8601_seconds("2026-08-02T10:19:22.546+07:00"),
            Some(1785640762)
        );
    }

    #[aidoku_test]
    fn iso8601_invalid() {
        assert_eq!(iso8601_seconds("kemarin"), None);
    }

    #[aidoku_test]
    fn form_encode_spaces_and_reserved() {
        assert_eq!(form_encode("one piece & more"), "one+piece+%26+more");
    }

    #[aidoku_test]
    fn form_encode_unreserved_untouched() {
        assert_eq!(form_encode("ABC-_.~xyz0129"), "ABC-_.~xyz0129");
    }

    #[aidoku_test]
    fn remove_size_suffix_strips_dimensions() {
        assert_eq!(
            remove_size_suffix("https://natsu.one/wp-content/uploads/2025/09/a-320x427.png"),
            "https://natsu.one/wp-content/uploads/2025/09/a.png"
        );
    }

    #[aidoku_test]
    fn remove_size_suffix_keeps_plain_url() {
        assert_eq!(
            remove_size_suffix("https://cdn.natsu.id/img/o/one-piece/0/1.jpg"),
            "https://cdn.natsu.id/img/o/one-piece/0/1.jpg"
        );
    }

    #[aidoku_test]
    fn remove_size_suffix_keeps_numeric_suffix() {
        assert_eq!(
            remove_size_suffix("https://x.example/img/2025-06.png"),
            "https://x.example/img/2025-06.png"
        );
    }
}
