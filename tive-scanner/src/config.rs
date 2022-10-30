use std::{fmt::Debug, str::FromStr};

pub struct ScannerOptions {
    /// Time until mac address is considered expired, in seconds
    pub mac_addr_timeout: u64,
    /// Interval that ARP requests are sent, in seconds
    pub arp_scan_period: u64,
    /// Interval at which mac cache size is logged, in seconds
    pub mac_cache_log_period: u64,
    /// Whether to log 'trace' level information
    pub trace: bool,
}

fn load_env_var<T>(key: &str) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    dotenvy::var(key)
        .expect(format!("unable to load {}", key).as_str())
        .parse()
        .expect(format!("unable to parse {}", key).as_str())
}

pub fn load_scanner_opts() -> ScannerOptions {
    ScannerOptions {
        mac_addr_timeout: load_env_var("MAC_ADDR_TIMEOUT_SECS"),
        arp_scan_period: load_env_var("ARP_SCAN_PERIOD_SECS"),
        mac_cache_log_period: load_env_var("MAC_CACHE_LOG_PERIOD_SECS"),
        trace: load_env_var("TRACE"),
    }
}
