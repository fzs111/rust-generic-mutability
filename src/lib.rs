#![no_std]

#![warn(clippy::undocumented_unsafe_blocks)]

mod erased_mut_ref;
mod mutability;
mod genref;

pub use erased_mut_ref::ErasedMutRef;
pub use mutability::{ Mutability, Mutable, Shared, MutabilityEnum, IsShared, IsMutable };
pub use genref::GenRef;