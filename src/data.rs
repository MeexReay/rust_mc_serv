
pub struct VarInt;

impl VarInt {
	// Константы объявляются внутри блока impl
	pub const MAX_VARINT_SIZE: i32 = 5;
	pub const DATA_BITS_MASK: i32 = 127;
	pub const CONTINUATION_BIT_MASK: i32 = 128;
	pub const DATA_BITS_PER_BYTE: i32 = 7;

	pub fn getByteSize(i: i32) -> i32 {
		for j in 1..5 {
			if (i & -1 << (j * 7)) == 0 {
				return j;
			}
		}
		return 5;
	}

	pub fn hasContinuationBit(b0: u8) -> bool {
		return (b0 & 128) == 128
	}

	
}