use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

pub const SUMMARY_FIELDS: usize = 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HashPathSummary {
    pub inputs: u64,
    pub bytes: u64,
    pub digest_checksum: u64,
    pub last_digest_head: u64,
}

pub fn read_all_summary(paths: &[String]) -> io::Result<HashPathSummary> {
    let mut summary = HashPathSummary::default();
    for path in paths {
        let data = std::fs::read(Path::new(path))?;
        let digest = Sha256::digest(&data);
        summary.inputs += 1;
        summary.bytes = summary.bytes.wrapping_add(data.len() as u64);
        summary.digest_checksum = mix_digest(summary.digest_checksum, &digest);
        summary.last_digest_head = digest_head(&digest);
    }
    Ok(summary)
}

pub fn streaming_summary(paths: &[String], buffer_size: usize) -> io::Result<HashPathSummary> {
    let mut summary = HashPathSummary::default();
    let mut buffer = vec![0u8; buffer_size.max(1)];

    for path in paths {
        let mut file = File::open(Path::new(path))?;
        let mut hasher = Sha256::new();
        let mut file_bytes = 0u64;
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            file_bytes = file_bytes.wrapping_add(read as u64);
        }
        let digest = hasher.finalize();
        summary.inputs += 1;
        summary.bytes = summary.bytes.wrapping_add(file_bytes);
        summary.digest_checksum = mix_digest(summary.digest_checksum, &digest);
        summary.last_digest_head = digest_head(&digest);
    }

    Ok(summary)
}

#[inline]
fn mix_digest(mut checksum: u64, digest: &[u8]) -> u64 {
    for chunk in digest.chunks(8) {
        let mut value = 0u64;
        for &byte in chunk {
            value = (value << 8) | byte as u64;
        }
        checksum = mix64(checksum ^ value);
    }
    checksum
}

#[inline]
fn digest_head(digest: &[u8]) -> u64 {
    let mut value = 0u64;
    for &byte in &digest[..8] {
        value = (value << 8) | byte as u64;
    }
    value
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^ (value >> 33)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_are_stable() {
        assert_eq!(read_all_summary(&[]).unwrap(), HashPathSummary::default());
        assert_eq!(
            streaming_summary(&[], 64 * 1024).unwrap(),
            HashPathSummary::default()
        );
    }
}
