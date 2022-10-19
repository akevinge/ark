use std::fmt::Display;

pub enum ArpScannerErr {
    OpenChannelError,
    InterfaceError(InterfaceErr),
    UnsupportedMask,
}

impl Display for ArpScannerErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ArpScannerErr::OpenChannelError => "unable to open channel for network interface",
            ArpScannerErr::UnsupportedMask => "network has unsupported subnet mask",
            ArpScannerErr::InterfaceError(interface_err) => match interface_err {
                InterfaceErr::InvalidMask => "chosen network interface is missing ipv4 subnet mask",
                InterfaceErr::NoIpv4 => "chosen network interface is missing ipv4 address",
                InterfaceErr::NoMac => "chosen network interface is missing mac address",
                InterfaceErr::NotFound => "unable to choose default network interface",
            },
        };
        write!(f, "[scanner error]: {}", message)
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
