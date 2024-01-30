#![no_std]

#![forbid(unsafe_op_in_unsafe_fn)]

mod mutability;
mod genref;
mod genref_enum;
mod genref_macro;
mod compile_fail_tests;

pub use mutability::{ Mutability, Immutable, Mutable };
pub use genref::GenRef;
pub use genref_enum::{ GenRefEnum, IncorrectMutability };