use anyhow::{Context, anyhow};
use data_encoding::Specification;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

pub const BYTES_RV: [u8; 16] = *b"RV\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
pub const NAMESPACE_RV: Uuid = Uuid::from_bytes(BYTES_RV);
pub const SPEC_BASE32: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

// A Remote Viewing UID is like a UUID, but easy to write down, and based on UUIDv5.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rvuid {
    pub uuid: Uuid,
    pub rvuid: String,
}

impl Rvuid {
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("failed to read bytes from {}", path.display()))?;
        let uuid = uuid_from_bytes(&bytes);
        let rvuid = Self::rvuid_from_uuid(uuid);
        Ok(Self { uuid, rvuid })
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        let uuid = uuid_from_bytes(data);
        let rvuid = Self::rvuid_from_uuid(uuid);
        Self { uuid, rvuid }
    }
    pub fn rvuid_from_uuid(uuid: Uuid) -> String {
        // Crockford-like Base32 alphabet (no I, L, O, U since they're a big ambiguous")
        let mut spec = Specification::new();
        spec.symbols.push_str(SPEC_BASE32);
        let crockford = spec.encoding().unwrap();
        let encoded = crockford.encode(uuid.as_bytes());
        let part1 = &encoded[..4];
        let part2 = &encoded[4..8];
        let part3 = &encoded[8..];
        format!("R-{}-{}-{}", part1, part2, part3)
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

        if bytes.len() != 16 {
            return Err(anyhow!("expected 16 bytes for UUID, got {}", bytes.len()));
        }

        let uuid = Uuid::from_slice(&bytes)?;

        Ok(Rvuid {
            uuid,
            rvuid: s.to_string(),
        })
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
}
