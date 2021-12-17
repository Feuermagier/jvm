use std::collections::HashMap;

use crate::model::value::JvmValue;

use super::{
    heap::HeapIndex,
    types::JvmType,
    value::{JvmDouble, JvmFloat, JvmInt, JvmLong, JvmReference},
    visibility::Visibility,
};

#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    pub name: String,
    pub visibility: Visibility,
    pub ty: JvmType,
    pub constant_value: Option<JvmValue>,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldInfo {
    pub offset: usize,
    pub ty: JvmType,
}

#[derive(Clone, Debug)]
pub struct FieldLayout {
    length: usize,
    fields: HashMap<String, (usize, JvmType, Option<JvmValue>)>,
    spaces: Vec<EmptySpace>, // Ordered list of runs of not used bytes (ordered by their starting index)
}

impl FieldLayout {
    pub fn empty() -> Self {
        Self {
            length: 0,
            fields: HashMap::new(),
            spaces: Vec::new(),
        }
    }
}

impl FieldLayout {
    pub fn resolve(&self, name: &str) -> Result<FieldInfo, FieldError> {
        if let Some((offset, ty, _)) = self.fields.get(name) {
            Ok(FieldInfo {
                offset: *offset,
                ty: *ty,
            })
        } else {
            Err(FieldError::UnknownField(name.to_string()))
        }
    }
}

pub fn layout_fields(parent_layout: &FieldLayout, fields: &Vec<FieldDescriptor>) -> FieldLayout {
    // Sort descending by an inverted comparison
    let mut fields_to_place = (*fields).clone();
    fields_to_place
        .sort_by(|first_field, second_field| second_field.ty.size().cmp(&first_field.ty.size()));

    let mut field_mappings = parent_layout.fields.clone();

    let mut spaces = parent_layout.spaces.clone();
    let mut length = parent_layout.length;
    'field: for field in fields_to_place {
        for i in 0..spaces.len() {
            if spaces[i].length == field.ty.size() {
                field_mappings.insert(
                    field.name.clone(),
                    (spaces[i].index, field.ty, field.constant_value),
                );
                spaces.remove(i);
                break 'field;
            } else if spaces[i].length > field.ty.size() {
                field_mappings.insert(
                    field.name.clone(),
                    (spaces[i].index, field.ty, field.constant_value),
                );
                spaces[i].length -= field.ty.size();
                spaces[i].index += field.ty.size();
                break 'field;
            }
        }

        // If we are here, no matching empty space has been found and the field will be layouted at the end of all fields
        let alignment_space = if length % field.ty.alignment() == 0 {
            0
        } else {
            let space = field.ty.alignment() - (length % field.ty.alignment());
            spaces.push(EmptySpace {
                index: length,
                length: space,
            });
            space
        };

        field_mappings.insert(
            field.name.clone(),
            (length + alignment_space, field.ty, field.constant_value),
        );
        length += alignment_space + field.ty.size();
    }

    FieldLayout {
        length,
        fields: field_mappings,
        spaces,
    }
}

#[derive(Debug, Clone, Copy)]
struct EmptySpace {
    index: usize,
    length: usize,
}

#[repr(transparent)]
pub struct Fields {
    fields: Vec<u8>,
}

impl Fields {
    pub fn init_from_layout(layout: &FieldLayout) -> Self {
        let mut fields = Self {
            fields: vec![0; layout.length],
        };

        // todo
        /*
        for (offset, _, constant_value) in layout.fields.values() {
            if let Some(value) = constant_value {
                fields.set_value(*offset, *value);
            }
        }
        */
        fields
    }

    pub fn set_value(&mut self, offset: usize, ty: JvmType, value: JvmValue) {
        match ty {
            JvmType::Void => {},
            JvmType::Integer => self.set_int(offset, value.int()),
            JvmType::Long => self.set_long(offset, value.long()),
            JvmType::Float => self.set_float(offset, value.float()),
            JvmType::Double => self.set_double(offset, value.double()),
            JvmType::Reference => self.set_reference(offset, value.reference()),
            _ => todo!()
        }
    }

    pub fn set_int(&mut self, offset: usize, value: JvmInt) {
        let bytes = value.0.to_be_bytes();
        self.fields[offset + 0] = bytes[0];
        self.fields[offset + 1] = bytes[1];
        self.fields[offset + 2] = bytes[2];
        self.fields[offset + 3] = bytes[3];
    }

    pub fn set_float(&mut self, offset: usize, value: JvmFloat) {
        let bytes = value.0.to_be_bytes();
        self.fields[offset + 0] = bytes[0];
        self.fields[offset + 1] = bytes[1];
        self.fields[offset + 2] = bytes[2];
        self.fields[offset + 3] = bytes[3];
    }

    pub fn set_long(&mut self, offset: usize, value: JvmLong) {
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

    pub fn set_double(&mut self, offset: usize, value: JvmDouble) {
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

    pub fn set_reference(&mut self, offset: usize, value: JvmReference) {
        let bytes = unsafe { value.0.into_raw().to_be_bytes() };
        self.fields[offset + 0] = bytes[0];
        self.fields[offset + 1] = bytes[1];
        self.fields[offset + 2] = bytes[2];
        self.fields[offset + 3] = bytes[3];
        self.fields[offset + 4] = bytes[4];
        self.fields[offset + 5] = bytes[5];
        self.fields[offset + 6] = bytes[6];
        self.fields[offset + 7] = bytes[7];
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
        unsafe {
            JvmReference(HeapIndex::from_raw(u64::from_be_bytes([
                self.fields[offset + 0],
                self.fields[offset + 1],
                self.fields[offset + 2],
                self.fields[offset + 3],
                self.fields[offset + 4],
                self.fields[offset + 5],
                self.fields[offset + 6],
                self.fields[offset + 7],
            ])))
        }
    }

    pub fn get_value(&self, offset: usize, ty: JvmType) -> JvmValue {
        match ty {
            JvmType::Void => JvmValue::VOID,
            JvmType::Integer => JvmValue {
                int: self.get_int(offset).into(),
            },
            JvmType::Long => JvmValue {
                long: self.get_long(offset).into(),
            },
            JvmType::Float => JvmValue {
                float: self.get_float(offset).into(),
            },
            JvmType::Double => JvmValue {
                double: self.get_double(offset).into(),
            },
            JvmType::Reference => JvmValue {
                reference: self.get_reference(offset).to_heap_index(),
            },
            JvmType::Byte => todo!(),
            JvmType::Char => todo!(),
            JvmType::Short => todo!(),
            JvmType::Boolean => todo!(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FieldError {
    #[error("No field with name {0}")]
    UnknownField(String),
}
