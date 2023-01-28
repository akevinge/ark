use std::time::{SystemTime, UNIX_EPOCH};

use log::log;
use reqwest::{
    blocking::{Client, Response},
    Result,
};
use serde::Serialize;

// Generic wrapper for local and API logger
pub struct CacheLogger<'a> {
    inner: Box<dyn Logger + 'a>,
}

impl<'a> Logger for CacheLogger<'a> {
    fn log(&mut self, location: String, device_count: u64) {
        self.inner.as_mut().log(location, device_count)
    }
}

impl<'a> CacheLogger<'a> {
    pub fn new(
        url: Option<String>,
        api_key: Option<String>,
        max_retries: Option<u64>,
        failure_cb: impl Fn() + 'a,
    ) -> Self {
        if let (Some(u), Some(m)) = (url, max_retries) {
            Self {
                inner: Box::new(APILogger::new(u, api_key, m, Box::new(failure_cb))),
            }
        } else {
            Self {
                inner: Box::new(LocalLogger {}),
            }
        }
    }
}

pub trait Logger {
    fn log(&mut self, _location: String, _device_count: u64) {}
}

// Logger for APIs
// Takes failure callback
struct APILogger<'a> {
    max_retries: u64,
    url: String,
    retries_exceeded_cb: Box<dyn Fn() + 'a>,
    http_client: Client,
    api_key: Option<String>,
}

impl<'a> APILogger<'a> {
    pub fn new(
        url: String,
        api_key: Option<String>,
        max_retries: u64,
        retries_exceeded_cb: Box<dyn Fn() + 'a>,
    ) -> Self {
        Self {
            url,
            max_retries,
            retries_exceeded_cb,
            http_client: Client::new(),
            api_key,
        }
    }
}

#[derive(Serialize)]
struct LogBody {
    location: String,
    device_count: u64,
    created_at: u64,
}

impl<'a> Logger for APILogger<'a> {
    fn log(&mut self, location: String, device_count: u64) {
        let mut failure_count = 0;

        let epoch_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(t) => t.as_secs(),
            Err(_) => panic!("System time is before UNIX_EPOCH"),
        };

        while failure_count < self.max_retries {
            let response = send_request(
                &self.http_client,
                &self.url,
                &self.api_key,
                &LogBody {
                    location: location.clone(),
                    device_count,
                    created_at: epoch_time,
                },
            );

            match response {
                Ok(_) => {
                    log!(
                        log::Level::Info,
                        "successfully logged cache size to api: {}",
                        device_count
                    );

                    return;
                }
                Err(e) => {
                    failure_count += 1;
                    log!(
                        log::Level::Error,
                        "failed attempted to log to: {}, error: {}",
                        &self.url,
                        e
                    );
                    self.retries_exceeded_cb.as_ref()();
                }
            }
        }
    }
}

fn send_request<T>(
    client: &Client,
    url: &String,
    api_key: &Option<String>,
    body: &T,
) -> Result<Response>
where
    T: Serialize,
{
    let mut request = client.post(url).json(body);

    if let Some(api_key) = api_key {
        request = request.header("x-api-key", api_key);
    }

    match request.send() {
        Ok(r) => r.error_for_status(),
        e => e,
    }
}

// Logger if API url is absent
struct LocalLogger {}

impl Logger for LocalLogger {
    fn log(&mut self, _: String, device_count: u64) {
        log!(log::Level::Info, "mac cache size: {}", device_count)
    }
}
