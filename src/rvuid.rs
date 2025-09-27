use data_encoding::Specification;
use std::fmt;
use uuid::Uuid;

pub const BYTES_RV: [u8; 16] = *b"RV\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
pub const NAMESPACE_RV: Uuid = Uuid::from_bytes(BYTES_RV);

// A Remote Viewing UID is like a UUID, but easy to write down, and based on UUIDv5.
pub struct Rvuid {
    pub uuid: Uuid,
    pub rvuid: String,
}

impl Rvuid {
    pub fn from_bytes(data: &[u8]) -> Self {
        let uuid = uuid_from_bytes(data);
        let rvuid = Self::rvuid_from_uuid(uuid);
        Self { uuid, rvuid }
    }
    pub fn rvuid_from_uuid(uuid: Uuid) -> String {
        // Crockford-like Base32 alphabet (no I, L, O, U since they're a big ambiguous")
        let mut spec = Specification::new();
        spec.symbols.push_str("0123456789ABCDEFGHJKMNPQRSTVWXYZ");
        let crockford = spec.encoding().unwrap();
        let encoded = crockford.encode(uuid.as_bytes());
        let part1 = &encoded[..4];
        let part2 = &encoded[4..8];
        format!("R-{}-{}", part1, part2)
    }
}

impl fmt::Display for Rvuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rvuid)
    }
}

fn uuid_from_bytes(data: &[u8]) -> Uuid {
    Uuid::new_v5(&NAMESPACE_RV, data)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
