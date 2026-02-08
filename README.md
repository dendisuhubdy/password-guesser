# password-guesser

Smart, profile-driven password guesser for **educational cybersecurity research**. Generates targeted wordlists and cracks hashes by building intelligent password candidates from known information about a target.

> **Disclaimer:** This tool is intended for authorized security testing, CTF competitions, and educational purposes only. Unauthorized use against systems you do not own or have explicit permission to test is illegal.

## Features

- **Profile-based generation** — Build password candidates from personal info, interests, network details, and custom words defined in a TOML profile
- **Mutation engine** — Automatically applies case variations, leet speak, reversals, word combinations, numeric/symbol suffixes, and common prefixes
- **Hash cracking** — Crack MD5, SHA1, SHA256, SHA512, and bcrypt hashes
- **WiFi cracking** — Crack WPA/WPA2 handshakes via aircrack-ng or hashcat
- **Tiered depth control** — Three generation depths: fast (~5K candidates), medium (~20-50K), deep (~100-500K)
- **Parallel processing** — Uses rayon for multi-threaded hash cracking

## Installation

```sh
cargo build --release
```

The binary will be at `target/release/password-guesser`.

## Usage

### 1. Create a target profile

Create a TOML file with known information about the target. See [`examples/target_profile.toml`](examples/target_profile.toml) for a full example:

```toml
[personal]
first_name = "John"
last_name = "Smith"
nickname = "Johnny"
birthdate = "1990-05-15"
partner_name = "Jane"
pet_name = "Buddy"
children_names = ["Emma", "Liam"]
phone = "+1-555-867-5309"

[network]
ssid = "SmithFamily"
router_brand = "Netgear"

[interests]
favorite_team = "Lakers"
favorite_band = "Metallica"
hobbies = ["fishing", "gaming", "guitar"]
favorite_color = "blue"
favorite_number = "7"

[custom]
words = ["mustang", "texas"]
numbers = ["1234", "42"]
```

All fields are optional — fill in whatever you know.

### 2. Generate a wordlist

```sh
password-guesser generate \
  --profile examples/target_profile.toml \
  --output wordlist.txt \
  --depth 2
```

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --profile` | Path to target profile TOML | required |
| `-o, --output` | Output wordlist file | required |
| `-d, --depth` | Generation depth (1=fast, 2=medium, 3=deep) | 2 |
| `--min-length` | Minimum password length | 6 |
| `--max-length` | Maximum password length | 32 |

### 3. Crack hashes

```sh
# Single hash
password-guesser crack-hash \
  --hash "5f4dcc3b5aa765d61d8327deb882cf99" \
  --algo md5 \
  --profile examples/target_profile.toml

# Multiple hashes from file
password-guesser crack-hash \
  --hash-file hashes.txt \
  --algo sha256 \
  --profile examples/target_profile.toml \
  --depth 3
```

Supported algorithms: `md5`, `sha1`, `sha256`, `sha512`, `bcrypt`

### 4. Crack WiFi handshakes

```sh
# Using aircrack-ng (default)
password-guesser crack-wifi \
  --handshake capture.cap \
  --profile examples/target_profile.toml

# Using hashcat
password-guesser crack-wifi \
  --handshake capture.hccapx \
  --profile examples/target_profile.toml \
  --use-hashcat
```

WiFi mode requires `aircrack-ng` or `hashcat` to be installed. Minimum password length defaults to 8 (WPA requirement).

## How it works

The generator builds candidates in tiers:

1. **Common passwords** — Embedded list of frequently-used passwords
2. **Mutated seed words** — Profile words with case mutations, leet speak, reversals, and doubling
3. **Affixed seeds** — Seed words combined with numeric suffixes, symbol suffixes, common prefixes, and profile-specific numbers
4. **Word combinations** — Pairs of seed words joined, underscored, dotted, and reversed
5. **Keyboard patterns** — Common keyboard walks and number runs
6. **Deep mutations** (depth 3 only) — Mutations applied to combinations, plus mutated seeds with all numeric suffixes

Each tier deduplicates candidates and filters by length constraints.

## Project structure

```
src/
├── main.rs          # CLI entry point and subcommands
├── profile.rs       # TOML profile loading and seed extraction
├── generator.rs     # Tiered candidate generation engine
├── mutations.rs     # Mutation and mangling rules
├── common.rs        # Embedded common passwords, patterns, and affixes
├── wordlist.rs      # Wordlist file I/O
└── cracker/
    ├── mod.rs       # Hash algorithm types and crack result
    ├── hash.rs      # Parallel hash cracking (MD5/SHA/bcrypt)
    └── wifi.rs      # WiFi cracking via aircrack-ng/hashcat
```

## License

For educational and authorized security testing use only.
