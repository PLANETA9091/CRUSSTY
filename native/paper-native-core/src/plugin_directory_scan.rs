use std::fs;
use std::io;
use std::path::Path;

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const DIRECTORY_SCAN_TAG: u64 = 0xC4D8_A9E2_4B36_7195;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PluginDirectoryScanSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_walk_depth1_summary(
    iterations: usize,
    directory: &Path,
) -> io::Result<PluginDirectoryScanSummary> {
    run_summary(iterations, directory, Mode::OldWalkDepth1)
}

pub fn new_list_summary(
    iterations: usize,
    directory: &Path,
) -> io::Result<PluginDirectoryScanSummary> {
    run_summary(iterations, directory, Mode::NewList)
}

pub fn directory_stream_summary(
    iterations: usize,
    directory: &Path,
) -> io::Result<PluginDirectoryScanSummary> {
    run_summary(iterations, directory, Mode::DirectoryStream)
}

#[derive(Clone, Copy)]
enum Mode {
    OldWalkDepth1,
    NewList,
    DirectoryStream,
}

fn run_summary(
    iterations: usize,
    directory: &Path,
    mode: Mode,
) -> io::Result<PluginDirectoryScanSummary> {
    if iterations == 0 {
        return Ok(PluginDirectoryScanSummary::default());
    }

    let path_digest = mix64(
        DIRECTORY_SCAN_TAG
            ^ (java_string_hash(&directory.to_string_lossy()) as i64 as u64)
            ^ ((iterations as u64) << 11),
    );
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let value = match mode {
            Mode::OldWalkDepth1 => scan_walk_depth1(directory)?,
            Mode::NewList => scan_list(directory)?,
            Mode::DirectoryStream => scan_directory_stream(directory)?,
        } as u64;

        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ path_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)),
        );
    }

    Ok(PluginDirectoryScanSummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    })
}

fn scan_walk_depth1(directory: &Path) -> io::Result<usize> {
    let mut files = Vec::new();
    if is_valid_file(directory)? {
        files.push(directory.to_path_buf());
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if is_valid_file(&path)? {
            files.push(path);
        }
    }
    Ok(files.len())
}

fn scan_list(directory: &Path) -> io::Result<usize> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if is_valid_file(&path)? {
            files.push(path);
        }
    }
    Ok(files.len())
}

fn scan_directory_stream(directory: &Path) -> io::Result<usize> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if is_valid_file(&path)? {
            files.push(path);
        }
    }
    Ok(files.len())
}

fn is_valid_file(path: &Path) -> io::Result<bool> {
    Ok(fs::metadata(path)?.is_file() && !path.starts_with("."))
}

fn java_string_hash(value: &str) -> i32 {
    let mut hash = 0i32;
    for unit in value.encode_utf16() {
        hash = hash.wrapping_mul(31).wrapping_add(i32::from(unit));
    }
    hash
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe1a85_ec53);
    value ^ (value >> 33)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scan_modes_match_on_small_directory() {
        let root = std::env::temp_dir().join(format!(
            "paper-native-plugin-scan-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create temp directory");
        fs::write(root.join("A.jar"), b"a").expect("write file");
        fs::write(root.join("B.jar"), b"b").expect("write file");
        fs::create_dir(root.join("nested")).expect("create nested directory");

        let walk = old_walk_depth1_summary(16, &root).expect("walk summary");
        let list = new_list_summary(16, &root).expect("list summary");
        let stream = directory_stream_summary(16, &root).expect("stream summary");

        assert_eq!(walk, list);
        assert_eq!(list, stream);
        assert_eq!(walk.last_total, 2);

        fs::remove_dir_all(root).expect("remove temp directory");
    }
}
