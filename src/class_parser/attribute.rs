pub use attribute::*;

#[allow(dead_code)]
pub mod attribute {
    pub const CONSTANT_VALUE: &str = "ConstantValue";
    pub const CODE: &str = "Code";
    pub const STACK_MAP_TABLE: &str = "StackMapTable";
    pub const BOOTSTRAP_METHODS: &str = "BootstrapMethods";
    pub const NEST_HOST: &str = "NestHost";
    pub const NEST_MEMBERS: &str = "NestMembers";
    pub const PERMITTED_SUBCLASSES: &str = "PermittedSubclasses";
    pub const EXCEPTIONS: &str = "Exceptions";
    pub const INNER_CLASSES: &str = "InnerClasses";
    pub const ENCLOSING_METHOD: &str = "EnclosingMethod";
    pub const SYNTHETIC: &str = "Synthetic";
    pub const SIGNATURE: &str = "Signature";
    pub const RECORD: &str = "Record";
    pub const SOURCE_FILE: &str = "SourceFile";
    pub const LINE_NUMBER_TABLE: &str = "LineNumberTable";
    pub const LOCAL_VARIABLE_TABLE: &str = "LocalVariableTable";
    pub const LOCAL_VARIABLE_TYPE_TABLE: &str = "LocalVariableTypeTable";
    pub const SOURCE_DEBUG_EXTENSIONS: &str = "SourceDebugExtensions";
    pub const DEPRECATED: &str = "Deprecated";
    pub const RUNTIME_VISIBLE_ANNOTATIONS: &str = "RuntimeVisibleAnnotations";
    pub const RUNTIME_INVISIBLE_ANNOTATIONS: &str = "RuntimeInvisibleAnnotations";
    pub const RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS: &str = "RuntimeVisibleParameterAnnotations";
    pub const RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS: &str =
        "RuntimeInvisibleParameterAnnotations";
    pub const RUNTIME_VISIBLE_TYPE_ANNOTATIONS: &str = "RuntimeVisibleTypeAnnotations";
    pub const RUNTIME_INVISIBLE_TYPE_ANNOTATIONS: &str = "RuntimeInvisibleTypeAnnotations";
    pub const ANNOTATION_DEFAULT: &str = "AnnotationDefault";
    pub const METHOD_PARAMETERS: &str = "MethodParameters";
    pub const MODULE: &str = "Module";
    pub const MODULE_PACKAGES: &str = "ModulePackages";
    pub const MODULE_MAIN_CLASS: &str = "ModuleMainClass";
}
