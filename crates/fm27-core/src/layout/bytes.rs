//! Little-endian reads shared by the save parsers.

pub fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(*read_slice::<2>(bytes, offset)?))
}

pub fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(*read_slice::<4>(bytes, offset)?))
}

pub fn read_i16(bytes: &[u8], offset: usize) -> Option<i16> {
    Some(i16::from_le_bytes(*read_slice::<2>(bytes, offset)?))
}

pub fn read_i32(bytes: &[u8], offset: usize) -> Option<i32> {
    Some(i32::from_le_bytes(*read_slice::<4>(bytes, offset)?))
}

pub fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(*read_slice::<8>(bytes, offset)?))
}

pub fn read_u8(bytes: &[u8], offset: usize) -> Option<u8> {
    bytes.get(offset).copied()
}

fn read_slice<const N: usize>(bytes: &[u8], offset: usize) -> Option<&[u8; N]> {
    bytes.get(offset..)?.first_chunk::<N>()
}
