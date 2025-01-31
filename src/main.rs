use clap::Parser;
use ed25519_dalek::Keypair;
use indicatif::{HumanDuration, ProgressState, ProgressStyle};
use logfather::{Level, Logger};
use num_format::{Locale, ToFormattedString};
use rand::RngCore;
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

fn main() {
    let args = Args::parse();
    validate_target(&args.target);
    
    let mut logger = Logger::new();
    logger.log_format("[{timestamp} {level}] {message}");
    logger.timestamp_format("%Y-%m-%d %H:%M:%S");
    logger.level(Level::Info);

    println!("╔════════════════════════════════════════════════╗");
    println!("║          Solana Vanity Address Finder          ║");
    println!("╠════════════════════════════════════════════════╣");
    println!("║ Target: {:38} ║", args.target);
    println!("║ GPUs: {:40} ║", args.gpus);
    println!("║ Batch Size: {:34}M ║", args.batch_size);
    println!("║ Case-sensitive: {:31} ║", !args.case_insensitive);
    println!("╚════════════════════════════════════════════════╝\n");

    grind(args);
}

fn grind(args: Args) {
    let total_count = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();
    let found = Arc::new(AtomicBool::new(false));

    // Initialize GPU context
    let devices = Device::all(Framework::Cuda).expect("Failed to get CUDA devices");
    let program = Program::from_bytes(include_bytes!("./kernel.cubin"), Framework::Cuda)
        .expect("Failed to load CUDA kernel");

    // Setup progress bar
    let style = ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos} keys ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| {
            write!(w, "ETA: {}", HumanDuration(state.eta())).unwrap()
        })
        .progress_chars("#>-");

    let progress = indicatif::ProgressBar::new_spinner();
    progress.set_style(style);
    progress.enable_steady_tick(Duration::from_millis(100));

    // GPU processing loop
    while !found.load(Ordering::Relaxed) {
        let mut handles = vec![];
        
        for device in &devices[..args.gpus as usize] {
            let args_clone = args.clone();
            let total_count = Arc::clone(&total_count);
            let found = Arc::clone(&found);
            
            let handle = std::thread::spawn(move || {
                let mut rng = rand::thread_rng();
                let mut buffer = vec![0u8; (args_clone.batch_size * 1_000_000) as usize * 32];
                rng.fill_bytes(&mut buffer);

                let result = device
                    .execute(
                        &program,
                        &[buffer.as_slice(), args_clone.target.as_bytes()],
                        (args_clone.batch_size * 1_000_000) as usize,
                    )
                    .expect("GPU execution failed");

                for i in 0..result.len() {
                    if found.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    let keypair = Keypair::from_bytes(&buffer[i*32..(i+1)*32]).unwrap();
                    let pubkey = bs58::encode(keypair.public.to_bytes()).into_string();
                    
                    total_count.fetch_add(1, Ordering::Relaxed);
                    progress.set_position(total_count.load(Ordering::Relaxed) as u64);

                    if check_match(&pubkey, &args_clone.target, args_clone.case_insensitive) {
                        found.store(true, Ordering::Relaxed);
                        print_match(&pubkey, &buffer[i*32..(i+1)*32]);
                        break;
                    }
                }
            });
            
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    progress.finish_and_clear();
    println!(
        "\nFinished. Total checked: {} in {:.2} seconds",
        total_count.load(Ordering::Relaxed).to_formatted_string(&Locale::en),
        start_time.elapsed().as_secs_f64()
    );
}

fn check_match(pubkey: &str, target: &str, case_insensitive: bool) -> bool {
    if case_insensitive {
        pubkey.to_lowercase().starts_with(&target.to_lowercase())
    } else {
        pubkey.starts_with(target)
    }
}

fn print_match(pubkey: &str, seed: &[u8]) {
    println!("\n\n╔════════════════════════════════════════════════╗");
    println!("║               MATCH FOUND!               ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║ Public Address: {:25} ║", pubkey);
    println!("║ Private Seed: {:27} ║", bs58::encode(seed).into_string());
    println!("╚══════════════════════════════════════════╝");
}

fn validate_target(target: &str) {
    let valid_chars = bs58::alphabet::BITCOIN.chars().collect::<Vec<_>>();
    
    for c in target.chars() {
        if !valid_chars.contains(&c) {
            panic!("Invalid character '{}' in target. Only Base58 characters allowed.", c);
        }
    }
    
    if target.len() < 3 {
        panic!("Target must be at least 3 characters long");
    }
}
