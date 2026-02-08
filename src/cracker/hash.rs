use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use anyhow::{bail, Result};
use colored::Colorize;
use digest::Digest;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use super::{CrackResult, HashAlgorithm};

/// Crack one or more hashes against a list of candidates.
pub fn crack_hashes(
    hashes: &[String],
    algo: HashAlgorithm,
    candidates: &[String],
) -> Result<Vec<CrackResult>> {
    if hashes.is_empty() {
        bail!("No hashes provided");
    }

    println!(
        "{} Cracking {} hash(es) with {} algorithm using {} candidates...",
        ">>".cyan().bold(),
        hashes.len(),
        algo,
        candidates.len()
    );

    match algo {
        HashAlgorithm::Bcrypt => crack_bcrypt(hashes, candidates),
        _ => crack_fast_hash(hashes, algo, candidates),
    }
}

/// Crack fast hashes (MD5, SHA1, SHA256, SHA512) using rayon.
fn crack_fast_hash(
    hashes: &[String],
    algo: HashAlgorithm,
    candidates: &[String],
) -> Result<Vec<CrackResult>> {
    let target_hashes: Vec<String> = hashes.iter().map(|h| h.to_lowercase()).collect();
    let total_hashes = target_hashes.len();
    let found_count = AtomicUsize::new(0);
    let all_found = AtomicBool::new(false);
    let results: Mutex<Vec<CrackResult>> = Mutex::new(Vec::new());
    let checked = AtomicUsize::new(0);

    let pb = ProgressBar::new(candidates.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏ "),
    );

    candidates.par_iter().for_each(|candidate| {
        if all_found.load(Ordering::Relaxed) {
            return;
        }

        let hash_hex = compute_hash(algo, candidate);

        // Check against all target hashes
        for target in &target_hashes {
            if hash_hex == *target {
                let mut res = results.lock().unwrap();
                res.push(CrackResult {
                    hash: target.clone(),
                    plaintext: candidate.clone(),
                    algorithm: algo,
                });
                let count = found_count.fetch_add(1, Ordering::Relaxed) + 1;
                pb.println(format!(
                    "  {} Found: {} -> {}",
                    "✓".green().bold(),
                    target.dimmed(),
                    candidate.green().bold()
                ));
                if count >= total_hashes {
                    all_found.store(true, Ordering::Relaxed);
                }
            }
        }

        let prev = checked.fetch_add(1, Ordering::Relaxed);
        if prev % 1000 == 0 {
            pb.set_position(prev as u64);
        }
    });

    pb.finish_and_clear();

    let results = results.into_inner().unwrap();
    Ok(results)
}

/// Crack bcrypt hashes (much slower, uses bcrypt::verify).
fn crack_bcrypt(hashes: &[String], candidates: &[String]) -> Result<Vec<CrackResult>> {
    let results: Mutex<Vec<CrackResult>> = Mutex::new(Vec::new());
    let total_hashes = hashes.len();
    let found_count = AtomicUsize::new(0);
    let all_found = AtomicBool::new(false);

    let pb = ProgressBar::new(candidates.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.set_message("(bcrypt is slow ~100/sec)");

    candidates.par_iter().enumerate().for_each(|(i, candidate)| {
        if all_found.load(Ordering::Relaxed) {
            return;
        }

        for target in hashes {
            if let Ok(true) = bcrypt::verify(candidate, target) {
                let mut res = results.lock().unwrap();
                res.push(CrackResult {
                    hash: target.clone(),
                    plaintext: candidate.clone(),
                    algorithm: HashAlgorithm::Bcrypt,
                });
                let count = found_count.fetch_add(1, Ordering::Relaxed) + 1;
                pb.println(format!(
                    "  {} Found: {} -> {}",
                    "✓".green().bold(),
                    target.dimmed(),
                    candidate.green().bold()
                ));
                if count >= total_hashes {
                    all_found.store(true, Ordering::Relaxed);
                }
            }
        }

        if i % 10 == 0 {
            pb.set_position(i as u64);
        }
    });

    pb.finish_and_clear();

    let results = results.into_inner().unwrap();
    Ok(results)
}

/// Compute the hex-encoded hash of a candidate.
fn compute_hash(algo: HashAlgorithm, input: &str) -> String {
    match algo {
        HashAlgorithm::Md5 => {
            let mut hasher = md5::Md5::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Bcrypt => {
            // bcrypt doesn't produce a hex hash for comparison
            unreachable!("bcrypt uses verify, not hash comparison")
        }
    }
}
