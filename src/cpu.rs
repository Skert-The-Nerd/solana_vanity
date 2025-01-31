use ed25519_dalek::SigningKey;
use rand::RngCore;
use rayon::prelude::*;

pub fn grind(target: String, case_insensitive: bool, num_threads: u32) {
    (0..num_threads).into_par_iter().for_each(|_| {
        let mut rng = rand::thread_rng();
        let mut buffer = [0u8; 32];

        loop {
            rng.fill_bytes(&mut buffer);
            let signing_key = SigningKey::from_bytes(&buffer);

            let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
            let check_pubkey = if case_insensitive {
                pubkey.to_lowercase()
            } else {
                pubkey.clone()
            };

            if check_pubkey.starts_with(&target) {
                println!("MATCH FOUND: {}", pubkey);
                break;
            }
        }
    });
}
