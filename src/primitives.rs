//! This module contains primitives for working with *structures*.
//! 
//! # Structures
//! 
//! Structures are a way to encode arbitrary data, while allowing to convert certain parts between references (`&`/`&mut`), `NonNull` pointers and `GenRef` references.
//! 
//! Structures come in 3 flavors, reference-strucures, `NonNull`-structures and `GenRef`-structures.
//! 
//! These conversions are available:
//! 
//! - The `StructureMutIntoNonNull` and `StructureImmutIntoNonNull` traits convert reference-structures into `NonNull`-structures, erasing the mutability information in the process.
//! - The `StructureNonNullIntoGenRef` trait unsafely converts a `NonNull`-structure into a `GenRef`-structure.
//! - `GenRef`-structures cannot be converted further, they are designed to be destructured/matched on by user code.
//! 
//! The following types are considered structures:
//! 
//! - Flavored data (references in reference-structures, `NonNull` in `NonNull`-structures and `GenRef` in `GenRef`-structures).
//! - `Untouched(T)`: Contains arbitrary data that is unaffected by structure conversions. If contains flavored data, it will not be converted.
//! - Binary tuples `(S1, S2)` where `S1` and `S2` are structures. Structs and n-ary tuples can be encoded as a binary tree of tuples `(T, (U, (V, ...)))`.
//! - Two-variant enum `OneOf<S1, S2>` where `S1` and `S2` are structures. n-variant enums can be encoded as a binary-tree-like structure of `OneOf` enums `OneOf<T, OneOf<U, OneOf<V, ...>>>`
//! - The unit type `()`.
//! 
//! Note that in all reference- and `GenRef`-structures, the mutability and lifetime of each reference must be the same.
//! 
//! Structures are only necessary because there's no widespread support for generic mutability, so round-tripping between regular references and `GenRef`s is required.
//! With sufficient library and compiler support, generic mutability can be just as easy to use as working with regular references.
//! 
//! ## Examples of conversions:
//! 
//! ```rust, ignore
//! struct Foo<'a>{
//!     bar: i32,
//!     slice: &'a mut [u8]
//! }
//! 
//! // can be represented with the following mutable reference-structure:
//! 
//! (Untouched<i32>, &mut [u8])
//! 
//! // ...which can be converted into a `NonNull`-structure (via `StructureMutIntoNonNull`):
//! 
//! (Untouched<i32>, NonNull<[u8]>)
//! 
//! // ...which can be converted into a `GenRef`-structure (via `StructureNonNullIntoGenRef`):
//! 
//! (Untouched<i32>, GenRef<'_, M, [u8]>)
//! ```
//! 
//! ```rust, ignore
//! Result<&MyStructEntry, MyError>
//! // could become
//! OneOf<&MyStructEntry, MyError>
//! // ...which can be converted to:
//! OneOf<NonNull<MyStructEntry>, MyError>
//! // ...and then:
//! OneOf<GenRef<'_, M, MyStructEntry>, MyError>
//! // ...and finally it can be turned back into a `Result`:
//! Result<GenRef<'_, M, MyStructEntry>, MyError>
//! ```
//! 
//! ```rust, ignore
//! struct U64s<'a>{
//!     first: &'a mut u64, 
//!     second: &'a mut u64, 
//!     option: Option<&'a mut u64>,
//!     dont_unwrap: &'static u64
//! }
//! // ↓
//! (&'a mut u64, (&'a mut u64, (OneOf<&'a mut u64, ()>, Untouched<&'static u64>)))
//! // ↓
//! (NonNull<u64>, (NonNull<u64>, (OneOf<NonNull<u64>, ()>, Untouched<&'static u64>)))
//! // ↓
//! (GenRef<'a, M, u64>, (GenRef<'a, M, u64>, (OneOf<GenRef<'a, M, u64>, ()>, Untouched<&'static u64>)))
//! ```
//! 
//! ```rust, ignore
//! enum Message<'a> {
//!     Quit,
//!     Move { x: i32, y: i32 },
//!     Write(&'a str),
//!     ChangeColor(i32, i32, i32),
//! }
//! // may become this (note that the tree the enums form may be balanced):
//! OneOf<OneOf<(), Untouched<(i32, i32)>>, OneOf<&'a str, Untouched<(i32, i32, i32)>>>
//! // ↓
//! OneOf<OneOf<(), Untouched<(i32, i32)>>, OneOf<NonNull<str>, Untouched<(i32, i32, i32)>>>
//! // ↓
//! OneOf<OneOf<(), Untouched<(i32, i32)>>, OneOf<GenRef<'_, M, str>, Untouched<(i32, i32, i32)>>>


use core::ptr::NonNull;

use crate::{GenRef, Mutability};

/// Primitive for encoding enums as structures.
/// 
/// n-variant enums can be encoded as a binary tree-like structure of `OneOf` enums.
/// Such a binary tree may be either balanced (`OneOf<OneOf<T, U>, OneOf<V, W>>`) or degenerate (`OneOf<T, OneOf<U, OneOf<V, ...>>>`).
/// 
/// See the module documentation for more details.
pub enum OneOf<T, U>{
    First(T),
    Second(U)
}

/// Primitive for inserting arbitrary data inside structures.
/// 
/// Data wrapped in `Untouched` will not be changed by structure conversions.
/// 
/// See the module documentation for more details.
pub struct Untouched<T>(pub T);

