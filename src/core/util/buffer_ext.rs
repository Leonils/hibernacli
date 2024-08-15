use std::array::TryFromSliceError;

pub trait BufferExt {
    fn read_u64_from_le(&mut self, offset: usize) -> Result<u64, TryFromSliceError>;
    fn read_u128_from_le(&mut self, offset: usize) -> Result<u128, TryFromSliceError>;
}

impl BufferExt for Vec<u8> {
    fn read_u64_from_le(&mut self, offset: usize) -> Result<u64, TryFromSliceError> {
        Ok(u64::from_le_bytes(self[offset..offset + 8].try_into()?))
    }

    fn read_u128_from_le(&mut self, offset: usize) -> Result<u128, TryFromSliceError> {
        Ok(u128::from_le_bytes(self[offset..offset + 16].try_into()?))
    }
}
