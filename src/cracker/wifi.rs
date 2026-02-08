use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;

/// Crack a WiFi handshake using aircrack-ng.
pub fn crack_with_aircrack(handshake: &Path, wordlist: &Path) -> Result<()> {
    // Check if aircrack-ng is available
    if !command_exists("aircrack-ng") {
        bail!(
            "aircrack-ng not found. Install it:\n\
             - macOS: brew install aircrack-ng\n\
             - Ubuntu/Debian: sudo apt install aircrack-ng\n\
             - Arch: sudo pacman -S aircrack-ng"
        );
    }

    if !handshake.exists() {
        bail!("Handshake file not found: {}", handshake.display());
    }

    println!(
        "{} Running aircrack-ng with wordlist ({} entries)...",
        ">>".cyan().bold(),
        count_lines(wordlist)?
    );

    let output = Command::new("aircrack-ng")
        .arg("-w")
        .arg(wordlist.as_os_str())
        .arg(handshake.as_os_str())
        .output()
        .context("Failed to execute aircrack-ng")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stdout.contains("KEY FOUND!") {
        println!("{}", stdout);
        println!("{} WiFi key cracked!", "SUCCESS".green().bold());
    } else {
        println!("{}", stdout);
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        println!(
            "{} Key not found in wordlist. Try increasing --depth or adding more profile data.",
            "FAILED".red().bold()
        );
    }

    Ok(())
}

/// Crack a WiFi handshake using hashcat.
pub fn crack_with_hashcat(handshake: &Path, wordlist: &Path) -> Result<()> {
    // Check if hashcat is available
    if !command_exists("hashcat") {
        bail!(
            "hashcat not found. Install it:\n\
             - macOS: brew install hashcat\n\
             - Ubuntu/Debian: sudo apt install hashcat\n\
             - Arch: sudo pacman -S hashcat\n\
             - Or download from https://hashcat.net/hashcat/"
        );
    }

    if !handshake.exists() {
        bail!("Handshake file not found: {}", handshake.display());
    }

    // Convert .cap to .hccapx if needed
    let hccapx_path = if handshake.extension().map_or(false, |e| e == "cap" || e == "pcap") {
        let hccapx = handshake.with_extension("hccapx");
        convert_cap_to_hccapx(handshake, &hccapx)?;
        hccapx
    } else {
        handshake.to_path_buf()
    };

    println!(
        "{} Running hashcat with wordlist ({} entries)...",
        ">>".cyan().bold(),
        count_lines(wordlist)?
    );

    // hashcat mode 22000 for WPA-PBKDF2-PMKID+EAPOL (newer)
    // Fall back to mode 2500 for WPA/WPA2
    let output = Command::new("hashcat")
        .arg("-m")
        .arg("2500")
        .arg("-a")
        .arg("0")
        .arg(hccapx_path.as_os_str())
        .arg(wordlist.as_os_str())
        .arg("--force")
        .output()
        .context("Failed to execute hashcat")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("{}", stdout);
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }

    if output.status.success() {
        println!("{} hashcat completed.", "DONE".green().bold());
    } else {
        println!(
            "{} hashcat exited with code {}",
            "WARNING".yellow().bold(),
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

/// Convert .cap file to .hccapx using aircrack-ng.
fn convert_cap_to_hccapx(cap: &Path, hccapx: &Path) -> Result<()> {
    if !command_exists("aircrack-ng") {
        bail!(
            "aircrack-ng is needed to convert .cap to .hccapx. Install it first."
        );
    }

    println!(
        "{} Converting {} to hccapx format...",
        ">>".cyan().bold(),
        cap.display()
    );

    let output = Command::new("aircrack-ng")
        .arg(cap.as_os_str())
        .arg("-J")
        .arg(hccapx.with_extension("").as_os_str())
        .output()
        .context("Failed to convert cap to hccapx")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to convert capture file: {}", stderr);
    }

    Ok(())
}

/// Check if a command exists on PATH.
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Count lines in a file.
fn count_lines(path: &Path) -> Result<usize> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    Ok(content.lines().count())
}
