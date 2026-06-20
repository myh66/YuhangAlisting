#![allow(dead_code)]

use std::time::Duration;

pub async fn http_ok(url: &str, timeout: Duration) -> bool {
    reqwest::Client::new()
        .get(url)
        .timeout(timeout)
        .send()
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}
