#![no_std]

#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::missing_safety_doc)]
#![deny(unsafe_op_in_unsafe_fn)]

//! This crate enables to create methods, traits or even structs generic over mutability.
//! 
//! The main use case is allowing crates to write pairs of getter functions of the form
//! ```
//! fn get<'a>(&'a T, ...) -> &'a U
//! fn get_mut<'a>(&'a mut T, ...) -> &'a mut U
//! ```
//! as a single function
//! ```
//! fn get_gen<'a, M: Mutability>(GenRef<'a, M, T>, ...) -> GenRef<'a, M, U>
//! ```
//! 
//! # Usage for end users
//! 
//! *You need to interface with an API that is generic over mutability, but you do not use generic mutability to call it.*
//! 
//! You will need to create a `GenRef<'_, Mutable, T>` or `GenRef<'_, Shared, T>` via the `From<&mut T>` or `From<&T>` implementations, respectively.
//! Then pass the `GenRef` to the function(s) you would like to call.
//! At the end, you can use the resulting `GenRef` mostly the same way you would use a reference, but you can also unwrap it into a normal reference using the `into_mut` or `into_shared` functions.
//! 
//! # Usage for libraries
//! 
//! *You are implementing an API that is supposed to be generic over mutability.*
//! 
//! You should start by introducing a *mutability parameter*. It is a generic argument (usually named `M`) with a `M: Mutability` bound on it .
//! 
//! Then you can take a `GenRef` as an argument, which is the generic-mutability equivalent of safe reference types (`&` and `&mut`).
//! 
//! `GenRef` always provides immutable access to te pointed-to value through the `Deref` trait.
//! 
//! Then, to map the generic reference into one of another type, you can do one of these:
//! 
//! - If the API you are calling has generic mutability accessors, you can pass the `GenRef` directly to them.
//!     Unlike normal references, which are automatically reborrowed, you may need to use `GenRef::reborrow` to perform a reborrow. 
//!     You can also call `as_deref` to perform a dereference.
//! 
//! - If the API you're calling *does not* have generic mutability, you can use one of the following ways to unwrap and reconstruct the `GenRef`:
//!     - `map`
//!     - `field!` macro for accessing fields
//!     - `gen_mut!` macro
//!     - branch to cases on `Mutability::mutability()` and use `gen_{into,from}_{mut,shared}` with the proof provided by the return value of `Mutability::mutability()`. 
//!         See the Examples section on how to do this.


mod erased_mut_ref;
mod mutability;
mod genref;
mod macros;

pub use erased_mut_ref::ErasedMutRef;
pub use mutability::{ Mutability, Mutable, Shared, MutabilityEnum, IsShared, IsMutable };
pub use genref::GenRef;