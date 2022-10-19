use std::{thread, time::Duration};

mod network;

fn main() {
    network::init_arp_scanner();

    thread::sleep(Duration::from_secs(60 * 60));
}
