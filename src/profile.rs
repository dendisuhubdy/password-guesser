use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// A target profile loaded from TOML.
#[derive(Debug, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub personal: Personal,
    #[serde(default)]
    pub network: Network,
    #[serde(default)]
    pub interests: Interests,
    #[serde(default)]
    pub custom: Custom,
}

#[derive(Debug, Default, Deserialize)]
pub struct Personal {
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub birthdate: Option<String>, // YYYY-MM-DD
    #[serde(default)]
    pub partner_name: Option<String>,
    #[serde(default)]
    pub pet_name: Option<String>,
    #[serde(default)]
    pub children_names: Vec<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Network {
    #[serde(default)]
    pub ssid: Option<String>,
    #[serde(default)]
    pub router_brand: Option<String>,
    #[serde(default)]
    pub isp: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Interests {
    #[serde(default)]
    pub favorite_team: Option<String>,
    #[serde(default)]
    pub favorite_band: Option<String>,
    #[serde(default)]
    pub hobbies: Vec<String>,
    #[serde(default)]
    pub favorite_color: Option<String>,
    #[serde(default)]
    pub favorite_number: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Custom {
    #[serde(default)]
    pub words: Vec<String>,
    #[serde(default)]
    pub numbers: Vec<String>,
}

impl Profile {
    /// Load a profile from a TOML file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read profile: {}", path.display()))?;
        let profile: Profile =
            toml::from_str(&content).with_context(|| "Failed to parse profile TOML")?;
        Ok(profile)
    }

    /// Extract all seed words from the profile (lowercased, non-empty).
    pub fn seed_words(&self) -> Vec<String> {
        let mut words = Vec::new();

        let p = &self.personal;
        push_opt(&mut words, &p.first_name);
        push_opt(&mut words, &p.last_name);
        push_opt(&mut words, &p.nickname);
        push_opt(&mut words, &p.partner_name);
        push_opt(&mut words, &p.pet_name);
        for name in &p.children_names {
            push_word(&mut words, name);
        }

        let n = &self.network;
        push_opt(&mut words, &n.ssid);
        push_opt(&mut words, &n.router_brand);
        push_opt(&mut words, &n.isp);

        let i = &self.interests;
        push_opt(&mut words, &i.favorite_team);
        push_opt(&mut words, &i.favorite_band);
        for h in &i.hobbies {
            push_word(&mut words, h);
        }
        push_opt(&mut words, &i.favorite_color);

        for w in &self.custom.words {
            push_word(&mut words, w);
        }

        words
    }

    /// Extract seed numbers from the profile.
    pub fn seed_numbers(&self) -> Vec<String> {
        let mut numbers = Vec::new();

        // Birthdate decomposition
        if let Some(ref bd) = self.personal.birthdate {
            numbers.extend(decompose_date(bd));
        }

        // Phone digits
        if let Some(ref phone) = self.personal.phone {
            let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                numbers.push(digits.clone());
                // Last 4 digits
                if digits.len() >= 4 {
                    numbers.push(digits[digits.len() - 4..].to_string());
                }
            }
        }

        // Favorite number
        if let Some(ref n) = self.interests.favorite_number {
            numbers.push(n.clone());
        }

        // Custom numbers
        for n in &self.custom.numbers {
            if !n.is_empty() {
                numbers.push(n.clone());
            }
        }

        numbers
    }
}

fn push_opt(words: &mut Vec<String>, opt: &Option<String>) {
    if let Some(ref s) = opt {
        push_word(words, s);
    }
}

fn push_word(words: &mut Vec<String>, s: &str) {
    let trimmed = s.trim();
    if !trimmed.is_empty() {
        // Add the whole thing lowercased
        words.push(trimmed.to_lowercase());
        // If it contains spaces/hyphens, also add individual parts
        for part in trimmed.split(|c: char| c == ' ' || c == '-' || c == '_') {
            let p = part.trim().to_lowercase();
            if !p.is_empty() && p != trimmed.to_lowercase() {
                words.push(p);
            }
        }
    }
}

/// Decompose a date string (YYYY-MM-DD) into useful number fragments.
fn decompose_date(date: &str) -> Vec<String> {
    let mut frags = Vec::new();
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        let year = parts[0];
        let month = parts[1];
        let day = parts[2];

        frags.push(year.to_string()); // 1990
        if year.len() == 4 {
            frags.push(year[2..].to_string()); // 90
        }
        frags.push(month.to_string()); // 05
        frags.push(day.to_string()); // 15
        frags.push(format!("{}{}", month, day)); // 0515
        frags.push(format!("{}{}", day, month)); // 1505
        frags.push(format!("{}{}{}", month, day, year)); // 05151990
        frags.push(format!("{}{}{}", day, month, year)); // 15051990
        if year.len() == 4 {
            frags.push(format!("{}{}{}", month, day, &year[2..])); // 051590
            frags.push(format!("{}{}{}", day, month, &year[2..])); // 150590
        }
    }
    frags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_date() {
        let frags = decompose_date("1990-05-15");
        assert!(frags.contains(&"1990".to_string()));
        assert!(frags.contains(&"90".to_string()));
        assert!(frags.contains(&"0515".to_string()));
        assert!(frags.contains(&"051590".to_string()));
    }
}
