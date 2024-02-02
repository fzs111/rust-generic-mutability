#![no_std]

mod mutability;
mod genref;
mod genref_enum;
mod genref_macro;
mod compile_fail_tests;

pub mod primitives;

pub use mutability::{ Mutability, Immutable, Mutable };
pub use genref::GenRef;
pub use genref_enum::{ GenRefEnum, IncorrectMutability };