use std::collections::HashMap;

pub const SUMMARY_FIELDS: usize = 11;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NbtCompoundMapCapacitySummary {
    pub chunks: u64,
    pub compounds: u64,
    pub entries: u64,
    pub max_entries: u64,
    pub bucket0: u64,
    pub bucket1_to_2: u64,
    pub bucket3_to_4: u64,
    pub bucket5_to_6: u64,
    pub bucket7_to_13: u64,
    pub bucket14_plus: u64,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NbtCompoundMapCapacityError {
    LengthMismatch,
    InvalidRange,
    UnexpectedEof,
    InvalidData,
}

pub fn parse_capacity_summary(
    data: &[u8],
    offsets: &[i32],
    lengths: &[i32],
    capacity: usize,
) -> Result<NbtCompoundMapCapacitySummary, NbtCompoundMapCapacityError> {
    if offsets.len() != lengths.len() {
        return Err(NbtCompoundMapCapacityError::LengthMismatch);
    }

    let mut stats = Stats::default();
    let mut checksum = 0x9E37_79B9_7F4A_7C15u64;

    for index in 0..offsets.len() {
        let offset = usize::try_from(offsets[index]).map_err(|_| NbtCompoundMapCapacityError::InvalidRange)?;
        let length = usize::try_from(lengths[index]).map_err(|_| NbtCompoundMapCapacityError::InvalidRange)?;
        let end = offset
            .checked_add(length)
            .ok_or(NbtCompoundMapCapacityError::InvalidRange)?;
        if end > data.len() {
            return Err(NbtCompoundMapCapacityError::InvalidRange);
        }
        let chunk = &data[offset..end];
        let mut parser = Parser::new(chunk, capacity, Some(&mut stats));
        checksum = mix(checksum, parser.parse_root()?);
    }

    Ok(NbtCompoundMapCapacitySummary {
        chunks: offsets.len() as u64,
        compounds: stats.compounds,
        entries: stats.entries,
        max_entries: stats.max_entries as u64,
        bucket0: stats.bucket0,
        bucket1_to_2: stats.bucket1_to_2,
        bucket3_to_4: stats.bucket3_to_4,
        bucket5_to_6: stats.bucket5_to_6,
        bucket7_to_13: stats.bucket7_to_13,
        bucket14_plus: stats.bucket14_plus,
        checksum,
    })
}

#[derive(Default)]
struct Stats {
    compounds: u64,
    entries: u64,
    max_entries: usize,
    bucket0: u64,
    bucket1_to_2: u64,
    bucket3_to_4: u64,
    bucket5_to_6: u64,
    bucket7_to_13: u64,
    bucket14_plus: u64,
}

impl Stats {
    fn record(&mut self, entry_count: usize) {
        self.compounds += 1;
        self.entries += entry_count as u64;
        self.max_entries = self.max_entries.max(entry_count);
        if entry_count == 0 {
            self.bucket0 += 1;
        } else if entry_count <= 2 {
            self.bucket1_to_2 += 1;
        } else if entry_count <= 4 {
            self.bucket3_to_4 += 1;
        } else if entry_count <= 6 {
            self.bucket5_to_6 += 1;
        } else if entry_count <= 13 {
            self.bucket7_to_13 += 1;
        } else {
            self.bucket14_plus += 1;
        }
    }
}

struct Parser<'a> {
    data: &'a [u8],
    offset: usize,
    compound_capacity: usize,
    stats: Option<&'a mut Stats>,
}

impl<'a> Parser<'a> {
    fn new(data: &'a [u8], compound_capacity: usize, stats: Option<&'a mut Stats>) -> Self {
        Self {
            data,
            offset: 0,
            compound_capacity,
            stats,
        }
    }

    fn parse_root(&mut self) -> Result<u64, NbtCompoundMapCapacityError> {
        let tag_type = self.read_u8()?;
        if tag_type == 0 {
            return Ok(0);
        }
        let _ = self.read_utf_hash()?;
        self.parse_payload(tag_type)
    }

