use std::{env, fs, io, process};

use clap::Parser;
use log::log;

mod cache;
mod config;
mod error;
mod network;
mod scanner;

fn main() {
    let is_prod = match env::var("RS_ENV") {
        Ok(v) => v == "production",
        Err(_) => false,
    };

    let scanner_options = config::ScannerOptions::parse();

    if let Err(e) = init_logger(is_prod) {
        eprintln!("{}", e);
        process::exit(1);
    }

    match scanner::init_arp_scanner(scanner_options) {
        Err(e) => log!(log::Level::Error, "{}", e),
        _ => {
            log!(log::Level::Info, "exiting scanner...");
            process::exit(0);
        }
    }
}

pub fn init_logger(is_prod: bool) -> Result<(), fern::InitError> {
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
        .level(log::LevelFilter::Trace)
        .chain(fern::log_file("scanner.log")?);

    if !is_prod {
        dispatch = dispatch.chain(io::stdout());
    }

    dispatch.apply()?;

    Ok(())
}
