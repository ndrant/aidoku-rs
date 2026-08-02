use aidoku::AidokuError;
use aidoku::prelude::*;

/// Wraps an error with the URL that caused it.
pub fn with_url<S: AsRef<str>>(url: S, error: AidokuError) -> AidokuError {
    let url = url.as_ref();
    match error {
        AidokuError::Message(message) => AidokuError::message(format!("{message} | {url}")),
        other => AidokuError::message(format!("{other:?} | {url}")),
    }
}

/// Wraps an error with the site that raised it.
pub fn with_site<S: AsRef<str>>(site: S, error: AidokuError) -> AidokuError {
    let site = site.as_ref();
    match error {
        AidokuError::Message(message) => AidokuError::message(format!("[{site}] {message}")),
        other => AidokuError::message(format!("[{site}] {other:?}")),
    }
}
