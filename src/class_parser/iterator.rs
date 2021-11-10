use super::ParsingError;

pub(super) struct ClassFileIterator<'b> {
    bytes: &'b [u8],
    offset: usize,
}

impl<'b> ClassFileIterator<'b> {
    pub(super) fn new(bytes: &'b [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(super) fn take_bytes(&mut self, count: usize) -> Result<&'_ [u8], ParsingError> {
        if self.offset + count > self.bytes.len() {
            Err(ParsingError::UnexpectedEOF)
        } else {
            let bytes = &self.bytes[self.offset..self.offset + count];
            self.offset += count;
            Ok(bytes)
        }
    }

    pub(super) fn skip_bytes(&mut self, count: usize) -> Result<(), ParsingError> {
        if self.offset + count > self.bytes.len() {
            Err(ParsingError::UnexpectedEOF)
        } else {
            self.offset += count;
            Ok(())
        }
    }

    pub(super) fn byte(&mut self) -> Result<u8, ParsingError> {
        let byte = match self.bytes.get(self.offset) {
            Some(byte) => Ok(*byte),
            None => Err(ParsingError::UnexpectedEOF),
        };
        self.offset += 1;
        byte
    }

    pub(super) fn u16(&mut self) -> Result<u16, ParsingError> {
        Ok(u16::from_be_bytes([self.byte()?, self.byte()?]))
    }

    pub(super) fn u32(&mut self) -> Result<u32, ParsingError> {
        Ok(u32::from_be_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }

    pub(super) fn i32(&mut self) -> Result<i32, ParsingError> {
        Ok(i32::from_be_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }

    pub(super) fn i64(&mut self) -> Result<i64, ParsingError> {
        Ok(i64::from_be_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }

    pub(super) fn f32(&mut self) -> Result<f32, ParsingError> {
        Ok(f32::from_be_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }

    pub(super) fn f64(&mut self) -> Result<f64, ParsingError> {
        Ok(f64::from_be_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }
}