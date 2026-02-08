/// Embedded common passwords, keyboard patterns, and common affixes.

/// Top common passwords embedded at compile time.
pub const COMMON_PASSWORDS: &str = include_str!("../data/common_passwords.txt");

/// Returns deduplicated list of common passwords.
pub fn common_passwords() -> Vec<String> {
    COMMON_PASSWORDS
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Common keyboard walk patterns.
pub fn keyboard_patterns() -> Vec<String> {
    vec![
        // Row walks
        "qwerty", "qwertyuiop", "qwert", "asdfgh", "asdfghjkl", "zxcvbn", "zxcvbnm",
        // Diagonal walks
        "qazwsx", "1qaz2wsx", "1qaz2wsx3edc", "zaq1xsw2",
        // Number runs
        "123456", "1234567", "12345678", "123456789", "1234567890",
        "0987654321", "987654321", "654321", "54321",
        // Numpad patterns
        "147258369", "159357", "789456123", "321654987",
        // Repeats
        "aaaaaa", "000000", "111111", "222222", "555555", "666666", "777777", "88888888",
        "999999", "112233", "123123", "121212", "131313", "123321",
        // Short keyboard
        "qwer", "asdf", "zxcv", "1234", "4321",
        // Other common patterns
        "abcdef", "abcdefg", "abcdefgh", "abcd1234", "1234abcd",
        "abc123", "123abc", "aaa111", "zzz999",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Common numeric suffixes appended to words.
pub fn numeric_suffixes() -> Vec<String> {
    let mut suffixes = Vec::new();

    // Single digits
    for i in 0..=9 {
        suffixes.push(format!("{}", i));
    }
    // Common double digits
    for &n in &[10, 11, 12, 13, 21, 22, 23, 69, 77, 88, 99] {
        suffixes.push(format!("{}", n));
    }
    // Common triple digits
    for &n in &[100, 111, 123, 321, 234, 420, 666, 777, 007, 911] {
        suffixes.push(format!("{}", n));
    }
    // Years 1950-2026
    for y in 1950..=2026 {
        suffixes.push(format!("{}", y));
    }
    // Short years
    for y in 0..=26 {
        suffixes.push(format!("{:02}", y));
    }

    suffixes
}

/// Common symbol suffixes.
pub fn symbol_suffixes() -> Vec<String> {
    vec![
        "!", "!!", "!!!", "@", "#", "$", "!@#", "!1", "@1", "#1",
        "!!", "?", "*", ".", "!", "!@", "@#", "#$",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Common prefixes prepended to words.
pub fn common_prefixes() -> Vec<String> {
    vec![
        "my", "the", "i", "its", "mr", "ms", "im", "iam", "ilove", "ilike",
        "my1", "the1", "super", "mega", "big", "lil",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