macro_rules! impl_ref_into_nonnull {
    ($trait:ident, $mut_or_not:ident, $mut_or_not_article:literal, $mut_or_not_human_readable:literal) => {
        #[doc = "Converts "]
        #[doc = $mut_or_not_article]
        #[doc = $mut_or_not_human_readable]
        #[doc = " reference-structure into a `NonNull`-structure. See the module documentation.\n\n"]
        #[doc = "# Safety\n\n"]
        #[doc = "All `NonNull` pointers in the result (excluding inside `Untouched`) must be valid for "]
        #[doc = $mut_or_not_human_readable]
        #[doc = " accesses for the lifetime `'a`."]
        pub unsafe trait $trait<'a>{
            type Output;
            fn into_nonnull_structure(ref_structure: Self) -> Self::Output;
        }

        unsafe impl<'a, T: ?Sized> $trait<'a> for $mut_or_not<'a, T>
        {
            type Output = NonNull<T>;
            fn into_nonnull_structure(reference: Self) -> NonNull<T> {
                NonNull::from(reference)
            }
        }

        unsafe impl<'a> $trait<'a> for () {
            type Output = ();
            fn into_nonnull_structure((): Self) -> () {
                ()
            }
        }

        unsafe impl<'a, T, U> $trait<'a> for (T, U)
            where 
                T: $trait<'a>,
                U: $trait<'a>
        {
            type Output = (T::Output, U::Output);
            fn into_nonnull_structure((t, u): Self) -> (T::Output, U::Output) {
                ($trait::into_nonnull_structure(t), $trait::into_nonnull_structure(u))
            }
        }

        unsafe impl<'a, T, U> $trait<'a> for OneOf<T, U>
            where 
                T: $trait<'a>,
                U: $trait<'a>
        {
            type Output = OneOf<T::Output, U::Output>;
            fn into_nonnull_structure(one_of: Self) -> OneOf<T::Output, U::Output> {
                match one_of {
                    OneOf::First(t) => OneOf::First($trait::into_nonnull_structure(t)),
                    OneOf::Second(u) => OneOf::Second($trait::into_nonnull_structure(u))
                }
            }
        }

        unsafe impl<'a, T> $trait<'a> for Untouched<T>
        {
            type Output = Untouched<T>;
            fn into_nonnull_structure(t: Self) -> Untouched<T> {
                t
            }
        }
    }
}

// These type aliases make the macro a bit more readable to implement
type MutRef<'a, T> = &'a mut T;
type ImmutRef<'a, T> = &'a T;

impl_ref_into_nonnull!(StructureMutIntoNonNull,   MutRef,   "a ",  "mutable");
impl_ref_into_nonnull!(StructureImmutIntoNonNull, ImmutRef, "an ", "immutable");

/// Converts a `NonNull`-structure into a `GenRef`-structure.
/// 
/// # Safety
/// 
/// All implementations must conform to the contract of the `into_genref_structure` function.
pub unsafe trait StructureNonNullIntoGenRef<'a, M: Mutability>{
    type Output;

    /// Converts a `NonNull`-structure into a `GenRef`-structure.
    /// 
    /// # Safety
    /// 
    /// All `NonNull` pointers in the input structure (except inside `Untouched`) must satisfy the requirements of `GenRef::new` for lifetime `'a` and mutability `M`.
    unsafe fn into_genref_structure(nonnull_structure: Self) -> Self::Output;
}

unsafe impl<'a, M: Mutability, T: 'a + ?Sized> StructureNonNullIntoGenRef<'a, M> for NonNull<T>
{
    type Output = GenRef<'a, M, T>;
    unsafe fn into_genref_structure(nonnull: Self) -> GenRef<'a, M, T> {
        GenRef::new(nonnull)
    }
}

unsafe impl<'a, M: Mutability> StructureNonNullIntoGenRef<'a, M> for () {
    type Output = ();
    unsafe fn into_genref_structure((): Self) -> () {
        ()
    }
}

unsafe impl<'a, M: Mutability, T, U> StructureNonNullIntoGenRef<'a, M> for (T, U)
    where 
        T: StructureNonNullIntoGenRef<'a, M>,
        U: StructureNonNullIntoGenRef<'a, M>
{
    type Output = (T::Output, U::Output);
    unsafe fn into_genref_structure((t, u): Self) -> (T::Output, U::Output) {
        (StructureNonNullIntoGenRef::into_genref_structure(t), StructureNonNullIntoGenRef::into_genref_structure(u))
    }
}

unsafe impl<'a, M: Mutability, T, U> StructureNonNullIntoGenRef<'a, M> for OneOf<T, U>
    where 
        T: StructureNonNullIntoGenRef<'a, M>,
        U: StructureNonNullIntoGenRef<'a, M>
{
    type Output = OneOf<T::Output, U::Output>;
    unsafe fn into_genref_structure(one_of: Self) -> OneOf<T::Output, U::Output> {
        match one_of {
            OneOf::First(t) => OneOf::First(StructureNonNullIntoGenRef::into_genref_structure(t)),
            OneOf::Second(u) => OneOf::Second(StructureNonNullIntoGenRef::into_genref_structure(u))
        }
    }
}

unsafe impl<'a, M: Mutability, T> StructureNonNullIntoGenRef<'a, M> for Untouched<T>
{
    type Output = Untouched<T>;
    unsafe fn into_genref_structure(t: Self) -> Untouched<T> {
        t
    }
}