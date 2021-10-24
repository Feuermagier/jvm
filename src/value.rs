pub enum JvmValue {
    Int { val: i32 },
    Long { val: i64 },
    Float { var: f32 },
    Double { var: f64 }
}