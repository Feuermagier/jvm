use crate::model::value::JvmValue;

use super::{types::JvmType, visibility::Visibility};

#[derive(Debug)]
pub struct FieldDescriptor {
    pub name: String,
    pub visibility: Visibility,
    pub ty: JvmType,
    pub constant_value: Option<JvmValue>,
}
