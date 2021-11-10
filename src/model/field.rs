use crate::model::value::JvmValue;

use super::{class::ConstantPoolIndex, types::JvmType, visibility::Visibility};

pub struct FieldDescriptor {
    pub name: String,
    pub visibility: Visibility,
    pub ty: JvmType,
    pub constant_value: Option<JvmValue>,
}
