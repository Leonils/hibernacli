pub struct BufferReadError;

pub trait BufferExt {
    fn read_u64_from_le(&mut self, offset: usize) -> Result<u64, BufferReadError>;
    fn read_u128_from_le(&mut self, offset: usize) -> Result<u128, BufferReadError>;
}

impl BufferExt for Vec<u8> {
    fn read_u64_from_le(&mut self, offset: usize) -> Result<u64, BufferReadError> {
        Ok(u64::from_le_bytes(
            self.get(offset..offset + 8)
                .ok_or(BufferReadError)?
                .try_into()
                .map_err(|_| BufferReadError)?,
        ))
    }

    fn read_u128_from_le(&mut self, offset: usize) -> Result<u128, BufferReadError> {
        Ok(u128::from_le_bytes(
            self.get(offset..offset + 16)
                .ok_or(BufferReadError)?
                .try_into()
                .map_err(|_| BufferReadError)?,
        ))
    }
}
