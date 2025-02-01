use clap::Parser;
use ed25519_dalek::Keypair;
use indicatif::{ProgressBar, ProgressStyle};
use logfather::Logger;
use num_format::{Locale, ToFormattedString};
use rayon::iter::IntoParallelIterator; // Fixed import
use rust_gpu_tools::{Device, Framework, Program};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "Solana Vanity GPU")]
struct Args {
    #[arg(short, long)]
    target: String,
    #[arg(short, long, default_value_t = 4)]
    gpus: u32,
    #[arg(short, long, default_value_t = 1_000_000)]
    batch_size: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let devices = Device::all(Framework::Cuda)?;
    let program = Program::from_bytes(include_bytes!("./kernel.cubin"), Framework::Cuda)?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} [{elapsed}] Keys: {pos} ({per_sec})")?
    );

    let start = Instant::now();
    let total_count = Arc::new(AtomicU64::new(0));

    'search: loop {
        let mut handles = vec![];
        for device in &devices[..args.gpus as usize] {
            let device = device.clone();
            let program = program.clone();
            let target = args.target.as_bytes().to_vec();
            let batch_size = args.batch_size;
            let total_count = Arc::clone(&total_count);
            
            handles.push(std::thread::spawn(move || -> anyhow::Result<()> {
                let mut seeds = vec![0u8; (batch_size * 32) as usize];
                let mut results = vec![0u8; batch_size as usize];

                device.execute(
                    &program,
                    &[&target, &results, &(target.len() as u32).to_ne_bytes(), &batch_size.to_ne_bytes(), &seeds],
                    batch_size as usize,
                )?;

                for i in 0..batch_size {
                    if results[i as usize] == 1 {
                        let seed = &seeds[(i*32) as usize..(i+1)*32 as usize];
                        let keypair = Keypair::from_bytes(seed)?;
                        let pubkey = bs58::encode(keypair.public.to_bytes()).into_string();
                        if pubkey.starts_with(&args.target) {
                            println!("\nFound: {}", pubkey);
                            println!("Seed: {}", bs58::encode(seed));
                            break 'search;
                        }
                    }
                }
                
                total_count.fetch_add(batch_size, Ordering::Relaxed);
                pb.set_position(total_count.load(Ordering::Relaxed));
                pb.set_message(format!(
                    "{:.2}M keys/s", 
                    total_count.load(Ordering::Relaxed) as f64 / start.elapsed().as_secs_f64() / 1_000_000.0
                ));
                
                Ok(())
            }));
        }

        for handle in handles {
            handle.join().unwrap()?;
        }
    }

    pb.finish_with_message("Done!");
    Ok(())
}
