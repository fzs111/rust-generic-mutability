#![no_std]

#![warn(clippy::undocumented_unsafe_blocks)]

mod mutability;
mod genref;
mod macros;

pub use mutability::{ Mutability, Mutable, Shared, MutabilityEnum, IsShared, IsMutable };
pub use genref::{ GenRef, GenRefMethods };