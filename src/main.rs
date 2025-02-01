use std::sync::{Arc, Mutex};
use std::time::Instant;
use rayon::prelude::*;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use rand::Rng;

const TARGET_PREFIX: &str = "Van1"; // Change this to your desired prefix

fn generate_random_keypair() -> (String, Pubkey) {
    let keypair = Keypair::new();
    let pubkey_str = keypair.pubkey().to_string();
    (pubkey_str, keypair.pubkey())
}

fn main() {
    let start_time = Instant::now();
    let found = Arc::new(Mutex::new(false));

    println!("Starting Solana Vanity Address Finder...");
    println!("Looking for addresses starting with: {}", TARGET_PREFIX);

    let result = (0..num_cpus::get()) // Use all available CPU cores
        .into_par_iter() // Use Rayon for parallel execution
        .map(|_| {
            loop {
                let (pubkey_str, pubkey) = generate_random_keypair();

                if pubkey_str.starts_with(TARGET_PREFIX) {
                    let mut found_lock = found.lock().unwrap();
                    if !*found_lock {
                        *found_lock = true;
                        return Some((pubkey_str, pubkey));
                    }
                }

                if *found.lock().unwrap() {
                    break;
                }
            }
            None
        })
        .find_any(|res| res.is_some());

    if let Some(Some((pubkey_str, _))) = result {
        let elapsed_time = start_time.elapsed().as_secs_f64();
        
        // ✅ FIX: Correctly format f64 as a string
        let formatted_time = format!("{:.2}", elapsed_time);

        println!("✅ Found matching address: {}", pubkey_str);
        println!("⏱️ Time elapsed: {} seconds", formatted_time);
    } else {
        println!("❌ No matching address found.");
    }
}
