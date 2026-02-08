/// Mutation and mangling rules engine.

/// Apply all basic mutations to a word, returning new variants.
pub fn mutate_word(word: &str) -> Vec<String> {
    let mut results = Vec::new();
    let lower = word.to_lowercase();

    // Original
    results.push(lower.clone());

    // Capitalize first letter
    results.push(capitalize_first(&lower));

    // ALL CAPS
    results.push(lower.to_uppercase());

    // aLtErNaTiNg case
    results.push(alternating_case(&lower));

    // Reverse
    let reversed: String = lower.chars().rev().collect();
    results.push(reversed.clone());
    results.push(capitalize_first(&reversed));

    // Full leet speak
    results.push(full_leet(&lower));
    results.push(capitalize_first(&full_leet(&lower)));

    // Single-position leet substitutions (to avoid exponential blowup)
    results.extend(single_leet_variants(&lower));

    results
}

/// Apply mutations suitable for combined words (lighter set).
pub fn mutate_combined(word: &str) -> Vec<String> {
    let mut results = Vec::new();
    let lower = word.to_lowercase();

    results.push(lower.clone());
    results.push(capitalize_first(&lower));
    results.push(lower.to_uppercase());
    results.push(full_leet(&lower));

    results
}

/// Capitalize the first letter of a string.
pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// aLtErNaTiNg case.
fn alternating_case(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i % 2 == 0 {
                c.to_lowercase().to_string()
            } else {
                c.to_uppercase().to_string()
            }
        })
        .collect()
}

/// Full leet speak substitution.
pub fn full_leet(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a' => '@',
            'e' => '3',
            'i' => '1',
            'o' => '0',
            's' => '$',
            't' => '7',
            'l' => '1',
            _ => c,
        })
        .collect()
}

/// Single-position leet variants: replace one character at a time.
fn single_leet_variants(s: &str) -> Vec<String> {
    let leet_map: &[(char, &[char])] = &[
        ('a', &['@', '4']),
        ('e', &['3']),
        ('i', &['1', '!']),
        ('o', &['0']),
        ('s', &['$', '5']),
        ('t', &['7', '+']),
        ('l', &['1']),
    ];

    let chars: Vec<char> = s.chars().collect();
    let mut variants = Vec::new();

    for (i, &ch) in chars.iter().enumerate() {
        for &(from, replacements) in leet_map {
            if ch == from {
                for &replacement in replacements {
                    let mut new_chars = chars.clone();
                    new_chars[i] = replacement;
                    variants.push(new_chars.into_iter().collect());
                }
            }
        }
    }

    variants
}

/// Generate combined forms of two words.
pub fn combine_words(a: &str, b: &str) -> Vec<String> {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_cap = capitalize_first(&a_lower);
    let b_cap = capitalize_first(&b_lower);

    vec![
        format!("{}{}", a_lower, b_lower),       // johnsmith
        format!("{}{}", a_cap, b_cap),            // JohnSmith
        format!("{}{}", a_cap, b_lower),          // Johnsmith
        format!("{}_{}", a_lower, b_lower),       // john_smith
        format!("{}_{}", a_cap, b_cap),           // John_Smith
        format!("{}.{}", a_lower, b_lower),       // john.smith
        format!("{}{}", b_lower, a_lower),        // smithjohn
        format!("{}{}", b_cap, a_cap),            // SmithJohn
    ]
}

/// Generate word + number combinations.
pub fn combine_word_number(word: &str, number: &str) -> Vec<String> {
    let lower = word.to_lowercase();
    let cap = capitalize_first(&lower);

    vec![
        format!("{}{}", lower, number),  // john123
        format!("{}{}", cap, number),    // John123
        format!("{}{}", number, lower),  // 123john
        format!("{}{}", number, cap),    // 123John
    ]
}

/// Apply suffix to a word.
pub fn apply_suffix(word: &str, suffix: &str) -> Vec<String> {
    let lower = word.to_lowercase();
    let cap = capitalize_first(&lower);

    vec![
        format!("{}{}", lower, suffix),
        format!("{}{}", cap, suffix),
    ]
}

/// Apply prefix to a word.
pub fn apply_prefix(prefix: &str, word: &str) -> Vec<String> {
    let lower = word.to_lowercase();
    let cap = capitalize_first(&lower);

    vec![
        format!("{}{}", prefix, lower),
        format!("{}{}", prefix, cap),
    ]
}

/// Double a word.
pub fn double_word(word: &str) -> Vec<String> {
    let lower = word.to_lowercase();
    let cap = capitalize_first(&lower);

    vec![
        format!("{}{}", lower, lower),       // johnjohn
        format!("{}_{}", lower, lower),      // john_john
        format!("{}{}", cap, cap),           // JohnJohn
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn test_full_leet() {
        assert_eq!(full_leet("password"), "p@$$w0rd");
        assert_eq!(full_leet("leet"), "1337");
    }

    #[test]
    fn test_mutate_word() {
        let variants = mutate_word("test");
        assert!(variants.contains(&"test".to_string()));
        assert!(variants.contains(&"Test".to_string()));
        assert!(variants.contains(&"TEST".to_string()));
        assert!(variants.contains(&"tset".to_string())); // reversed
        assert!(variants.contains(&"7es7".to_string())); // leet variants
    }

    #[test]
    fn test_combine_words() {
        let combos = combine_words("john", "smith");
        assert!(combos.contains(&"johnsmith".to_string()));
        assert!(combos.contains(&"JohnSmith".to_string()));
        assert!(combos.contains(&"john_smith".to_string()));
    }
}
