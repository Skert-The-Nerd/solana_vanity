use clap::Parser;
use ed25519_dalek::Keypair;
use indicatif::{HumanDuration, ProgressBar, ProgressState, ProgressStyle};
use logfather::{Level, Logger};
use num_format::{Locale, ToFormattedString};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rust_gpu_tools::{Device, Framework, Program};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
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

    /// Number of GPU devices to use
    #[arg(short, long, default_value_t = 4)]
    gpus: u32,

    /// Batch size per GPU (millions)
    #[arg(short, long, default_value_t = 10)]
    batch_size: u32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    validate_target(&args.target)?;
    
    let mut logger = Logger::new();
    logger.log_format("[{timestamp} {level}] {message}");
    logger.timestamp_format("%Y-%m-%d %H:%M:%S");
    logger.level(Level::Info);

    println!("╔{}╗", "═".repeat(46));
    println!("║{:^46}║", "Solana Vanity Address Finder");
    println!("╠{}╣", "═".repeat(46));
    println!("║ Target: {:36} ║", args.target);
    println!("║ GPUs: {:38} ║", args.gpus);
    println!("║ Batch Size: {:32}M ║", args.batch_size);
    println!("║ Case-sensitive: {:29} ║", !args.case_insensitive);
    println!("╚{}╝", "═".repeat(46));

    grind(args)
}

fn grind(args: Args) -> anyhow::Result<()> {
    let total_count = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();
    let found = Arc::new(AtomicBool::new(false));

    // Initialize CUDA
    let devices = Device::all(Framework::Cuda)?;
    let program = Program::from_bytes(include_bytes!("./kernel.cubin"), Framework::Cuda)?;

    // Progress bar setup
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} [{elapsed}] {msg}")
        .unwrap()
        .tick_strings(&["◜", "◠", "◝", "◞", "◡", "◟"]));
    pb.enable_steady_tick(Duration::from_millis(100));

    // Main processing loop
    while !found.load(Ordering::Relaxed) {
        let mut handles = vec![];
        
        for device in devices.iter().take(args.gpus as usize) {
            let args_clone = args.clone();
            let total_count = Arc::clone(&total_count);
            let found = Arc::clone(&found);
            let pb = pb.clone();
            
            let handle = std::thread::spawn(move || -> anyhow::Result<()> {
                let batch_size = (args_clone.batch_size * 1_000_000) as u64;
                let mut seeds = vec![0u8; (batch_size * 32) as usize];
                let mut results = vec![0u8; batch_size as usize];

                // Execute CUDA kernel
                device.execute(
                    &program,
                    &[
                        args_clone.target.as_bytes(),
                        results.as_slice(),
                        &(args_clone.target.len() as u32).to_ne_bytes(),
                        &batch_size.to_ne_bytes(),
                        seeds.as_slice(),
                    ],
                    batch_size as usize,
                )?;

                // Process results
                for i in 0..batch_size {
                    if found.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    if results[i as usize] == 1 {
                        let seed = &seeds[(i*32) as usize..((i+1)*32) as usize];
                        let keypair = Keypair::from_bytes(seed)?;
                        let pubkey = bs58::encode(keypair.public.to_bytes()).into_string();
                        
                        if check_match(&pubkey, &args_clone.target, args_clone.case_insensitive) {
                            found.store(true, Ordering::Relaxed);
                            print_match(&pubkey, seed);
                        }
                    }
                    
                    total_count.fetch_add(1, Ordering::Relaxed);
                    pb.set_message(format!(
                        "Checked: {} | Speed: {}/sec", 
                        total_count.load(Ordering::Relaxed).to_formatted_string(&Locale::en),
                        (total_count.load(Ordering::Relaxed) as f64 / start_time.elapsed().as_secs_f64()).to_formatted_string(&Locale::en)
                    ));
                }
                
                Ok(())
            });
            
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()?;
        }
    }

    pb.finish_with_message("Done!");
    println!(
        "\nFinished. Total checked: {} in {:.2} seconds",
        total_count.load(Ordering::Relaxed).to_formatted_string(&Locale::en),
        start_time.elapsed().as_secs_f64()
    );
    
    Ok(())
}

fn check_match(pubkey: &str, target: &str, case_insensitive: bool) -> bool {
    if case_insensitive {
        pubkey.to_lowercase().starts_with(&target.to_lowercase())
    } else {
        pubkey.starts_with(target)
    }
}

fn print_match(pubkey: &str, seed: &[u8]) {
    let seed_str = bs58::encode(seed).into_string();
    let max_len = pubkey.len().max(seed_str.len()) + 4;
    
    println!("\n\n╔{}╗", "═".repeat(max_len + 2));
    println!("║{:^width$}║", "MATCH FOUND!", width = max_len);
    println!("╠{}╣", "═".repeat(max_len + 2));
    println!("║ Public Address: {:<width$}║", pubkey, width = max_len - 18);
    println!("║ Private Seed: {:<width$}║", seed_str, width = max_len - 16);
    println!("╚{}╝", "═".repeat(max_len + 2));
}

fn validate_target(target: &str) -> anyhow::Result<()> {
    const BASE58_CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    
    for c in target.chars() {
        if !BASE58_CHARS.contains(c) {
            anyhow::bail!("Invalid character '{}' in target. Only Base58 characters allowed.", c);
        }
    }
    
    if target.len() < 3 {
        anyhow::bail!("Target must be at least 3 characters long");
    }
    
    Ok(())
}
