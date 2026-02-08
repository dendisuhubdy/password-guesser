mod common;
mod cracker;
mod generator;
mod mutations;
mod profile;
mod wordlist;

use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "password-guesser",
    about = "Smart password guesser for educational cybersecurity research",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a wordlist from a target profile
    Generate {
        /// Path to the target profile TOML file
        #[arg(short, long)]
        profile: PathBuf,

        /// Output wordlist file
        #[arg(short, long)]
        output: PathBuf,

        /// Generation depth (1=fast ~5K, 2=medium ~20-50K, 3=deep ~100-500K)
        #[arg(short, long, default_value = "2", value_parser = clap::value_parser!(u8).range(1..=3))]
        depth: u8,

        /// Minimum password length
        #[arg(long, default_value = "6")]
        min_length: usize,

        /// Maximum password length
        #[arg(long, default_value = "32")]
        max_length: usize,
    },

    /// Crack hash(es) using a target profile
    CrackHash {
        /// Single hash to crack
        #[arg(long)]
        hash: Option<String>,

        /// File containing hashes (one per line)
        #[arg(long)]
        hash_file: Option<PathBuf>,

        /// Hash algorithm (md5, sha1, sha256, sha512, bcrypt, ntlm)
        #[arg(short, long)]
        algo: String,

        /// Path to the target profile TOML file
        #[arg(short, long)]
        profile: PathBuf,

        /// Generation depth (1-3)
        #[arg(short, long, default_value = "2", value_parser = clap::value_parser!(u8).range(1..=3))]
        depth: u8,

        /// Minimum password length
        #[arg(long, default_value = "6")]
        min_length: usize,

        /// Maximum password length
        #[arg(long, default_value = "32")]
        max_length: usize,
    },

    /// Crack a WiFi handshake using a target profile
    CrackWifi {
        /// Path to the handshake capture file (.cap/.pcap/.hccapx)
        #[arg(long)]
        handshake: PathBuf,

        /// Path to the target profile TOML file
        #[arg(short, long)]
        profile: PathBuf,

        /// Use hashcat instead of aircrack-ng
        #[arg(long)]
        use_hashcat: bool,

        /// Generation depth (1-3)
        #[arg(short, long, default_value = "2", value_parser = clap::value_parser!(u8).range(1..=3))]
        depth: u8,

        /// Minimum password length (WiFi requires 8+)
        #[arg(long, default_value = "8")]
        min_length: usize,

        /// Maximum password length
        #[arg(long, default_value = "63")]
        max_length: usize,
    },
}

fn main() -> Result<()> {
    print_banner();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            profile,
            output,
            depth,
            min_length,
            max_length,
        } => cmd_generate(&profile, &output, depth, min_length, max_length),

        Commands::CrackHash {
            hash,
            hash_file,
            algo,
            profile,
            depth,
            min_length,
            max_length,
        } => cmd_crack_hash(hash, hash_file, &algo, &profile, depth, min_length, max_length),

        Commands::CrackWifi {
            handshake,
            profile,
            use_hashcat,
            depth,
            min_length,
            max_length,
        } => cmd_crack_wifi(&handshake, &profile, use_hashcat, depth, min_length, max_length),
    }
}

fn print_banner() {
    let banner = r#"
  ╔═══════════════════════════════════════════╗
  ║       Smart Password Guesser v0.1.0       ║
  ║     Educational Cybersecurity Research     ║
  ╚═══════════════════════════════════════════╝
"#;
    println!("{}", banner.cyan());
}

