/// Borrowed from https://snowbridge-rust-docs.snowfork.com/src/parity_wasm/elements/primitives.rs.html
use crate::io;
use super::{Error, Deserialize, Serialize};

/// Unsigned variable-length integer, limited to 32 bits,
/// represented by at most 5 bytes that may contain padding 0x80 bytes.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarUint32(u32);

impl From<VarUint32> for usize {
	fn from(var: VarUint32) -> usize {
		var.0 as usize
	}
}

impl From<VarUint32> for u32 {
	fn from(var: VarUint32) -> u32 {
		var.0
	}
}

impl From<u32> for VarUint32 {
	fn from(i: u32) -> VarUint32 {
		VarUint32(i)
	}
}

impl From<usize> for VarUint32 {
	fn from(i: usize) -> VarUint32 {
		assert!(i <= u32::max_value() as usize);
		VarUint32(i as u32)
	}
}

impl Deserialize for VarUint32 {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let mut res = 0;
		let mut shift = 0;
		let mut u8buf = [0u8; 1];
		loop {
			if shift > 31 { return Err(Error::InvalidVarUint32); }

			reader.read(&mut u8buf)?;
			let b = u8buf[0] as u32;
			res |= (b & 0x7f).checked_shl(shift).ok_or(Error::InvalidVarUint32)?;
			shift += 7;
			if (b >> 7) == 0 {
				if shift >= 32 && (b as u8).leading_zeros() < 4 {
					return Err(Error::InvalidVarInt32);
				}
				break;
			}
		}
		Ok(VarUint32(res))
	}
}

impl Serialize for VarUint32 {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut buf = [0u8; 1];
		let mut v = self.0;
		loop {
			buf[0] = (v & 0b0111_1111) as u8;
			v >>= 7;
			if v > 0 {
				buf[0] |= 0b1000_0000;
			}
			writer.write(&buf[..])?;
			if v == 0 { break; }
		}

		Ok(())
	}
}