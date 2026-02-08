use std::collections::HashSet;

use indicatif::{ProgressBar, ProgressStyle};

use crate::common;
use crate::mutations;
use crate::profile::Profile;

/// Depth controls how many tiers of candidates are generated.
#[derive(Debug, Clone, Copy)]
pub struct GeneratorConfig {
    pub depth: u8,       // 1-3
    pub min_length: usize,
    pub max_length: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            depth: 2,
            min_length: 6,
            max_length: 32,
        }
    }
}

/// Generate all candidate passwords based on profile and config.
pub fn generate_candidates(profile: &Profile, config: &GeneratorConfig) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg} [{elapsed_precise}]")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );

    let seed_words = profile.seed_words();
    let seed_numbers = profile.seed_numbers();

    // Tier 1: Common passwords
    pb.set_message("Tier 1: Common passwords...");
    let common = common::common_passwords();
    add_unique(&mut candidates, &mut seen, common.into_iter(), config);
    pb.set_message(format!("Tier 1 done: {} candidates", candidates.len()));

    // Tier 2: Mutated seed words
    pb.set_message("Tier 2: Mutating seed words...");
    let mut tier2 = Vec::new();
    for word in &seed_words {
        tier2.extend(mutations::mutate_word(word));
        tier2.extend(mutations::double_word(word));
    }
    add_unique(&mut candidates, &mut seen, tier2.into_iter(), config);
    pb.set_message(format!("Tier 2 done: {} candidates", candidates.len()));

    // Tier 3: Seeds + affixes
    if config.depth >= 2 {
        pb.set_message("Tier 3: Applying affixes...");
        let mut tier3 = Vec::new();

        let num_suffixes = common::numeric_suffixes();
        let sym_suffixes = common::symbol_suffixes();
        let prefixes = common::common_prefixes();

        for word in &seed_words {
            // Numeric suffixes
            for suffix in &num_suffixes {
                tier3.extend(mutations::apply_suffix(word, suffix));
            }
            // Symbol suffixes
            for suffix in &sym_suffixes {
                tier3.extend(mutations::apply_suffix(word, suffix));
            }
            // Prefixes
            for prefix in &prefixes {
                tier3.extend(mutations::apply_prefix(prefix, word));
            }
            // Seed numbers as suffixes
            for num in &seed_numbers {
                tier3.extend(mutations::combine_word_number(word, num));
            }
        }

        // Also add seed numbers with common words
        for num in &seed_numbers {
            tier3.push(num.clone());
        }

        add_unique(&mut candidates, &mut seen, tier3.into_iter(), config);
        pb.set_message(format!("Tier 3 done: {} candidates", candidates.len()));
    }

    // Tier 4: Word combinations
    if config.depth >= 2 {
        pb.set_message("Tier 4: Combining words...");
        let mut tier4 = Vec::new();

        for (i, a) in seed_words.iter().enumerate() {
            for b in seed_words.iter().skip(i + 1) {
                tier4.extend(mutations::combine_words(a, b));
            }
            // Word + seed number combos
            for num in &seed_numbers {
                tier4.extend(mutations::combine_word_number(a, num));
            }
        }

        add_unique(&mut candidates, &mut seen, tier4.into_iter(), config);
        pb.set_message(format!("Tier 4 done: {} candidates", candidates.len()));
    }

    // Tier 5: Keyboard patterns
    if config.depth >= 2 {
        pb.set_message("Tier 5: Keyboard patterns...");
        let patterns = common::keyboard_patterns();
        add_unique(&mut candidates, &mut seen, patterns.into_iter(), config);
        pb.set_message(format!("Tier 5 done: {} candidates", candidates.len()));
    }

    // Tier 6: Deep mutations on combinations (depth=3 only)
    if config.depth >= 3 {
        pb.set_message("Tier 6: Deep mutations on combinations...");
        let mut tier6 = Vec::new();

        // Mutate Tier 4 style combinations
        for (i, a) in seed_words.iter().enumerate() {
            for b in seed_words.iter().skip(i + 1) {
                let combos = mutations::combine_words(a, b);
                for combo in &combos {
                    tier6.extend(mutations::mutate_combined(combo));
                    // Add suffixes to combos
                    for suffix in &["123", "!", "1", "12", "1!"] {
                        tier6.push(format!("{}{}", combo, suffix));
                    }
                }
            }
        }

        // Mutated seeds + affixes
        for word in &seed_words {
            let mutated = mutations::mutate_word(word);
            let num_suffixes = common::numeric_suffixes();
            for m in &mutated {
                for suffix in &num_suffixes {
                    tier6.extend(mutations::apply_suffix(m, suffix));
                }
            }
        }

        add_unique(&mut candidates, &mut seen, tier6.into_iter(), config);
        pb.set_message(format!("Tier 6 done: {} candidates", candidates.len()));
    }

    pb.finish_with_message(format!("Generated {} unique candidates", candidates.len()));
    candidates
}

/// Add items to candidates if they pass filters and haven't been seen.
fn add_unique(
    candidates: &mut Vec<String>,
    seen: &mut HashSet<String>,
    items: impl Iterator<Item = String>,
    config: &GeneratorConfig,
) {
    for item in items {
        if item.len() >= config.min_length
            && item.len() <= config.max_length
            && seen.insert(item.clone())
        {
            candidates.push(item);
        }
    }
}
