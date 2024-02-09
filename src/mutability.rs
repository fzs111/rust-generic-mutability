use core::ptr::NonNull;

use crate::primitives::{StructureImmutIntoNonNull, StructureMutIntoNonNull};

pub(crate) mod seal{
    pub trait MutabilitySealed {}
}

/// This is one of the types implementing the `Mutability` trait, representing the mutability of a shared reference (`&T`).
/// It is an uninhabited type, only used as typestate.
///
/// Note that mutation of interior mutability types (built around `UnsafeCell`) **is allowed** by `Immutable`, just as it is allowed for shared references.
///
/// For more information, visit the documentation on the `Mutability` trait.
pub enum Immutable{}

/// This is one of the types implementing the `Mutability` trait, representing the mutability of a unique reference (`&mut T`).
/// It is an uninhabited type, only used as typestate.
///
/// For more information, visit the documentation on the `Mutability` trait.
pub enum Mutable{}

impl seal::MutabilitySealed for Mutable{}

impl seal::MutabilitySealed for Immutable{}

/// This is the core primitive of this crate.
/// It exposes raw, pointer-to-pointer operations for both mutability values.
/// This trait is used as a bound for all items that are generic over mutability.
///
/// It is implemented for two types, `Mutable` and `Immutable`, and it is sealed, so no other types can implement it.
pub unsafe trait Mutability: seal::MutabilitySealed {
    
    /// Casts the pointer to a reference of the given mutability, maps it into a reference-structure of the same mutability using either `fn_mut` or `fn_immut` depending on the mutability, and returns is as a `NonNull`-structure.
    /// For more information on structures, see the module documentation of the `primitives` module.
    ///
    /// Although only one of the closures is ever called, capturing the same values with both closures will not work: instead, you can use `moved` helper argument to move the values into the closure that is actually called.
    /// If you do not need to capture any values, you can pass `()` to ignore it.
    ///
    /// # Safety
    ///
    /// `ptr` will be dereferenced and converted to a reference of the chosen mutability. As such:
    ///
    /// - The pointer must be properly aligned.
    /// - The pointer must point to an initialized instance of `T`.
    /// - The lifetime `'a` is arbitrarily chosen and doesn't reflect the actual lifetime of the data. Extra care must be taken to ensure that the correct lifetime is used.
    /// - Furthermore:
    ///     - If the mutability is `Immutable`:
    ///         - The pointer must be valid for reads for lifetime `'a`.
    ///         - The pointed-to value must not be written to by other pointers and no mutable references to it may exist during `'a`.
    ///     - If the mutability is `Mutable`:
    ///         - The pointer must be valid for reads and writes for lifetime `'a`.
    ///         - The pointed-to value must not be accessed (read or written) by other pointers, and no references to it may exist during `'a`.
    /// 
    /// # Guarantees
    /// 
    /// The returned `NonNull`-structure also satisfies all requirements listed in the Safety section. `unsafe` code may rely on this guarantee.

    // The only reason this function is not named `map` is to avoid confusing it with `Mutability::map` on the `main` branch.
    unsafe fn map_to_structure<'a, T, U, UM, UIM, X>(
        ptr: NonNull<T>,
        moved: X,
        fn_mut:   impl FnOnce(&'a mut T, X) -> UM,
        fn_immut: impl FnOnce(&'a     T, X) -> UIM
    ) -> U
        where
            T: 'a + ?Sized,
            UM: StructureMutIntoNonNull<'a, Output = U>,
            UIM: StructureImmutIntoNonNull<'a, Output = U>;

    /// Is `true` if the mutability is `Mutable` and `false` if the mutability is `Immutable`.
    ///
    /// This can be used to observe the mutability of the value at runtime.
    /// Consider using `dispatch` or `map` instead for compile-time resolution.
    /// 
    /// # Guarantees
    ///
    /// This constant has to be implemented correctly and `unsafe` code can rely on this guarantee.
    const IS_MUTABLE: bool;
}

unsafe impl Mutability for Mutable {

    unsafe fn map_to_structure<'a, T, U, UM, UIM, X>(
        ptr: NonNull<T>,
        moved: X,
        fn_mut:    impl FnOnce(&'a mut T, X) -> UM,
        _fn_immut: impl FnOnce(&'a     T, X) -> UIM
    ) -> U
        where
            T: 'a + ?Sized,
            UM: StructureMutIntoNonNull<'a, Output = U>,
            UIM: StructureImmutIntoNonNull<'a, Output = U>
    {
        let reference = unsafe{
            // SAFETY: the caller must uphold the safety contract
            &mut *ptr.as_ptr()
        };
        StructureMutIntoNonNull::into_nonnull_structure(fn_mut(reference, moved))
    }

    const IS_MUTABLE: bool = true;
}
unsafe impl Mutability for Immutable {
    unsafe fn map_to_structure<'a, T, U, UM, UIM, X>(
        ptr: NonNull<T>,
        moved: X,
        _fn_mut:  impl FnOnce(&'a mut T, X) -> UM,
        fn_immut: impl FnOnce(&'a     T, X) -> UIM
    ) -> U
        where
            T: 'a + ?Sized,
            UM: StructureMutIntoNonNull<'a, Output = U>,
            UIM: StructureImmutIntoNonNull<'a, Output = U>
    {
        let reference = unsafe{
            // SAFETY: the caller must uphold the safety contract
            & *ptr.as_ptr()
        };
        StructureImmutIntoNonNull::into_nonnull_structure(fn_immut(reference, moved))
    }

    const IS_MUTABLE: bool = false;
}