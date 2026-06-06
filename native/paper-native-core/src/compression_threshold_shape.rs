use std::convert::TryFrom;

use crate::varint;

pub const SUMMARY_FIELDS: usize = 8;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const SUMMARY_TAG: u64 = 0x434F_4D50_5448_5245;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompressionThresholdShapeSummary {
    pub packets: u64,
    pub thresholds: u64,
    pub total_payload_bytes: u64,
    pub bypassed_packets: u64,
    pub compressed_packets: u64,
    pub framed_bytes: u64,
    pub compression_input_bytes: u64,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompressionThresholdShapeError {
    InvalidPacketLength,
    InvalidThreshold,
    InvalidIterations,
}

pub fn threshold_summary(
    packet_lengths: &[i32],
    thresholds: &[i32],
    iterations: usize,
) -> Result<CompressionThresholdShapeSummary, CompressionThresholdShapeError> {
    if iterations == 0 {
        return Err(CompressionThresholdShapeError::InvalidIterations);
    }

    let mut summary = CompressionThresholdShapeSummary {
        packets: (packet_lengths.len() as u64)
            .wrapping_mul(thresholds.len() as u64)
            .wrapping_mul(iterations as u64),
        thresholds: thresholds.len() as u64,
        checksum: mix64(
            SUMMARY_TAG
                ^ packet_lengths.len() as u64
                ^ ((thresholds.len() as u64) << 21)
                ^ ((iterations as u64) << 42),
        ),
        ..CompressionThresholdShapeSummary::default()
    };

    for iteration in 0..iterations {
        for (threshold_index, &threshold) in thresholds.iter().enumerate() {
            if threshold < -1 {
                return Err(CompressionThresholdShapeError::InvalidThreshold);
            }
            for (packet_index, &packet_len) in packet_lengths.iter().enumerate() {
                let packet_len = usize::try_from(packet_len)
                    .map_err(|_| CompressionThresholdShapeError::InvalidPacketLength)?;
                summary.total_payload_bytes =
                    summary.total_payload_bytes.wrapping_add(packet_len as u64);

                let shape = packet_shape(packet_len, threshold)?;
                summary.bypassed_packets = summary
                    .bypassed_packets
                    .wrapping_add(u64::from(!shape.compressed));
                summary.compressed_packets = summary
                    .compressed_packets
                    .wrapping_add(u64::from(shape.compressed));
                summary.framed_bytes = summary.framed_bytes.wrapping_add(shape.framed_bytes);
                summary.compression_input_bytes = summary
                    .compression_input_bytes
                    .wrapping_add(shape.compression_input_bytes);
                summary.checksum = mix64(
                    summary.checksum
                        ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                        ^ ((threshold_index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                        ^ ((packet_index as u64).wrapping_mul(0xD6E8_FD9A_2C4F_6B1D))
                        ^ ((threshold as i64 as u64) << 3)
                        ^ ((packet_len as u64) << 17)
                        ^ shape.framed_bytes.rotate_left(11)
                        ^ shape.compression_input_bytes.rotate_left(29),
                );
            }
        }
    }

    Ok(summary)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PacketShape {
    compressed: bool,
    framed_bytes: u64,
    compression_input_bytes: u64,
}

fn packet_shape(
    packet_len: usize,
    threshold: i32,
) -> Result<PacketShape, CompressionThresholdShapeError> {
    if packet_len > i32::MAX as usize {
        return Err(CompressionThresholdShapeError::InvalidPacketLength);
    }
    if threshold < 0 {
        let frame_len = varint::varint_size(packet_len as i32) + packet_len;
        return Ok(PacketShape {
            compressed: false,
            framed_bytes: frame_len as u64,
            compression_input_bytes: 0,
        });
    }

    let threshold =
        usize::try_from(threshold).map_err(|_| CompressionThresholdShapeError::InvalidThreshold)?;
    if packet_len < threshold {
        let packet_body_len = 1usize + packet_len;
        let frame_len = varint::varint_size(packet_body_len as i32) + packet_body_len;
        Ok(PacketShape {
            compressed: false,
            framed_bytes: frame_len as u64,
            compression_input_bytes: 0,
        })
    } else {
        let compressed_len = modeled_compressed_len(packet_len);
        let data_len_size = varint::varint_size(packet_len as i32);
        let packet_body_len = data_len_size + compressed_len;
        let frame_len = varint::varint_size(packet_body_len as i32) + packet_body_len;
        Ok(PacketShape {
            compressed: true,
            framed_bytes: frame_len as u64,
            compression_input_bytes: packet_len as u64,
        })
    }
}

#[inline]
fn modeled_compressed_len(packet_len: usize) -> usize {
    if packet_len == 0 {
        return 8;
    }
    let repeated_payload_estimate = packet_len / 3;
    let entropy_overhead = packet_len / 128 + 8;
    repeated_payload_estimate + entropy_overhead
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
    fn threshold_counts_bypass_and_compressed_packets() {
        let lengths = [32, 128, 255, 256, 1024];
        let thresholds = [-1, 256];
        let summary = threshold_summary(&lengths, &thresholds, 3).unwrap();

        assert_eq!(summary.packets, 30);
        assert_eq!(summary.thresholds, 2);
        assert_eq!(summary.compressed_packets, 6);
        assert_eq!(summary.bypassed_packets, 24);
        assert_eq!(summary.compression_input_bytes, (256 + 1024) * 3);
    }

    #[test]
    fn lower_threshold_compresses_more() {
        let lengths = [64, 128, 256, 512, 2048];
        let high = threshold_summary(&lengths, &[1024], 4).unwrap();
        let low = threshold_summary(&lengths, &[128], 4).unwrap();

        assert!(low.compressed_packets > high.compressed_packets);
        assert!(low.compression_input_bytes > high.compression_input_bytes);
    }

    #[test]
    fn disabled_threshold_has_no_compression_input() {
        let lengths = [64, 128, 256, 512, 2048];
        let summary = threshold_summary(&lengths, &[-1], 2).unwrap();

        assert_eq!(summary.compressed_packets, 0);
        assert_eq!(summary.compression_input_bytes, 0);
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(
            threshold_summary(&[-1], &[256], 1),
            Err(CompressionThresholdShapeError::InvalidPacketLength)
        );
        assert_eq!(
            threshold_summary(&[1], &[-2], 1),
            Err(CompressionThresholdShapeError::InvalidThreshold)
        );
        assert_eq!(
            threshold_summary(&[1], &[256], 0),
            Err(CompressionThresholdShapeError::InvalidIterations)
        );
    }
}
