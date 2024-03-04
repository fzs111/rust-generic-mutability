#![no_std]

#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::missing_safety_doc)]
#![deny(unsafe_op_in_unsafe_fn)]

mod erased_mut_ref;
mod mutability;
mod genref;
mod macros;

pub use erased_mut_ref::ErasedMutRef;
pub use mutability::{ Mutability, Mutable, Shared, MutabilityEnum, IsShared, IsMutable };
pub use genref::GenRef;