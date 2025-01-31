use clap::Parser;
use ed25519_dalek::SigningKey;
use logfather::{Level, Logger};
use num_format::{Locale, ToFormattedString};
use rand::RngCore;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

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

    grind(args);
}

fn grind(args: Args) {
    let total_count = AtomicU64::new(0);
    let start_time = Instant::now();

    (0..args.threads).into_par_iter().for_each(|_| {
        let mut rng = rand::thread_rng();
        let mut buffer = [0u8; 32];

        loop {
            if EXIT.load(Ordering::Acquire) {
                return;
            }

            // Generate random seed
            rng.fill_bytes(&mut buffer);
            let signing_key = SigningKey::from_bytes(&buffer);

            // Get base58 public key
            let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
            let check_pubkey = if args.case_insensitive {
                pubkey.to_lowercase()
            } else {
                pubkey.clone()
            };

            // Update counters
            total_count.fetch_add(1, Ordering::Relaxed);

            // Check for match
            if check_pubkey.starts_with(&args.target) {
                EXIT.store(true, Ordering::Release);
                
                println!("\n\n╔════════════════════════════════════════════════╗");
                println!("║               MATCH FOUND!               ║");
                println!("╠══════════════════════════════════════════╣");
                println!("║ Public Address: {:25} ║", pubkey);
                println!("║ Private Seed: {:27} ║", bs58::encode(buffer).into_string());
                println!("╚══════════════════════════════════════════╝");
                return;
            }

            // Print progress every 1 million checks
            if total_count.load(Ordering::Relaxed) % 1_000_000 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let cps = total_count.load(Ordering::Relaxed) as f64 / elapsed;
                println!("Checked: {} | Speed: {:>8.1} CPS", 
                         total_count.load(Ordering::Relaxed).to_formatted_string(&Locale::en),
                         cps);
            }
        }
    });

    println!(
        "\nFinished. Total checked: {} in {:.2} seconds",
        total_count.load(Ordering::Relaxed).to_formatted_string(&Locale::en),
        start_time.elapsed().as_secs_f64()
    );
}

fn validate_target(target: &str) {
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
