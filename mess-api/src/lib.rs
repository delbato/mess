#![warn(missing_docs)]

pub mod container;

pub mod function;

pub mod interface;

pub mod module;

pub mod value;

pub mod var_type;

#[cfg(feature = "exec-vm")]
pub mod adapter;

pub mod prelude {
    pub use super::container::Container;
    pub use super::function::Function;
    pub use super::interface::Interface;
    pub use super::module::Module;
    pub use super::value::{ Value };
    pub use super::var_type::Type;
    #[cfg(feature = "exec-vm")]
    pub use super::adapter::{ Adapter, AdapterImpl };
}