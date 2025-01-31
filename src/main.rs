use clap::Parser;
use logfather::{Level, Logger};
use num_format::{Locale, ToFormattedString};
use rayon::prelude::*;
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
    io::{self, Write},
};

#[cfg(feature = "gpu")]
mod gpu;

#[cfg(not(feature = "gpu"))]
mod cpu;

use std::env;

static EXIT: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Parser)]
#[command(name = "Solana Vanity Address Generator")]
struct Args {
    /// Target prefix to search for (case-sensitive)
    #[arg(short, long)]
    target: String,

    /// Use case-insensitive search
    #[arg(short, long, default_value_t = false)]
    case_insensitive: bool,

    /// Number of CPU threads to use
    #[arg(short, long, default_value_t = 0)]
    threads: u32,
}

fn main() {
    let args = Args::parse();
    validate_target(&args.target);

    let mut logger = Logger::new();
    logger.log_format("[{timestamp} {level}] {message}");
    logger.timestamp_format("%Y-%m-%d %H:%M:%S");
    logger.level(Level::Info);

    let num_threads = if args.threads == 0 {
        rayon::current_num_threads() as u32
    } else {
        args.threads
    };

    println!("╔════════════════════════════════════════════════╗");
    println!("║          Solana Vanity Address Finder          ║");
    println!("╠════════════════════════════════════════════════╣");
    println!("║ Target: {:38} ║", args.target);
    println!("║ Threads: {:36} ║", num_threads);
    println!("║ Case-sensitive: {:31} ║", !args.case_insensitive);
    println!("╚════════════════════════════════════════════════╝\n");

    #[cfg(feature = "gpu")]
    {
        gpu::grind(args.target, args.case_insensitive, num_threads);
    }

    #[cfg(not(feature = "gpu"))]
    {
        cpu::grind(args.target, args.case_insensitive, num_threads);
    }
}

fn validate_target(target: &str) {
    // Create a vector of valid Base58 characters
    let valid_chars: Vec<char> = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
        .chars()
        .collect();
    
    for c in target.chars() {
        if !valid_chars.contains(&c) {
            panic!("Invalid character '{}' in target. Only Base58 characters allowed.", c);
        }
    }
    
    if target.len() < 3 {
        panic!("Target must be at least 3 characters long");
    }
}
