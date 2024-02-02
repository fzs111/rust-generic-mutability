use core::ptr::NonNull;
use core::fmt::{ Debug, Display };

use crate::{ Mutability, GenRef };

/// This enum allows to erase the statically known mutability of a `GenRef`.
/// It doesn't have helper methods, you are supposed to `match` over it and reconstruct it when necessary.
/// Using this type has some (small) dynamic overhead compared to the entirely zero-cost `GenRef`, but it gives a little more flexibility.
///
/// This type has a `From<GenRef<'_, M, T>>` implementation, while `GenRef` has a `TryFrom<GenRefEnum<'_, T>>`.
#[derive(Debug)]
pub enum GenRefEnum<'s, T: ?Sized>{
    Mutable(&'s mut T),
    Immutable(&'s T),
}

impl<'a, M: Mutability, T: ?Sized> From<GenRef<'a, M, T>> for GenRefEnum<'a, T> {
    fn from(genref: GenRef<'a, M, T>) -> Self {
        genref.dispatch(
            |r| GenRefEnum::Mutable(r),
            |r| GenRefEnum::Immutable(r),
        )
    }
}

/// Error type for `TryFrom<GenRefEnum<'_, T>>`, when the mutability of the `GenRefEnum` does not match the generic mutability parameter `M`.
///
/// It implements `std::error::Error` when the `std` feature is enabled. This restriction might be lifted if this gets stabilized: https:// github.com/rust-lang/rust/issues/103765
///
/// Note that although converting from `GenRefEnum<'_, T>::Mutable` to `GenRef<'_, Immutable, T>` is technically sound it is disallowed, because it is usually not what one wants to do.
/// If you really need to do that, convert the reference to an immutable one first.
pub struct IncorrectMutability{
    target_mutable: bool,
}

// This shouldn't require std when this is stabilized: https:// github.com/rust-lang/rust/issues/103765
#[cfg(feature = "std")]
impl std::error::Error for IncorrectMutability{}

impl Debug for IncorrectMutability{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self.target_mutable {
            true => "IncorrectMutability(immut -> mut)",
            false => "IncorrectMutability(mut -> immut)"
        })
    }
}

impl Display for IncorrectMutability{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("\
Failed to convert GenRefEnum<'_, T> into GenRef<'_, {target}, T>:
Mutability of target differs from source

Expected enum variant GenRefEnum::{target}
   Found enum variant GenRefEnum::{source}",
            target = match self.target_mutable {
                true => "Mutable",
                false => "Immutable"
            },
            source = match !self.target_mutable {
                true => "Mutable",
                false => "Immutable"
            }
        ))
    }
}

/// Attempts to convert `GenRefEnum` to a `GenRef` with generic mutability parameter `M`. 
///
/// The conversion is fallible because the variant of the enum is impossible to determine statically, while the value of `M` is determined at compile time.
/// This is essentially an downcast operation.
///
/// This operation has a little runtime cost. Whenever possible, prefer the zero-cost methods on `GenRef`, like `map`, `dispatch` and `split`.
///
/// If the `M` is not a generic mutability parameter, you can use the `From<&T>` or `From<&mut T>` implementations of `GenRef` instead.
///
/// For an unchecked alternative, use `GenRef::new`.
///
/// Note that although converting from `GenRefEnum<'_, T>::Mutable` to `GenRef<'_, Immutable, T>` is technically sound it is disallowed, because it is usually not what one wants to do.
/// If you really need to do that, convert the reference to an immutable one first.
impl<'a, M: Mutability, T: ?Sized> TryFrom<GenRefEnum<'a, T>> for GenRef<'a, M, T> {
    type Error = IncorrectMutability;
    fn try_from(genref_enum: GenRefEnum<'a, T>) -> Result<Self, Self::Error> {
        match (M::IS_MUTABLE, genref_enum) {
            (true, GenRefEnum::Mutable(r)) => {
                unsafe{
                    // SAFETY: `M::IS_MUTABLE` guarantees correct result, so `M` must be `Mutable`
                    // SAFETY: the pointer is obtained from a unique reference,
                    //         so it satisfies all invariants of `GenRef<'a, Mutable, T>`
                    Ok(GenRef::new(NonNull::from(r)))
                }
            },
            (false, GenRefEnum::Immutable(r)) => {
                unsafe{
                    // SAFETY: `M::IS_MUTABLE` guarantees correct result, so `M` must be `Immutable`
                    // SAFETY: the pointer is obtained from a shared reference,
                    //         so it satisfies all invariants of `GenRef<'a, Immutable, T>`
                    Ok(GenRef::new(NonNull::from(r)))
                }
            },
            (target_mutable, _) => {
                Err(IncorrectMutability{ target_mutable })
            }
        }
    }
}