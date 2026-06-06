use sha2::{Digest, Sha256};

pub const SHA256_DIGEST_LEN: usize = 32;

#[inline]
pub fn sha256_digest(data: &[u8]) -> [u8; SHA256_DIGEST_LEN] {
    let digest = Sha256::digest(data);
    let mut out = [0u8; SHA256_DIGEST_LEN];
    out.copy_from_slice(&digest);
    out
}

#[inline]
pub fn sha256_digest_into(data: &[u8], dst: &mut [u8]) -> Option<usize> {
    if dst.len() < SHA256_DIGEST_LEN {
        return None;
    }
    let digest = sha256_digest(data);
    dst[..SHA256_DIGEST_LEN].copy_from_slice(&digest);
    Some(SHA256_DIGEST_LEN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_lower(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0F) as usize] as char);
        }
        out
    }

    #[test]
    fn matches_known_vectors() {
        assert_eq!(
            hex_lower(&sha256_digest(b"")),
            "e3b0c44298fc1c149afbf4c8996fb924\
             27ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex_lower(&sha256_digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223\
             b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn writes_digest_into_existing_buffer() {
        let mut out = [0x55u8; SHA256_DIGEST_LEN + 4];
        assert_eq!(sha256_digest_into(b"abc", &mut out), Some(SHA256_DIGEST_LEN));
        assert_eq!(&out[..SHA256_DIGEST_LEN], &sha256_digest(b"abc"));
        assert_eq!(&out[SHA256_DIGEST_LEN..], &[0x55; 4]);
        assert_eq!(sha256_digest_into(b"abc", &mut [0u8; SHA256_DIGEST_LEN - 1]), None);
    }
}
