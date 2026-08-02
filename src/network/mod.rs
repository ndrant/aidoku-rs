use aidoku::alloc::String;
use aidoku::imports::html::Document;
use aidoku::imports::net::{HttpMethod, Request, Response};
use aidoku::prelude::*;
use aidoku::serde::de::DeserializeOwned;
use aidoku::{AidokuError, Result};

use crate::error;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                             (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

const TIMEOUT_SECONDS: f64 = 20.0;
const MAX_RETRIES: u32 = 2;

/// Sends a GET request and returns the raw response.
pub fn get(url: &str) -> Result<Response> {
    send(url, HttpMethod::Get)
}

/// Sends a GET request and returns the body as a string.
pub fn get_string(url: &str) -> Result<String> {
    let response = get(url)?;
    response
        .get_string()
        .map_err(|request_error| error::with_url(url, request_error))
}

/// Sends a GET request and returns the body as a parsed HTML document.
pub fn get_html(url: &str) -> Result<Document> {
    let response = get(url)?;
    response
        .get_html()
        .map_err(|request_error| error::with_url(url, request_error.into()))
}

/// Sends a GET request and deserializes the body into a JSON value.
pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T> {
    let response = get(url)?;
    response
        .get_json_owned()
        .map_err(|aidoku_error| error::with_url(url, aidoku_error))
}

/// Sends a POST request with a form body and returns the body as a string.
///
/// `body` is sent verbatim (use [crate::utils::form_encode] on field values)
/// and `extra_headers` are merged over the shared request headers.
pub fn post_form_string(url: &str, body: &str, extra_headers: &[(&str, &str)]) -> Result<String> {
    let response = send_raw(url, HttpMethod::Post, Some(body), extra_headers)?;
    response
        .get_string()
        .map_err(|request_error| error::with_url(url, request_error))
}

fn send(url: &str, method: HttpMethod) -> Result<Response> {
    send_raw(url, method, None, &[])
}

fn send_raw(
    url: &str,
    method: HttpMethod,
    body: Option<&str>,
    extra_headers: &[(&str, &str)],
) -> Result<Response> {
    for attempt in 0..=MAX_RETRIES {
        let mut request = Request::new(url, method)
            .map_err(|request_error| error::with_url(url, request_error.into()))?;
        request = request
            .header("User-Agent", USER_AGENT)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/json;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7")
            .timeout(TIMEOUT_SECONDS);
        if let Some(body) = body {
            request = request
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body.as_bytes());
        }
        for (key, value) in extra_headers {
            request = request.header(key, value);
        }
        let response = request.send();

        match response {
            Ok(response) => {
                let status = response.status_code();
                if status >= 400 {
                    return Err(AidokuError::message(format!(
                        "HTTP {status} | {url} | attempt {}",
                        attempt + 1
                    )));
                }
                return Ok(response);
            }
            Err(request_error) => {
                if attempt < MAX_RETRIES {
                    continue;
                }
                return Err(error::with_url(
                    url,
                    AidokuError::message(format!(
                        "request failed after {} attempt(s): {request_error:?}",
                        attempt + 1
                    )),
                ));
            }
        }
    }
    Err(error::with_url(
        url,
        AidokuError::message("request failed: no attempts made"),
    ))
}
