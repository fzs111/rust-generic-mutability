#![no_std]

#![forbid(unsafe_op_in_unsafe_fn)]

//! This crate enables to create methods, traits or even structs generic over mutability.
//! 
//! The biggest use case is allowing crates to write pairs of getter functions of the form
//! ```
//! fn get<'a>(&'a T, ...) -> &'a U
//! fn get_mut<'a>(&'a mut T, ...) -> &'a mut U
//! ```
//! as a single function
//! ```
//! fn get_gen<'a, M: Mutability>(GenRef<'a, M, T>, ...) -> GenRef<'a, M, U>
//! ```
//! 
//! # Usage for libraries
//! 
//! You should start by introducing a *mutability parameter*. It is a normal generic argument (usually named `M`) with a `M: Mutability` bound on it.
//! 
//! Then you can take a `GenRef` as an argument, which is the generic-mutability equivalent of safe reference types (`&` and `&mut`).
//! 
//! `GenRef` provides immutable access to te pointed-to value using the `as_immut` method.
//! 
//! Then, to map the generic reference into one of another type, you can do one of these:
//! 
//! - If the API you are calling has generic mutability accessors, you can pass the `GenRef` directly to them.
//!     The `reborrow` and `call` methods may come in handy.
//! - If the API you're calling *does not* has generic mutability, you can use one of the following to unwrap and reconstruct the `GenRef`:
//!     - `map`
//!     - `map_to_option` to return an optional value
//!     - `split` to do borrow splitting
//!     - unwrap with `dispatch` or `Into<GenRefEnum<'_, M, T>>` and reconstruct with 

mod mutability;
mod genref;
mod genref_enum;
mod genref_macro;
mod compile_fail_tests;

pub use mutability::{ Mutability, Immutable, Mutable };
pub use genref::GenRef;
pub use genref_enum::{ GenRefEnum, IncorrectMutability };