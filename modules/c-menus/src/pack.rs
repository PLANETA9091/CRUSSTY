//! Embedded resource pack (committed artifact) + SHA-1 for the client hash.

use sha1_smol::Sha1;

/// The built pack zip (see scripts/build_pack.py; committed and CI-verified).
pub const PACK_ZIP: &[u8] = include_bytes!("../pack/cmenus-pack.zip");

pub fn sha1_hex(data: &[u8]) -> String {
    let mut h = Sha1::new();
    h.update(data);
    h.digest().to_string()
}

pub fn pack_sha1_hex() -> String {
    sha1_hex(PACK_ZIP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_zip_magic() {
        assert_eq!(&PACK_ZIP[0..2], b"PK", "pack zip must be a real zip");
        assert!(PACK_ZIP.len() > 512, "pack zip looks too small");
    }

    #[test]
    fn sha1_known() {
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    }
}
