//   Info on ARP requests:
// - http://www.cs.newpaltz.edu/~easwaran/CCN/Week13/ARP.pdf
// - https://www.sciencedirect.com/topics/computer-science/address-resolution-protocol-request#:~:text=ARP%20Packets,same%20way%20as%20IP%20packets

use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{net::IpAddr, thread};

use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet_datalink::{Channel, DataLinkReceiver, DataLinkSender, MacAddr, NetworkInterface};

use crate::cache::MacCache;
use crate::error::{ArpScannerErr, InterfaceErr};
use crate::network::{compute_subnet_ips, gen_arp_request, select_default_interface};

const MAC_ADDR_TIMEOUT_SECS: u64 = 60 * 5;
/// How often the mac cache checks for overdue macs
const MAC_CACHE_CLEAN_PERIOD: Duration = Duration::from_secs(10);
/// How often the network is scanned
const ARP_SCAN_PERIOD: Duration = Duration::from_secs(5);

pub fn init_arp_scanner() -> Result<(), ArpScannerErr> {
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

    let cache_clone = Arc::clone(&mac_cache);
    handles.push(thread::spawn(move || mac_cache_periodic_clean(cache_clone)));

    // ARP packet receiever
    handles.push(thread::spawn(move || {
        receive_arp_packets(rx, Arc::clone(&mac_cache), source_mac)
    }));

    // ARP request sender
    handles.push(thread::spawn(move || {
        send_arp_packets_to_ips(tx, ips, interface, source_mac, source_ip)
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
fn mac_cache_periodic_clean(mac_cache: Arc<Mutex<MacCache>>) {
    loop {
        thread::sleep(MAC_CACHE_CLEAN_PERIOD);

        let mut macs_to_remove: Vec<MacAddr> = vec![];

        let mut cache = mac_cache.lock().unwrap();

        for (mac, instant) in cache.iter() {
            if instant.elapsed().as_secs() > MAC_ADDR_TIMEOUT_SECS {
                macs_to_remove.push(mac.clone());
            }
        }

        for mac in macs_to_remove {
            println!("deleting... {}", &mac);
            cache.delete(&mac);
        }
    }
}

fn receive_arp_packets(
    mut rx: Box<dyn DataLinkReceiver>,
    mac_cache: Arc<Mutex<MacCache>>,
    source_mac: MacAddr,
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

        let mut cache = mac_cache.lock().unwrap();

        println!("adding...: {}, {}", &packet_mac, cache.size());
        cache.add(packet_mac);
    }
}

fn send_arp_packets_to_ips(
    mut tx: Box<dyn DataLinkSender>,
    ips: Vec<Ipv4Addr>,
    interface: NetworkInterface,
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
) {
    loop {
        for ip in &ips {
            if let Some(arp_request) = gen_arp_request(source_mac, source_ip, ip.clone()) {
                tx.send_to(&arp_request, Some(interface.clone()));
            }
        }

        thread::sleep(ARP_SCAN_PERIOD);
    }
}
