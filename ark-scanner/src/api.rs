use serde::Serialize;

#[derive(Serialize)]
struct LogBody {
    location: String,
    device_count: u64,
}

pub fn log_cache(
    url: &str,
    location: String,
    device_count: u64,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    match reqwest::blocking::Client::new()
        .post(url)
        .json(&LogBody {
            location,
            device_count,
        })
        .send()
    {
        Ok(res) => res.error_for_status(),
        Err(e) => Err(e),
    }
}
