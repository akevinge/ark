//   Info on ARP requests:
// - http://www.cs.newpaltz.edu/~easwaran/CCN/Week13/ARP.pdf
// - https://www.sciencedirect.com/topics/computer-science/address-resolution-protocol-request#:~:text=ARP%20Packets,same%20way%20as%20IP%20packets

use std::{
    net::Ipv4Addr,
    process::Command,
    sync::{Arc, RwLock},
};

use log::log;
use pnet::{
    packet::{
        arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket},
        ethernet::{EtherTypes, MutableEthernetPacket},
        Packet,
    },
    util::MacAddr,
};
use pnet_datalink::NetworkInterface;

const ARP_PACKET_SIZE: usize = 28;
const ETHERNET_HW_ADDR_LEN: u8 = 6;
const IPV4_ADDR_LEN: u8 = 4;
const ETHERNET_FRAME_SIZE: usize = 42; // ARP_PACKET_SIZE + 14 for the ethernet header

// Generates ARP message wrapped in an Ethernet frame
pub fn gen_arp_request(
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) -> Option<[u8; ETHERNET_FRAME_SIZE]> {
    let target_mac = MacAddr::broadcast();

    let mut arp_buf = [0u8; ARP_PACKET_SIZE];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buf).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);

    arp_packet.set_hw_addr_len(ETHERNET_HW_ADDR_LEN);
    arp_packet.set_proto_addr_len(IPV4_ADDR_LEN);

    arp_packet.set_operation(ArpOperations::Request);

    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_sender_proto_addr(source_ip);

    arp_packet.set_target_hw_addr(target_mac);
    arp_packet.set_target_proto_addr(target_ip);

    let mut eth_buf = [0u8; ETHERNET_FRAME_SIZE];
    let mut eth_packet = MutableEthernetPacket::new(&mut eth_buf).unwrap();

    eth_packet.set_destination(target_mac);
    eth_packet.set_source(source_mac);
    eth_packet.set_ethertype(EtherTypes::Arp);
    eth_packet.set_payload(arp_packet.packet());

    eth_packet.packet().try_into().ok()
}

pub fn select_default_interface(interfaces: &[NetworkInterface]) -> Option<NetworkInterface> {
    interfaces
        .iter()
        .find(|interface| {
            interface.is_up()
                && interface.is_broadcast()
                && !interface.is_loopback()
                && !interface.ips.is_empty()
                && interface.ips.iter().any(|ip| ip.is_ipv4())
                && interface.mac.is_some()
        })
        .cloned()
}

pub fn is_interface_connected(interface: &NetworkInterface) -> bool {
    let interfaces = pnet_datalink::interfaces();

    match interfaces.iter().find(|iface| iface.name == interface.name) {
        Some(iface) => iface.is_running(),
        None => false,
    }
}

// Returns all IPs that fall within the same subnet as sample_ip
// https://github.com/google/gopacket/blob/3aa782ce48d4a525acaebab344cedabfb561f870/examples/arpscan/arpscan.go
pub fn compute_subnet_ips(sample_up: Ipv4Addr, subnet_mask: Ipv4Addr) -> Vec<Ipv4Addr> {
    let network = sample_up.octets();
    let raw_network = ((network[0] as u32) << 24)
        + ((network[1] as u32) << 16)
        + ((network[2] as u32) << 8)
        + (network[3] as u32);

    let raw_mask = subnet_mask.octets();

    // convert to 32 bit word
    let raw_mask: u32 = ((raw_mask[0] as u32) << 24)
        + ((raw_mask[1] as u32) << 16)
        + ((raw_mask[2] as u32) << 8)
        + (raw_mask[3] as u32);

    let first = raw_network & raw_mask;
    let last = raw_network | !raw_mask;

    let mut out: Vec<Ipv4Addr> = vec![];

    // skip broadcast network
    for raw in (first + 1)..last {
        out.push(Ipv4Addr::new(
            (raw >> 24) as u8,
            (raw >> 16) as u8,
            (raw >> 8) as u8,
            raw as u8,
        ))
    }

    out
}

// Disallows parallel running of command
pub struct NetworkCommandLimiter {
    cmd: NetworkCommand,
    is_running: Arc<RwLock<bool>>,
}

impl NetworkCommandLimiter {
    pub fn new(cmd: &String) -> Self {
        Self {
            cmd: NetworkCommand::new(cmd),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn run(&self) {
        let running_rd = self.is_running.read().unwrap();

        if !*running_rd {
            drop(running_rd); // drop so write lock can be obtained

            let mut running_w = self.is_running.write().unwrap();
            *running_w = true;
            drop(running_w); // drop so other threads can access read lock

            self.cmd.run();

            let mut running_w = self.is_running.write().unwrap();
            *running_w = false;
        }
    }
}

// A reusable command runner
// Generates new command on each run
pub struct NetworkCommand {
    cmd: String,
    args: Vec<String>,
}

impl NetworkCommand {
    pub fn new(cmd: &String) -> Self {
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

    pub fn run(&self) {
        match Command::new(&self.cmd).args(&self.args).status() {
            Ok(s) => log!(log::Level::Info, "Reconnect status: {}", s.to_string()),
            Err(e) => log!(log::Level::Error, "{}", e.to_string()),
        }
    }
}
