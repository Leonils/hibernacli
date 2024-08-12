use std::array::TryFromSliceError;

pub trait BufferExt {
    fn read_u64_from_le(&mut self, offset: usize) -> Result<u64, TryFromSliceError>;
}

impl BufferExt for Vec<u8> {
    fn read_u64_from_le(&mut self, offset: usize) -> Result<u64, TryFromSliceError> {
        Ok(u64::from_le_bytes(self[offset..offset + 8].try_into()?))
    }
}
