use super::value::{JvmDouble, JvmFloat, JvmInt, JvmLong, JvmReference, JvmValue};

#[repr(transparent)]
pub struct Fields {
    fields: Vec<u8>,
}

impl Fields {
    pub fn new(size: usize) -> Self {
        Self {
            fields: vec![0; size],
        }
    }

    pub fn set_value(&mut self, offset: usize, value: JvmValue) {
        match value {
            JvmValue::Int(value) => {
                let bytes = value.0.to_be_bytes();
                self.fields[offset + 0] = bytes[0];
                self.fields[offset + 1] = bytes[1];
                self.fields[offset + 2] = bytes[2];
                self.fields[offset + 3] = bytes[3];
            }
            JvmValue::Double(value) => {
                let bytes = value.0.to_be_bytes();
                self.fields[offset + 0] = bytes[0];
                self.fields[offset + 1] = bytes[1];
                self.fields[offset + 2] = bytes[2];
                self.fields[offset + 3] = bytes[3];
                self.fields[offset + 4] = bytes[4];
                self.fields[offset + 5] = bytes[5];
                self.fields[offset + 6] = bytes[6];
                self.fields[offset + 7] = bytes[7];
            }
            JvmValue::Reference(value) => {
                let bytes = value.0.to_be_bytes();
                self.fields[offset + 0] = bytes[0];
                self.fields[offset + 1] = bytes[1];
            }
            JvmValue::Long(value) => {
                let bytes = value.0.to_be_bytes();
                self.fields[offset + 0] = bytes[0];
                self.fields[offset + 1] = bytes[1];
                self.fields[offset + 2] = bytes[2];
                self.fields[offset + 3] = bytes[3];
                self.fields[offset + 4] = bytes[4];
                self.fields[offset + 5] = bytes[5];
                self.fields[offset + 6] = bytes[6];
                self.fields[offset + 7] = bytes[7];
            }
            JvmValue::Float(value) => {
                let bytes = value.0.to_be_bytes();
                self.fields[offset + 0] = bytes[0];
                self.fields[offset + 1] = bytes[1];
                self.fields[offset + 2] = bytes[2];
                self.fields[offset + 3] = bytes[3];
            }
            JvmValue::Void => {}
        }
    }

    pub fn get_int(&self, offset: usize) -> JvmInt {
        JvmInt(i32::from_be_bytes([
            self.fields[offset + 0],
            self.fields[offset + 1],
            self.fields[offset + 2],
            self.fields[offset + 3],
        ]))
    }

    pub fn get_long(&self, offset: usize) -> JvmLong {
        JvmLong(i64::from_be_bytes([
            self.fields[offset + 0],
            self.fields[offset + 1],
            self.fields[offset + 2],
            self.fields[offset + 3],
            self.fields[offset + 4],
            self.fields[offset + 5],
            self.fields[offset + 6],
            self.fields[offset + 7],
        ]))
    }

    pub fn get_float(&self, offset: usize) -> JvmFloat {
        JvmFloat(f32::from_be_bytes([
            self.fields[offset + 0],
            self.fields[offset + 1],
            self.fields[offset + 2],
            self.fields[offset + 3],
        ]))
    }

    pub fn get_double(&self, offset: usize) -> JvmDouble {
        JvmDouble(f64::from_be_bytes([
            self.fields[offset + 0],
            self.fields[offset + 1],
            self.fields[offset + 2],
            self.fields[offset + 3],
            self.fields[offset + 4],
            self.fields[offset + 5],
            self.fields[offset + 6],
            self.fields[offset + 7],
        ]))
    }

    pub fn get_reference(&self, offset: usize) -> JvmReference {
        JvmReference(u16::from_be_bytes([
            self.fields[offset + 0],
            self.fields[offset + 1],
        ]))
    }
}
