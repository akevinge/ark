use std::{fmt::Display, io::ErrorKind};

pub enum ArpScannerErr {
    OpenChannelError(ErrorKind),
    InterfaceError(InterfaceErr),
    UnsupportedMask,
}

impl Display for ArpScannerErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ArpScannerErr::OpenChannelError(reason) => format!(
                "{}: {:?}",
                "unable to open channel for network interface", &reason
            ),
            ArpScannerErr::UnsupportedMask => String::from("network has unsupported subnet mask"),
            ArpScannerErr::InterfaceError(interface_err) => match interface_err {
                InterfaceErr::InvalidMask => {
                    String::from("chosen network interface is missing ipv4 subnet mask")
                }
                InterfaceErr::NoIpv4 => {
                    String::from("chosen network interface is missing ipv4 address")
                }
                InterfaceErr::NoMac => {
                    String::from("chosen network interface is missing mac address")
                }
                InterfaceErr::NotFound => {
                    String::from("unable to choose default network interface")
                }
            },
        };
        write!(f, "[arp scanner error]: {}", message)
    }
}

pub enum InterfaceErr {
    /// No default interface is found
    NotFound,
    /// No IPv4 mask is found for cooresponding interface
    InvalidMask,
    /// Interface has no mac address
    NoMac,
    /// Interface has no ip address
    NoIpv4,
}
