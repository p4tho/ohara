use lopdf::{ Object };

pub fn obj_to_f32(obj: &Object) -> f32 {
    match obj {
        Object::Real(v) => *v,
        Object::Integer(v) => *v as f32,
        _ => 0.0,
    }
}