use clap::Parser;
use ed25519_dalek::SigningKey;
use logfather::{Level, Logger};
use num_format::{Locale, ToFormattedString};
use rand::RngCore;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    sync::Arc,
    time::Instant,
    io::{self, Write},
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
    let total_count = Arc::new(AtomicU64::new(0));
    let matches_found = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();

    let total_count_clone = Arc::clone(&total_count);
    let matches_clone = Arc::clone(&matches_found);
    
    // Live status thread for real-time updates
    std::thread::spawn(move || {
        loop {
            if EXIT.load(Ordering::Acquire) {
                break;
            }
            let elapsed = start_time.elapsed().as_secs_f64();
            let total_checked = total_count_clone.load(Ordering::Relaxed);
            let cps = if elapsed > 0.0 { total_checked as f64 / elapsed } else { 0.0 };
            let matches = matches_clone.load(Ordering::Relaxed);
            
            print!(
                "\rChecked: {} | Speed: {:>8.1} CPS | Matches: {}   ",
                total_checked.to_formatted_string(&Locale::en),
                cps,
                matches.to_formatted_string(&Locale::en)
            );
            io::stdout().flush().unwrap();
            
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    (0..args.threads).into_par_iter().for_each(|_| {
        let mut rng = rand::thread_rng();
        let mut buffer = [0u8; 32];

        loop {
            if EXIT.load(Ordering::Acquire) {
                return;
            }

            // Generate random seed
            rng.fill_bytes(&mut buffer);
            
            // Create signing key
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
                matches_found.fetch_add(1, Ordering::Relaxed);
                
                println!("\n\n╔════════════════════════════════════════════════╗");
                println!("║               MATCH FOUND!               ║");
                println!("╠══════════════════════════════════════════╣");
                println!("║ Public Address: {:25} ║", pubkey);
                println!("║ Private Seed: {:27} ║", bs58::encode(buffer).into_string());
                println!("╚══════════════════════════════════════════╝");
                return;
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
