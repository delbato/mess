#![warn(missing_docs)]

extern crate enum_primitive_derive as epd;

pub mod codegen;

pub mod exec;

pub use codegen::compiler::Compiler;
pub use exec::core::Core;
