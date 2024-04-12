#![no_std]

#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::missing_safety_doc)]
#![deny(unsafe_op_in_unsafe_fn)]

//! This crate enables the creation of functions, methods, traits or even structs generic over mutability.
//! 
//! The main use case is allowing crates to write pairs of getter functions of the form
//! ```rust,ignore
//! fn get<'a>(&'a T, ...) -> &'a U
//! fn get_mut<'a>(&'a mut T, ...) -> &'a mut U
//! ```
//! as a single function
//! ```rust,ignore
//! fn get_gen<'a, M: Mutability>(GenRef<'a, M, T>, ...) -> GenRef<'a, M, U>
//! ```

mod mutability;
mod genref;
mod macros;

pub use mutability::{ Mutability, Mutable, Shared, MutabilityEnum, IsShared, IsMutable };
pub use genref::{ GenRef, GenRefMethods };