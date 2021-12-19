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
    fields: HashMap<String, (usize, JvmType)>,
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

    pub fn resolve(&self, name: &str) -> Result<FieldInfo, FieldError> {
        if let Some((offset, ty)) = self.fields.get(name) {
            Ok(FieldInfo {
                offset: *offset,
                ty: *ty,
            })
        } else {
            Err(FieldError::UnknownField(name.to_string()))
        }
    }

    pub fn byte_length(&self) -> usize {
        self.length
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
                field_mappings.insert(field.name.clone(), (spaces[i].index, field.ty));
                spaces.remove(i);
                break 'field;
            } else if spaces[i].length > field.ty.size() {
                field_mappings.insert(field.name.clone(), (spaces[i].index, field.ty));
                spaces[i].length -= field.ty.size();
                spaces[i].index += field.ty.size();
                break 'field;
            }
        }

        // If we are here, no matching empty space has been found and the field will be layouted after all other fields
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

        field_mappings.insert(field.name.clone(), (length + alignment_space, field.ty));
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
    fields: *mut u8,
}

impl Fields {
    pub unsafe fn at(position: *mut u8) -> Self {
        Self {
            fields: position,
        }
    }

    pub unsafe fn init_from_layout_at(
        position: *mut u8,
        layout: &FieldLayout,
        descriptors: &[FieldDescriptor],
    ) -> Self {
        let mut fields = Self { fields: position };
        for field in descriptors {
            if let Some(constant_value) = field.constant_value {
                fields.set_value(
                    layout.resolve(&field.name).unwrap().offset,
                    field.ty,
                    constant_value,
                );
            }
        }
        fields
    }

    pub fn set_value(&mut self, offset: usize, ty: JvmType, value: JvmValue) {
        match ty {
            JvmType::Void => {}
            JvmType::Integer => self.set_int(offset, value.int()),
            JvmType::Long => self.set_long(offset, value.long()),
            JvmType::Float => self.set_float(offset, value.float()),
            JvmType::Double => self.set_double(offset, value.double()),
            JvmType::Reference => self.set_reference(offset, value.reference()),
            _ => todo!(),
        }
    }

    pub fn set_int(&mut self, offset: usize, value: JvmInt) {
        let bytes = value.0.to_be_bytes();
        unsafe {
            *self.fields.offset(offset as isize + 0) = bytes[0];
            *self.fields.offset(offset as isize + 1) = bytes[1];
            *self.fields.offset(offset as isize + 2) = bytes[2];
            *self.fields.offset(offset as isize + 3) = bytes[3];
        }
    }

    pub fn set_float(&mut self, offset: usize, value: JvmFloat) {
        let bytes = value.0.to_be_bytes();
        unsafe {
            *self.fields.offset(offset as isize + 0) = bytes[0];
            *self.fields.offset(offset as isize + 1) = bytes[1];
            *self.fields.offset(offset as isize + 2) = bytes[2];
            *self.fields.offset(offset as isize + 3) = bytes[3];
        }
    }

    pub fn set_long(&mut self, offset: usize, value: JvmLong) {
        let bytes = value.0.to_be_bytes();
        unsafe {
            *self.fields.offset(offset as isize + 0) = bytes[0];
            *self.fields.offset(offset as isize + 1) = bytes[1];
            *self.fields.offset(offset as isize + 2) = bytes[2];
            *self.fields.offset(offset as isize + 3) = bytes[3];
            *self.fields.offset(offset as isize + 4) = bytes[4];
            *self.fields.offset(offset as isize + 5) = bytes[5];
            *self.fields.offset(offset as isize + 6) = bytes[6];
            *self.fields.offset(offset as isize + 7) = bytes[7];
        }
    }

