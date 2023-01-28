use std::{env, fs, io};

use config::load_scanner_opts;
use log::log;

mod cache;
mod cache_logger;
mod config;
mod error;
mod network;
mod scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = match args.get(1) {
        Some(p) => p,
        None => {
            println!("did not specify .env path, defaulting to ./.scanner.dev.env");
            "./.scanner.dev.env"
        }
    };
    if let Err(e) = dotenvy::from_path(path) {
        panic!("unable to load {path}, {e}")
    }

    let scanner_options = load_scanner_opts();

    if let Err(e) = init_logger(scanner_options.trace) {
        panic!("{}", e);
    }

    match scanner::init_arp_scanner(scanner_options) {
        Err(e) => log!(log::Level::Error, "{}", e),
        _ => {
            log!(log::Level::Info, "exiting scanner...");
        }
    }
}

pub fn init_logger(trace: bool) -> Result<(), fern::InitError> {
    let _ = fs::remove_file("scanner.log");

    let mut dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file("scanner.log")?);

    if trace {
        dispatch = dispatch.level(log::LevelFilter::Trace)
    } else {
        dispatch = dispatch.level(log::LevelFilter::Debug)
    }

    dispatch = dispatch.chain(io::stdout());

    dispatch.apply()?;

    Ok(())
}
