//   Info on ARP requests:
// - http://www.cs.newpaltz.edu/~easwaran/CCN/Week13/ARP.pdf
// - https://www.sciencedirect.com/topics/computer-science/address-resolution-protocol-request#:~:text=ARP%20Packets,same%20way%20as%20IP%20packets

use std::net::Ipv4Addr;
use std::process::{self, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{net::IpAddr, thread};

use log::log;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet_datalink::{Channel, DataLinkReceiver, DataLinkSender, MacAddr, NetworkInterface};

use crate::cache::MacCache;
use crate::config::ScannerOptions;
use crate::error::{ArpScannerErr, InterfaceErr};
use crate::network::{
    compute_subnet_ips, gen_arp_request, is_interface_connected, select_default_interface,
};

pub fn init_arp_scanner(options: ScannerOptions) -> Result<(), ArpScannerErr> {
    let interfaces = pnet_datalink::interfaces();

    let interface = match select_default_interface(&interfaces) {
        Some(interface) => interface,
        _ => return Err(ArpScannerErr::InterfaceError(InterfaceErr::NotFound)),
    };

    let source_mac = match interface.mac {
        Some(mac) => mac,
        None => return Err(ArpScannerErr::InterfaceError(InterfaceErr::NoMac)),
    };

    let network = match interface.ips.iter().find(|ip| ip.is_ipv4()) {
        Some(net) => net,
        None => return Err(ArpScannerErr::InterfaceError(InterfaceErr::NoIpv4)),
    };

    let source_ip = match network.ip() {
        IpAddr::V4(addr) => addr,
        _ => return Err(ArpScannerErr::InterfaceError(InterfaceErr::NoIpv4)),
    };

    let subnet_mask = match network.mask() {
        IpAddr::V4(addr) => addr,
        _ => return Err(ArpScannerErr::InterfaceError(InterfaceErr::InvalidMask)),
    };

    compute_subnet_ips(source_ip, subnet_mask);

    let ips = compute_subnet_ips(source_ip, subnet_mask);

    log::log!(
        log::Level::Info,
        "Selected interface {}, ip: {}, subnet mask: {}, subnet ip range: {}-{}",
        interface.name,
        source_ip,
        subnet_mask,
        ips.first().unwrap(),
        ips.last().unwrap(),
    );

    let (tx, rx) = match pnet_datalink::channel(
        &interface,
        pnet_datalink::Config {
            ..pnet_datalink::Config::default()
        },
    ) {
        Ok(channel) => match channel {
            Channel::Ethernet(tx, rx) => (tx, rx),
            // this should never happen
            _ => return Err(ArpScannerErr::OpenChannelError(std::io::ErrorKind::Other)),
        },
        Err(e) => return Err(ArpScannerErr::OpenChannelError(e.kind())),
    };

    let mac_cache = Arc::new(Mutex::new(MacCache::new()));

    thread::scope(|s| {
        s.spawn(|| clean_mac_cache_periodic(Arc::clone(&mac_cache), &options));
        s.spawn(|| receive_arp_packets_constant(rx, Arc::clone(&mac_cache), &source_mac));
        s.spawn(|| {
            send_arp_req_to_ips_periodic(tx, ips, &interface, &source_mac, &source_ip, &options)
        });
        s.spawn(|| log_mac_cache_periodic(Arc::clone(&mac_cache), &options));
        s.spawn(|| check_interface_connectivity(&interface, &options));
    });

    Ok(())
}

fn clean_mac_cache_periodic(mac_cache: Arc<Mutex<MacCache>>, options: &ScannerOptions) {
    loop {
        thread::sleep(Duration::from_secs(5));

        let mut macs_to_remove: Vec<MacAddr> = vec![];

        let mut cache = mac_cache.lock().unwrap();

        log!(log::Level::Trace, "running cache janitor...");
        for (mac, instant) in cache.iter() {
            if instant.elapsed().as_secs() > options.mac_addr_timeout {
                macs_to_remove.push(*mac);
            }
        }

        for mac in macs_to_remove {
            log!(log::Level::Trace, "deleting mac: {}", mac);
            cache.delete(&mac);
        }
    }
}

fn log_mac_cache_periodic(mac_cache: Arc<Mutex<MacCache>>, options: &ScannerOptions) {
    loop {
        thread::sleep(Duration::from_secs(options.mac_cache_log_period));

        let cache = mac_cache.lock().unwrap();

        log!(log::Level::Info, "mac cache size: {}", cache.size());
    }
}

fn receive_arp_packets_constant(
    mut rx: Box<dyn DataLinkReceiver>,
    mac_cache: Arc<Mutex<MacCache>>,
    source_mac: &MacAddr,
) {
    loop {
        let packet = match rx.next() {
            Ok(buf) => buf,
            Err(_) => continue,
        };

        let eth_packet = match EthernetPacket::new(packet) {
            Some(packet) => packet,
            None => continue,
        };

        let packet_mac = eth_packet.get_source();

        // skip if machine pings itself
        if packet_mac == *source_mac {
            continue;
        }

        // skip if is not an ARP packet
        if eth_packet.get_ethertype().0 != EtherTypes::Arp.0 {
            continue;
        }

        log!(log::Level::Trace, "incoming arp packet mac: {}", packet_mac);

        let mut cache = mac_cache.lock().unwrap();

        cache.add(packet_mac);
    }
}

fn send_arp_req_to_ips_periodic(
    mut tx: Box<dyn DataLinkSender>,
    ips: Vec<Ipv4Addr>,
    interface: &NetworkInterface,
    source_mac: &MacAddr,
    source_ip: &Ipv4Addr,
    options: &ScannerOptions,
) {
    loop {
        for ip in &ips {
            if let Some(arp_request) = gen_arp_request(*source_mac, *source_ip, *ip) {
                tx.send_to(&arp_request, Some(interface.clone()));
            }
        }

        thread::sleep(Duration::from_secs(options.arp_scan_period));
    }
}

fn check_interface_connectivity(interface: &NetworkInterface, options: &ScannerOptions) {
    let ar: &Vec<&str> = &options.reconnect_cmd.split(" ").collect();

    loop {
        thread::sleep(Duration::from_millis(100));
        if !is_interface_connected(interface) {
            log!(log::Level::Error, "{:?} no longer connected", interface);

            let mut cmd = Command::new(ar.get(0).expect("Invalid reconnect command"));

            if ar.len() > 1 {
                cmd.args(&ar[1..ar.len()]);
            }

            match cmd.status() {
                Ok(s) => log!(log::Level::Info, "Reconnect status: {}", s.to_string()),
                Err(e) => log!(log::Level::Error, "{}", e.to_string()),
            }
        }
    }
}
