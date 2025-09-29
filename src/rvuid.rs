use anyhow::{Context, anyhow};
use data_encoding::Specification;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

pub const BYTES_RV: [u8; 16] = *b"RV\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
pub const NAMESPACE_RV: Uuid = Uuid::from_bytes(BYTES_RV);
pub const SPEC_BASE32: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

// A Remote Viewing UID is like a UUID, but easy to write down, and based on UUIDv5.
#[derive(Clone, Debug)]
pub struct Rvuid {
    pub uuid: Uuid,
    pub rvuid: String,
    pub missing_bits: bool,
    pub prefix40: u64,
}

impl PartialEq for Rvuid {
    /// If one or both is a shortened prefix, just compare the first 40 bits.
    /// Otherwise, compare the exact uuid.
    fn eq(&self, other: &Self) -> bool {
        if self.missing_bits || other.missing_bits {
            self.prefix40 == other.prefix40
        } else {
            self.uuid == other.uuid
        }
    }
}

// Marking Eq here is problematic because we don't have transitivity when it comes to RVUID now that I'm
// adding the 40-bit prefix support.
// Here's the problem. A remote viewer might only want to write the first 40 bits, like so:
// R-1234-5678 , which would be ignoring the final 88 bits after R-1234-5678-...
// We want to support the user comparing R-1234-5678 against the full 128-bit version, even if they
// don't provide those bits. So if `x` is user provided as R-1234-5678 and `y` is the actual RVUID
// of R-1234-5678-0123456789123456 , then PartialEq will say they're equal, and if we have `z`
// which is a phenomenally statistically lucky RVUID of R-1234-5678-AB1C123D00E92842 and just
// _happens_ to have the same 40-bit prefix... then `x` == `y` and `x` == `z` but `y` != `z`
// Transitivity fails, and thus Eq should not be marked. Everything with Eq should support full
// equality, being reflexive, symmetric, and transitive.

impl Eq for Rvuid {}

impl Hash for Rvuid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Rust has a contract with PartialEq that if a == b, then hash(a) == hash(b)
        // We have to _only_ hash the prefix due to this.
        // Since we're not getting
        if self.missing_bits {
            panic!("You can't hash RVUIDs that are only the 40-bit prefix!");
        }
        self.uuid.hash(state);
    }
}

impl Rvuid {
    pub fn new(uuid: Uuid, missing_bits: bool) -> Self {
        let rvuid = Self::rvuid_from_uuid(uuid, missing_bits);
        let prefix40 = Self::prefix40_from_uuid(uuid);
        Self {
            uuid,
            rvuid,
            missing_bits,
            prefix40,
        }
    }
    pub fn prefix40_from_uuid(uuid: Uuid) -> u64 {
        let bytes = uuid.as_bytes();
        ((bytes[0] as u64) << 32)
            | ((bytes[1] as u64) << 24)
            | ((bytes[2] as u64) << 16)
            | ((bytes[3] as u64) << 8)
            | (bytes[4] as u64)
    }
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("failed to read bytes from {}", path.display()))?;
        let uuid = uuid_from_bytes(&bytes);
        Ok(Self::new(uuid, false))
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        let uuid = uuid_from_bytes(data);
        Self::new(uuid, false)
    }
    pub fn rvuid_from_uuid(uuid: Uuid, missing_bits: bool) -> String {
        // Crockford-like Base32 alphabet (no I, L, O, U since they're a big ambiguous")
        let mut spec = Specification::new();
        spec.symbols.push_str(SPEC_BASE32);
        let crockford = spec.encoding().unwrap();
        let encoded = crockford.encode(uuid.as_bytes());
        let part1 = &encoded[..4];
        let part2 = &encoded[4..8];
        if missing_bits {
            format!("R-{}-{}", part1, part2)
        } else {
            let part3 = &encoded[8..];
            format!("R-{}-{}-{}", part1, part2, part3)
        }
    }
}

impl fmt::Display for Rvuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rvuid)
    }
}

impl FromStr for Rvuid {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("R-") {
            anyhow::bail!("invalid RVUID: must start with 'R-' prefix.");
        }

        let raw = s.trim_start_matches("R-").replace('-', "");

        let mut spec = Specification::new();
        spec.symbols.push_str(SPEC_BASE32);
        let decoder = spec.encoding().unwrap();

        let bytes = decoder
            .decode(raw.as_bytes())
            .map_err(|e| anyhow!("base32 decode error: {}", e))?;

        match bytes.len() {
            16 => {
                let uuid = Uuid::from_slice(&bytes)?;
                Ok(Rvuid::new(uuid, false))
            }
            5 => {
                let mut full_bytes = [0u8; 16];
                full_bytes[..5].copy_from_slice(&bytes[..5]);
                let uuid = Uuid::from_slice(&full_bytes)?;
                Ok(Rvuid::new(uuid, true))
            }
            x => Err(anyhow!("expected 16 or 5 bytes for RVUID, got {}", x)),
        }
    }
}

impl<'de> Deserialize<'de> for Rvuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // parse from string
        Rvuid::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for Rvuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.rvuid)
    }
}

fn uuid_from_bytes(data: &[u8]) -> Uuid {
    Uuid::new_v5(&NAMESPACE_RV, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_uuid_from_bytes() {
        let empty: [u8; 0] = [];
        let empty2: [u8; 0] = [];
        let oneval: [u8; 1] = [99];
        let oneval2: [u8; 1] = [99];
        assert_eq!(
            uuid_from_bytes(&empty),
            Uuid::new_v5(&NAMESPACE_RV, &empty2)
        );
        assert_eq!(
            uuid_from_bytes(&oneval),
            Uuid::new_v5(&NAMESPACE_RV, &oneval2)
        );
    }

    #[derive(Debug, Deserialize)]
    struct Foo {
        id: Rvuid,
    }

    #[test]
    fn test_rvuid_deserialize_from_yaml() {
        let yaml = r#"
id: "R-2DTH-GZW5-W9FMX29F6HJ52Q8N9C"
"#;

        let foo: Foo = serde_yaml::from_str(yaml).expect("failed to deserialize");
        assert_eq!(foo.id.to_string(), "R-2DTH-GZW5-W9FMX29F6HJ52Q8N9C");
    }

    #[test]
    fn test_rvuid_partial_eq() {
        let r1 = Rvuid::from_str("R-2DTH-GZW5-W9FMX29F6HJ52Q8N9C").unwrap();
        let r2 = Rvuid::from_str("R-2DTH-GZW5-W9FMX29F6HJ52Q8N9C").unwrap();
        let r3 = Rvuid::from_str("R-2DTH-GZW5").unwrap();
        let r_close_but_not = Rvuid::from_str("R-2DTH-GZW0").unwrap();
        let r5 = Rvuid::from_str("R-2DTH-GZW5-123400000000000000").unwrap();
        let r6 = Rvuid::from_str("R-2DTH-GZW5-567800000000000000").unwrap();
        // These are equal.
        assert_eq!(r1, r2);
        assert_eq!(r1, r3);
        assert_eq!(r2, r3);
        assert_eq!(r3, r5);
        assert_eq!(r3, r6);
        // These are NOT equal.
        assert_ne!(r1, r_close_but_not);
        assert_ne!(r2, r_close_but_not);
        assert_ne!(r3, r_close_but_not);
        assert_ne!(r1, r5);
        assert_ne!(r2, r5);
        assert_ne!(r5, r6);
    }
}
