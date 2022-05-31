use std::any::Any;

use serde::Deserialize;

pub trait Value {
    fn is_foreign() -> bool { false }
    fn size() -> usize;
}

impl Value for i32 {
    fn size() -> usize {
        4
    }
}

impl Value for bool {
    fn size() -> usize {
        1
    }
}

impl Value for f32 {
    fn size() -> usize {
        4
    }
}