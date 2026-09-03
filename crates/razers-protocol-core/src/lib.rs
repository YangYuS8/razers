// SPDX-License-Identifier: GPL-2.0-or-later

//! Pure protocol codecs with no device or operating-system I/O.

use thiserror::Error;

/// Total size of the classic Razer vendor report.
pub const REPORT_90_SIZE: usize = 90;
/// Maximum number of argument bytes carried by a classic report.
pub const REPORT_90_ARGUMENT_CAPACITY: usize = 80;
/// Byte position containing the report checksum.
pub const REPORT_90_CRC_INDEX: usize = 88;
/// Byte position containing the reserved trailing byte.
pub const REPORT_90_RESERVED_INDEX: usize = 89;

/// A decoded classic 90-byte Razer vendor report.
///
/// The representation is intentionally explicit instead of relying on packed
/// structs or unsafe memory conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report90 {
    pub status: u8,
    pub transaction_id: u8,
    pub remaining_packets: u16,
    pub protocol_type: u8,
    pub command_class: u8,
    pub command_id: u8,
    arguments: Vec<u8>,
    pub reserved: u8,
}

/// Errors produced while encoding or decoding a classic report.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum Report90Error {
    #[error("expected a {REPORT_90_SIZE}-byte report, received {actual} bytes")]
    InvalidLength { actual: usize },
    #[error(
        "report arguments are {actual} bytes, exceeding the {REPORT_90_ARGUMENT_CAPACITY}-byte limit"
    )]
    ArgumentsTooLong { actual: usize },
    #[error("report declares an invalid data size of {data_size} bytes")]
    InvalidDataSize { data_size: usize },
    #[error("report checksum mismatch: expected 0x{expected:02x}, received 0x{actual:02x}")]
    ChecksumMismatch { expected: u8, actual: u8 },
}

impl Report90 {
    /// Construct a host-to-device command with default status and protocol fields.
    pub fn command(
        command_class: u8,
        command_id: u8,
        arguments: impl Into<Vec<u8>>,
    ) -> Result<Self, Report90Error> {
        let arguments = arguments.into();
        validate_argument_length(arguments.len())?;

        Ok(Self {
            status: 0,
            transaction_id: 0,
            remaining_packets: 0,
            protocol_type: 0,
            command_class,
            command_id,
            arguments,
            reserved: 0,
        })
    }

    /// Set the transaction identifier used to match a request and response.
    pub fn with_transaction_id(mut self, transaction_id: u8) -> Self {
        self.transaction_id = transaction_id;
        self
    }

    /// Return only the meaningful argument bytes declared by `data_size`.
    pub fn arguments(&self) -> &[u8] {
        &self.arguments
    }

    /// Encode this report into its fixed-size wire representation.
    pub fn encode(&self) -> Result<[u8; REPORT_90_SIZE], Report90Error> {
        validate_argument_length(self.arguments.len())?;

        let mut bytes = [0_u8; REPORT_90_SIZE];
        bytes[0] = self.status;
        bytes[1] = self.transaction_id;
        bytes[2..4].copy_from_slice(&self.remaining_packets.to_be_bytes());
        bytes[4] = self.protocol_type;
        bytes[5] = self.arguments.len() as u8;
        bytes[6] = self.command_class;
        bytes[7] = self.command_id;
        bytes[8..8 + self.arguments.len()].copy_from_slice(&self.arguments);
        bytes[REPORT_90_CRC_INDEX] = calculate_crc(&bytes);
        bytes[REPORT_90_RESERVED_INDEX] = self.reserved;
        Ok(bytes)
    }

    /// Decode and validate an exact 90-byte wire report.
    pub fn decode(bytes: &[u8]) -> Result<Self, Report90Error> {
        if bytes.len() != REPORT_90_SIZE {
            return Err(Report90Error::InvalidLength {
                actual: bytes.len(),
            });
        }

        let data_size = bytes[5] as usize;
        if data_size > REPORT_90_ARGUMENT_CAPACITY {
            return Err(Report90Error::InvalidDataSize { data_size });
        }

        let packet: &[u8; REPORT_90_SIZE] = bytes
            .try_into()
            .expect("report length was checked before conversion");
        let expected = calculate_crc(packet);
        let actual = bytes[REPORT_90_CRC_INDEX];
        if actual != expected {
            return Err(Report90Error::ChecksumMismatch { expected, actual });
        }

        Ok(Self {
            status: bytes[0],
            transaction_id: bytes[1],
            remaining_packets: u16::from_be_bytes([bytes[2], bytes[3]]),
            protocol_type: bytes[4],
            command_class: bytes[6],
            command_id: bytes[7],
            arguments: bytes[8..8 + data_size].to_vec(),
            reserved: bytes[REPORT_90_RESERVED_INDEX],
        })
    }
}

fn validate_argument_length(actual: usize) -> Result<(), Report90Error> {
    if actual > REPORT_90_ARGUMENT_CAPACITY {
        return Err(Report90Error::ArgumentsTooLong { actual });
    }
    Ok(())
}

/// Calculate the XOR checksum over byte positions 2 through 87 inclusive.
///
/// The CRC and reserved bytes are intentionally excluded.
pub fn calculate_crc(bytes: &[u8; REPORT_90_SIZE]) -> u8 {
    bytes[2..REPORT_90_CRC_INDEX]
        .iter()
        .fold(0_u8, |checksum, byte| checksum ^ byte)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_the_documented_wire_layout() {
        let mut report = Report90::command(0x04, 0x05, [0x12, 0x34]).unwrap();
        report.status = 0x02;
        report.transaction_id = 0x1f;
        report.remaining_packets = 0x1234;

        let bytes = report.encode().unwrap();

        assert_eq!(bytes.len(), REPORT_90_SIZE);
        assert_eq!(
            &bytes[..10],
            &[0x02, 0x1f, 0x12, 0x34, 0, 2, 4, 5, 0x12, 0x34]
        );
        assert_eq!(bytes[REPORT_90_CRC_INDEX], calculate_crc(&bytes));
        assert_eq!(bytes[REPORT_90_RESERVED_INDEX], 0);
    }

    #[test]
    fn round_trips_a_report() {
        let mut report = Report90::command(0x07, 0x82, [0xaa, 0xbb, 0xcc]).unwrap();
        report.transaction_id = 0x3f;
        report.remaining_packets = 2;

        let decoded = Report90::decode(&report.encode().unwrap()).unwrap();
        assert_eq!(decoded, report);
    }

    #[test]
    fn rejects_a_corrupted_report() {
        let mut bytes = Report90::command(0x00, 0x81, [0_u8, 0_u8])
            .unwrap()
            .encode()
            .unwrap();
        bytes[8] ^= 0xff;

        assert!(matches!(
            Report90::decode(&bytes),
            Err(Report90Error::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn rejects_oversized_arguments() {
        assert_eq!(
            Report90::command(0, 0, vec![0; REPORT_90_ARGUMENT_CAPACITY + 1]),
            Err(Report90Error::ArgumentsTooLong {
                actual: REPORT_90_ARGUMENT_CAPACITY + 1
            })
        );
    }

    #[test]
    fn rejects_an_invalid_declared_data_size_before_slicing() {
        let mut bytes = [0_u8; REPORT_90_SIZE];
        bytes[5] = 81;
        bytes[REPORT_90_CRC_INDEX] = calculate_crc(&bytes);

        assert_eq!(
            Report90::decode(&bytes),
            Err(Report90Error::InvalidDataSize { data_size: 81 })
        );
    }
}
