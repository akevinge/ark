use clap::Parser;

/// Seconds until mac address is considered expired
pub const MAC_ADDR_TIMEOUT_SECS: u64 = 60 * 5;
/// How often the network is scanned
pub const ARP_SCAN_PERIOD_SECS: u64 = 10;
/// How often the mac cache is logged
pub const MAC_CACHE_LOG_PERIOD_SECS: u64 = 60 * 60;

#[derive(Parser, Debug, Copy, Clone)]
#[command(author, version)]
pub struct ScannerOptions {
    #[arg(
        short = 't',
        long = "timeout",
        help = "How long MAC addresses can sit in the cache before expiring (seconds)",
        default_value_t = MAC_ADDR_TIMEOUT_SECS
    )]
    pub mac_addr_timeout_secs: u64,

    #[arg(
        short = 's',
        long = "scan_period",
        help = "How often arp requests are sent out (seconds)",
        default_value_t = ARP_SCAN_PERIOD_SECS
    )]
    pub arp_scan_period: u64,

    #[arg(
        short = 'l',
        long = "log_period",
        help = "How often cache data is logged",
        default_value_t = MAC_CACHE_LOG_PERIOD_SECS
    )]
    pub mac_cache_log_period: u64,
    #[arg(
        short = 'p',
        long = "log_packets",
        help = "Whether to log all incoming ARP packets",
        default_value_t = false
    )]
    pub log_all_arp_packets: bool,

    #[arg(
        short = 'd',
        long = "log_cache",
        help = "Whether to log cache deletions",
        default_value_t = false
    )]
    pub log_cache_deletions: bool,
}
