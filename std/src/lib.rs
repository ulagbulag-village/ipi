#![feature(trivial_bounds)]

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate rkyv;

pub mod attention;
pub mod class;
pub mod class_cursor;
pub mod class_data;
pub mod data;
pub mod path;
pub mod storage;

// re-export core
pub use ::ipi_core as core;

// derived types
// pub use self::class::Class;
// #[cfg(feature = "ipi-std-derive")]
// pub use ::ipi_std_derive::Class;
