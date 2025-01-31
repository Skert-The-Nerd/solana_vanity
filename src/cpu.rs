// src/cpu.rs

use ed25519_dalek::SigningKey;
use rand::RngCore;
use serde_json;
use rayon::prelude::*;
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
    io::{self, Write},
};
use num_format::{Locale, ToFormattedString};

use crate::EXIT;

pub fn grind(target: String, case_insensitive: bool, num_threads: u32) {
    let total_count = AtomicU64::new(0);
    let start_time = Instant::now();

    (0..num_threads).into_par_iter().for_each(|_| {
        let mut rng = rand::thread_rng();
        let mut count = 0_u64;
        let mut buffer = [0u8; 32];

        loop {
            if EXIT.load(Ordering::Acquire) {
                return;
            }

            // Generate random seed
            rng.fill_bytes(&mut buffer);
            
            // Create signing key directly
            let signing_key = SigningKey::from_bytes(&buffer).unwrap();

            // Get verifying key (public key)
            let verifying_key = signing_key.verifying_key();
            let pubkey_bytes = verifying_key.to_bytes();

            // Encode public key in Base58
            let pubkey = bs58::encode(pubkey_bytes).into_string();
            let check_pubkey = if case_insensitive {
                pubkey.to_lowercase()
            } else {
                pubkey.clone()
            };

            // Update counters
            count += 1;
            total_count.fetch_add(1, Ordering::Relaxed);

            // Update progress every 1M attempts
            if count % 1_000_000 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let cps = total_count.load(Ordering::Relaxed) as f64 / elapsed;
                let remaining = 58f64.powf(target.len() as f64) / cps;
                
                print!(
                    "\rChecked: {} | Speed: {:>8.1} CPS | ETA: {:>6.1}h ",
                    total_count.load(Ordering::Relaxed).to_formatted_string(&Locale::en),
                    cps,
                    remaining / 3600.0
                );
                io::stdout().flush().unwrap();
            }

            // Check for match
            if check_pubkey.starts_with(&target) {
                EXIT.store(true, Ordering::Release);
                
                // Concatenate seed and public key to form the full keypair
                let mut full_keypair = Vec::with_capacity(64);
                full_keypair.extend_from_slice(&buffer); // First 32 bytes: Seed
                full_keypair.extend_from_slice(&pubkey_bytes); // Last 32 bytes: Public Key

                // Serialize the full keypair as a JSON array of numbers
                let keypair_json = serde_json::to_string(&full_keypair).unwrap();

                println!("\n\n╔════════════════════════════════════════════════╗");
                println!("║               MATCH FOUND!               ║");
                println!("╠══════════════════════════════════════════╣");
                println!("║ Public Address: {:25} ║", pubkey);
                println!("║ Private Key (JSON Array):\n{} ║", keypair_json);
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
