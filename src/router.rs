use regex::Regex;
use reqwest::Response;
use reqwest;
use std::error::Error;
use std::fmt;

/// Pattern for capturing the mac addresses from the page.
const PATTERN: &str = r#"<tr><td align="center" class="tabdata">\d+</td><td align="center" class="tabdata">([0-9A-Z]{2}:[0-9A-Z]{2}:[0-9A-Z]{2}:[0-9A-Z]{2}:[0-9A-Z]{2}:[0-9A-Z]{2})</td></tr>"#;

/// Max number of retry if we have unsuccessful status code from the
/// web request.
const MAX_RETRY: u8 = 3;

/// Error struct that contains the Response object.
#[derive(Debug)]
struct NotSuccess(Response);

/// Configuration struct for request
#[derive(Debug)]
pub struct Config {
    pub host: String,
    pub username: String,
    pub password: String,
}

impl fmt::Display for NotSuccess {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}\n{}", self.0.status(), self.0.headers())
    }
}

impl Error for NotSuccess {
    fn description(&self) -> &str { "Unsuccessful request" }
}

/// Single request to get the page.
fn get(config: &Config) -> reqwest::Result<Response> {
    let client = reqwest::Client::new();
    let url = format!("http://{}/status/status_deviceinfo.htm", config.host);
    client.get(&url)
        .basic_auth(&config.username[..], Some(&config.password[..]))
        .send()
}

/// Retry the request when we have unsuccessful http reply
fn retry_get(config: &Config) -> Result<Response, Box<Error>> {
    let mut resp = get(config)?;
    for _ in 0..MAX_RETRY {
        if resp.status().is_success() {
            return Ok(resp);
        }
        resp = get(config)?;
    }
    Err(Box::new(NotSuccess(resp)))
}

/// Run the request and return the mac addresses
pub fn run(config: &Config) -> Result<Vec<String>, Box<Error>> {
    let mut resp = retry_get(config)?;
    let body = resp.text()?;
    lazy_static! {
        static ref RE: Regex = Regex::new(PATTERN).unwrap();
    }
    let mac_addresses = RE.captures_iter(&body)
        .map(|cap| cap[1].to_string())
        .collect();
    Ok(mac_addresses)
}
