#![no_std]

#![warn(clippy::undocumented_unsafe_blocks)]

mod erased_mutability_ref;
mod mutability;
mod genref;
mod macros;

pub use erased_mutability_ref::ErasedMutabilityRef;
pub use mutability::{ Mutability, Mutable, Shared, MutabilityEnum, IsShared, IsMutable };
pub use genref::GenRef;