use frame::prelude::*;

#[derive(
    Encode,
    Decode,
    TypeInfo,
    Clone,
    PartialEq,
    DecodeWithMemTracking,
    Default,
    Eq,
    MaxEncodedLen,
    Debug,
)]
pub enum BaseRight {
    #[default]
    Update,
    Impersonate,
    Dispute,
}

#[derive(
    Encode,
    Decode,
    Clone,
    Eq,
    PartialEq,
    Default,
    TypeInfo,
    MaxEncodedLen,
    Debug,
    DecodeWithMemTracking,
)]
pub struct ContentId {
    prefix: [u8; 4],
    hash: [u8; 32],
}

impl ContentId {
    pub fn new(prefix: &[u8], hash: &[u8]) -> Self {
        let mut content_id = ContentId::default();
        content_id.prefix.copy_from_slice(prefix);
        content_id.hash.copy_from_slice(hash);
        content_id
    }
}

#[cfg(feature = "std")]
impl serde::Serialize for ContentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // "cid:<hex>"
        let s = format!("cid:{}", hex::encode(&self.hash));
        serializer.serialize_str(&s)
    }
}

#[cfg(feature = "std")]
impl<'de> serde::Deserialize<'de> for ContentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // expect "cid:<hex>"
        let pref = s
            .get(0..4)
            .ok_or_else(|| serde::de::Error::custom("invalid cid"))?;
        if pref != "cid:" {
            return Err(serde::de::Error::custom("missing cid: prefix"));
        }
        let hex_part = &s[4..];
        let bytes = hex::decode(hex_part).map_err(|e| serde::de::Error::custom(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("invalid hash length"));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(ContentId {
            prefix: *b"cid:",
            hash,
        })
    }
}
