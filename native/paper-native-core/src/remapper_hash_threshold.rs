use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::hash;

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemapperHashThresholdError {
    InvalidIterations,
    EmptyInputs,
    Io,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RemapperHashThresholdSummary {
    pub count: u64,
    pub total_entries: u64,
    pub checksum: u64,
    pub last_digest: u64,
}

pub fn compute_if_absent_summary(
    paths: &[String],
    iterations: usize,
) -> Result<RemapperHashThresholdSummary, RemapperHashThresholdError> {
    run_summary(paths, iterations, Mode::ComputeIfAbsent)
}

pub fn put_summary(
    paths: &[String],
    iterations: usize,
) -> Result<RemapperHashThresholdSummary, RemapperHashThresholdError> {
    run_summary(paths, iterations, Mode::Put)
}

pub fn hybrid_summary(
    paths: &[String],
    iterations: usize,
) -> Result<RemapperHashThresholdSummary, RemapperHashThresholdError> {
    run_summary(paths, iterations, Mode::Hybrid)
}

pub fn parallel_summary(
    paths: &[String],
    iterations: usize,
) -> Result<RemapperHashThresholdSummary, RemapperHashThresholdError> {
    run_summary(paths, iterations, Mode::Parallel)
}

#[derive(Clone, Copy)]
enum Mode {
    ComputeIfAbsent,
    Put,
    Hybrid,
    Parallel,
}

impl Mode {
    fn tag(self) -> u64 {
        match self {
            Self::ComputeIfAbsent => 0xC011_1F_AB5E_17,
            Self::Put => 0x9015_EA5E,
            Self::Hybrid => 0xA11C_E51D,
            Self::Parallel => 0x9A8A_11E1,
        }
    }
}

fn run_summary(
    paths: &[String],
    iterations: usize,
    mode: Mode,
) -> Result<RemapperHashThresholdSummary, RemapperHashThresholdError> {
    if iterations == 0 {
        return Err(RemapperHashThresholdError::InvalidIterations);
    }
    if paths.is_empty() {
        return Err(RemapperHashThresholdError::EmptyInputs);
    }

    let mut total_entries = 0u64;
    let mut checksum = 0u64;
    let mut last_digest = 0u64;
    for iteration in 0..iterations {
        let map = build_hashes(paths, mode)?;
        let digest = digest_map(paths, &map);
        total_entries += map.len() as u64;
        last_digest = digest;
        checksum = mix64(
            checksum
                ^ digest
                ^ mode.tag()
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)),
        );
    }

    Ok(RemapperHashThresholdSummary {
        count: iterations as u64,
        total_entries,
        checksum,
        last_digest,
    })
}

fn build_hashes(
    paths: &[String],
    mode: Mode,
) -> Result<HashMap<&str, [u8; hash::SHA256_DIGEST_LEN]>, RemapperHashThresholdError> {
    let mut hashes = HashMap::with_capacity(paths.len() * 2);
    match mode {
        Mode::ComputeIfAbsent => {
            for path in paths {
                if !hashes.contains_key(path.as_str()) {
                    let digest = sha256_file(path)?;
                    hashes.insert(path.as_str(), digest);
                }
            }
        }
        Mode::Put => {
            for path in paths {
                let digest = sha256_file(path)?;
                hashes.insert(path.as_str(), digest);
            }
        }
        Mode::Hybrid => {
            if paths.len() == 1 {
                let path = &paths[0];
                let digest = sha256_file(path)?;
                hashes.insert(path.as_str(), digest);
            } else {
                for path in paths {
                    let digest = sha256_file(path)?;
                    hashes.insert(path.as_str(), digest);
                }
            }
        }
        Mode::Parallel => {
            std::thread::scope(|scope| {
                let mut handles = Vec::with_capacity(paths.len());
                for path in paths {
                    handles.push((path.as_str(), scope.spawn(move || sha256_file(path))));
                }
                for (path, handle) in handles {
                    let digest = handle
                        .join()
                        .map_err(|_| RemapperHashThresholdError::Io)??;
                    hashes.insert(path, digest);
                }
                Ok::<(), RemapperHashThresholdError>(())
            })?;
        }
    }
    Ok(hashes)
}

fn sha256_file(path: &str) -> Result<[u8; hash::SHA256_DIGEST_LEN], RemapperHashThresholdError> {
    let bytes = fs::read(Path::new(path)).map_err(|_| RemapperHashThresholdError::Io)?;
    Ok(hash::sha256_digest(&bytes))
}

fn digest_map(paths: &[String], map: &HashMap<&str, [u8; hash::SHA256_DIGEST_LEN]>) -> u64 {
    let mut digest = 0u64;
    for path in paths {
        digest = digest_string(digest, path);
        if let Some(hash) = map.get(path.as_str()) {
            digest = digest_hash_hex_upper(digest, hash);
        }
    }
    digest ^ (map.len() as u64)
}

fn digest_string(mut digest: u64, value: &str) -> u64 {
    for &byte in value.as_bytes() {
        digest = mix64(digest ^ byte as u64);
    }
    digest
}

fn digest_hash_hex_upper(mut digest: u64, value: &[u8; hash::SHA256_DIGEST_LEN]) -> u64 {
    for &byte in value {
        digest = mix64(digest ^ hex_upper((byte >> 4) & 0x0F) as u64);
        digest = mix64(digest ^ hex_upper(byte & 0x0F) as u64);
    }
    digest
}

#[inline]
fn hex_upper(value: u8) -> u8 {
    match value {
        0..=9 => b'0' + value,
        _ => b'A' + (value - 10),
    }
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
    fn rejects_bad_inputs() {
        assert_eq!(
            compute_if_absent_summary(&[], 1),
            Err(RemapperHashThresholdError::EmptyInputs)
        );
        assert_eq!(
            compute_if_absent_summary(&[String::from("/missing")], 0),
            Err(RemapperHashThresholdError::InvalidIterations)
        );
    }
}
