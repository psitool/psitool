use uuid::Uuid;

pub const BYTES_RV: [u8; 16] = *b"RV\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
pub const NAMESPACE_RV: Uuid = Uuid::from_bytes(BYTES_RV);


/// Hash arbitrary length data with SHA1, then generate a UUID using UUIDv5
pub fn rv_uuid_from_bytes(data: &[u8]) -> Uuid {
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
        assert_eq!(rv_uuid_from_bytes(&empty), Uuid::new_v5(&NAMESPACE_RV, &empty2));
        assert_eq!(rv_uuid_from_bytes(&oneval), Uuid::new_v5(&NAMESPACE_RV, &oneval2));
    }
}
