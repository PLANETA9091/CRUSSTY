pub const MAX_VARINT_SIZE: usize = 5;
pub const MAX_VARLONG_SIZE: usize = 10;

#[inline]
pub fn varint_size(value: i32) -> usize {
    let data = value as u32;
    for size in 1..MAX_VARINT_SIZE {
        if (data & (!0u32 << (size * 7))) == 0 {
            return size;
        }
    }
    MAX_VARINT_SIZE
}

#[inline]
pub fn varlong_size(value: i64) -> usize {
    let data = value as u64;
    for size in 1..MAX_VARLONG_SIZE {
        if (data & (!0u64 << (size * 7))) == 0 {
            return size;
        }
    }
    MAX_VARLONG_SIZE
}

#[inline]
pub fn write_varint(value: i32, dst: &mut [u8]) -> Option<usize> {
    let mut value = value as u32;
    let mut written = 0usize;

    while (value & !0x7F) != 0 {
        if written >= dst.len() {
            return None;
        }
        dst[written] = ((value & 0x7F) | 0x80) as u8;
        written += 1;
        value >>= 7;
    }

    if written >= dst.len() {
        return None;
    }
    dst[written] = value as u8;
    Some(written + 1)
}

#[inline]
pub fn write_varlong(value: i64, dst: &mut [u8]) -> Option<usize> {
    let mut value = value as u64;
    let mut written = 0usize;

    while (value & !0x7F) != 0 {
        if written >= dst.len() {
            return None;
        }
        dst[written] = ((value & 0x7F) | 0x80) as u8;
        written += 1;
        value >>= 7;
    }

    if written >= dst.len() {
        return None;
    }
    dst[written] = value as u8;
    Some(written + 1)
}

pub fn write_varint_batch(values: &[i32], dst: &mut [u8]) -> Option<usize> {
    let required = values
        .iter()
        .fold(0usize, |total, value| total + varint_size(*value));
    if required > dst.len() {
        return None;
    }

    let mut offset = 0usize;
    for &value in values {
        offset += write_varint(value, &mut dst[offset..])?;
    }
    Some(offset)
}

pub fn write_varlong_batch(values: &[i64], dst: &mut [u8]) -> Option<usize> {
    let required = values
        .iter()
        .fold(0usize, |total, value| total + varlong_size(*value));
    if required > dst.len() {
        return None;
    }

    let mut offset = 0usize;
    for &value in values {
        offset += write_varlong(value, &mut dst[offset..])?;
    }
    Some(offset)
}

#[inline]
pub fn read_varint(src: &[u8]) -> Result<(i32, usize), VarIntError> {
    let mut value = 0u32;
    for index in 0..MAX_VARINT_SIZE {
        let Some(byte) = src.get(index).copied() else {
            return Err(VarIntError::Truncated);
        };
        value |= ((byte & 0x7F) as u32) << (index * 7);
        if (byte & 0x80) == 0 {
            return Ok((value as i32, index + 1));
        }
    }
    Err(VarIntError::TooBig)
}

#[inline]
pub fn read_varlong(src: &[u8]) -> Result<(i64, usize), VarIntError> {
    let mut value = 0u64;
    for index in 0..MAX_VARLONG_SIZE {
        let Some(byte) = src.get(index).copied() else {
            return Err(VarIntError::Truncated);
        };
        value |= ((byte & 0x7F) as u64) << (index * 7);
        if (byte & 0x80) == 0 {
            return Ok((value as i64, index + 1));
        }
    }
    Err(VarIntError::TooBig)
}

pub fn read_varint_batch(src: &[u8], dst: &mut [i32]) -> Result<usize, VarIntError> {
    let mut offset = 0usize;
    for value in dst {
        let (decoded, consumed) = read_varint(&src[offset..])?;
        *value = decoded;
        offset += consumed;
    }
    Ok(offset)
}

pub fn read_varlong_batch(src: &[u8], dst: &mut [i64]) -> Result<usize, VarIntError> {
    let mut offset = 0usize;
    for value in dst {
        let (decoded, consumed) = read_varlong(&src[offset..])?;
        *value = decoded;
        offset += consumed;
    }
    Ok(offset)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VarIntError {
    TooBig,
    Truncated,
}

#[cfg(test)]
mod tests {
    use super::*;

    const INT_CASES: &[i32] = &[
        0,
        1,
        2,
        127,
        128,
        255,
        16_383,
        16_384,
        2_097_151,
        2_097_152,
        268_435_455,
        268_435_456,
        i32::MAX,
        -1,
        i32::MIN,
    ];

    const LONG_CASES: &[i64] = &[
        0,
        1,
        127,
        128,
        16_383,
        16_384,
        2_097_151,
        2_097_152,
        268_435_455,
        268_435_456,
        i32::MAX as i64,
        i64::MAX,
        -1,
        i64::MIN,
    ];

    #[test]
    fn varint_round_trips() {
        for &value in INT_CASES {
            let mut buf = [0u8; MAX_VARINT_SIZE];
            let written = write_varint(value, &mut buf).unwrap();
            assert_eq!(written, varint_size(value), "size mismatch for {value}");
            let (decoded, consumed) = read_varint(&buf[..written]).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(consumed, written);
        }
    }

    #[test]
    fn varlong_round_trips() {
        for &value in LONG_CASES {
            let mut buf = [0u8; MAX_VARLONG_SIZE];
            let written = write_varlong(value, &mut buf).unwrap();
            assert_eq!(written, varlong_size(value), "size mismatch for {value}");
            let (decoded, consumed) = read_varlong(&buf[..written]).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(consumed, written);
        }
    }

    #[test]
    fn batch_round_trips() {
        let mut int_buf = [0u8; INT_CASES.len() * MAX_VARINT_SIZE];
        let mut int_decoded = [0i32; INT_CASES.len()];
        let int_written = write_varint_batch(INT_CASES, &mut int_buf).unwrap();
        let int_read = read_varint_batch(&int_buf[..int_written], &mut int_decoded).unwrap();
        assert_eq!(int_read, int_written);
        assert_eq!(&int_decoded, INT_CASES);

        let mut long_buf = [0u8; LONG_CASES.len() * MAX_VARLONG_SIZE];
        let mut long_decoded = [0i64; LONG_CASES.len()];
        let long_written = write_varlong_batch(LONG_CASES, &mut long_buf).unwrap();
        let long_read = read_varlong_batch(&long_buf[..long_written], &mut long_decoded).unwrap();
        assert_eq!(long_read, long_written);
        assert_eq!(&long_decoded, LONG_CASES);
    }

    #[test]
    fn detects_invalid_inputs() {
        assert_eq!(read_varint(&[0x80; MAX_VARINT_SIZE]), Err(VarIntError::TooBig));
        assert_eq!(read_varint(&[0x80, 0x80]), Err(VarIntError::Truncated));
        assert_eq!(write_varint(-1, &mut [0u8; 4]), None);
        assert_eq!(write_varlong(-1, &mut [0u8; 9]), None);
    }
}