    pub fn set_double(&mut self, offset: usize, value: JvmDouble) {
        let bytes = value.0.to_be_bytes();
        unsafe {
            *self.fields.offset(offset as isize + 0) = bytes[0];
            *self.fields.offset(offset as isize + 1) = bytes[1];
            *self.fields.offset(offset as isize + 2) = bytes[2];
            *self.fields.offset(offset as isize + 3) = bytes[3];
            *self.fields.offset(offset as isize + 4) = bytes[4];
            *self.fields.offset(offset as isize + 5) = bytes[5];
            *self.fields.offset(offset as isize + 6) = bytes[6];
            *self.fields.offset(offset as isize + 7) = bytes[7];
        }
    }

    pub fn set_reference(&mut self, offset: usize, value: JvmReference) {
        let bytes = unsafe { value.0.into_raw().to_be_bytes() };
        unsafe {
            *self.fields.offset(offset as isize + 0) = bytes[0];
            *self.fields.offset(offset as isize + 1) = bytes[1];
            *self.fields.offset(offset as isize + 2) = bytes[2];
            *self.fields.offset(offset as isize + 3) = bytes[3];
            *self.fields.offset(offset as isize + 4) = bytes[4];
            *self.fields.offset(offset as isize + 5) = bytes[5];
            *self.fields.offset(offset as isize + 6) = bytes[6];
            *self.fields.offset(offset as isize + 7) = bytes[7];
        }
    }

    pub fn get_int(&self, offset: usize) -> JvmInt {
        unsafe {
            JvmInt(i32::from_be_bytes([
                *self.fields.offset(offset as isize + 0),
                *self.fields.offset(offset as isize + 1),
                *self.fields.offset(offset as isize + 2),
                *self.fields.offset(offset as isize + 3),
            ]))
        }
    }

    pub fn get_long(&self, offset: usize) -> JvmLong {
        unsafe {
            JvmLong(i64::from_be_bytes([
                *self.fields.offset(offset as isize + 0),
                *self.fields.offset(offset as isize + 1),
                *self.fields.offset(offset as isize + 2),
                *self.fields.offset(offset as isize + 3),
                *self.fields.offset(offset as isize + 4),
                *self.fields.offset(offset as isize + 5),
                *self.fields.offset(offset as isize + 6),
                *self.fields.offset(offset as isize + 7),
            ]))
        }
    }

    pub fn get_float(&self, offset: usize) -> JvmFloat {
        unsafe {
            JvmFloat(f32::from_be_bytes([
                *self.fields.offset(offset as isize + 0),
                *self.fields.offset(offset as isize + 1),
                *self.fields.offset(offset as isize + 2),
                *self.fields.offset(offset as isize + 3),
            ]))
        }
    }

    pub fn get_double(&self, offset: usize) -> JvmDouble {
        unsafe {
            JvmDouble(f64::from_be_bytes([
                *self.fields.offset(offset as isize + 0),
                *self.fields.offset(offset as isize + 1),
                *self.fields.offset(offset as isize + 2),
                *self.fields.offset(offset as isize + 3),
                *self.fields.offset(offset as isize + 4),
                *self.fields.offset(offset as isize + 5),
                *self.fields.offset(offset as isize + 6),
                *self.fields.offset(offset as isize + 7),
            ]))
        }
    }

    pub fn get_reference(&self, offset: usize) -> JvmReference {
        unsafe {
            JvmReference(HeapIndex::from_raw(u64::from_be_bytes([
                *self.fields.offset(offset as isize + 0),
                *self.fields.offset(offset as isize + 1),
                *self.fields.offset(offset as isize + 2),
                *self.fields.offset(offset as isize + 3),
                *self.fields.offset(offset as isize + 4),
                *self.fields.offset(offset as isize + 5),
                *self.fields.offset(offset as isize + 6),
                *self.fields.offset(offset as isize + 7),
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

pub unsafe fn init_fields_at(position: *mut u8, layout: FieldLayout, fields: &[FieldDescriptor]) {
    for field in fields {}
}

#[derive(thiserror::Error, Debug)]
pub enum FieldError {
    #[error("No field with name {0}")]
    UnknownField(String),
}
