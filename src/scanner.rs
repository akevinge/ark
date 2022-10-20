//   Info on ARP requests:
// - http://www.cs.newpaltz.edu/~easwaran/CCN/Week13/ARP.pdf
// - https://www.sciencedirect.com/topics/computer-science/address-resolution-protocol-request#:~:text=ARP%20Packets,same%20way%20as%20IP%20packets

use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{net::IpAddr, thread};

use log::log;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet_datalink::{Channel, DataLinkReceiver, DataLinkSender, MacAddr, NetworkInterface};

use crate::cache::MacCache;
use crate::config::ScannerOptions;
use crate::error::{ArpScannerErr, InterfaceErr};
use crate::network::{compute_subnet_ips, gen_arp_request, select_default_interface};

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

    let ips = match compute_subnet_ips(source_ip, subnet_mask) {
        Some(ips) => ips,
        None => return Err(ArpScannerErr::UnsupportedMask),
    };

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
            _ => return Err(ArpScannerErr::OpenChannelError),
        },
        Err(_) => return Err(ArpScannerErr::OpenChannelError),
    };

    let mac_cache = Arc::new(Mutex::new(MacCache::new()));

    let mut handles = Vec::with_capacity(3);

    let cache_clone_1 = Arc::clone(&mac_cache);
    // Clean ARP cache periodically
    handles.push(thread::spawn(move || {
        clean_mac_cache_periodic(cache_clone_1, &options)
    }));

    let cache_clone_2 = Arc::clone(&mac_cache);
    // Recieve incoming ARP packets
    handles.push(thread::spawn(move || {
        receive_arp_packets_constant(rx, cache_clone_2, source_mac, &options)
    }));

    // Send ARP requests periodically
    handles.push(thread::spawn(move || {
        send_arp_req_to_ips_periodic(tx, ips, interface, source_mac, source_ip, &options)
    }));

    let cache_clone_3 = Arc::clone(&mac_cache);
    handles.push(thread::spawn(move || {
        log_mac_cache_periodic(cache_clone_3, &options)
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
fn clean_mac_cache_periodic(mac_cache: Arc<Mutex<MacCache>>, options: &ScannerOptions) {
    loop {
        let mut macs_to_remove: Vec<MacAddr> = vec![];

        let mut cache = mac_cache.lock().unwrap();

        for (mac, instant) in cache.iter() {
            if instant.elapsed().as_secs() > options.mac_addr_timeout_secs {
                macs_to_remove.push(mac.clone());
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

        if options.log_cache {
            log!(log::Level::Info, "mac cache size: {}", cache.size());
        }
    }
}

fn receive_arp_packets_constant(
    mut rx: Box<dyn DataLinkReceiver>,
    mac_cache: Arc<Mutex<MacCache>>,
    source_mac: MacAddr,
    options: &ScannerOptions,
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
        if packet_mac == source_mac {
            continue;
        }

        // skip if is not an ARP packet
        if eth_packet.get_ethertype().0 != EtherTypes::Arp.0 {
            continue;
        }

        if options.log_packets {
            log!(log::Level::Trace, "incoming arp packet mac: {}", packet_mac);
        }

        let mut cache = mac_cache.lock().unwrap();

        cache.add(packet_mac);
    }
}

fn send_arp_req_to_ips_periodic(
    mut tx: Box<dyn DataLinkSender>,
    ips: Vec<Ipv4Addr>,
    interface: NetworkInterface,
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
    options: &ScannerOptions,
) {
    loop {
        for ip in &ips {
            if let Some(arp_request) = gen_arp_request(source_mac, source_ip, ip.clone()) {
                tx.send_to(&arp_request, Some(interface.clone()));
            }
        }

        thread::sleep(Duration::from_secs(options.arp_scan_period));
    }
}