    fn parse_payload(&mut self, tag_type: u8) -> Result<u64, NbtCompoundMapCapacityError> {
        match tag_type {
            0 => Ok(0),
            1 => Ok(as_signed_u64(self.read_i8()? as i64)),
            2 => Ok(as_signed_u64(self.read_i16()? as i64)),
            3 => Ok(as_signed_u64(self.read_i32()? as i64)),
            4 => Ok(self.read_i64()? as u64),
            5 => Ok(as_signed_u64(self.read_i32()? as i64)),
            6 => Ok(self.read_i64()? as u64),
            7 => self.skip_array(1),
            8 => Ok(as_signed_u64(self.read_utf_hash()? as i64)),
            9 => self.parse_list(),
            10 => self.parse_compound(),
            11 => self.skip_array(4),
            12 => self.skip_array(8),
            _ => Err(NbtCompoundMapCapacityError::InvalidData),
        }
    }

    fn parse_list(&mut self) -> Result<u64, NbtCompoundMapCapacityError> {
        let element_type = self.read_u8()?;
        let length = self.read_i32()?;
        if length < 0 {
            return Err(NbtCompoundMapCapacityError::InvalidData);
        }
        let mut checksum = mix(0xD1B5_4A32_D192_ED03, element_type as u64);
        checksum = mix(checksum, as_signed_u64(length as i64));
        for _ in 0..length {
            checksum = mix(checksum, self.parse_payload(element_type)?);
        }
        Ok(checksum)
    }

    fn parse_compound(&mut self) -> Result<u64, NbtCompoundMapCapacityError> {
        let mut names = HashMap::with_capacity(self.compound_capacity);
        let mut checksum = 0x94D0_49BB_1331_11EBu64;
        let mut entries = 0usize;

        loop {
            let tag_type = self.read_u8()?;
            if tag_type == 0 {
                break;
            }
            let name_hash = self.read_utf_hash()?;
            let child = self.parse_payload(tag_type)?;
            names.insert(name_hash, ());
            checksum = mix(checksum, tag_type as u64);
            checksum = mix(checksum, as_signed_u64(name_hash as i64));
            checksum = mix(checksum, child);
            entries += 1;
        }

        if let Some(stats) = &mut self.stats {
            stats.record(entries);
        }
        Ok(mix(checksum, entries as u64))
    }

    fn skip_array(&mut self, element_bytes: usize) -> Result<u64, NbtCompoundMapCapacityError> {
        let length = self.read_i32()?;
        if length < 0 {
            return Err(NbtCompoundMapCapacityError::InvalidData);
        }
        let bytes = (length as usize)
            .checked_mul(element_bytes)
            .ok_or(NbtCompoundMapCapacityError::InvalidData)?;
        self.skip_fully(bytes)?;
        Ok(mix(as_signed_u64(length as i64), bytes as u64))
    }

    fn skip_fully(&mut self, bytes: usize) -> Result<(), NbtCompoundMapCapacityError> {
        let end = self
            .offset
            .checked_add(bytes)
            .ok_or(NbtCompoundMapCapacityError::UnexpectedEof)?;
        if end > self.data.len() {
            return Err(NbtCompoundMapCapacityError::UnexpectedEof);
        }
        self.offset = end;
        Ok(())
    }

