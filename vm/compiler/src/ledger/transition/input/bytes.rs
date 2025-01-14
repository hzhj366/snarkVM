// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

impl<N: Network> FromBytes for Input<N> {
    /// Reads the input from a buffer.
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let variant = Variant::read_le(&mut reader)?;
        let literal = match variant {
            0 => {
                let plaintext_hash: Field<N> = FromBytes::read_le(&mut reader)?;
                let plaintext_exists: bool = FromBytes::read_le(&mut reader)?;
                let plaintext = match plaintext_exists {
                    true => Some(FromBytes::read_le(&mut reader)?),
                    false => None,
                };

                Self::Constant(plaintext_hash, plaintext)
            }
            1 => {
                let plaintext_hash: Field<N> = FromBytes::read_le(&mut reader)?;
                let plaintext_exists: bool = FromBytes::read_le(&mut reader)?;
                let plaintext = match plaintext_exists {
                    true => Some(FromBytes::read_le(&mut reader)?),
                    false => None,
                };
                Self::Public(plaintext_hash, plaintext)
            }
            2 => {
                let ciphertext_hash: Field<N> = FromBytes::read_le(&mut reader)?;
                let ciphertext_exists: bool = FromBytes::read_le(&mut reader)?;
                let ciphertext = match ciphertext_exists {
                    true => Some(FromBytes::read_le(&mut reader)?),
                    false => None,
                };
                Self::Private(ciphertext_hash, ciphertext)
            }
            3 => {
                // Read the serial number.
                let serial_number: Field<N> = FromBytes::read_le(&mut reader)?;
                // Read the tag.
                let tag: Field<N> = FromBytes::read_le(&mut reader)?;
                // Read the origin type.
                let origin_type = u8::read_le(&mut reader)?;
                // Read the origin.
                match origin_type {
                    0 => Self::Record(serial_number, tag, Origin::Commitment(FromBytes::read_le(&mut reader)?)),
                    1 => Self::Record(serial_number, tag, Origin::StateRoot(FromBytes::read_le(&mut reader)?)),
                    _ => {
                        return Err(error(format!("Failed to decode transition input with origin type {origin_type}")));
                    }
                }
            }
            4 => Self::ExternalRecord(FromBytes::read_le(&mut reader)?),
            5.. => return Err(error(format!("Failed to decode transition input variant {variant}"))),
        };
        Ok(literal)
    }
}

impl<N: Network> ToBytes for Input<N> {
    /// Writes the input to a buffer.
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        match self {
            Self::Constant(plaintext_hash, plaintext) => {
                (0 as Variant).write_le(&mut writer)?;
                plaintext_hash.write_le(&mut writer)?;
                match plaintext {
                    Some(plaintext) => {
                        true.write_le(&mut writer)?;
                        plaintext.write_le(&mut writer)
                    }
                    None => false.write_le(&mut writer),
                }
            }
            Self::Public(plaintext_hash, plaintext) => {
                (1 as Variant).write_le(&mut writer)?;
                plaintext_hash.write_le(&mut writer)?;
                match plaintext {
                    Some(plaintext) => {
                        true.write_le(&mut writer)?;
                        plaintext.write_le(&mut writer)
                    }
                    None => false.write_le(&mut writer),
                }
            }
            Self::Private(ciphertext_hash, ciphertext) => {
                (2 as Variant).write_le(&mut writer)?;
                ciphertext_hash.write_le(&mut writer)?;
                match ciphertext {
                    Some(ciphertext) => {
                        true.write_le(&mut writer)?;
                        ciphertext.write_le(&mut writer)
                    }
                    None => false.write_le(&mut writer),
                }
            }
            Self::Record(serial_number, tag, origin) => {
                (3 as Variant).write_le(&mut writer)?;
                serial_number.write_le(&mut writer)?;
                tag.write_le(&mut writer)?;
                match origin {
                    Origin::Commitment(commitment) => {
                        0u8.write_le(&mut writer)?;
                        commitment.write_le(&mut writer)
                    }
                    Origin::StateRoot(root) => {
                        1u8.write_le(&mut writer)?;
                        root.write_le(&mut writer)
                    }
                }
            }
            Self::ExternalRecord(input_commitment) => {
                (4 as Variant).write_le(&mut writer)?;
                input_commitment.write_le(&mut writer)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::network::Testnet3;

    type CurrentNetwork = Testnet3;

    #[test]
    fn test_bytes() {
        for (_, expected) in crate::ledger::transition::input::test_helpers::sample_inputs() {
            // Check the byte representation.
            let expected_bytes = expected.to_bytes_le().unwrap();
            assert_eq!(expected, Input::read_le(&expected_bytes[..]).unwrap());
            assert!(Input::<CurrentNetwork>::read_le(&expected_bytes[1..]).is_err());
        }
    }
}
