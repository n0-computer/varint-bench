//! QUIC Variable-Length Integer Encoding.
//!
//! This is an unsigned integer type encoded 1, 2, 4, or 8 bytes and can store values up to
//! 2**62.
//!
//! See https://www.rfc-editor.org/rfc/rfc9000.html#name-variable-length-integer-enc.

use std::fmt;
use std::io::{Read, Write};

use anyhow::{bail, Error, Result};

/// An integer less than 2^62
///
/// This is defined to be identical to QUIC's variable-length integer.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VarInt(u64);

impl VarInt {
    /// The largest representable value
    pub const MAX: VarInt = VarInt((1 << 62) - 1);
    /// The largest encoded value length
    pub const MAX_SIZE: usize = 8;

    /// Compute the number of bytes needed to encode this value
    pub fn size(self) -> usize {
        let x = self.0;
        if x < 2u64.pow(6) {
            1
        } else if x < 2u64.pow(14) {
            2
        } else if x < 2u64.pow(30) {
            4
        } else if x < 2u64.pow(62) {
            8
        } else {
            unreachable!("malformed VarInt");
        }
    }

    /// Encodes the VarInt to a writer.
    pub fn encode<W: Write>(&self, mut writer: W) -> Result<()> {
        let x = self.0;
        if x < 2u64.pow(6) {
            let src = [x as u8];
            writer.write_all(&src)?;
        } else if x < 2u64.pow(14) {
            let n = 0b01 << 14 | x as u16;
            writer.write_all(&n.to_be_bytes())?;
        } else if x < 2u64.pow(30) {
            let n = 0b10 << 30 | x as u32;
            writer.write_all(&n.to_be_bytes())?;
        } else if x < 2u64.pow(62) {
            let n = 0b11 << 62 | x;
            writer.write_all(&n.to_be_bytes())?;
        } else {
            unreachable!("malformed VarInt")
        }
        Ok(())
    }

    /// Length of an encoded value from its first byte
    pub fn encoded_size(first: u8) -> usize {
        2usize.pow((first >> 6) as u32)
    }

    /// Decodes a VarInt from a reader.
    pub fn decode<R: Read>(mut reader: R) -> Result<Self> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf[0..1])?;
        let num_bytes = Self::encoded_size(buf[0]);
        buf[0] &= 0b0011_1111;
        let val = match num_bytes {
            1 => VarInt::from(buf[0]),
            2 => {
                reader.read_exact(&mut buf[1..2])?;
                let val = u16::from_be_bytes(buf[..2].try_into()?);
                VarInt::from(val)
            }
            4 => {
                reader.read_exact(&mut buf[1..4])?;
                let val = u32::from_be_bytes(buf[..4].try_into()?);
                VarInt::from(val)
            }
            8 => {
                reader.read_exact(&mut buf[1..8])?;
                VarInt::try_from(u64::from_be_bytes(buf))?
            }
            _ => bail!("Invalid VarInt tag"),
        };
        Ok(val)
    }
}

impl From<VarInt> for u64 {
    fn from(x: VarInt) -> u64 {
        x.0
    }
}

impl From<u8> for VarInt {
    fn from(x: u8) -> Self {
        VarInt(x.into())
    }
}

impl From<u16> for VarInt {
    fn from(x: u16) -> Self {
        VarInt(x.into())
    }
}

impl From<u32> for VarInt {
    fn from(x: u32) -> Self {
        VarInt(x.into())
    }
}

impl TryFrom<u64> for VarInt {
    type Error = Error;

    fn try_from(x: u64) -> Result<Self> {
        if x < 2u64.pow(62) {
            Ok(VarInt(x))
        } else {
            Err(Error::msg("VarInt bounds exceeded"))
        }
    }
}

impl TryFrom<usize> for VarInt {
    type Error = Error;

    fn try_from(x: usize) -> Result<Self> {
        let x: u64 = x.try_into()?;
        VarInt::try_from(x)
    }
}

impl fmt::Display for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
