use bitcoin::{
    key::{
        secp256k1::{rand, SecretKey},
        Keypair,
    },
    Address, Network, XOnlyPublicKey, PublicKey, 
};
use clap::Parser;
use rayon::prelude::*;
use std::fs::{OpenOptions, read_to_string, File};
use std::io::Write;
use std::time::Instant;

/// Command line arguments parser using clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    
	/// Search for patterns as suffix instead of prefix
	#[arg(short = 's', long, default_value_t = false)]
    suffix: bool,
	
	/// Single pattern to search for (instead of filename)
	#[arg(short = 'p', long)]
    pattern: Option<String>,
	
	/// Filename to load patterns from (default: patterns.txt)
	#[arg(short = 'f', long)]
	filename: Option<String>,
	
	/// Optional single pattern for payment address (p2wpkh)
	#[arg(short = 'y', long)]
    patternpay: Option<String>,
}

/// Load patterns from a file, one per line, ignoring empty lines
fn load_patterns_from_file(file: &str) -> Vec<String> {
    read_to_string(file)
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Save patterns back to file (used to remove found patterns)
fn save_patterns_to_file(file: &str, patterns: &Vec<String>) {
    let mut f = File::create(file).unwrap(); 
    for p in patterns {
        writeln!(f, "{}", p).unwrap();
    }
}

fn main() {
    
	// Parse CLI arguments
	let args = Args::parse();
	
	// Initialize secp256k1 context
    let secp = bitcoin::key::Secp256k1::new();
	
	// Determine pattern filenames (default to patterns.txt / patterns_payment.txt)
	let file = args.filename.as_deref().unwrap_or("patterns.txt");
	let filepay = args.filename.as_deref().unwrap_or("patterns_payment.txt");
	
	// Load patterns from argument or file
	let mut patterns = if let Some(p) = &args.pattern {
		vec![p.clone()]
	} else {
		load_patterns_from_file(file)
	};
	
	// Load payment patterns from argument or file
	let patternspay = if let Some(pay) = &args.patternpay {
		vec![pay.clone()]
	} else {
		load_patterns_from_file(filepay)
	};
	
	// Info: show what patterns are being searched
	if let Some(_) = args.pattern.as_ref() {
		println!("Pattern from console input.");
	}
	else {
		println!("pattern from file: {}", file);
	}
	
	if let Some(_) = args.patternpay.as_ref() {
		println!("Pay-pattern from console input.");
	}
	else {
		println!("Pay-pattern from file: patterns_payment.txt");
	}
	
	if args.suffix {
		
		println!("Search for suffix(es):");
		patterns.iter().for_each(|p| println!("{}", p));
		if !patternspay.is_empty() {
			println!("...with payment (p2wpkh) suffix(es):");
			patternspay.iter().for_each(|pay| println!("{}", pay));	
		}
		else {
			println!("...without any payment (p2wpkh) suffix(es).");
		}
	
	}
	else {
		
		println!("Search for prefix(es):");
		patterns.iter().for_each(|p| println!("{}", p));
		
		if !patternspay.is_empty() {
			println!("...with payment (p2wpkh) prefix(es):");
			patternspay.iter().for_each(|pay| println!("{}", pay));	
		}
		else {
			println!("...without any payment (p2wpkh) prefix(es).");
		}
		
	}
	
	// Main loop: continue until all patterns are found
	while !patterns.is_empty() {
		
		let payload = vec![0; 1_000_000];
        let kickoff = Instant::now();
		
		 // Generate addresses and check for pattern matches
		let upshot = payload
            .par_iter()
            .map(|_| {
                
				// Generate new secret key and derive keypair
				let sec_key = SecretKey::new(&mut rand::thread_rng());
                let keypair = Keypair::from_secret_key(&secp, &sec_key);
				let pub_key = PublicKey::new(keypair.public_key());
				let (x_only_public_key, _) = XOnlyPublicKey::from_keypair(&keypair);
				
				// Generate Taproot (p2tr) and legacy/witness (p2wpkh) addresses
                let p2tr = Address::p2tr(&secp, x_only_public_key, None, Network::Bitcoin);
				let p2wpkh = Address::p2wpkh(&pub_key, Network::Bitcoin).ok()?;
				
                let p2tr_str = format!("{}", p2tr);
				let p2wpkh_str = format!("{}", p2wpkh);
				
				// Check if any pattern matches the generated addresses
                if args.suffix {					
					for pat in &patterns {
						if p2tr_str.ends_with(pat) {
							if !patternspay.is_empty() {
								for patpay in &patternspay {
									if p2wpkh_str.ends_with(patpay) {
										return Some((p2tr, p2wpkh, keypair, pat.clone()));
									}
								}
							}
							else{
								return Some((p2tr, p2wpkh, keypair, pat.clone()));
							}
						}
					}					
				} else {					
					for pat in &patterns {
						if p2tr_str.starts_with(&format!("bc1p{}", pat)) {
							if !patternspay.is_empty() {
								for patpay in &patternspay {
									if p2wpkh_str.starts_with(&format!("bc1p{}", patpay)) {
										return Some((p2tr, p2wpkh, keypair, pat.clone()));
									}
								}
							}
							else{
								return Some((p2tr, p2wpkh, keypair, pat.clone()));
							}
						}
					}					
				}
				None
            })
            .filter(|e| e.is_some())
            .collect::<Vec<_>>();
			
		// Calculate performance in H/s
		let duration = kickoff.elapsed().as_secs_f64();
		let hps = payload.len() as f64 / duration;
		println!(
			"{:.2} {}",
			if hps >= 1e9 { hps / 1e9 } else if hps >= 1e6 { hps / 1e6 } else if hps >= 1e3 { hps / 1e3 } else { hps },
			if hps >= 1e9 { "GH/s" } else if hps >= 1e6 { "MH/s" } else if hps >= 1e3 { "kH/s" } else { "H/s" }
		);
		
		// Process any hits
		if upshot.len() > 0 {
            
			let (p2tr, p2wpkh, keypair, pattern) = upshot[0].clone().unwrap();
			let (x_only_public_key, _) = XOnlyPublicKey::from_keypair(&keypair);
			
			println!("HIT!");
			println!("pattern: {}", pattern);
            println!("p2tr: {}", p2tr);
			if !patternspay.is_empty() {
				println!("p2wpkh: {}", p2wpkh);
			}
            println!("private key: {}", keypair.display_secret());
			println!("public key: {}", keypair.public_key());
			println!("XOnlyPublicKey: {}", x_only_public_key);
			println!("ScriptPubKey: {}", p2tr.script_pubkey());

            // Remove found pattern from memory and file
            patterns.retain(|p| p != &pattern);
			save_patterns_to_file(file, &patterns);

            // Append results to results.txt
            let mut f = OpenOptions::new().append(true).create(true).open("results.txt").unwrap();
			writeln!(
				f,
				"Pattern: {}\np2tr: {}\np2wpkh: {}\nPrivateKey: {}\nPublicKey: {}\nXOnlyPublicKey: {}\nScriptPubKey: {}\n",
				pattern,
				p2tr,
				p2wpkh,
				keypair.display_secret(),
				keypair.public_key(),
				x_only_public_key,         
				p2tr.script_pubkey()     
			).unwrap();
			
        }
		
	}
	
}