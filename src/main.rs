mod cache;
mod error;
mod network;
mod scanner;

fn main() {
    match scanner::init_arp_scanner() {
        Err(e) => eprintln!("{}", e),
        _ => println!("exiting scanner gracefully..."),
    }
}