fn cmd_generate(
    profile_path: &PathBuf,
    output: &PathBuf,
    depth: u8,
    min_length: usize,
    max_length: usize,
) -> Result<()> {
    let profile = profile::Profile::load(profile_path)?;

    let config = generator::GeneratorConfig {
        depth,
        min_length,
        max_length,
    };

    println!(
        "{} Profile: {} | Depth: {} | Length: {}-{}",
        ">>".cyan().bold(),
        profile_path.display(),
        depth,
        min_length,
        max_length,
    );

    let seeds = profile.seed_words();
    println!(
        "{} Seed words: {}",
        ">>".cyan().bold(),
        seeds.join(", ").dimmed()
    );

    let candidates = generator::generate_candidates(&profile, &config);

    wordlist::write_wordlist(output, &candidates)?;

    println!(
        "\n{} Wrote {} candidates to {}",
        "SUCCESS".green().bold(),
        candidates.len(),
        output.display()
    );

    Ok(())
}

fn cmd_crack_hash(
    hash: Option<String>,
    hash_file: Option<PathBuf>,
    algo_str: &str,
    profile_path: &PathBuf,
    depth: u8,
    min_length: usize,
    max_length: usize,
) -> Result<()> {
    let algo = cracker::HashAlgorithm::from_str(algo_str);
    let algo = match algo {
        Some(a) => a,
        None => bail!(
            "Unknown algorithm: {}. Supported: md5, sha1, sha256, sha512, bcrypt, ntlm",
            algo_str
        ),
    };

    // Collect hashes
    let mut hashes = Vec::new();
    if let Some(ref h) = hash {
        hashes.push(h.clone());
    }
    if let Some(ref path) = hash_file {
        let file_hashes = wordlist::read_wordlist(path)?;
        hashes.extend(file_hashes);
    }
    if hashes.is_empty() {
        bail!("Provide --hash or --hash-file");
    }

    // Generate candidates
    let profile = profile::Profile::load(profile_path)?;
    let config = generator::GeneratorConfig {
        depth,
        min_length,
        max_length,
    };

    println!(
        "{} Profile: {} | Algo: {} | Depth: {}",
        ">>".cyan().bold(),
        profile_path.display(),
        algo,
        depth,
    );

    let seeds = profile.seed_words();
    println!(
        "{} Seed words: {}",
        ">>".cyan().bold(),
        seeds.join(", ").dimmed()
    );

    let candidates = generator::generate_candidates(&profile, &config);

    // Crack
    let results = cracker::hash::crack_hashes(&hashes, algo, &candidates)?;

    // Summary
    println!();
    if results.is_empty() {
        println!(
            "{} No hashes cracked. Try increasing --depth or enriching the profile.",
            "RESULT".yellow().bold()
        );
    } else {
        println!(
            "{} Cracked {}/{} hash(es):",
            "RESULT".green().bold(),
            results.len(),
            hashes.len()
        );
        for r in &results {
            println!("  {} {}", "→".green(), r);
        }
    }

    Ok(())
}

fn cmd_crack_wifi(
    handshake: &PathBuf,
    profile_path: &PathBuf,
    use_hashcat: bool,
    depth: u8,
    min_length: usize,
    max_length: usize,
) -> Result<()> {
    let profile = profile::Profile::load(profile_path)?;
    let config = generator::GeneratorConfig {
        depth,
        min_length,
        max_length,
    };

    println!(
        "{} Profile: {} | Depth: {} | Tool: {}",
        ">>".cyan().bold(),
        profile_path.display(),
        depth,
        if use_hashcat { "hashcat" } else { "aircrack-ng" },
    );

    let candidates = generator::generate_candidates(&profile, &config);

    // Write to temp file
    let tmp_dir = std::env::temp_dir();
    let wordlist_path = tmp_dir.join("password_guesser_wordlist.txt");
    wordlist::write_wordlist(&wordlist_path, &candidates)?;

    println!(
        "{} Wordlist written to {} ({} candidates)",
        ">>".cyan().bold(),
        wordlist_path.display(),
        candidates.len()
    );

    if use_hashcat {
        cracker::wifi::crack_with_hashcat(handshake, &wordlist_path)?;
    } else {
        cracker::wifi::crack_with_aircrack(handshake, &wordlist_path)?;
    }

    // Clean up temp file
    let _ = std::fs::remove_file(&wordlist_path);

    Ok(())
}