    fn read_utf_hash(&mut self) -> Result<i32, NbtCompoundMapCapacityError> {
        let len = self.read_u16()? as usize;
        let end = self
            .offset
            .checked_add(len)
            .ok_or(NbtCompoundMapCapacityError::UnexpectedEof)?;
        if end > self.data.len() {
            return Err(NbtCompoundMapCapacityError::UnexpectedEof);
        }

        let mut hash = 0i32;
        while self.offset < end {
            let byte = self.data[self.offset];
            self.offset += 1;
            let code_unit = if byte & 0x80 == 0 {
                byte as u16
            } else if byte & 0xE0 == 0xC0 {
                if self.offset >= end {
                    return Err(NbtCompoundMapCapacityError::UnexpectedEof);
                }
                let b2 = self.data[self.offset];
                self.offset += 1;
                if b2 & 0xC0 != 0x80 {
                    return Err(NbtCompoundMapCapacityError::InvalidData);
                }
                (((byte & 0x1F) as u16) << 6) | ((b2 & 0x3F) as u16)
            } else if byte & 0xF0 == 0xE0 {
                if self.offset + 1 >= end {
                    return Err(NbtCompoundMapCapacityError::UnexpectedEof);
                }
                let b2 = self.data[self.offset];
                let b3 = self.data[self.offset + 1];
                self.offset += 2;
                if b2 & 0xC0 != 0x80 || b3 & 0xC0 != 0x80 {
                    return Err(NbtCompoundMapCapacityError::InvalidData);
                }
                (((byte & 0x0F) as u16) << 12)
                    | (((b2 & 0x3F) as u16) << 6)
                    | ((b3 & 0x3F) as u16)
            } else {
                return Err(NbtCompoundMapCapacityError::InvalidData);
            };
            hash = hash.wrapping_mul(31).wrapping_add(code_unit as i32);
        }

        if self.offset != end {
            return Err(NbtCompoundMapCapacityError::InvalidData);
        }
        Ok(hash)
    }

    fn read_u8(&mut self) -> Result<u8, NbtCompoundMapCapacityError> {
        if self.offset >= self.data.len() {
            return Err(NbtCompoundMapCapacityError::UnexpectedEof);
        }
        let value = self.data[self.offset];
        self.offset += 1;
        Ok(value)
    }

    fn read_i8(&mut self) -> Result<i8, NbtCompoundMapCapacityError> {
        Ok(self.read_u8()? as i8)
    }

    fn read_u16(&mut self) -> Result<u16, NbtCompoundMapCapacityError> {
        let bytes = self.read_exact::<2>()?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn read_i16(&mut self) -> Result<i16, NbtCompoundMapCapacityError> {
        let bytes = self.read_exact::<2>()?;
        Ok(i16::from_be_bytes(bytes))
    }

    fn read_i32(&mut self) -> Result<i32, NbtCompoundMapCapacityError> {
        let bytes = self.read_exact::<4>()?;
        Ok(i32::from_be_bytes(bytes))
    }

    fn read_i64(&mut self) -> Result<i64, NbtCompoundMapCapacityError> {
        let bytes = self.read_exact::<8>()?;
        Ok(i64::from_be_bytes(bytes))
    }

    fn read_exact<const N: usize>(&mut self) -> Result<[u8; N], NbtCompoundMapCapacityError> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or(NbtCompoundMapCapacityError::UnexpectedEof)?;
        if end > self.data.len() {
            return Err(NbtCompoundMapCapacityError::UnexpectedEof);
        }
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(&self.data[self.offset..end]);
        self.offset = end;
        Ok(bytes)
    }
}

#[inline]
fn as_signed_u64(value: i64) -> u64 {
    value as u64
}

#[inline]
fn mix(current: u64, value: u64) -> u64 {
    let mixed = current
        ^ value
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(current << 6)
            .wrapping_add(current >> 2);
    if mixed == 0 { 1 } else { mixed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_small_compound() {
        let data = [
            10, 0, 0, // root compound, empty name
            1, 0, 1, b'a', 7, // byte a=7
            8, 0, 1, b's', 0, 2, b'o', b'k', // string s="ok"
            0, // end
        ];
        let summary = parse_capacity_summary(&data, &[0], &[data.len() as i32], 4).unwrap();
        assert_eq!(summary.chunks, 1);
        assert_eq!(summary.compounds, 1);
        assert_eq!(summary.entries, 2);
        assert_eq!(summary.bucket1_to_2, 1);
        assert_ne!(summary.checksum, 0);
    }

    #[test]
    fn rejects_bad_ranges() {
        assert_eq!(
            parse_capacity_summary(&[0], &[0], &[2], 4),
            Err(NbtCompoundMapCapacityError::InvalidRange)
        );
        assert_eq!(
            parse_capacity_summary(&[0], &[0, 1], &[1], 4),
            Err(NbtCompoundMapCapacityError::LengthMismatch)
        );
    }
}
