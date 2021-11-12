mod attribute;
mod iterator;

use std::str::Utf8Error;

use unicode_segmentation::UnicodeSegmentation;

use crate::{class_parser::iterator::ClassFileIterator, model::{class::{ClassIndex, LoadedClasses}, class_file::ClassFile, constant_pool::{ConstantPool, ConstantPoolEntry, ConstantPoolError, ConstantPoolIndex}, field::FieldDescriptor, method::{Method, MethodCode}, types::JvmType, value::JvmValue, visibility::Visibility}};

pub fn parse(
    bytes: &[u8],
    classes: &mut LoadedClasses,
) -> Result<(ClassFile, ClassIndex), ParsingError> {
    let mut iter = ClassFileIterator::new(bytes);

    // Magic number
    if iter.u32()? != 0xCAFEBABE {
        return Err(ParsingError::MissingMagicNumber);
    }

    // Version info
    let minor_version = iter.u16()?;
    let major_version = iter.u16()?;

    // Constant pool
    let constant_pool = parse_constants(&mut iter).unwrap();

    // Visibility
    let access_flags = iter.u16()?;
    let visibility = Visibility::from_access_flags(access_flags);

    // This
    let this_class: ConstantPoolIndex = iter.u16()?.into();

    // Super
    let super_class: ConstantPoolIndex = iter.u16()?.into();

    // Interfaces
    let interface_count = iter.u16()? as usize;
    let mut interfaces: Vec<ConstantPoolIndex> = Vec::with_capacity(interface_count);
    for _ in 0..interface_count {
        interfaces.push(iter.u16()?.into());
    }

    // Fields
    let (static_fields, fields) = parse_fields(&mut iter, &constant_pool)?;

    // Methods
    let (static_methods, methods) = parse_methods(&mut iter, &constant_pool)?;

    // Attributes
    parse_attributes(&mut iter, &&constant_pool, |_, _, _| Ok(false))?;

    // Create the actual class file
    let class_file = ClassFile::new(minor_version, major_version);

    // Create the actual class
    let class = classes.load(
        constant_pool,
        visibility,
        this_class,
        super_class,
        interfaces,
        static_fields,
        fields,
        static_methods,
        methods,
    )?;

    Ok((class_file, class))
}

fn parse_constants(iter: &mut ClassFileIterator) -> Result<ConstantPool, ParsingError> {
    let count = iter.u16()? - 1; // For some obscure reason the number in the class file is the size of the constant pool plus one
    let mut constants = Vec::with_capacity(count as usize);

    let mut i = 0; // We can't use for because some entries requires us to skip the next entry
    while i < count {
        let tag = iter.byte()?;
        match tag {
            // CONSTANT_Utf8
            1 => {
                let length = iter.u16()? as usize;
                let name = std::str::from_utf8(iter.take_bytes(length)?)
                    .map_err(|err| ParsingError::InvalidUtf8Constant(i, err))?;
                constants.push(ConstantPoolEntry::Utf8(name.to_string()));
            }

            // CONSTANT_Integer
            3 => constants.push(ConstantPoolEntry::Integer(iter.i32()?)),

            // CONSTANT_Float
            4 => constants.push(ConstantPoolEntry::Float(iter.f32()?)),

            // CONSTANT_Long
            5 => {
                constants.push(ConstantPoolEntry::Long(iter.i64()?));
                constants.push(ConstantPoolEntry::Empty);
                i += 1; // Longs and doubles take up two slots
            }

            // CONSTANT_Double
            6 => {
                constants.push(ConstantPoolEntry::Double(iter.f64()?));
                constants.push(ConstantPoolEntry::Empty);
                i += 1; // Longs and doubles take up two slots
            }

            // CONSTANT_Class
            7 => constants.push(ConstantPoolEntry::Class {
                name: iter.u16()?.into(),
            }),

            // CONSTANT_Fieldref
            9 => constants.push(ConstantPoolEntry::FieldReference {
                class: iter.u16()?.into(),
                name_and_type: iter.u16()?.into(),
            }),

            // CONSTANT_Methodref
            10 => constants.push(ConstantPoolEntry::MethodReference {
                class: iter.u16()?.into(),
                name_and_type: iter.u16()?.into(),
            }),

            // CONSTANT_InterfaceMethodref
            11 => constants.push(ConstantPoolEntry::InterfaceMethodReference {
                class: iter.u16()?.into(),
                name_and_type: iter.u16()?.into(),
            }),

            // CONSTANT_NameAndType
            12 => constants.push(ConstantPoolEntry::NameAndType {
                name: iter.u16()?.into(),
                ty: iter.u16()?.into(),
            }),

            _ => return Err(ParsingError::UnknownConstantTag(tag)),
        }
        i += 1;
    }

    Ok(ConstantPool::new(constants))
}

