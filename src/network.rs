//   Info on ARP requests:
// - http://www.cs.newpaltz.edu/~easwaran/CCN/Week13/ARP.pdf
// - https://www.sciencedirect.com/topics/computer-science/address-resolution-protocol-request#:~:text=ARP%20Packets,same%20way%20as%20IP%20packets

use std::net::Ipv4Addr;

use pnet::{
    packet::{
        arp::{
            ArpHardwareTypes::{self},
            ArpOperations, MutableArpPacket,
        },
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

/// Generates ARP message wrapped in an Ethernet frame
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
        })
        .cloned()
}

/// Returns all IPs that fall within the same IP as sample_ip
///
/// Excludes subnet and broadcast IPs
///
/// Follows: [rfc 1878](https://www.ietf.org/rfc/rfc1878.txt)
///
/// Working (verified) for only the following subnet masks: \
/// 255.255.192.0/18 \
/// 255.255.224.0/19 \
/// 255.255.240.0/20 \
/// 255.255.248.0/21 \
/// 255.255.252.0/22 \
/// 255.255.254.0/23 \
/// 255.255.255.0/24
pub fn compute_subnet_ips(sample_ip: Ipv4Addr, mask: Ipv4Addr) -> Option<Vec<Ipv4Addr>> {
    // first two octects stay constant for all ips
    let network_part = [sample_ip.octets()[0], sample_ip.octets()[1]];

    let sample_subnet = sample_ip.octets()[2];

    let raw = mask.octets();

    // convert to 32 bit word
    let raw_bits: u32 = ((raw[0] as u32) << 24)
        + ((raw[1] as u32) << 16)
        + ((raw[2] as u32) << 8)
        + (raw[3] as u32);

    // get 0-indexed number of subnets
    // ex: /19 address has 3 bits for subnet
    // so subnet_count = 0b111 (8)
    let mut subnet_count = raw_bits & 0x0000FFFF;
    let mut subnet_right_offset = 0;

    while subnet_count & 1 == 0 {
        subnet_count >>= 1;
        subnet_right_offset += 1;
    }

    for subnet in 0..(subnet_count + 1) {
        let mut subnet_ips: Vec<Ipv4Addr> = vec![];

        // subnet range

        // inclusive of first possible address
        // 0x0000ZZ00 binary
        let subnet_start = subnet << subnet_right_offset;
        // exclusive of last possible address
        // basically the 0x0000ZZ00 for the next subnet
        let mut subnet_end = (subnet + 1) << subnet_right_offset;

        if (subnet_start >> 8) == 254 {
            continue;
        }

        if (subnet_end >> 8) == 256 {
            subnet_end = 255 << 8;
        }

        let sample_falls_in_range = sample_subnet >= (subnet_start >> 8) as u8
            && sample_subnet <= ((subnet_end >> 8) - 1) as u8;

        // skip subnet range that the sample subnet doesn't fall into
        if !sample_falls_in_range {
            continue;
        }

        // subnet_start + 1 to skip subnet ip
        // (first address, all 0's in host part of address)
        // subnet_end - 1 to exclude broadcast ip
        // (last address, all 1's in host)
        for ip in (subnet_start + 1)..(subnet_end - 1) {
            subnet_ips.push(Ipv4Addr::new(
                network_part[0],
                network_part[1],
                ((ip & 0x0000FF00) >> 8) as u8,
                (ip & 0x000000FF) as u8,
            ));
        }

        return Some(subnet_ips);
    }

    None
}
