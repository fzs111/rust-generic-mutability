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
//!
//! The main items of this crate are the `GenRef` struct, which represents a safe reference (like `&` and `&mut`) that is generic over mutability; and the `Mutability` trait, which is used as a bound on *generic mutability parameters*.

mod conv_traits;
mod gen_struct;
mod genref;
mod macros;
mod mutability;

pub use conv_traits::{GenFrom, GenInto};
pub use genref::genref_methods::GenRefMethods;
pub use genref::GenRef;
pub use mutability::{IsMutable, IsShared, Mutability, MutabilityEnum, Mutable, Shared};
