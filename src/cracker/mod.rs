pub mod hash;
pub mod wifi;

use std::fmt;

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Bcrypt,
}

impl HashAlgorithm {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "md5" => Some(Self::Md5),
            "sha1" => Some(Self::Sha1),
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            "bcrypt" => Some(Self::Bcrypt),
            _ => None,
        }
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Md5 => write!(f, "MD5"),
            Self::Sha1 => write!(f, "SHA1"),
            Self::Sha256 => write!(f, "SHA256"),
            Self::Sha512 => write!(f, "SHA512"),
            Self::Bcrypt => write!(f, "bcrypt"),
        }
    }
}

/// Result of cracking a single hash.
#[derive(Debug)]
pub struct CrackResult {
    pub hash: String,
    pub plaintext: String,
    pub algorithm: HashAlgorithm,
}

impl fmt::Display for CrackResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {} ({})", self.hash, self.plaintext, self.algorithm)
    }
}