fn parse_fields(
    iter: &mut ClassFileIterator,
    constant_pool: &ConstantPool,
) -> Result<(Vec<FieldDescriptor>, Vec<FieldDescriptor>), ParsingError> {
    let mut static_fields = Vec::new();
    let mut fields = Vec::new();

    let field_count = iter.u16()?;
    for _ in 0..field_count {
        let access_flags = iter.u16()?;
        let visibility = Visibility::from_access_flags(access_flags);

        let name_index = iter.u16()?;
        let name = constant_pool.get_utf8(name_index.into())?.to_string();

        let descriptor_index = iter.u16()?;
        let type_string = constant_pool.get_utf8(descriptor_index.into())?;
        let ty = parse_field_type(type_string)?;

        let mut constant_value = None;

        parse_attributes(iter, constant_pool, |attr_name, _, iter| {
            match attr_name {
                attribute::ConstantValue => {
                    let value_index = iter.u16()?;
                    let constant = constant_pool.get(value_index.into())?;
                    let value = match constant {
                        ConstantPoolEntry::Integer(value) => JvmValue::Int(*value),
                        ConstantPoolEntry::Long(value) => JvmValue::Long(*value),
                        ConstantPoolEntry::Float(value) => JvmValue::Float(*value),
                        ConstantPoolEntry::Double(value) => JvmValue::Double(*value),
                        //TODO parse strings
                        _ => {
                            return Err(ParsingError::InvalidConstantValue(format!(
                                "{:?}",
                                constant
                            )))
                        }
                    };
                    constant_value = Some(value);
                    Ok(true)
                }
                _ => Ok(false),
            }
        })?;

        if access_flags & 0x0008 != 0 {
            // ACC_STATIC
            static_fields.push(FieldDescriptor {
                name,
                visibility,
                ty,
                constant_value,
            });
        } else {
            fields.push(FieldDescriptor {
                name,
                visibility,
                ty,
                constant_value,
            });
        }
    }

    Ok((static_fields, fields))
}

fn parse_field_type(string: &str) -> Result<JvmType, ParsingError> {
    let mut iter = string.graphemes(true);
    let tag = iter.next();
    if tag.is_none() {
        return Err(ParsingError::InvalidFieldType(string.to_string()));
    }

    match tag.unwrap() {
        "B" => Ok(JvmType::Byte),
        "C" => Ok(JvmType::Char),
        "D" => Ok(JvmType::Double),
        "F" => Ok(JvmType::Float),
        "I" => Ok(JvmType::Integer),
        "J" => Ok(JvmType::Long),
        "S" => Ok(JvmType::Long),
        "Z" => Ok(JvmType::Long),
        "L" => {
            let class = iter.take_while(|c| *c != ";").collect::<String>();
            //Ok(JvmType::Reference(class))
            Ok(JvmType::Reference)
        }
        _ => Err(ParsingError::InvalidFieldType(string.to_string())),
    }
}

fn parse_methods(
    iter: &mut ClassFileIterator,
    constant_pool: &ConstantPool,
) -> Result<(Vec<Method>, Vec<Method>), ParsingError> {
    let mut methods = Vec::new();
    let mut static_methods = Vec::new();

    let method_count = iter.u16()?;
    for _ in 0..method_count {
        let access_flags = iter.u16()?;
        let visibility = Visibility::from_access_flags(access_flags);

        let name_index = iter.u16()?;
        let name = constant_pool.get_utf8(name_index.into())?.to_string();
        dbg!(&name);

        let descriptor_index = iter.u16()?;
        let descriptor = constant_pool.get_utf8(descriptor_index.into())?.to_string();

        let mut code = None;
        let mut max_stack = 0;
        let mut max_locals = 0;
        parse_attributes(iter, constant_pool, |attribute_name, _, iter| {
            match attribute_name {
                attribute::Code => {
                    max_stack = iter.u16()? as usize;
                    max_locals = iter.u16()? as usize;
                    let code_length = iter.u32()?;
                    code = Some(iter.take_bytes(code_length as usize)?.to_vec());
                    let exception_table_length = iter.u16()?;
                    //TODO
                    // Skip the exception table for now
                    iter.skip_bytes(exception_table_length as usize * 8)?;

                    //TODO
                    // Skip the attributes
                    parse_attributes(iter, constant_pool, |_, _, _| Ok(false))?;

                    Ok(true)
                }
                _ => Ok(false),
            }
        })?;

        let code = if let Some(bytecode) = code {
            MethodCode::Bytecode(bytecode)
        } else if is_native(access_flags) {
            MethodCode::Native(None)
        } else {
            return Err(ParsingError::MissingCode(name));
        };

        let method = Method {
            name,
            descriptor,
            visibility,
            max_locals,
            max_stack,
            code,
        };
        dbg!(&method);

        if access_flags & 0x0008 != 0 {
            // ACC_STATIC
            static_methods.push(method);
        } else {
            methods.push(method);
        }
    }

    Ok((static_methods, methods))
}

fn parse_attributes<H>(
    iter: &mut ClassFileIterator,
    constant_pool: &ConstantPool,
    mut handler: H,
) -> Result<(), ParsingError>
where
    H: FnMut(&str, usize, &mut ClassFileIterator) -> Result<bool, ParsingError>,
{
    let count = iter.u16()?;
    for _ in 0..count {
        let name_index = iter.u16()?;
        let name = constant_pool.get_utf8(name_index.into())?;
        let length = iter.u32()? as usize;
        if !handler(name, length, iter)? {
            log::info!("Skipping attribute '{}'", name);
            iter.take_bytes(length)?;
        }
    }
    Ok(())
}

fn is_native(access_flags: u16) -> bool {
    access_flags & 0x0100 != 0
}

#[derive(thiserror::Error, Debug)]
pub enum ParsingError {
    #[error("mising magic number")]
    MissingMagicNumber,

    #[error("unexpected end of file")]
    UnexpectedEOF,

    #[error("unknown constant tag {0}")]
    UnknownConstantTag(u8),

    #[error("invalid utf string at constant index {0}: {1}")]
    InvalidUtf8Constant(u16, Utf8Error),

    #[error("invalid field type {0}")]
    InvalidFieldType(String),

    #[error("unexpected attribute while reading an element of type '{0}': {1}")]
    UnexpectedAttribute(String, String),

    #[error("the constant value is of the invalid type {0}")]
    InvalidConstantValue(String),

    #[error("no code attribute found for methode {0}")]
    MissingCode(String),

    #[error("constant pool error")]
    ConstantPool {
        #[from]
        source: ConstantPoolError,
    },
}
