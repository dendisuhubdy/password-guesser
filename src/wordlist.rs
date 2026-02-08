use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};

/// Write a list of candidates to a file, one per line.
pub fn write_wordlist(path: &Path, candidates: &[String]) -> Result<()> {
    let file = std::fs::File::create(path)
        .with_context(|| format!("Failed to create wordlist: {}", path.display()))?;
    let mut writer = BufWriter::new(file);

    for candidate in candidates {
        writeln!(writer, "{}", candidate)
            .with_context(|| "Failed to write to wordlist")?;
    }

    writer.flush().with_context(|| "Failed to flush wordlist")?;
    Ok(())
}

/// Read a wordlist from a file, one entry per line.
pub fn read_wordlist(path: &Path) -> Result<Vec<String>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open wordlist: {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut words = Vec::new();

    for line in reader.lines() {
        let line = line.with_context(|| "Failed to read line from wordlist")?;
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            words.push(trimmed);
        }
    }

    Ok(words)
}
