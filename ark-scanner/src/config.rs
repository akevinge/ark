use std::{
    fmt::Debug,
    process::{Command, ExitStatus},
    str::FromStr,
};

pub struct ScannerOptions {
    /// Time until mac address is considered expired, in seconds
    pub mac_addr_timeout: u64,
    /// Interval that ARP requests are sent, in seconds
    pub arp_scan_period: u64,
    /// Interval at which mac cache size is logged, in seconds
    pub mac_cache_log_period: u64,
    /// Whether to log 'trace' level information
    pub trace: bool,
    /// Command or script to force network reconnect
    pub reconnect_cmd: SharedCommand,
    /// URL to send log request to
    /// Optional in .env file
    pub log_api_url: Option<String>,
    /// Scanner location
    /// Optional in .env file, defaults to 'dev-location'
    pub location: String,
}

fn load_env_var<T>(key: &str) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    dotenvy::var(key)
        .unwrap_or_else(|_| panic!("unable to load {}", key))
        .parse()
        .unwrap_or_else(|_| panic!("unable to parse {}", key))
}

fn load_env_var_optional<T>(key: &str) -> Option<T>
where
    T: FromStr,
{
    match dotenvy::var(key) {
        Ok(v) => v.parse().ok(),
        Err(_) => None,
    }
}

// Reuseable wrapper around Command
pub struct SharedCommand {
    cmd: String,
    args: Vec<String>,
}

impl SharedCommand {
    pub fn new(cmd: String) -> Self {
        let ar: Vec<&str> = cmd.split(' ').collect();

        if ar.is_empty() {
            panic!("invalid command: {}", cmd);
        }

        let ar_str: Vec<String> = ar.into_iter().map(String::from).collect();

        Self {
            cmd: String::from(&ar_str[0]),
            args: ar_str[1..ar_str.len()].to_vec(),
        }
    }

    pub fn run(&self) -> Result<ExitStatus, std::io::Error> {
        Command::new(&self.cmd).args(&self.args).status()
    }
}

pub fn load_scanner_opts() -> ScannerOptions {
    ScannerOptions {
        mac_addr_timeout: load_env_var("MAC_ADDR_TIMEOUT_SECS"),
        arp_scan_period: load_env_var("ARP_SCAN_PERIOD_SECS"),
        mac_cache_log_period: load_env_var("MAC_CACHE_LOG_PERIOD_SECS"),
        trace: load_env_var("TRACE"),
        reconnect_cmd: SharedCommand::new(load_env_var("RECONNECT_CMD")),
        log_api_url: load_env_var_optional("LOG_API_URL"),
        location: load_env_var_optional("SCANNER_LOCATION")
            .unwrap_or_else(|| String::from("dev-location")),
    }
}
